use workspace_core::{Core, SupervisorState};

#[test]
fn initial_status_reports_the_execution_owner_without_starting_work() {
    let status = Core::new().status();
    assert_eq!(status.state, SupervisorState::Ready);
    assert_eq!(status.supervisor_pid, std::process::id());
    assert_eq!(status.application_version, "0.1.0");
}
