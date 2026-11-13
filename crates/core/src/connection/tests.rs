use super::*;
use std::{cell::RefCell, collections::VecDeque, io::Cursor, rc::Rc};

fn snapshot() -> SupervisorSnapshot {
    Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
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
    let mut connection =
        SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
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
    let mut desktop = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    desktop.snapshot_with(&mut scenario).unwrap();
    drop(desktop);
    assert_eq!(scenario.calls, ["prepare", "snapshot", "snapshot"]);
    let mut reopened = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
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
    let mut desktop = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    desktop.snapshot_with(&mut scenario).unwrap();
    assert!(desktop.snapshot_with(&mut scenario).is_err());
    assert!(!scenario.calls.contains(&"start"));
    let mut reopened = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
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

struct TransportScenario {
    input: Cursor<Vec<u8>>,
    chunk_size: usize,
    timeouts: RefCell<Vec<Duration>>,
    write_chunk_size: usize,
    fault: Option<TransportFault>,
    trace: Rc<RefCell<TransportTrace>>,
}

#[derive(Clone, Copy)]
enum TransportFault {
    ReadTimeout,
    WriteTimeout,
    WriteAfter(usize),
    Read,
}

#[derive(Default)]
struct TransportTrace {
    output: Vec<u8>,
    reads: usize,
    writes: usize,
    drops: usize,
}

impl TransportScenario {
    fn new(input: Vec<u8>) -> Self {
        Self {
            input: Cursor::new(input),
            chunk_size: usize::MAX,
            timeouts: RefCell::new(vec![]),
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
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        if matches!(self.fault, Some(TransportFault::ReadTimeout)) {
            return Err(io::ErrorKind::Unsupported.into());
        }
        self.timeouts.borrow_mut().push(timeout.unwrap());
        Ok(())
    }
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        assert_eq!(timeout, Some(IO_TIMEOUT));
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
        application_version: version.into(),
        after,
        repository: None,
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
fn repository_protocol_returns_typed_storage_failure_without_claiming_empty_navigation() {
    let core = Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
    };
    let mut input = serde_json::to_vec(&Request {
        protocol: PROTOCOL_VERSION,
        application_version: env!("CARGO_PKG_VERSION").into(),
        after: None,
        repository: Some(RepositoryRequest::List {
            after: None,
            refresh: false,
        }),
    })
    .unwrap();
    input.push(b'\n');
    let mut stream = TransportScenario::new(input);
    serve_stream(&core, &mut stream).unwrap();
    let Response::Repository { result: Err(error) } =
        serde_json::from_slice(&stream.trace.borrow().output).unwrap()
    else {
        panic!("storage failure must be an observation error")
    };
    assert_eq!(error.code, ErrorCode::Storage);
    assert!(error.message.contains("execution owner"));
}

#[test]
fn oversized_incomplete_and_malformed_frames_are_rejected_without_output() {
    let core = Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
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
        *stream.timeouts.borrow(),
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
    trace: Rc<RefCell<TransportTrace>>,
    calls: Vec<ClientCall>,
    time: Instant,
    times: VecDeque<Instant>,
}

impl ClientScenario {
    fn new(input: Vec<u8>) -> Self {
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
