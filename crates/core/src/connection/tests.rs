use super::*;
use std::{cell::RefCell, collections::VecDeque, io::Cursor, rc::Rc};

#[test]
fn desktop_admission_leaves_status_and_intent_independent_of_stalled_observations() {
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    connection
        .snapshot_with(&mut DiscoveryScenario::new(vec![
            Ok(snapshot()),
            Ok(snapshot()),
        ]))
        .unwrap();
    let read = || RepositoryRequest::List {
        after: None,
        refresh: false,
    };
    let first = connection
        .prepare_repository_request("owner-a".into(), read())
        .unwrap();
    let second = connection
        .prepare_repository_request("owner-a".into(), read())
        .unwrap();
    let Err(RequestFailure::Transport { error }) =
        connection.prepare_repository_request("owner-a".into(), read())
    else {
        panic!("third observation must fail before spawn")
    };
    assert_eq!(error.code, TransportCode::Busy);
    assert_eq!(error.delivery, Delivery::NotSent);
    let status = connection.prepare_snapshot().unwrap();
    let intent = connection
        .prepare_repository_request(
            "owner-a".into(),
            RepositoryRequest::Register {
                request: crate::workspaces::RegisterRepository {
                    action_id: crate::workspaces::ActionId(uuid::Uuid::nil()),
                    path: "/repo".into(),
                    create_initial_workspace: false,
                },
            },
        )
        .unwrap();
    assert!(connection.prepare_snapshot().is_err());
    drop((status, intent, first));
    assert!(
        connection
            .prepare_repository_request("owner-a".into(), read())
            .is_ok()
    );
    drop(second);
}

#[test]
fn abandoned_waiter_keeps_physical_admission_until_exchange_completes() {
    use std::sync::mpsc;
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    connection
        .snapshot_with(&mut DiscoveryScenario::new(vec![
            Ok(snapshot()),
            Ok(snapshot()),
        ]))
        .unwrap();
    let read = || RepositoryRequest::List {
        after: None,
        refresh: false,
    };
    let (started, entered) = mpsc::channel();
    let (finish, release) = mpsc::channel();
    let (reply, waiter) = mpsc::channel();
    let checking = connection.clone();
    let admitted = connection
        .prepare_repository_request_with("owner-a".into(), read(), move |_, _| {
            // Acquiring admission/state while physically exchanging must never block on a state lock.
            assert!(checking.observed.try_lock().is_ok());
            started.send(()).unwrap();
            release.recv().unwrap();
            Err(unavailable("controlled lost reply"))
        })
        .unwrap();
    let worker = std::thread::spawn(move || {
        let _ = reply.send(admitted.run());
    });
    entered.recv().unwrap();
    drop(waiter);
    let second = connection
        .prepare_repository_request("owner-a".into(), read())
        .unwrap();
    assert!(
        connection
            .prepare_repository_request("owner-a".into(), read())
            .is_err()
    );
    assert!(connection.prepare_snapshot().is_ok());
    finish.send(()).unwrap();
    worker.join().unwrap();
    assert!(
        connection
            .prepare_repository_request("owner-a".into(), read())
            .is_ok()
    );
    drop(second);
}

#[test]
fn admitted_failure_unwind_and_unstarted_drop_release_their_slots() {
    let slots = ConnectionSlots::new(1);
    let failed = Admitted {
        permit: slots.try_acquire().unwrap(),
        work: || Err::<(), _>("failure"),
    };
    assert!(failed.run().is_err());
    let unwinds = Admitted {
        permit: slots.try_acquire().unwrap(),
        work: || panic!("controlled worker unwind"),
    };
    // Tokio's permit is not UnwindSafe; this controlled panic mutates no shared policy state.
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unwinds.run())).is_err());
    drop(Admitted {
        permit: slots.try_acquire().unwrap(),
        work: || (),
    });
    assert!(slots.try_acquire().is_some());
    let server = ConnectionSlots::new(8);
    let peers: Vec<_> = (0..8).map(|_| server.try_acquire().unwrap()).collect();
    assert!(server.try_acquire().is_none());
    drop(peers);
    assert!(server.try_acquire().is_some());
}

#[test]
fn only_one_discovery_is_admitted_and_status_does_not_lock_across_effects() {
    struct InspectDiscovery<'a> {
        connection: &'a SupervisorConnection,
        effects: DiscoveryScenario,
    }
    impl Discovery for InspectDiscovery<'_> {
        fn prepare(&mut self, path: &Path) -> io::Result<()> {
            assert!(self.connection.observed.try_lock().is_ok());
            let Err(RequestFailure::Transport { error }) = self.connection.prepare_snapshot()
            else {
                panic!("concurrent discovery admitted")
            };
            assert_eq!(error.code, TransportCode::Busy);
            self.effects.prepare(path)
        }
        fn snapshot(
            &mut self,
            state: &Path,
            after: Option<SnapshotCursor>,
        ) -> io::Result<SupervisorSnapshot> {
            assert!(self.connection.observed.try_lock().is_ok());
            self.effects.snapshot(state, after)
        }
        fn owner_present(&mut self, path: &Path) -> io::Result<bool> {
            self.effects.owner_present(path)
        }
        fn start(&mut self, executable: &Path, path: &Path) -> io::Result<()> {
            self.effects.start(executable, path)
        }
        fn elapsed(&self) -> Duration {
            self.effects.elapsed()
        }
        fn wait(&mut self, duration: Duration) {
            self.effects.wait(duration);
        }
    }
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    let mut effects = InspectDiscovery {
        connection: &connection,
        effects: DiscoveryScenario::new(vec![
            failure(io::ErrorKind::NotFound),
            Ok(snapshot()),
            failure(io::ErrorKind::UnexpectedEof),
            Ok(snapshot()),
        ]),
    };
    assert!(connection.snapshot_with(&mut effects).is_err());
    assert!(connection.snapshot_with(&mut effects).is_ok());
    assert_eq!(
        effects
            .effects
            .calls
            .iter()
            .filter(|&&call| call == "start")
            .count(),
        1
    );
    assert_eq!(
        effects
            .effects
            .calls
            .iter()
            .filter(|&&call| call == "prepare")
            .count(),
        1
    );
}

#[test]
fn two_physical_exchanges_allow_status_and_control_effects_before_either_finishes() {
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    connection
        .snapshot_with(&mut DiscoveryScenario::new(vec![
            Ok(snapshot()),
            Ok(snapshot()),
        ]))
        .unwrap();
    let read = || RepositoryRequest::List {
        after: None,
        refresh: false,
    };
    connection
        .prepare_repository_request_with("owner-a".into(), read(), |_, _| {
            connection
                .prepare_repository_request_with("owner-a".into(), read(), |_, _| {
                    assert!(
                        connection
                            .prepare_repository_request("owner-a".into(), read())
                            .is_err()
                    );
                    assert!(
                        connection
                            .snapshot_with(&mut DiscoveryScenario::new(vec![Ok(snapshot())]))
                            .is_ok()
                    );
                    let register = RepositoryRequest::Register {
                        request: crate::workspaces::RegisterRepository {
                            action_id: crate::workspaces::ActionId(uuid::Uuid::nil()),
                            path: "/repo".into(),
                            create_initial_workspace: false,
                        },
                    };
                    connection
                        .prepare_repository_request_with(
                            "owner-a".into(),
                            register,
                            |_, request| {
                                assert!(matches!(
                                    request.repository,
                                    Some(RepositoryRequest::Register { .. })
                                ));
                                assert!(connection.observed.try_lock().is_ok());
                                Ok(Response::Repository {
                                    owner: "owner-a".into(),
                                    result: Err(RepositoryError::storage(
                                        "controlled storage outcome",
                                    )),
                                })
                            },
                        )
                        .unwrap()
                        .run()
                        .unwrap();
                    Err(unavailable("second exchange finishes later"))
                })
                .unwrap()
                .run()
                .unwrap_err();
            Err(unavailable("first exchange finishes last"))
        })
        .unwrap()
        .run()
        .unwrap_err();
}

#[test]
fn workspace_variants_share_observation_and_intent_budgets_with_repository_requests() {
    use serde_json::json;
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    connection
        .snapshot_with(&mut DiscoveryScenario::new(vec![
            Ok(snapshot()),
            Ok(snapshot()),
        ]))
        .unwrap();
    let id = "00000000-0000-4000-8000-000000000001";
    let observations = [
        json!({"operation":"list","repository_id":id,"after":null}),
        json!({"operation":"get","id":id}),
        json!({"operation":"action","action_id":id}),
        json!({"operation":"changes","workspace_id":id,"refresh":true,"comparison_id":null,"after":null}),
        json!({"operation":"change_preview","target":{"workspace_id":id,"comparison_id":id,"file_id":id},"load":true}),
    ];
    for value in observations {
        let request = serde_json::from_value(value).unwrap();
        let first = connection
            .prepare_workspace_request("owner-a".into(), request)
            .unwrap();
        let second = connection
            .prepare_repository_request(
                "owner-a".into(),
                RepositoryRequest::Action {
                    action_id: crate::workspaces::ActionId(uuid::Uuid::nil()),
                },
            )
            .unwrap();
        assert!(
            connection
                .prepare_workspace_request(
                    "owner-a".into(),
                    crate::workspaces::WorkspaceRequest::Get {
                        id: crate::workspaces::WorkspaceId(uuid::Uuid::nil())
                    }
                )
                .is_err()
        );
        drop((first, second));
    }
    for value in [
        json!({"operation":"create","action_id":id,"repository_id":id}),
        json!({"operation":"reconcile","action_id":id,"workspace_id":id}),
    ] {
        let request = serde_json::from_value(value).unwrap();
        let first = connection
            .prepare_workspace_request("owner-a".into(), request)
            .unwrap();
        assert!(
            connection
                .prepare_workspace_request(
                    "owner-a".into(),
                    crate::workspaces::WorkspaceRequest::Create {
                        action_id: crate::workspaces::ActionId(uuid::Uuid::nil()),
                        repository_id: crate::workspaces::RepositoryId(uuid::Uuid::nil())
                    }
                )
                .is_err()
        );
        assert!(
            connection
                .prepare_repository_request(
                    "owner-a".into(),
                    RepositoryRequest::List {
                        after: None,
                        refresh: true
                    }
                )
                .is_ok()
        );
        drop(first);
    }
    let uninitialized =
        SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    for _ in 0..4 {
        let Err(RequestFailure::Transport { error }) = uninitialized.prepare_repository_request(
            "owner-a".into(),
            RepositoryRequest::List {
                after: None,
                refresh: false,
            },
        ) else {
            panic!("uninitialized requests cannot send")
        };
        assert_eq!(error.code, TransportCode::NotConnected); // Rejected preparation never leaks its acquired permit.
    }
}

fn snapshot() -> SupervisorSnapshot {
    Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    }
    .snapshot(None)
}

struct DiscoveryScenario {
    observations: VecDeque<io::Result<SupervisorSnapshot>>,
    prepared: io::Result<()>,
    owner: io::Result<bool>,
    launch: io::Result<()>,
    calls: Vec<&'static str>,
    cursors: Vec<Option<SnapshotCursor>>,
    time: Duration,
    wait_step: Duration,
}
impl DiscoveryScenario {
    fn new(observations: Vec<io::Result<SupervisorSnapshot>>) -> Self {
        Self {
            observations: observations.into(),
            prepared: Ok(()),
            owner: Ok(false),
            launch: Ok(()),
            calls: vec![],
            cursors: vec![],
            time: Duration::ZERO,
            wait_step: Duration::from_millis(25),
        }
    }
}
impl Discovery for DiscoveryScenario {
    fn prepare(&mut self, state: &Path) -> io::Result<()> {
        assert_eq!(state, Path::new("/private/state"));
        self.calls.push("prepare");
        std::mem::replace(&mut self.prepared, Ok(()))
    }
    fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot> {
        assert_eq!(state, Path::new("/private/state"));
        self.calls.push("snapshot");
        self.cursors.push(after);
        self.observations
            .pop_front()
            .expect("unexpected observation/retry")
    }
    fn owner_present(&mut self, _: &Path) -> io::Result<bool> {
        self.calls.push("owner");
        std::mem::replace(&mut self.owner, Ok(false))
    }
    fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()> {
        assert_eq!(executable, Path::new("/app/supervisor"));
        assert_eq!(state, Path::new("/private/state"));
        self.calls.push("start");
        std::mem::replace(&mut self.launch, Ok(()))
    }
    fn elapsed(&self) -> Duration {
        self.time
    }
    fn wait(&mut self, duration: Duration) {
        assert_eq!(duration, Duration::from_millis(25));
        self.calls.push("wait");
        self.time += self.wait_step;
    }
}
fn failure(kind: io::ErrorKind) -> io::Result<SupervisorSnapshot> {
    Err(io::Error::from(kind))
}
fn discover_scenario(scenario: &mut DiscoveryScenario) -> io::Result<()> {
    discover(
        Path::new("/private/state"),
        Path::new("/app/supervisor"),
        scenario,
    )
}

#[test]
fn existing_owner_is_discovered_without_lock_probe_or_start() {
    let mut scenario = DiscoveryScenario::new(vec![Ok(snapshot())]);
    discover_scenario(&mut scenario).unwrap();
    assert_eq!(scenario.calls, ["prepare", "snapshot"]);
}

#[test]
fn absent_owner_starts_once_and_waits_for_readiness() {
    let mut scenario = DiscoveryScenario::new(vec![
        failure(io::ErrorKind::NotFound),
        failure(io::ErrorKind::ConnectionRefused),
        failure(io::ErrorKind::UnexpectedEof),
        Ok(snapshot()),
    ]);
    discover_scenario(&mut scenario).unwrap();
    assert_eq!(
        scenario.calls,
        [
            "prepare", "snapshot", "owner", "start", "snapshot", "wait", "snapshot", "wait",
            "snapshot"
        ]
    );
}

#[test]
fn concurrent_owner_startup_is_waited_for_without_starting_a_competitor() {
    let mut scenario = DiscoveryScenario::new(vec![
        failure(io::ErrorKind::ConnectionRefused),
        failure(io::ErrorKind::NotFound),
        Ok(snapshot()),
    ]);
    scenario.owner = Ok(true);
    discover_scenario(&mut scenario).unwrap();
    assert_eq!(
        scenario.calls,
        [
            "prepare", "snapshot", "owner", "snapshot", "wait", "snapshot"
        ]
    );
}

#[test]
fn unsafe_state_or_incompatible_endpoint_never_starts_an_owner() {
    let mut scenario = DiscoveryScenario::new(vec![]);
    scenario.prepared = Err(io::ErrorKind::PermissionDenied.into());
    assert_eq!(
        discover_scenario(&mut scenario).unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(scenario.calls, ["prepare"]);
    for kind in [
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::InvalidData,
        io::ErrorKind::TimedOut,
        io::ErrorKind::UnexpectedEof,
    ] {
        let mut scenario = DiscoveryScenario::new(vec![failure(kind)]);
        assert_eq!(discover_scenario(&mut scenario).unwrap_err().kind(), kind);
        assert_eq!(scenario.calls, ["prepare", "snapshot"]);
    }
}

#[test]
fn lock_probe_and_spawn_failures_remain_failures_without_retry() {
    let mut scenario = DiscoveryScenario::new(vec![failure(io::ErrorKind::NotFound)]);
    scenario.owner = Err(io::ErrorKind::PermissionDenied.into());
    assert!(discover_scenario(&mut scenario).is_err());
    assert_eq!(scenario.calls, ["prepare", "snapshot", "owner"]);
    let mut scenario = DiscoveryScenario::new(vec![failure(io::ErrorKind::NotFound)]);
    scenario.launch = Err(io::ErrorKind::NotFound.into());
    assert!(discover_scenario(&mut scenario).is_err());
    assert_eq!(scenario.calls, ["prepare", "snapshot", "owner", "start"]);
}

#[test]
fn startup_timeout_is_controlled_and_does_not_launch_again() {
    let mut scenario = DiscoveryScenario::new(vec![
        failure(io::ErrorKind::NotFound),
        failure(io::ErrorKind::NotFound),
        failure(io::ErrorKind::ConnectionRefused),
    ]);
    scenario.wait_step = Duration::from_secs(5);
    let error = discover_scenario(&mut scenario).unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert!(error.to_string().contains("Do not remove the owner lock"));
    assert_eq!(
        scenario
            .calls
            .iter()
            .filter(|&&call| call == "start")
            .count(),
        1
    );
    assert_eq!(
        scenario
            .calls
            .iter()
            .filter(|&&call| call == "wait")
            .count(),
        1
    );
}

#[test]
fn non_transient_readiness_failure_is_not_retried() {
    let mut scenario = DiscoveryScenario::new(vec![
        failure(io::ErrorKind::NotFound),
        failure(io::ErrorKind::InvalidData),
    ]);
    assert_eq!(
        discover_scenario(&mut scenario).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert!(!scenario.calls.contains(&"wait"));
}

#[test]
fn refresh_after_disconnect_retrieves_cursor_without_restart_or_replay() {
    let first = snapshot();
    let mut recovered = first.clone();
    recovered.reset = false;
    let mut scenario = DiscoveryScenario::new(vec![
        Ok(first.clone()),
        Ok(first.clone()),
        failure(io::ErrorKind::ConnectionRefused),
        Ok(recovered),
    ]);
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    connection.snapshot_with(&mut scenario).unwrap();
    let error = connection.snapshot_with(&mut scenario).unwrap_err();
    assert!(error.to_string().contains("no actions have been replayed"));
    assert_eq!(
        connection.snapshot_with(&mut scenario).unwrap().cursor,
        first.cursor
    );
    assert_eq!(
        scenario.calls,
        ["prepare", "snapshot", "snapshot", "snapshot", "snapshot"]
    );
    assert_eq!(
        scenario.cursors,
        [None, None, Some(first.cursor.clone()), Some(first.cursor)]
    );
}

#[test]
fn spawn_command_preserves_executable_and_single_state_argument() {
    let command = supervisor_command(
        Path::new("/app with spaces/supervisor"),
        Path::new("/private/state with spaces"),
    );
    assert_eq!(command.get_program(), "/app with spaces/supervisor");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["/private/state with spaces"]
    );
}

#[test]
fn dropping_desktop_and_reopening_observes_the_same_owner_without_starting() {
    let before = snapshot();
    let mut scenario = DiscoveryScenario::new(vec![
        Ok(before.clone()),
        Ok(before.clone()),
        Ok(before.clone()),
        Ok(before.clone()),
    ]);
    let desktop = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    desktop.snapshot_with(&mut scenario).unwrap();
    drop(desktop);
    assert_eq!(scenario.calls, ["prepare", "snapshot", "snapshot"]);
    let reopened = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    assert_eq!(
        reopened.snapshot_with(&mut scenario).unwrap().cursor,
        before.cursor
    );
    assert_eq!(
        scenario.calls,
        [
            "prepare", "snapshot", "snapshot", "prepare", "snapshot", "snapshot"
        ]
    );
}

#[test]
fn lost_owner_never_restarts_on_refresh_but_reopen_can_admit_a_new_owner() {
    let before = snapshot();
    let after = Core {
        owner: "owner-b".into(),
        pid: 43,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    }
    .snapshot(Some(before.cursor.clone()));
    let mut scenario = DiscoveryScenario::new(vec![
        Ok(before.clone()),
        Ok(before),
        failure(io::ErrorKind::ConnectionRefused),
        failure(io::ErrorKind::ConnectionRefused),
        Ok(after.clone()),
        Ok(after.clone()),
    ]);
    let desktop = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    desktop.snapshot_with(&mut scenario).unwrap();
    assert!(desktop.snapshot_with(&mut scenario).is_err());
    assert!(!scenario.calls.contains(&"start"));
    let reopened = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    let observed = reopened.snapshot_with(&mut scenario).unwrap();
    assert!(observed.reset);
    assert_eq!(observed.cursor, after.cursor);
    assert_eq!(
        scenario
            .calls
            .iter()
            .filter(|&&call| call == "start")
            .count(),
        1
    );
}

pub(super) struct TransportScenario {
    input: Cursor<Vec<u8>>,
    chunk_size: usize,
    timeouts: RefCell<Vec<Duration>>,
    read_waits: RefCell<Vec<Duration>>,
    write_chunk_size: usize,
    fault: Option<TransportFault>,
    pub(super) trace: Rc<RefCell<TransportTrace>>,
}

#[derive(Clone, Copy)]
enum TransportFault {
    ReadTimeout,
    ReadTimeoutAfterFirstRead,
    ReadWait,
    WriteTimeout,
    WriteAfter(usize),
    Read,
}

#[derive(Default)]
pub(super) struct TransportTrace {
    pub(super) output: Vec<u8>,
    reads: usize,
    writes: usize,
    drops: usize,
}

impl TransportScenario {
    pub(super) fn new(input: Vec<u8>) -> Self {
        Self {
            input: Cursor::new(input),
            chunk_size: usize::MAX,
            timeouts: RefCell::new(vec![]),
            read_waits: RefCell::new(vec![]),
            write_chunk_size: usize::MAX,
            fault: None,
            trace: Rc::default(),
        }
    }
}
impl Read for TransportScenario {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.trace.borrow_mut().reads += 1;
        if matches!(self.fault, Some(TransportFault::Read)) {
            return Err(io::ErrorKind::ConnectionReset.into());
        }
        let length = bytes.len().min(self.chunk_size);
        self.input.read(&mut bytes[..length])
    }
}
impl Write for TransportScenario {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut trace = self.trace.borrow_mut();
        trace.writes += 1;
        let mut length = bytes.len().min(self.write_chunk_size);
        if let Some(TransportFault::WriteAfter(limit)) = self.fault {
            if trace.output.len() >= limit {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            length = length.min(limit - trace.output.len());
        }
        trace.output.extend_from_slice(&bytes[..length]);
        Ok(length)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl FrameStream for TransportScenario {
    fn wait_readable(&self, timeout: Duration) -> io::Result<()> {
        self.read_waits.borrow_mut().push(timeout);
        if matches!(self.fault, Some(TransportFault::ReadWait)) {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(())
    }
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        if matches!(self.fault, Some(TransportFault::ReadTimeout)) {
            return Err(io::ErrorKind::Unsupported.into());
        }
        // macOS rejects timeout changes after a peer closes, even with unread bytes.
        if matches!(self.fault, Some(TransportFault::ReadTimeoutAfterFirstRead))
            && self.trace.borrow().reads > 0
        {
            return Err(io::Error::from_raw_os_error(libc::EINVAL));
        }
        self.timeouts.borrow_mut().push(timeout.unwrap());
        Ok(())
    }
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        assert!(timeout.is_some_and(|timeout| !timeout.is_zero() && timeout <= IO_TIMEOUT));
        if matches!(self.fault, Some(TransportFault::WriteTimeout)) {
            return Err(io::ErrorKind::Unsupported.into());
        }
        Ok(())
    }
}
impl Drop for TransportScenario {
    fn drop(&mut self) {
        self.trace.borrow_mut().drops += 1;
    }
}
fn request(protocol: u32, version: &str, after: Option<SnapshotCursor>) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(&Request {
        protocol,
        subscribe: None,
        application_version: version.into(),
        after,
        expected_owner: None,
        repository: None,
        workspace: None,
    })
    .unwrap();
    bytes.push(b'\n');
    bytes
}

#[test]
fn status_exchange_and_version_rejection_use_real_protocol_parsers() {
    let core = Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    };
    let mut stream = TransportScenario::new(request(
        PROTOCOL_VERSION,
        env!("CARGO_PKG_VERSION"),
        Some(snapshot().cursor),
    ));
    serve_stream(&core, &mut stream).unwrap();
    let Response::Snapshot { snapshot } =
        serde_json::from_slice(&stream.trace.borrow().output).unwrap()
    else {
        panic!("missing snapshot")
    };
    assert_eq!(snapshot.status.supervisor_pid, 42);
    assert_eq!(snapshot.status.state, crate::SupervisorState::Ready);
    assert!(!snapshot.reset);
    for (protocol, version) in [
        (4, env!("CARGO_PKG_VERSION")),
        (5, env!("CARGO_PKG_VERSION")),
        (6, env!("CARGO_PKG_VERSION")),
        (PROTOCOL_VERSION + 1, env!("CARGO_PKG_VERSION")),
        (999, env!("CARGO_PKG_VERSION")),
        (PROTOCOL_VERSION, "99.0.0"),
    ] {
        let mut stream = TransportScenario::new(request(protocol, version, None));
        serve_stream(&core, &mut stream).unwrap();
        let Response::Incompatible { message } =
            serde_json::from_slice(&stream.trace.borrow().output).unwrap()
        else {
            panic!("incompatible client accepted")
        };
        assert!(message.contains("Install matching desktop and supervisor builds"));
    }
}

#[test]
fn inspection_uses_the_existing_workspace_protocol_and_reports_owner_failure() {
    use crate::workspaces::{WorkspaceId, WorkspaceRequest};
    let core = Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    };
    let mut input = serde_json::to_vec(&Request {
        protocol: PROTOCOL_VERSION,
        application_version: env!("CARGO_PKG_VERSION").into(),
        after: None,
        expected_owner: Some("owner-a".into()),
        repository: None,
        workspace: Some(WorkspaceRequest::Changes {
            workspace_id: WorkspaceId(uuid::Uuid::from_u128(7)),
            refresh: true,
            comparison_id: None,
            after: None,
        }),
        subscribe: None,
    })
    .unwrap();
    input.push(b'\n');
    let mut stream = TransportScenario::new(input);
    serve_stream(&core, &mut stream).unwrap();
    let Response::Workspace {
        result: Err(error), ..
    } = serde_json::from_slice(&stream.trace.borrow().output).unwrap()
    else {
        panic!("inspection must retain workspace routing and typed failure")
    };
    assert_eq!(error.code, ErrorCode::Storage);
}

#[test]
fn repository_protocol_returns_typed_storage_failure_without_claiming_empty_navigation() {
    let core = Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    };
    let mut input = serde_json::to_vec(&Request {
        protocol: PROTOCOL_VERSION,
        application_version: env!("CARGO_PKG_VERSION").into(),
        after: None,
        expected_owner: Some("owner-a".into()),
        workspace: None,
        repository: Some(RepositoryRequest::List {
            after: None,
            refresh: false,
        }),
        subscribe: None,
    })
    .unwrap();
    input.push(b'\n');
    let mut stream = TransportScenario::new(input);
    serve_stream(&core, &mut stream).unwrap();
    let Response::Repository {
        result: Err(error), ..
    } = serde_json::from_slice(&stream.trace.borrow().output).unwrap()
    else {
        panic!("storage failure must be an observation error")
    };
    assert_eq!(error.code, ErrorCode::Storage);
    assert!(error.message.contains("execution owner"));
}

#[test]
fn oversized_registration_is_a_definite_capacity_rejection_before_transport() {
    let client = SupervisorClient::new("/unused-private-state".into());
    for path in [
        format!("/{}", "a".repeat(4096)),
        format!("/{}", "é".repeat(2048)),
        format!("/{}", "a".repeat(MAX_FRAME_BYTES)),
    ] {
        let request = RepositoryRequest::Register {
            request: crate::workspaces::RegisterRepository {
                create_initial_workspace: false,
                action_id: crate::workspaces::ActionId(uuid::Uuid::from_u128(1)),
                path,
            },
        };
        let error = client
            .repository_request_with("owner-a", request, |_, _| {
                panic!("oversized input must not access state, connect, or send")
            })
            .unwrap_err();
        let RequestFailure::Rejected { error } = error else {
            panic!("expected local rejection")
        };
        assert_eq!(error.code, ErrorCode::Capacity);
        assert!(error.message.contains("4096 UTF-8 bytes"));
    }
}

#[test]
fn bounded_registration_uses_transport_and_preserves_uncertain_failures() {
    let client = SupervisorClient::new("/unused-private-state".into());
    for path in [
        format!("/{}", "a".repeat(4095)),
        format!("/{}a", "é".repeat(2047)),
        format!("/{}", "\u{1}".repeat(4095)),
    ] {
        let request = RepositoryRequest::Register {
            request: crate::workspaces::RegisterRepository {
                create_initial_workspace: false,
                action_id: crate::workspaces::ActionId(uuid::Uuid::from_u128(1)),
                path,
            },
        };
        let mut calls = 0;
        for kind in [
            io::ErrorKind::BrokenPipe,
            io::ErrorKind::ConnectionReset,
            io::ErrorKind::TimedOut,
        ] {
            let error = client
                .repository_request_with("owner-a", request.clone(), |state, envelope| {
                    calls += 1;
                    assert_eq!(state, Path::new("/unused-private-state"));
                    let Some(RepositoryRequest::Register { request: sent }) = &envelope.repository
                    else {
                        panic!("expected the targeted registration request")
                    };
                    let RepositoryRequest::Register { request: expected } = &request else {
                        unreachable!()
                    };
                    assert_eq!(sent, expected);
                    let mut frame = Vec::new();
                    write_frame(&mut frame, envelope).unwrap();
                    assert!(frame.len() <= MAX_FRAME_BYTES);
                    Err(unavailable(io::Error::from(kind)))
                })
                .unwrap_err();
            let RequestFailure::Transport { error } = error else {
                panic!("expected transport failure")
            };
            assert_eq!(error.code, TransportCode::ReplyUnavailable);
            assert!(error.message.contains("recorded action status"));
        }
        assert_eq!(calls, 3);
    }
}

#[test]
fn oversized_incomplete_and_malformed_frames_are_rejected_without_output() {
    let core = Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    };
    for input in [
        vec![b'x'; MAX_FRAME_BYTES + 1],
        b"{}".to_vec(),
        b"invalid\n".to_vec(),
    ] {
        let mut stream = TransportScenario::new(input);
        assert!(serve_stream(&core, &mut stream).is_err());
        assert!(stream.trace.borrow().output.is_empty());
    }
    let mut oversized = vec![];
    assert_eq!(
        write_frame(&mut oversized, &"x".repeat(MAX_FRAME_BYTES))
            .unwrap_err()
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert!(oversized.is_empty());
    let mut healthy =
        TransportScenario::new(request(PROTOCOL_VERSION, env!("CARGO_PKG_VERSION"), None));
    serve_stream(&core, &mut healthy).unwrap();
    assert!(!healthy.trace.borrow().output.is_empty());
}

#[test]
fn large_reply_survives_peer_close_after_first_chunk() {
    let expected = "x".repeat(45_000);
    let mut frame = serde_json::to_vec(&expected).unwrap();
    frame.push(b'\n');
    let mut stream = TransportScenario::new(frame);
    stream.fault = Some(TransportFault::ReadTimeoutAfterFirstRead);
    let start = Instant::now();
    let received: String = read_frame_with_clock(&mut stream, || start).unwrap();
    assert_eq!(received, expected);
    assert!(stream.trace.borrow().reads > 1);
}

#[test]
fn slow_trickled_frame_uses_an_absolute_deadline_without_waiting() {
    let mut stream = TransportScenario::new(b"{}\n".to_vec());
    stream.chunk_size = 1;
    let start = Instant::now();
    let mut times = VecDeque::from([
        start,
        start,
        start + Duration::from_secs(1),
        start + Duration::from_secs(3),
    ]);
    let error =
        read_frame_with_clock::<serde_json::Value>(&mut stream, || times.pop_front().unwrap())
            .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(
        *stream.read_waits.borrow(),
        [Duration::from_secs(2), Duration::from_secs(1)]
    );
}

#[derive(Debug, PartialEq, Eq)]
enum ClientCall {
    CheckPrivate(PathBuf, bool),
    Connect(PathBuf),
}

fn client_calls() -> [ClientCall; 3] {
    [
        ClientCall::CheckPrivate("/private/state".into(), true),
        ClientCall::CheckPrivate("/private/state/supervisor.sock".into(), false),
        ClientCall::Connect("/private/state/supervisor.sock".into()),
    ]
}

struct ClientScenario {
    checks: VecDeque<io::Result<()>>,
    connect_error: Option<io::ErrorKind>,
    stream: Option<TransportScenario>,
    pub(super) trace: Rc<RefCell<TransportTrace>>,
    calls: Vec<ClientCall>,
    time: Instant,
    times: VecDeque<Instant>,
}

impl ClientScenario {
    pub(super) fn new(input: Vec<u8>) -> Self {
        let stream = TransportScenario::new(input);
        Self {
            checks: VecDeque::from([Ok(()), Ok(())]),
            connect_error: None,
            trace: Rc::clone(&stream.trace),
            stream: Some(stream),
            calls: vec![],
            time: Instant::now(),
            times: VecDeque::new(),
        }
    }

    fn snapshot(&mut self) -> io::Result<SupervisorSnapshot> {
        SupervisorClient::new("/private/state".into()).snapshot_with(
            None,
            PROTOCOL_VERSION,
            env!("CARGO_PKG_VERSION"),
            self,
        )
    }
}

impl ClientEffects for ClientScenario {
    type Stream = TransportScenario;

    fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        self.calls
            .push(ClientCall::CheckPrivate(path.into(), directory));
        self.checks.pop_front().expect("unexpected private check")
    }

    fn connect(&mut self, socket: &Path) -> io::Result<Self::Stream> {
        self.calls.push(ClientCall::Connect(socket.into()));
        if let Some(kind) = self.connect_error {
            return Err(kind.into());
        }
        Ok(self.stream.take().expect("unexpected reconnect"))
    }

    fn now(&mut self) -> Instant {
        self.times.pop_front().unwrap_or(self.time)
    }
}

// Synthetic application-supervisor replies, not native-provider protocol evidence.
fn response(response: Response) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(&response).unwrap();
    bytes.push(b'\n');
    bytes
}

#[test]
fn client_sends_exact_versions_and_cursor_and_reads_a_chunked_snapshot() {
    for after in [None, Some(snapshot().cursor)] {
        let expected = snapshot();
        let mut scenario = ClientScenario::new(response(Response::Snapshot {
            snapshot: expected.clone(),
        }));
        let stream = scenario.stream.as_mut().unwrap();
        stream.chunk_size = 3;
        stream.write_chunk_size = 3;

        let observed = SupervisorClient::new("/private/state".into())
            .snapshot_with(after.clone(), 17, "client-version", &mut scenario)
            .unwrap();

        assert_eq!(observed.cursor, expected.cursor);
        assert_eq!(
            observed.status.supervisor_pid,
            expected.status.supervisor_pid
        );
        assert_eq!(observed.status.state, expected.status.state);
        assert_eq!(
            observed.status.application_version,
            expected.status.application_version
        );
        assert_eq!(observed.reset, expected.reset);
        assert_eq!(scenario.calls, client_calls());
        let trace = scenario.trace.borrow();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&trace.output).unwrap(),
            serde_json::json!({
                "protocol": 17,
                "application_version": "client-version",
                "after": after,
            })
        );
        assert_eq!(trace.output.last(), Some(&b'\n'));
        assert!(trace.writes > 1);
        assert!(trace.reads > 1);
        assert_eq!(trace.drops, 1);
    }
}

#[test]
fn client_preserves_incompatible_version_guidance_without_another_exchange() {
    let message = "Install matching desktop and supervisor builds";
    let mut scenario = ClientScenario::new(response(Response::Incompatible {
        message: message.into(),
    }));

    let error = scenario.snapshot().unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), message);
    assert_eq!(scenario.calls, client_calls());
    assert_eq!(scenario.trace.borrow().writes, 1);
    assert_eq!(scenario.trace.borrow().drops, 1);
}

#[test]
fn client_access_and_connect_failures_stop_before_transport_io() {
    for failed_step in 0..3 {
        let mut scenario = ClientScenario::new(vec![]);
        if failed_step < 2 {
            scenario.checks[failed_step] = Err(io::ErrorKind::PermissionDenied.into());
        } else {
            scenario.connect_error = Some(io::ErrorKind::ConnectionRefused);
        }

        let error = scenario.snapshot().unwrap_err();

        assert_eq!(
            error.kind(),
            if failed_step < 2 {
                io::ErrorKind::PermissionDenied
            } else {
                io::ErrorKind::ConnectionRefused
            }
        );
        assert_eq!(scenario.calls, client_calls()[..=failed_step]);
        let trace = scenario.trace.borrow();
        assert_eq!(trace.writes, 0);
        assert_eq!(trace.reads, 0);
        assert_eq!(trace.drops, 0);
    }
}

#[test]
fn client_timeout_configuration_failures_drop_the_stream_without_io() {
    for fault in [TransportFault::ReadTimeout, TransportFault::WriteTimeout] {
        let mut scenario = ClientScenario::new(vec![]);
        scenario.stream.as_mut().unwrap().fault = Some(fault);

        assert_eq!(
            scenario.snapshot().unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
        assert_eq!(scenario.calls, client_calls());
        let trace = scenario.trace.borrow();
        assert_eq!(trace.writes, 0);
        assert_eq!(trace.reads, 0);
        assert_eq!(trace.drops, 1);
    }
}

#[test]
fn client_partial_write_failure_neither_reads_nor_repeats_the_request() {
    let mut scenario = ClientScenario::new(vec![]);
    let stream = scenario.stream.as_mut().unwrap();
    stream.write_chunk_size = 3;
    stream.fault = Some(TransportFault::WriteAfter(7));

    let error = scenario.snapshot().unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
    assert_eq!(scenario.calls, client_calls());
    let trace = scenario.trace.borrow();
    assert_eq!(trace.output, b"{\"proto");
    assert_eq!(trace.reads, 0);
    assert_eq!(trace.drops, 1);
}

#[test]
fn client_rejects_an_oversized_request_before_writing() {
    let mut scenario = ClientScenario::new(vec![]);
    let error = SupervisorClient::new("/private/state".into())
        .snapshot_with(
            None,
            PROTOCOL_VERSION,
            &"v".repeat(MAX_FRAME_BYTES),
            &mut scenario,
        )
        .unwrap_err();

    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert_eq!(scenario.calls, client_calls());
    let trace = scenario.trace.borrow();
    assert_eq!(trace.writes, 0);
    assert_eq!(trace.reads, 0);
    assert_eq!(trace.drops, 1);
}

#[test]
fn client_bad_or_lost_replies_do_not_reconnect_or_repeat_the_request() {
    let mut truncated = response(Response::Snapshot {
        snapshot: snapshot(),
    });
    truncated.pop();
    for (input, expected_kind) in [
        (b"invalid\n".to_vec(), io::ErrorKind::Other),
        (b"{}\n".to_vec(), io::ErrorKind::Other),
        (truncated, io::ErrorKind::UnexpectedEof),
        (vec![b'x'; MAX_FRAME_BYTES + 1], io::ErrorKind::InvalidData),
    ] {
        let mut scenario = ClientScenario::new(input);

        assert_eq!(scenario.snapshot().unwrap_err().kind(), expected_kind);
        assert_eq!(scenario.calls, client_calls());
        let trace = scenario.trace.borrow();
        assert_eq!(trace.writes, 1);
        assert_eq!(trace.drops, 1);
    }
}

#[test]
fn client_readiness_failure_does_not_repeat_exchange_or_read() {
    let mut scenario = ClientScenario::new(b"{}\n".to_vec());
    scenario.stream.as_mut().unwrap().fault = Some(TransportFault::ReadWait);
    assert_eq!(
        scenario.snapshot().unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert_eq!(scenario.calls, client_calls());
    let trace = scenario.trace.borrow();
    assert_eq!(trace.writes, 1);
    assert_eq!(trace.reads, 0);
    assert_eq!(trace.drops, 1);
}

#[test]
fn client_read_failure_and_deadline_expiry_do_not_repeat_the_exchange() {
    for slow_reply in [false, true] {
        let mut scenario = ClientScenario::new(b"{}\n".to_vec());
        let stream = scenario.stream.as_mut().unwrap();
        let expected_kind = if slow_reply {
            stream.chunk_size = 1;
            let start = scenario.time;
            scenario.times = VecDeque::from([
                start,
                start,
                start + Duration::from_secs(1),
                start + Duration::from_secs(3),
            ]);
            io::ErrorKind::TimedOut
        } else {
            stream.fault = Some(TransportFault::Read);
            io::ErrorKind::ConnectionReset
        };

        assert_eq!(scenario.snapshot().unwrap_err().kind(), expected_kind);
        assert_eq!(scenario.calls, client_calls());
        assert!(scenario.times.is_empty());
        let trace = scenario.trace.borrow();
        assert_eq!(trace.writes, 1);
        assert_eq!(trace.reads, if slow_reply { 2 } else { 1 });
        assert_eq!(trace.drops, 1);
    }
}

#[test]
fn wrong_owner_rejects_every_target_before_domain_routing() {
    let core = Core {
        owner: "owner-b".into(),
        pid: 43,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    };
    for target in [
        serde_json::json!({"repository":{"operation":"register","request":{"action_id":"00000000-0000-0000-0000-000000000001","path":"/projects/one"}}}),
        serde_json::json!({"repository":{"operation":"list","after":null,"refresh":true}}),
        serde_json::json!({"workspace":{"operation":"create","action_id":"00000000-0000-0000-0000-000000000001","repository_id":"00000000-0000-0000-0000-000000000002"}}),
        serde_json::json!({"workspace":{"operation":"changes","workspace_id":"00000000-0000-0000-0000-000000000003","refresh":true,"comparison_id":null,"after":null}}),
    ] {
        let mut envelope = serde_json::json!({"protocol":PROTOCOL_VERSION,"application_version":env!("CARGO_PKG_VERSION"),"after":null,"expected_owner":"owner-a"});
        envelope
            .as_object_mut()
            .unwrap()
            .extend(target.as_object().unwrap().clone());
        dispatch_request(
            serde_json::from_value(envelope.clone()).unwrap(),
            "owner-b",
            |_| panic!("wrong owner must not route, persist intent, probe or queue inspection"),
        )
        .unwrap();
        let mut input = serde_json::to_vec(&envelope).unwrap();
        input.push(b'\n');
        let mut stream = TransportScenario::new(input);
        serve_stream(&core, &mut stream).unwrap();
        let reply: serde_json::Value =
            serde_json::from_slice(&stream.trace.borrow().output).unwrap();
        // Domain routing would return Storage from this unopened core.
        assert_eq!(reply["kind"], "transport");
        assert_eq!(reply["error"]["code"], "owner_changed");
        assert_eq!(reply["error"]["delivery"], "rejected");
    }
}

#[test]
fn targeted_client_rejects_replacement_kind_and_target_without_another_exchange() {
    let client = SupervisorClient::new("/unused-private-state".into());
    let id = crate::workspaces::ActionId(uuid::Uuid::from_u128(1));
    for response in [
        Response::Repository {
            owner: "owner-b".into(),
            result: Ok(RepositoryReply::Action { registration: None }),
        },
        Response::Workspace {
            owner: "owner-a".into(),
            result: Ok(crate::workspaces::WorkspaceReply::Action {
                action: None,
                workspace: None,
            }),
        },
        Response::Repository {
            owner: "owner-a".into(),
            result: Ok(RepositoryReply::Action {
                registration: Some(crate::workspaces::Registration {
                    action_id: crate::workspaces::ActionId(uuid::Uuid::from_u128(2)),
                    path: "/one".into(),
                    create_initial_workspace: false,
                    initial_workspace_id: None,
                    state: crate::workspaces::RegistrationState::Pending,
                }),
            }),
        },
    ] {
        let failure = client
            .repository_request_with(
                "owner-a",
                RepositoryRequest::Action { action_id: id },
                |_, request| {
                    assert_eq!(request.expected_owner.as_deref(), Some("owner-a"));
                    Ok(response)
                },
            )
            .unwrap_err();
        let RequestFailure::Transport { error } = failure else {
            panic!("must preserve uncertain delivery")
        };
        assert_eq!(error.code, TransportCode::ReplyUnavailable);
        assert_eq!(error.delivery, Delivery::Unknown);
    }
}

#[test]
fn application_corpus_matches_pure_rust_serialization() {
    use crate::workspaces::*;
    use serde_json::{Value, json};
    let corpus: Value =
        serde_json::from_str(include_str!("../../../../contracts/observations-v7.json")).unwrap();
    let owner = "owner-a";
    let repository_id = RepositoryId(uuid::Uuid::from_u128(1));
    let action_id = ActionId(uuid::Uuid::from_u128(2));
    let workspace_id = WorkspaceId(uuid::Uuid::from_u128(3));
    let comparison_id = ComparisonId(uuid::Uuid::from_u128(4));
    let file_id = FileId(uuid::Uuid::from_u128(5));
    let core = Core {
        owner: owner.into(),
        pid: 41,
        workspaces: None,
        journal: Arc::new(crate::observations::Journal::default()),
    };
    assert_eq!(corpus["protocol"], PROTOCOL_VERSION);
    assert_eq!(
        corpus["snapshot"],
        serde_json::to_value(core.snapshot(None)).unwrap()
    );
    assert_eq!(corpus["desktop_pid"], 42);
    let repository_case =
        |request: RepositoryRequest,
         result: std::result::Result<RepositoryReply, RepositoryError>| {
            json!({
                "request": request, "reply": OwnedReply { owner: owner.into(), result }
            })
        };
    let register = RegisterRepository {
        action_id,
        path: "/projects/one".into(),
        create_initial_workspace: false,
    };
    let repositories = vec![
        repository_case(
            RepositoryRequest::List {
                after: None,
                refresh: false,
            },
            Ok(RepositoryReply::Repositories {
                page: RepositoryPage {
                    repositories: vec![Repository {
                        id: repository_id,
                        name: "one".into(),
                        path: "/projects/one".into(),
                        availability: Availability::Available,
                    }],
                    next: None,
                },
            }),
        ),
        repository_case(
            RepositoryRequest::Register {
                request: register.clone(),
            },
            Ok(RepositoryReply::Registration {
                registration: Registration {
                    action_id,
                    path: register.path,
                    create_initial_workspace: false,
                    initial_workspace_id: None,
                    state: RegistrationState::Completed { repository_id },
                },
            }),
        ),
        repository_case(
            RepositoryRequest::Action { action_id },
            Ok(RepositoryReply::Action { registration: None }),
        ),
        repository_case(
            RepositoryRequest::Get {
                id: repository_id,
                refresh: false,
            },
            Err(RepositoryError::new(
                ErrorCode::Storage,
                "Storage unavailable",
            )),
        ),
    ];
    assert_eq!(corpus["repositories"], json!(repositories));
    let workspace = Workspace {
        id: workspace_id,
        repository_id,
        creation_action_id: action_id,
        name: None,
        branch: None,
        path: None,
        legacy: false,
        provider: "codex".into(),
        base: None,
        artifacts: Artifacts {
            branch: BranchArtifact::Unknown,
            folder: FolderArtifact::Unknown,
        },
        status: CreationStatus::Queued,
        cleanup_unconfirmed: false,
    };
    let workspace_case = |request: WorkspaceRequest, reply: WorkspaceReply| {
        json!({
            "request": request, "reply": OwnedReply { owner: owner.into(), result: Ok(reply) }
        })
    };
    let target = PreviewTarget {
        workspace_id,
        comparison_id,
        file_id,
    };
    let workspaces = vec![
        workspace_case(
            WorkspaceRequest::Create {
                action_id,
                repository_id,
            },
            WorkspaceReply::Action {
                action: Some(WorkspaceAction::Create {
                    action_id,
                    repository_id,
                }),
                workspace: Some(workspace.clone()),
            },
        ),
        workspace_case(
            WorkspaceRequest::List {
                repository_id,
                after: None,
            },
            WorkspaceReply::Workspaces {
                page: WorkspacePage {
                    workspaces: vec![workspace],
                    next: None,
                },
            },
        ),
        workspace_case(
            WorkspaceRequest::Changes {
                workspace_id,
                refresh: false,
                comparison_id: None,
                after: None,
            },
            WorkspaceReply::Changes {
                observation: ChangesObservation {
                    workspace_id,
                    phase: InspectionPhase::Ready,
                    error: None,
                    page: Some(ChangePage {
                        comparison_id,
                        head: "a".repeat(40),
                        files: vec![],
                        total: 0,
                        next: None,
                    }),
                },
            },
        ),
        workspace_case(
            WorkspaceRequest::ChangePreview {
                target: target.clone(),
                load: true,
            },
            WorkspaceReply::ChangePreview {
                observation: PreviewObservation {
                    target,
                    phase: InspectionPhase::Ready,
                    error: None,
                    preview: Some(FilePreview {
                        comparison_id,
                        head: "a".repeat(40),
                        rows: vec![DiffRow {
                            kind: RowKind::Added,
                            old_line: None,
                            new_line: Some(1),
                            text: "hello".into(),
                        }],
                        limited: None,
                        unsupported: None,
                        note: None,
                    }),
                },
            },
        ),
    ];
    assert_eq!(corpus["workspaces"], json!(workspaces));
    let mut failures = vec![RequestFailure::Rejected {
        error: RepositoryError::new(ErrorCode::Capacity, "Local input limit"),
    }];
    for (code, delivery, message) in [
        (TransportCode::Busy, Delivery::NotSent, "Busy"),
        (
            TransportCode::NotConnected,
            Delivery::NotSent,
            "Not connected",
        ),
        (
            TransportCode::OwnerChanged,
            Delivery::Rejected,
            "Owner changed",
        ),
        (
            TransportCode::ReplyUnavailable,
            Delivery::Unknown,
            "Reply unavailable",
        ),
        (
            TransportCode::Incompatible,
            Delivery::Rejected,
            "Incompatible versions",
        ),
    ] {
        failures.push(RequestFailure::transport(code, delivery, message));
    }
    assert_eq!(corpus["failures"], json!(failures));
}

#[test]
fn targeted_exchange_reports_delivery_at_the_physical_io_boundary() {
    let request = Request {
        protocol: PROTOCOL_VERSION,
        application_version: env!("CARGO_PKG_VERSION").into(),
        after: None,
        expected_owner: Some("owner-a".into()),
        repository: Some(RepositoryRequest::List {
            after: None,
            refresh: false,
        }),
        workspace: None,
        subscribe: None,
    };
    for (fault, expected) in [
        (Some(TransportFault::ReadTimeout), Delivery::NotSent),
        (Some(TransportFault::WriteTimeout), Delivery::NotSent),
        (Some(TransportFault::WriteAfter(8)), Delivery::Unknown),
        (Some(TransportFault::Read), Delivery::Unknown),
        (None, Delivery::Unknown), // EOF after a complete request was sent.
    ] {
        let mut scenario = ClientScenario::new(vec![]);
        scenario.stream.as_mut().unwrap().fault = fault;
        let error =
            exchange_target(Path::new("/private/state"), &request, &mut scenario).unwrap_err();
        let RequestFailure::Transport { error } = error else {
            panic!("expected transport evidence")
        };
        assert_eq!(error.delivery, expected);
        let trace = scenario.trace.borrow();
        assert_eq!(trace.drops, 1);
        if expected == Delivery::NotSent {
            assert_eq!(trace.writes, 0);
        } else {
            assert!(trace.writes > 0);
        }
        assert_eq!(scenario.calls.len(), 3); // one connect; no discovery or replay.
    }
    let mut scenario = ClientScenario::new(vec![]);
    scenario.connect_error = Some(io::ErrorKind::ConnectionRefused);
    let RequestFailure::Transport { error } =
        exchange_target(Path::new("/private/state"), &request, &mut scenario).unwrap_err()
    else {
        panic!("expected transport evidence")
    };
    assert_eq!(error.code, TransportCode::NotConnected);
    assert_eq!(error.delivery, Delivery::NotSent);
    assert_eq!(scenario.trace.borrow().writes, 0);
}

#[test]
fn invalid_snapshot_cannot_replace_the_connections_observed_owner() {
    let first = snapshot();
    for invalid in [
        SupervisorSnapshot {
            cursor: SnapshotCursor {
                owner: "".into(),
                revision: 1,
            },
            ..first.clone()
        },
        SupervisorSnapshot {
            cursor: SnapshotCursor {
                owner: "owner-b".into(),
                revision: u64::MAX,
            },
            ..first.clone()
        },
        SupervisorSnapshot {
            status: SupervisorStatus {
                supervisor_pid: 0,
                ..first.status.clone()
            },
            ..first.clone()
        },
    ] {
        let mut scenario =
            DiscoveryScenario::new(vec![Ok(first.clone()), Ok(first.clone()), Ok(invalid)]);
        let connection =
            SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        connection.snapshot_with(&mut scenario).unwrap();
        assert_eq!(
            connection.snapshot_with(&mut scenario).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(
            connection.observed.lock().unwrap().cursor,
            Some(first.cursor.clone())
        );
    }
}

#[test]
fn subscription_owner_gate_and_target_exclusivity_precede_domain_dispatch() {
    let targeted = |owner: &str| Request {
        protocol: PROTOCOL_VERSION,
        application_version: env!("CARGO_PKG_VERSION").into(),
        after: None,
        expected_owner: Some(owner.into()),
        repository: None,
        workspace: None,
        subscribe: Some(subscription::Subscribe {
            subscription_id: "feed-1".into(),
            after: None,
        }),
    };
    let response = dispatch_request(targeted("old-owner"), "owner-a", |_| {
        panic!("wrong owner must not dispatch")
    })
    .unwrap();
    assert!(matches!(
        response,
        Response::Transport {
            error: TransportFailure {
                code: TransportCode::OwnerChanged,
                ..
            }
        }
    ));
    let mut mixed = targeted("owner-a");
    mixed.repository = Some(RepositoryRequest::List {
        after: None,
        refresh: false,
    });
    assert!(
        dispatch_request(mixed, "owner-a", |_| panic!(
            "mixed request must not dispatch"
        ))
        .is_err()
    );
}
#[test]
fn persistent_framing_preserves_buffered_frames_and_separates_idle_from_transfer_deadlines() {
    let stream = TransportScenario::new(b"1\n2\n".to_vec());
    let trace = stream.trace.clone();
    let mut reader = BufReader::new(stream);
    assert_eq!(
        read_buffered_frame::<u64, _>(&mut reader, Duration::from_secs(5)).unwrap(),
        1
    );
    assert_eq!(
        read_buffered_frame::<u64, _>(&mut reader, Duration::from_secs(5)).unwrap(),
        2
    );
    assert_eq!(trace.borrow().reads, 1);
    let mut stream = TransportScenario::new(b"{}\n".to_vec());
    stream.chunk_size = 1;
    let mut reader = BufReader::new(stream);
    let start = Instant::now();
    let mut times = VecDeque::from([
        start,
        start,
        start + Duration::from_secs(4),
        start + Duration::from_secs(5),
        start + Duration::from_secs(7),
    ]);
    let error = read_buffered_frame_with_clock::<serde_json::Value, _>(
        &mut reader,
        Duration::from_secs(15),
        &mut || times.pop_front().unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.kind(), io::ErrorKind::TimedOut);
    assert_eq!(
        *reader.get_ref().read_waits.borrow(),
        [Duration::from_secs(15), Duration::from_secs(1)]
    );
}
