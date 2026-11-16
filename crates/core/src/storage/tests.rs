use super::*;
fn tracked(control: &Control, action: uuid::Uuid) -> WriteReceipt {
    let mut state = control.admission.lock().unwrap();
    state.next_write += 1;
    let receipt = WriteReceipt {
        sequence: state.next_write,
        action,
    };
    state.writes.insert(receipt.sequence, action);
    receipt
}
fn slot() -> OwnedSemaphorePermit {
    Arc::new(Semaphore::new(1)).try_acquire_owned().unwrap()
}

#[tokio::test]
async fn lost_requester_keeps_completion_and_status_pending_until_physical_settlement() {
    let control = Arc::new(Control::new());
    let action = uuid::Uuid::nil();
    let receipt = tracked(&control, action);
    let (reply, receiver) = oneshot::channel();
    drop(receiver);
    let (finish, physical) = oneshot::channel();
    let owner = control.clone();
    let task = tokio::spawn(complete_job(
        owner,
        Some(receipt),
        reply,
        slot(),
        async move {
            physical.await.unwrap();
            Ok(Outcome::Written { revision: 7 })
        },
    ));
    assert!(
        tokio::time::timeout(Duration::ZERO, control.settled(Some(action)))
            .await
            .is_err()
    );
    finish.send(()).unwrap();
    task.await.unwrap();
    control.settled(Some(action)).await.unwrap();
    assert!(control.admission.lock().unwrap().writes.is_empty());
    assert!(control.admission.lock().unwrap().faults.is_empty());
}

#[tokio::test]
async fn failed_commit_receipt_waits_for_retirement_and_preserves_both_failures() {
    let (finish, physical) = oneshot::channel();
    let task = tokio::spawn(retire_failed(
        Arc::new(Control::new()),
        Error::Invalid("original commit failure"),
        true,
        async move {
            physical.await.unwrap();
            Err(Error::Invalid("close acknowledgment lost"))
        },
    ));
    assert!(!task.is_finished());
    finish.send(()).unwrap();
    match task.await.unwrap() {
        Error::Write {
            original,
            retirement: Some(retirement),
            uncertain: true,
        } => {
            assert!(matches!(
                *original,
                Error::Invalid("original commit failure")
            ));
            assert!(matches!(
                *retirement,
                Error::Invalid("close acknowledgment lost")
            ));
        }
        other => panic!("lost failure evidence: {other:?}"),
    }
}

#[tokio::test]
async fn retirement_unknown_fences_fresh_observation_and_retains_exact_action_after_reply_loss() {
    let control = Arc::new(Control::new());
    let action = uuid::Uuid::from_u128(42);
    let receipt = tracked(&control, action);
    let (reply, receiver) = oneshot::channel();
    drop(receiver);
    complete_job(control.clone(), Some(receipt), reply, slot(), async {
        Err(retire_failed(
            Arc::new(Control::new()),
            Error::Invalid("commit failed"),
            true,
            async { Err(Error::Invalid("close failed")) },
        )
        .await)
    })
    .await;
    assert!(matches!(
        control.settled(Some(action)).await,
        Err(Error::Fenced)
    ));
    let state = control.admission.lock().unwrap();
    assert!(state.fenced && state.retirement_unknown);
    assert_eq!(state.faults.len(), 1);
    assert_eq!(state.faults[0].action, Some(action));
}

#[tokio::test]
async fn known_retirement_allows_fresh_reconciliation_but_never_grants_write_replay() {
    let control = Arc::new(Control::new());
    let action = uuid::Uuid::nil();
    let receipt = tracked(&control, action);
    let (reply, receiver) = oneshot::channel();
    complete_job(control.clone(), Some(receipt), reply, slot(), async {
        Err(retire_failed(
            Arc::new(Control::new()),
            Error::Invalid("commit acknowledgment lost"),
            true,
            async { Ok(()) },
        )
        .await)
    })
    .await;
    assert!(receiver.await.unwrap().outcome.is_err());
    control.settled(Some(action)).await.unwrap();
    let state = control.admission.lock().unwrap();
    assert!(state.fenced && !state.retirement_unknown);
    assert!(matches!(
        &state.faults[0].error,
        Error::Write {
            uncertain: true,
            retirement: None,
            ..
        }
    ));
}

#[tokio::test]
async fn task_panic_is_owned_attributed_and_fenced() {
    let control = Arc::new(Control::new());
    let action = uuid::Uuid::from_u128(7);
    let receipt = tracked(&control, action);
    let (reply, receiver) = oneshot::channel();
    complete_job(control.clone(), Some(receipt), reply, slot(), async {
        panic!("controlled storage panic")
    })
    .await;
    assert!(matches!(receiver.await.unwrap().outcome, Err(Error::Panic)));
    assert!(matches!(control.settled(None).await, Err(Error::Fenced)));
    assert_eq!(
        control.admission.lock().unwrap().faults[0].action,
        Some(action)
    );
}

#[tokio::test]
async fn completed_undelivered_result_keeps_its_capacity_until_consumed_or_dropped() {
    let control = Arc::new(Control::new());
    let capacity = Arc::new(Semaphore::new(1));
    let (reply, receiver) = oneshot::channel();
    complete_job(
        control,
        None,
        reply,
        capacity.clone().try_acquire_owned().unwrap(),
        async { Ok(Outcome::Written { revision: 1 }) },
    )
    .await;
    assert!(capacity.clone().try_acquire_owned().is_err());
    let delivered = receiver.await.unwrap();
    assert!(capacity.clone().try_acquire_owned().is_err());
    drop(delivered);
    assert!(capacity.try_acquire_owned().is_ok());
}

#[test]
fn bounded_serialization_rejects_before_allocating_an_unbounded_encoded_copy() {
    assert!(matches!(
        encode(&"a".repeat(ITEM_BYTES), ITEM_BYTES),
        Err(Error::Oversized)
    ));
    assert_eq!(encode(&"abc", 5).unwrap(), "\"abc\"");
    assert!(matches!(
        Read::ProviderLogin("a".repeat(4097)).validate(),
        Err(Error::Oversized)
    ));
}

#[test]
fn current_records_retain_recovery_evidence_and_corruption_is_not_absence() {
    let record = serde_json::json!({
        "id": uuid::Uuid::nil(), "repository_id": uuid::Uuid::from_u128(1),
        "creation_action_id": uuid::Uuid::from_u128(2), "name":"marta", "branch":"workspace/marta",
        "path":"/state/workspaces/repo/marta/checkout", "legacy":false, "base":null,
        "artifacts":{"branch":"owned","folder":"partial"},
        "status":{"state":"uncertain","error":{"code":"git","message":"Effect acknowledgment lost"}},
        "cleanup_unconfirmed":true
    });
    let snapshot = Snapshot {
        revision: 9,
        records: vec![record.to_string()],
        next: None,
    };
    let workspaces = snapshot.records::<Workspace>().unwrap();
    assert!(workspaces[0].cleanup_unconfirmed);
    assert_eq!(workspaces[0].artifacts.folder, FolderArtifact::Partial);
    assert_eq!(serde_json::to_value(&workspaces[0]).unwrap(), record);
    let corrupt = Snapshot {
        revision: 9,
        records: vec!["{broken".into()],
        next: None,
    };
    assert!(corrupt.records::<Workspace>().is_err());
}

#[tokio::test]
async fn replacement_waits_for_physical_retirement_and_unknown_close_fences_it() {
    let control = Arc::new(Control::new());
    let (started, started_rx) = oneshot::channel();
    let (finish, physical) = oneshot::channel();
    let task = tokio::spawn(retire_failed(
        control.clone(),
        Error::Invalid("statement failed"),
        false,
        async move {
            started.send(()).unwrap();
            physical.await.unwrap();
            Err(Error::Invalid("retirement acknowledgment lost"))
        },
    ));
    started_rx.await.unwrap();
    assert!(
        tokio::time::timeout(Duration::ZERO, control.ready_connection())
            .await
            .is_err()
    );
    finish.send(()).unwrap();
    task.await.unwrap();
    assert!(matches!(
        control.ready_connection().await,
        Err(Error::Fenced)
    ));
}

#[tokio::test]
async fn aborted_retirement_cannot_reopen_replacement_admission() {
    let control = Arc::new(Control::new());
    let guard = Retirement::begin(control.clone());
    assert!(
        tokio::time::timeout(Duration::ZERO, control.ready_connection())
            .await
            .is_err()
    );
    drop(guard);
    assert!(matches!(
        control.ready_connection().await,
        Err(Error::Fenced)
    ));
}

#[tokio::test]
async fn checkpoint_failure_does_not_skip_either_close_barrier_or_erase_prior_errors() {
    let control = Arc::new(Control::new());
    control.fault(Some(uuid::Uuid::nil()), Error::Corrupt);
    let calls = Mutex::new(Vec::new());
    let report = close_barriers(
        &control,
        async {
            calls.lock().unwrap().push("checkpoint");
            Err(Error::Corrupt)
        },
        async {
            calls.lock().unwrap().push("readers");
        },
        async {
            calls.lock().unwrap().push("writer");
        },
    )
    .await;
    assert_eq!(*calls.lock().unwrap(), ["checkpoint", "readers", "writer"]);
    assert_eq!(report.faults.len(), 2);
    assert_eq!(report.faults[0].action, Some(uuid::Uuid::nil()));
    assert!(report.checkpoint.is_none());
}

#[tokio::test]
async fn later_unrelated_write_cannot_extend_an_admitted_reads_settlement_wait() {
    for same_action in [false, true] {
        for exact in [false, true] {
            let control = Control::new();
            let first = uuid::Uuid::from_u128(1);
            let later = if same_action {
                first
            } else {
                uuid::Uuid::from_u128(2)
            };
            let receipt = tracked(&control, first);
            let admitted_read = control.settled(exact.then_some(first));
            tracked(&control, later);
            control.settle(receipt);
            assert!(
                tokio::time::timeout(Duration::ZERO, admitted_read)
                    .await
                    .is_ok(),
                "A later write extended the admitted read barrier"
            );
        }
    }
}
