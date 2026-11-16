use super::super::tests::{run, support::*};
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
fn native_delivery_waits_for_frontend_acceptance_before_acknowledging() {
    run(async {
        let control = Control::default();
        let first = batch(1, 7, Sample::ResyncRequired { watermark: 7 });
        let second = batch(
            2,
            8,
            Sample::Hints {
                watermark: 8,
                targets: vec![crate::observations::Target::Repositories],
            },
        );
        let mut bytes = wire(&Response::Feed {
            batch: first.clone(),
        });
        bytes.extend(wire(&Response::Feed {
            batch: second.clone(),
        }));
        let stream = Stream::new(bytes);
        let trace = stream.0.clone();
        let accepted = Mutex::new(Vec::new());
        let result = run_subscribed(
            &mut FramedExchange::new(stream),
            "incarnation-a",
            "feed-1",
            Some(FeedCursor {
                incarnation: "incarnation-a".into(),
                sequence: 7,
            }),
            &control,
            &|event| {
                let FeedEvent::Batch { batch } = event else {
                    panic!()
                };
                assert_eq!(
                    frames::<FeedAck>(&trace.output()).len(),
                    accepted.lock().unwrap().len()
                );
                accepted.lock().unwrap().push(batch.acknowledgement.clone());
                control.acknowledge(&batch.acknowledgement)
            },
        )
        .await;
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(
            *accepted.lock().unwrap(),
            [first.acknowledgement, second.acknowledgement]
        );
        assert_eq!(
            frames::<FeedAck>(&trace.output()),
            *accepted.lock().unwrap()
        );
    });
}
#[test]
fn sink_failure_and_frontend_stall_close_without_acknowledgement() {
    run(async {
        for fail in [false, true] {
            let stream = Stream::new(wire(&Response::Feed {
                batch: batch(1, 1, Sample::ResyncRequired { watermark: 1 }),
            }));
            let trace = stream.0.clone();
            let result = run_subscribed(
                &mut FramedExchange::new(stream),
                "incarnation-a",
                "feed-1",
                None,
                &Control::default(),
                &|_| {
                    if fail {
                        Err(io::ErrorKind::BrokenPipe.into())
                    } else {
                        Ok(())
                    }
                },
            )
            .await;
            assert_eq!(
                result.unwrap_err().kind(),
                if fail {
                    io::ErrorKind::BrokenPipe
                } else {
                    io::ErrorKind::TimedOut
                }
            );
            assert!(trace.output().is_empty());
            assert_eq!(trace.0.lock().unwrap().drops, 1);
        }
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
fn server_stops_on_wrong_ack_and_preserves_changes_during_resync() {
    run(async {
        for wrong in [false, true] {
            let journal = Arc::new(crate::observations::Journal::default());
            let mut ack = batch(1, 1, Sample::ResyncRequired { watermark: 1 }).acknowledgement;
            if wrong {
                ack.batch_id += 1;
            }
            let stream = Stream::pending();
            stream.0.feed(&wire(&ack));
            let trace = stream.0.clone();
            let cancel = CancellationToken::new();
            let mut exchange = FramedExchange::new(stream);
            let future = serve(
                "incarnation-a",
                &journal,
                Subscribe {
                    subscription_id: "feed-1".into(),
                    after: None,
                },
                &mut exchange,
                &cancel,
            );
            tokio::pin!(future);
            if wrong {
                assert_eq!(future.await.unwrap_err().kind(), io::ErrorKind::InvalidData);
            } else {
                assert!(pending(future.as_mut()).await);
                journal.publish(vec![crate::observations::Target::Repositories]);
                journal.wake();
                assert!(pending(future.as_mut()).await);
                let output = frames::<Response>(&trace.output());
                assert_eq!(output.len(), 2);
                assert!(matches!(
                    &output[1],
                    Response::Feed {
                        batch: FeedBatch {
                            sample: Sample::Hints { watermark: 2, .. },
                            ..
                        }
                    }
                ));
                cancel.cancel();
                assert_eq!(future.await.unwrap_err().kind(), io::ErrorKind::Interrupted);
            }
        }
    });
}
#[test]
fn idle_server_and_frontend_ack_wait_are_cancellable_and_ack_cap_is_absolute() {
    run(async {
        let control = Control::default();
        control
            .offer(batch(1, 1, Sample::ResyncRequired { watermark: 1 }).acknowledgement)
            .unwrap();
        let future = control.wait_accepted(Instant::now() + ACK_TIMEOUT);
        tokio::pin!(future);
        assert!(pending(future.as_mut()).await);
        control.stop();
        assert_eq!(future.await.unwrap_err().kind(), io::ErrorKind::Interrupted);
        let journal = crate::observations::Journal::default();
        let cancel = CancellationToken::new();
        let mut exchange = FramedExchange::new(Stream::pending());
        let future = serve(
            "incarnation-a",
            &journal,
            Subscribe {
                subscription_id: "feed-1".into(),
                after: Some(FeedCursor {
                    incarnation: "incarnation-a".into(),
                    sequence: 1,
                }),
            },
            &mut exchange,
            &cancel,
        );
        tokio::pin!(future);
        assert!(pending(future.as_mut()).await);
        cancel.cancel();
        assert_eq!(future.await.unwrap_err().kind(), io::ErrorKind::Interrupted);
        let mut exchange = FramedExchange::new(Stream::pending());
        assert_eq!(
            serve(
                "incarnation-a",
                &journal,
                Subscribe {
                    subscription_id: "feed-1".into(),
                    after: None
                },
                &mut exchange,
                &CancellationToken::new()
            )
            .await
            .unwrap_err()
            .kind(),
            io::ErrorKind::TimedOut
        );
    });
}
#[test]
fn stop_joins_late_setup_and_dropped_stop_waiters_fence_replacement() {
    run(async {
        let connection = SupervisorConnection::new("/state".into(), "/supervisor".into());
        initialize(&connection);
        let (started, entered) = tokio::sync::oneshot::channel();
        let (release, finish) = tokio::sync::oneshot::channel();
        connection
            .start_subscription_with(
                "incarnation-a".into(),
                "feed-1".into(),
                Some(FeedCursor {
                    incarnation: "incarnation-a".into(),
                    sequence: 7,
                }),
                |_| Ok(()),
                async move |incarnation, id, after, control, _| {
                    assert_eq!(incarnation, "incarnation-a");
                    assert_eq!(id, "feed-1");
                    assert_eq!(after.unwrap().sequence, 7);
                    started.send(()).unwrap();
                    finish.await.unwrap();
                    assert!(control.cancel.is_cancelled());
                    Ok(())
                },
            )
            .unwrap();
        entered.await.unwrap();
        assert!(connection.subscription.try_acquire().is_none());
        assert!(connection.prepare_snapshot().is_ok());
        assert!(connection.observations.try_acquire().is_some());
        connection
            .stop_subscription("old-generation")
            .await
            .unwrap();
        {
            let stop = connection.stop_subscription("feed-1");
            tokio::pin!(stop);
            assert!(pending(stop.as_mut()).await);
            assert!(
                connection
                    .start_subscription("incarnation-a".into(), "feed-2".into(), None, |_| Ok(()))
                    .is_err()
            );
        }
        assert!(connection.subscription.try_acquire().is_none());
        release.send(()).unwrap();
        connection.stop_subscription("feed-1").await.unwrap();
        connection.stop_subscription("feed-1").await.unwrap();
        assert!(connection.subscription.try_acquire().is_some());
        assert!(connection.feed.current.lock().unwrap().is_none());
    });
}
#[test]
fn completed_error_and_panic_workers_are_joined_before_replacement() {
    run(async {
        for panic in [false, true] {
            let connection = SupervisorConnection::new("/state".into(), "/supervisor".into());
            initialize(&connection);
            let (started, entered) = tokio::sync::oneshot::channel();
            connection
                .start_subscription_with(
                    "incarnation-a".into(),
                    "feed-1".into(),
                    None,
                    |_| Ok(()),
                    async move |_, _, _, _, _| {
                        started.send(()).unwrap();
                        assert!(!panic, "controlled feed panic");
                        Err(io::ErrorKind::ConnectionReset.into())
                    },
                )
                .unwrap();
            entered.await.unwrap();
            assert_eq!(connection.stop_subscription("feed-1").await.is_err(), panic);
            assert!(connection.feed.current.lock().unwrap().is_none());
            assert!(connection.subscription.try_acquire().is_some());
        }
    });
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
        assert!(encode_payload(&Response::Feed { batch }).unwrap().len() + 4 <= DELIVERY_BYTES);
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
fn reloaded_frontend_joins_previous_feed_before_replacing_and_bounds_pending_replacements() {
    run(async {
        let connection = SupervisorConnection::new("/state".into(), "/supervisor".into());
        initialize(&connection);
        let (started, entered) = tokio::sync::oneshot::channel();
        let (release, finish) = tokio::sync::oneshot::channel();
        connection
            .start_subscription_with(
                "incarnation-a".into(),
                "old-page".into(),
                None,
                |_| Ok(()),
                async move |_, _, _, control, _| {
                    started.send(()).unwrap();
                    finish.await.unwrap();
                    assert!(control.cancel.is_cancelled());
                    Ok(())
                },
            )
            .unwrap();
        entered.await.unwrap();
        let (replacement_started, replacement_entered) = tokio::sync::oneshot::channel();
        let replacement = connection.replace_subscription_with(
            "incarnation-a".into(),
            "new-page".into(),
            None,
            |_| Ok(()),
            async move |incarnation, id, after, control, _| {
                assert_eq!(incarnation, "incarnation-a");
                assert_eq!(id, "new-page");
                assert!(after.is_none());
                replacement_started.send(()).unwrap();
                control.cancel.cancelled().await;
                Ok(())
            },
        );
        tokio::pin!(replacement);
        assert!(pending(replacement.as_mut()).await);
        assert!(connection.subscription.try_acquire().is_none());
        assert_eq!(
            connection.feed.current.lock().unwrap().as_ref().unwrap().id,
            "old-page"
        );
        let competing = connection.replace_subscription_with(
            "incarnation-a".into(),
            "competing-page".into(),
            None,
            |_| Ok(()),
            async |_, _, _, _, _| -> io::Result<()> {
                panic!("competing replacement must not start")
            },
        );
        assert!(matches!(
            competing.await.unwrap_err(),
            RequestFailure::Transport {
                error: TransportFailure {
                    code: TransportCode::Busy,
                    ..
                }
            }
        ));
        release.send(()).unwrap();
        replacement.await.unwrap();
        replacement_entered.await.unwrap();
        connection.stop_subscription("old-page").await.unwrap();
        assert_eq!(
            connection.feed.current.lock().unwrap().as_ref().unwrap().id,
            "new-page"
        );
        connection.stop_subscription("new-page").await.unwrap();
        assert!(connection.subscription.try_acquire().is_some());
    });
}

#[test]
fn invalid_reload_replacement_cannot_stop_the_current_feed() {
    run(async {
        for (incarnation, id) in [
            ("other-incarnation", "new-page"),
            ("incarnation-a", "invalid id"),
        ] {
            let connection = SupervisorConnection::new("/state".into(), "/supervisor".into());
            initialize(&connection);
            let (started, entered) = tokio::sync::oneshot::channel();
            connection
                .start_subscription_with(
                    "incarnation-a".into(),
                    "current-page".into(),
                    None,
                    |_| Ok(()),
                    async move |_, _, _, control, _| {
                        assert!(started.send(control.clone()).is_ok());
                        control.cancel.cancelled().await;
                        Ok(())
                    },
                )
                .unwrap();
            let control = entered.await.unwrap();
            assert!(
                connection
                    .replace_subscription_with(
                        incarnation.into(),
                        id.into(),
                        None,
                        |_| Ok(()),
                        async |_, _, _, _, _| -> io::Result<()> {
                            panic!("invalid replacement must not start")
                        }
                    )
                    .await
                    .is_err()
            );
            assert!(!control.cancel.is_cancelled());
            connection.stop_subscription("current-page").await.unwrap();
        }
    });
}

#[test]
fn cancelled_reload_waiter_keeps_old_feed_owned_until_a_later_replacement_joins_it() {
    run(async {
        let connection = SupervisorConnection::new("/state".into(), "/supervisor".into());
        initialize(&connection);
        let (release, finish) = tokio::sync::oneshot::channel();
        connection
            .start_subscription_with(
                "incarnation-a".into(),
                "old-page".into(),
                None,
                |_| Ok(()),
                async move |_, _, _, _, _| {
                    finish.await.unwrap();
                    Ok(())
                },
            )
            .unwrap();
        {
            let replacement = connection.replace_subscription_with(
                "incarnation-a".into(),
                "abandoned-page".into(),
                None,
                |_| Ok(()),
                async |_, _, _, _, _| -> io::Result<()> {
                    panic!("cancelled replacement must not start")
                },
            );
            tokio::pin!(replacement);
            assert!(pending(replacement.as_mut()).await);
        }
        assert!(connection.subscription.try_acquire().is_none());
        assert_eq!(
            connection.feed.current.lock().unwrap().as_ref().unwrap().id,
            "old-page"
        );
        let replacement = connection.replace_subscription_with(
            "incarnation-a".into(),
            "latest-page".into(),
            None,
            |_| Ok(()),
            async |_, _, _, control, _| {
                control.cancel.cancelled().await;
                Ok(())
            },
        );
        tokio::pin!(replacement);
        assert!(pending(replacement.as_mut()).await);
        release.send(()).unwrap();
        replacement.await.unwrap();
        assert_eq!(
            connection.feed.current.lock().unwrap().as_ref().unwrap().id,
            "latest-page"
        );
        connection.stop_subscription("latest-page").await.unwrap();
    });
}
