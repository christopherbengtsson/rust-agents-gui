use super::super::tests::run;
use super::*;
fn batch(id: u64, sequence: u64, sample: Sample) -> FeedBatch {
    FeedBatch {
        acknowledgement: FeedAck {
            subscription_id: "feed-1".into(),
            batch_id: id,
            cursor: FeedCursor {
                incarnation: "incarnation-a".into(),
                sequence,
            },
        },
        sample,
    }
}
async fn pending(future: std::pin::Pin<&mut impl Future>) -> bool {
    let mut future = future;
    std::future::poll_fn(|cx| std::task::Poll::Ready(future.as_mut().poll(cx).is_pending())).await
}
#[test]
fn acknowledgements_match_subscription_batch_and_cursor_exactly() {
    run(async {
        let control = Control::default();
        let ack = batch(1, 1, Sample::ResyncRequired { watermark: 1 }).acknowledgement;
        control.offer(ack.clone()).unwrap();
        assert!(control.offer(ack.clone()).is_err());
        for wrong in [
            FeedAck {
                subscription_id: "old".into(),
                ..ack.clone()
            },
            FeedAck {
                batch_id: 2,
                ..ack.clone()
            },
            FeedAck {
                cursor: FeedCursor {
                    incarnation: "other".into(),
                    sequence: 1,
                },
                ..ack.clone()
            },
            FeedAck {
                cursor: FeedCursor {
                    incarnation: "incarnation-a".into(),
                    sequence: 2,
                },
                ..ack.clone()
            },
        ] {
            assert!(control.acknowledge(&wrong).is_err());
        }
        control.acknowledge(&ack).unwrap();
        assert!(control.acknowledge(&ack).is_err());
        control
            .wait_accepted(Instant::now() + ACK_TIMEOUT)
            .await
            .unwrap();
        control.offer(ack).unwrap();
        assert_eq!(
            control
                .wait_accepted(Instant::now())
                .await
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );
        control.stop();
        assert!(
            control
                .wait_accepted(Instant::now() + ACK_TIMEOUT)
                .await
                .is_err()
        );
    });
}
#[test]
fn owner_batch_cursor_and_delivery_bounds_precede_frontend_delivery() {
    let good = batch(
        1,
        2,
        Sample::Hints {
            watermark: 2,
            targets: vec![crate::observations::Target::Repositories],
        },
    );
    assert!(validate_batch(&good, "incarnation-a", "feed-1", 0, Some(1)).is_ok());
    for (incarnation, id, previous, cursor) in [
        ("other", "feed-1", 0, Some(1)),
        ("incarnation-a", "old", 0, Some(1)),
        ("incarnation-a", "feed-1", 1, Some(1)),
        ("incarnation-a", "feed-1", 0, Some(2)),
        ("incarnation-a", "feed-1", u64::MAX, Some(1)),
    ] {
        assert!(validate_batch(&good, incarnation, id, previous, cursor).is_err());
    }
    assert!(
        validate_batch(
            &batch(
                1,
                2,
                Sample::Hints {
                    watermark: 2,
                    targets: vec![crate::observations::Target::Repositories; 33]
                }
            ),
            "incarnation-a",
            "feed-1",
            0,
            Some(1)
        )
        .is_err()
    );
}
#[test]
fn shared_corpus_matches_rust_feed_serialization_and_delivery_bounds() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../contracts/observations-native.json"
    ))
    .unwrap();
    for value in corpus["feed"].as_array().unwrap() {
        let batch: FeedBatch = serde_json::from_value(value.clone()).unwrap();
        assert_eq!(serde_json::to_value(&batch).unwrap(), *value);
        assert!(serde_json::to_vec(&batch).unwrap().len() <= DELIVERY_BYTES);
    }
}
#[test]
fn stopped_before_delivery_cannot_offer_or_acknowledge() {
    let control = Control::default();
    control.stop();
    let ack = batch(1, 1, Sample::ResyncRequired { watermark: 1 }).acknowledgement;
    assert!(control.offer(ack.clone()).is_err());
    assert!(control.acknowledge(&ack).is_err());
}

#[test]
fn direct_delivery_waits_for_acceptance_and_preserves_changes_during_resync() {
    run(async {
        let journal = crate::observations::Journal::default();
        let control = Control::default();
        let (send, mut receive) = tokio::sync::mpsc::unbounded_channel();
        let deliver = |event| {
            send.send(event).unwrap();
            Ok(())
        };
        let delivery = run_direct(
            &journal,
            "incarnation-a",
            "feed-1",
            None,
            &control,
            &deliver,
        );
        tokio::pin!(delivery);
        assert!(pending(delivery.as_mut()).await);
        let FeedEvent::Batch { batch } = receive.try_recv().unwrap() else {
            panic!()
        };
        journal.publish(vec![crate::observations::Target::Repositories]);
        journal.wake();
        assert!(pending(delivery.as_mut()).await);
        assert!(receive.try_recv().is_err());
        control.acknowledge(&batch.acknowledgement).unwrap();
        assert!(pending(delivery.as_mut()).await);
        let FeedEvent::Batch { batch } = receive.try_recv().unwrap() else {
            panic!()
        };
        assert!(matches!(batch.sample, Sample::Hints { watermark: 2, .. }));
        control.acknowledge(&batch.acknowledgement).unwrap();
        assert!(pending(delivery.as_mut()).await);
        tokio::time::advance(HEARTBEAT).await;
        assert!(pending(delivery.as_mut()).await);
        let FeedEvent::Batch { batch } = receive.try_recv().unwrap() else {
            panic!()
        };
        assert!(matches!(batch.sample, Sample::Heartbeat { watermark: 2 }));
        control.stop();
        assert_eq!(
            delivery.await.unwrap_err().kind(),
            io::ErrorKind::Interrupted
        );
    });
}

#[test]
fn direct_sink_failure_and_absolute_ack_deadline_stop_delivery() {
    run(async {
        for fail in [false, true] {
            let journal = crate::observations::Journal::default();
            let control = Control::default();
            let result = run_direct(&journal, "incarnation-a", "feed-1", None, &control, &|_| {
                if fail {
                    Err(io::ErrorKind::BrokenPipe.into())
                } else {
                    Ok(())
                }
            })
            .await;
            assert_eq!(
                result.unwrap_err().kind(),
                if fail {
                    io::ErrorKind::BrokenPipe
                } else {
                    io::ErrorKind::TimedOut
                }
            );
        }
        let journal = crate::observations::Journal::default();
        let control = Control::default();
        let deliver = |_| Ok(());
        let idle = run_direct(
            &journal,
            "incarnation-a",
            "feed-1",
            Some(FeedCursor {
                incarnation: "incarnation-a".into(),
                sequence: 1,
            }),
            &control,
            &deliver,
        );
        tokio::pin!(idle);
        assert!(pending(idle.as_mut()).await);
        control.stop();
        assert_eq!(idle.await.unwrap_err().kind(), io::ErrorKind::Interrupted);
    });
}

#[test]
fn abandoned_stop_keeps_worker_owned_and_collects_its_completion() {
    run(async {
        for panics in [false, true] {
            let owner = FeedOwner::default();
            let control = Arc::new(Control::default());
            let (release, finish) = tokio::sync::oneshot::channel();
            let retained = control.clone();
            *owner.current.lock().unwrap() = Some(NativeSubscription {
                id: "old-page".into(),
                control: control.clone(),
                worker: tokio::spawn(async move {
                    finish.await.unwrap();
                    assert!(retained.cancel.is_cancelled());
                    assert!(!panics, "controlled feed panic");
                }),
            });
            owner.stop("unrelated-page").await.unwrap();
            assert!(!control.cancel.is_cancelled());
            {
                let stop = owner.stop("old-page");
                tokio::pin!(stop);
                assert!(pending(stop.as_mut()).await);
            }
            assert!(control.cancel.is_cancelled());
            assert!(owner.current.lock().unwrap().is_some());
            release.send(()).unwrap();
            assert_eq!(owner.stop_all().await.is_err(), panics);
            assert!(owner.current.lock().unwrap().is_none());
            assert_eq!(owner.stop_all().await.is_err(), panics);
        }
    });
}

#[test]
fn native_reload_bounds_replacements_and_retains_late_worker_after_waiter_loss() {
    run(async {
        let host = crate::native::NativeHost::controlled().await;
        let incarnation = host.snapshot().cursor.incarnation;
        let control = Arc::new(Control::default());
        let (release, finish) = tokio::sync::oneshot::channel();
        *host.feed.current.lock().unwrap() = Some(NativeSubscription {
            id: "old-page".into(),
            control: control.clone(),
            worker: tokio::spawn(async {
                finish.await.unwrap();
            }),
        });
        for (owner, id) in [
            ("retired", "new-page"),
            (incarnation.as_str(), "invalid id"),
        ] {
            assert!(
                host.replace_subscription(owner.into(), id.into(), None, |_| Ok(()))
                    .await
                    .is_err()
            );
            assert!(!control.cancel.is_cancelled());
        }
        {
            let replace =
                host.replace_subscription(incarnation.clone(), "new-page".into(), None, |_| Ok(()));
            tokio::pin!(replace);
            assert!(pending(replace.as_mut()).await);
            assert!(
                matches!(host.replace_subscription(incarnation.clone(), "competitor".into(), None, |_| Ok(())).await,
                Err(RequestFailure::Transport { error }) if error.code == TransportCode::Busy)
            );
        }
        assert!(control.cancel.is_cancelled());
        assert_eq!(
            host.feed.current.lock().unwrap().as_ref().unwrap().id,
            "old-page"
        );
        release.send(()).unwrap();
        let (send, mut receive) = tokio::sync::mpsc::unbounded_channel();
        host.replace_subscription(incarnation, "new-page".into(), None, move |event| {
            send.send(event).unwrap();
            Ok(())
        })
        .await
        .unwrap();
        host.stop_subscription("old-page").await.unwrap();
        assert_eq!(
            host.feed.current.lock().unwrap().as_ref().unwrap().id,
            "new-page"
        );
        host.stop_subscription("new-page").await.unwrap();
        while receive.try_recv().is_ok() {}
        tokio::task::yield_now().await;
        assert!(receive.try_recv().is_err());
        host.begin_quit();
        host.finished().await;
        assert!(host.quit_status().complete);
    });
}
