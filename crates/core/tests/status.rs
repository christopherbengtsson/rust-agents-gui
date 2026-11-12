use workspace_core::{Core, SupervisorState};

#[test]
fn initial_status_reports_the_execution_owner_without_starting_work() {
    let status = Core::new().status();
    assert_eq!(status.state, SupervisorState::Ready);
    assert_eq!(status.supervisor_pid, std::process::id());
    assert_eq!(status.application_version, "0.1.0");
}

#[test]
fn snapshot_catches_up_or_resets_observations_from_another_owner() {
    use workspace_core::SnapshotCursor;
    let core = Core::new();
    let first = core.snapshot(None);
    assert!(first.reset);
    let caught_up = core.snapshot(Some(first.cursor.clone()));
    assert!(!caught_up.reset);
    assert_eq!(caught_up.cursor, first.cursor);
    let stale = core.snapshot(Some(SnapshotCursor {
        owner: "previous-owner".into(),
        revision: 1,
    }));
    assert!(stale.reset);
    assert_eq!(stale.cursor, first.cursor);
    let future = core.snapshot(Some(SnapshotCursor {
        owner: first.cursor.owner,
        revision: 999,
    }));
    assert!(future.reset);
}
