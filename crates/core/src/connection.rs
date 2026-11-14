//! Versioned, bounded observations; transport loss never replays actions.
use crate::workspaces::{ErrorCode, RepositoryError, RepositoryReply, RepositoryRequest};
use crate::{
    Core, SnapshotCursor, SupervisorSnapshot, SupervisorStatus,
    supervisor::{check_private, owner_present, prepare_state, socket_path},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    os::{
        fd::AsRawFd,
        unix::{net::UnixStream, process::CommandExt},
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
mod admission;
pub use admission::{Admitted, ConnectionSlots};

const MAX_FRAME_BYTES: usize = 64 * 1024;
const IO_TIMEOUT: Duration = Duration::from_secs(2);
pub const PROTOCOL_VERSION: u32 = 6;

/// Evidence about this exchange only, never the outcome of an earlier attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    NotSent,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportCode {
    Busy,
    NotConnected,
    OwnerChanged,
    ReplyUnavailable,
    Incompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportFailure {
    pub code: TransportCode,
    pub delivery: Delivery,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RequestFailure {
    Transport {
        error: TransportFailure,
    },
    /// Local input rejection, before any transport access.
    Rejected {
        error: RepositoryError,
    },
}
impl RequestFailure {
    pub fn transport(code: TransportCode, delivery: Delivery, message: impl Into<String>) -> Self {
        Self::Transport {
            error: transport_failure(code, delivery, message),
        }
    }
}
fn transport_failure(
    code: TransportCode,
    delivery: Delivery,
    message: impl Into<String>,
) -> TransportFailure {
    TransportFailure {
        code,
        delivery,
        message: RepositoryError::new(ErrorCode::Connection, message).message,
    }
}

/// Serving identity is returned even when the domain rejected the request.
#[derive(Debug, Serialize, Deserialize)]
pub struct OwnedReply<T> {
    pub owner: String,
    pub result: Result<T, RepositoryError>,
}

fn unavailable(message: impl std::fmt::Display) -> RequestFailure {
    RequestFailure::transport(
        TransportCode::ReplyUnavailable,
        Delivery::Unknown,
        format!(
            "Supervisor reply unavailable: {message}. Refresh recorded action status; no action has been replayed."
        ),
    )
}

#[derive(Serialize, Deserialize)]
struct Request {
    protocol: u32,
    application_version: String,
    after: Option<SnapshotCursor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expected_owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    repository: Option<RepositoryRequest>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    workspace: Option<crate::workspaces::WorkspaceRequest>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Response {
    Transport {
        error: TransportFailure,
    },
    Workspace {
        owner: String,
        result: Result<crate::workspaces::WorkspaceReply, RepositoryError>,
    },
    Snapshot {
        snapshot: SupervisorSnapshot,
    },
    Incompatible {
        message: String,
    },
    Repository {
        owner: String,
        result: Result<RepositoryReply, RepositoryError>,
    },
}

/// Immutable exchange configuration is independent of short observation-state locks.
#[derive(Clone)]
pub struct SupervisorConnection {
    state: PathBuf,
    executable: PathBuf,
    observed: Arc<Mutex<ConnectionState>>,
    observations: ConnectionSlots,
    intents: ConnectionSlots,
    status: ConnectionSlots,
    // F04 owns the subscription resource; its budget cannot borrow observation slots.
    _subscription: ConnectionSlots,
}
#[derive(Default)]
struct ConnectionState {
    initialized: bool,
    cursor: Option<SnapshotCursor>,
}
impl SupervisorConnection {
    pub fn new(state: PathBuf, executable: PathBuf) -> Self {
        Self {
            state,
            executable,
            observed: Arc::new(Mutex::new(ConnectionState::default())),
            observations: ConnectionSlots::new(2),
            intents: ConnectionSlots::new(1),
            status: ConnectionSlots::new(1),
            _subscription: ConnectionSlots::new(1),
        }
    }

    /// Admission happens on the calling thread, before blocking work is spawned.
    pub fn prepare_repository_request(
        &self,
        expected_owner: String,
        request: RepositoryRequest,
    ) -> Result<
        Admitted<
            impl FnOnce() -> Result<OwnedReply<RepositoryReply>, RequestFailure> + Send + 'static,
        >,
        RequestFailure,
    > {
        self.prepare_repository_request_with(expected_owner, request, |state, request| {
            exchange_target(state, request, &mut LocalClient)
        })
    }

    fn prepare_repository_request_with<F>(
        &self,
        expected_owner: String,
        request: RepositoryRequest,
        exchange: F,
    ) -> Result<
        Admitted<impl FnOnce() -> Result<OwnedReply<RepositoryReply>, RequestFailure> + use<F>>,
        RequestFailure,
    >
    where
        F: FnOnce(&Path, &Request) -> Result<Response, RequestFailure>,
    {
        let lane = if matches!(request, RepositoryRequest::Register { .. }) {
            &self.intents
        } else {
            &self.observations
        };
        let permit = Self::admit(lane)?;
        self.require_connection(&expected_owner)?;
        let client = SupervisorClient::new(self.state.clone());
        Ok(Admitted {
            permit,
            work: move || client.repository_request_with(&expected_owner, request, exchange),
        })
    }

    pub fn prepare_workspace_request(
        &self,
        expected_owner: String,
        request: crate::workspaces::WorkspaceRequest,
    ) -> Result<
        Admitted<
            impl FnOnce() -> Result<OwnedReply<crate::workspaces::WorkspaceReply>, RequestFailure>
            + Send
            + 'static,
        >,
        RequestFailure,
    > {
        use crate::workspaces::WorkspaceRequest;
        let lane = if matches!(
            request,
            WorkspaceRequest::Create { .. } | WorkspaceRequest::Reconcile { .. }
        ) {
            &self.intents
        } else {
            &self.observations
        };
        let permit = Self::admit(lane)?;
        self.require_connection(&expected_owner)?;
        let client = SupervisorClient::new(self.state.clone());
        Ok(Admitted {
            permit,
            work: move || client.workspace_request(&expected_owner, request),
        })
    }

    pub fn prepare_snapshot(
        &self,
    ) -> Result<
        Admitted<impl FnOnce() -> io::Result<SupervisorSnapshot> + Send + 'static>,
        RequestFailure,
    > {
        let permit = Self::admit(&self.status)?;
        let connection = self.clone();
        Ok(Admitted {
            permit,
            work: move || connection.snapshot_admitted_with(&mut LocalDiscovery::new()),
        })
    }

    fn admit(lane: &ConnectionSlots) -> Result<admission::ConnectionPermit, RequestFailure> {
        lane.try_acquire().ok_or_else(|| RequestFailure::transport(
            TransportCode::Busy, Delivery::NotSent,
            "Desktop connection lane is busy. Wait for physical requests to finish before retrying explicitly.",
        ))
    }

    pub fn repository_request(
        &self,
        expected_owner: &str,
        request: RepositoryRequest,
    ) -> Result<OwnedReply<RepositoryReply>, RequestFailure> {
        self.prepare_repository_request(expected_owner.into(), request)?
            .run()
    }
    pub fn workspace_request(
        &self,
        expected_owner: &str,
        request: crate::workspaces::WorkspaceRequest,
    ) -> Result<OwnedReply<crate::workspaces::WorkspaceReply>, RequestFailure> {
        self.prepare_workspace_request(expected_owner.into(), request)?
            .run()
    }

    fn require_connection(&self, expected_owner: &str) -> Result<(), RequestFailure> {
        let observed = self.observed.lock().map_err(|e| {
            RequestFailure::transport(
                TransportCode::NotConnected,
                Delivery::NotSent,
                e.to_string(),
            )
        })?;
        if !observed.initialized || observed.cursor.is_none() {
            return Err(RequestFailure::transport(
                TransportCode::NotConnected,
                Delivery::NotSent,
                "Connect to the supervisor before submitting targeted requests.",
            ));
        }
        if observed
            .cursor
            .as_ref()
            .is_some_and(|cursor| cursor.owner != expected_owner)
        {
            return Err(RequestFailure::transport(
                TransportCode::OwnerChanged,
                Delivery::Rejected,
                "The observed supervisor owner has changed. Refresh status before continuing.",
            ));
        }
        Ok(())
    }

    pub fn snapshot(&self) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with(&mut LocalDiscovery::new())
    }
    fn snapshot_with(&self, effects: &mut impl Discovery) -> io::Result<SupervisorSnapshot> {
        let _permit = self
            .status
            .try_acquire()
            .ok_or_else(|| io::Error::new(io::ErrorKind::WouldBlock, "Status lane is busy"))?;
        self.snapshot_admitted_with(effects)
    }
    fn snapshot_admitted_with(
        &self,
        effects: &mut impl Discovery,
    ) -> io::Result<SupervisorSnapshot> {
        let (initialized, cursor) = {
            let observed = self
                .observed
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?;
            (observed.initialized, observed.cursor.clone())
        };
        if !initialized {
            discover(&self.state, &self.executable, effects)?;
            // Retain discovery success even if the subsequent status exchange fails.
            self.observed
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?
                .initialized = true;
        }
        let snapshot = effects.snapshot(&self.state, cursor)
            .map_err(|e| io::Error::new(e.kind(), format!(
                "Cannot read supervisor observations: {e}. Refresh to reconnect to this owner. If it has exited, reopen the desktop; no actions have been replayed.")))?;
        validate_snapshot(&snapshot)?;
        self.observed
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?
            .cursor = Some(snapshot.cursor.clone());
        Ok(snapshot)
    }
}

pub struct SupervisorClient {
    state: PathBuf,
}
impl SupervisorClient {
    pub fn workspace_request(
        &self,
        expected_owner: &str,
        request: crate::workspaces::WorkspaceRequest,
    ) -> Result<OwnedReply<crate::workspaces::WorkspaceReply>, RequestFailure> {
        self.workspace_request_with(expected_owner, request, |state, request| {
            exchange_target(state, request, &mut LocalClient)
        })
    }
    fn workspace_request_with(
        &self,
        expected_owner: &str,
        request: crate::workspaces::WorkspaceRequest,
        exchange: impl FnOnce(&Path, &Request) -> Result<Response, RequestFailure>,
    ) -> Result<OwnedReply<crate::workspaces::WorkspaceReply>, RequestFailure> {
        let response = exchange(
            &self.state,
            &Request {
                protocol: PROTOCOL_VERSION,
                application_version: env!("CARGO_PKG_VERSION").into(),
                after: None,
                expected_owner: Some(expected_owner.into()),
                repository: None,
                workspace: Some(request.clone()),
            },
        )?;
        match response {
            Response::Workspace { owner, result } => {
                if result
                    .as_ref()
                    .is_ok_and(|reply| !workspace_target(&request, reply))
                {
                    return Err(unavailable("Workspace reply has the wrong kind or target"));
                }
                owned_reply(expected_owner, owner, result)
            }
            other => Err(response_failure(other)),
        }
    }
    pub fn repository_request(
        &self,
        expected_owner: &str,
        request: RepositoryRequest,
    ) -> Result<OwnedReply<RepositoryReply>, RequestFailure> {
        self.repository_request_with(expected_owner, request, |state, request| {
            exchange_target(state, request, &mut LocalClient)
        })
    }
    fn repository_request_with(
        &self,
        expected_owner: &str,
        request: RepositoryRequest,
        exchange: impl FnOnce(&Path, &Request) -> Result<Response, RequestFailure>,
    ) -> Result<OwnedReply<RepositoryReply>, RequestFailure> {
        if let RepositoryRequest::Register { request } = &request {
            crate::workspaces::check_registration_path_budget(&request.path)
                .map_err(|error| RequestFailure::Rejected { error })?;
        }
        let response = exchange(
            &self.state,
            &Request {
                protocol: PROTOCOL_VERSION,
                application_version: env!("CARGO_PKG_VERSION").into(),
                after: None,
                expected_owner: Some(expected_owner.into()),
                repository: Some(request.clone()),
                workspace: None,
            },
        )?;
        match response {
            Response::Repository { owner, result } => {
                if result
                    .as_ref()
                    .is_ok_and(|reply| !repository_target(&request, reply))
                {
                    return Err(unavailable("Repository reply has the wrong kind or target"));
                }
                owned_reply(expected_owner, owner, result)
            }
            other => Err(response_failure(other)),
        }
    }
    pub fn new(state: PathBuf) -> Self {
        Self { state }
    }

    /// Opening the desktop may start an absent owner. Subsequent observations never start one.
    pub fn connect_or_start(state: PathBuf, executable: &Path) -> io::Result<Self> {
        discover(&state, executable, &mut LocalDiscovery::new())?;
        Ok(Self::new(state))
    }

    pub fn status(&self) -> io::Result<SupervisorStatus> {
        Ok(self.snapshot(None)?.status)
    }
    pub fn snapshot(&self, after: Option<SnapshotCursor>) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with_version(after, PROTOCOL_VERSION, env!("CARGO_PKG_VERSION"))
    }
    pub fn snapshot_with_version(
        &self,
        after: Option<SnapshotCursor>,
        protocol: u32,
        application_version: &str,
    ) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with(after, protocol, application_version, &mut LocalClient)
    }

    fn snapshot_with(
        &self,
        after: Option<SnapshotCursor>,
        protocol: u32,
        application_version: &str,
        effects: &mut impl ClientEffects,
    ) -> io::Result<SupervisorSnapshot> {
        effects.check_private(&self.state, true)?;
        let socket = socket_path(&self.state);
        effects.check_private(&socket, false)?;
        let mut stream = effects.connect(&socket)?;
        configure(&stream)?;
        write_frame(
            &mut stream,
            &Request {
                protocol,
                application_version: application_version.into(),
                after,
                expected_owner: None,
                repository: None,
                workspace: None,
            },
        )?;
        match read_frame_with_clock(&mut stream, || effects.now())? {
            Response::Snapshot { snapshot } => {
                validate_snapshot(&snapshot)?;
                Ok(snapshot)
            }
            Response::Incompatible { message } => {
                Err(io::Error::new(io::ErrorKind::InvalidData, message))
            }
            Response::Transport { .. }
            | Response::Workspace { .. }
            | Response::Repository { .. } => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unexpected repository reply to status request",
            )),
        }
    }
}

fn validate_snapshot(snapshot: &SupervisorSnapshot) -> io::Result<()> {
    if snapshot.status.supervisor_pid == 0
        || snapshot.status.application_version != env!("CARGO_PKG_VERSION")
        || snapshot.cursor.owner.is_empty()
        || snapshot.cursor.owner.len() > 128
        || snapshot
            .cursor
            .owner
            .chars()
            .any(|c| c.is_whitespace() || c.is_control())
        || snapshot.cursor.revision > 9_007_199_254_740_991
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid supervisor snapshot identity or numeric bounds",
        ));
    }
    Ok(())
}

fn repository_target(request: &RepositoryRequest, reply: &RepositoryReply) -> bool {
    match (request, reply) {
        (
            RepositoryRequest::Register { request },
            RepositoryReply::Registration { registration },
        ) => {
            registration.action_id == request.action_id
                && registration.path == request.path
                && registration.create_initial_workspace == request.create_initial_workspace
        }
        (RepositoryRequest::Action { action_id }, RepositoryReply::Action { registration }) => {
            registration
                .as_ref()
                .is_none_or(|registration| registration.action_id == *action_id)
        }
        (RepositoryRequest::Get { id, .. }, RepositoryReply::Repository { repository }) => {
            repository
                .as_ref()
                .is_none_or(|repository| repository.id == *id)
        }
        (RepositoryRequest::List { .. }, RepositoryReply::Repositories { page }) => {
            page.repositories.len() <= 32
        }
        _ => false,
    }
}
fn workspace_target(
    request: &crate::workspaces::WorkspaceRequest,
    reply: &crate::workspaces::WorkspaceReply,
) -> bool {
    use crate::workspaces::{WorkspaceAction as A, WorkspaceReply as R, WorkspaceRequest as Q};
    match (request, reply) {
        (Q::Get { id }, R::Workspace { workspace }) => {
            workspace.as_ref().is_none_or(|w| w.id == *id)
        }
        (Q::List { repository_id, .. }, R::Workspaces { page }) => {
            page.workspaces.len() <= 32
                && page
                    .workspaces
                    .iter()
                    .all(|w| w.repository_id == *repository_id)
        }
        (
            Q::Changes {
                workspace_id,
                comparison_id,
                ..
            },
            R::Changes { observation },
        ) => {
            observation.workspace_id == *workspace_id
                && comparison_id.is_none_or(|id| {
                    observation
                        .page
                        .as_ref()
                        .is_some_and(|page| page.comparison_id == id)
                })
        }
        (Q::ChangePreview { target, .. }, R::ChangePreview { observation }) => {
            observation.target == *target
        }
        (
            Q::Action { .. },
            R::Action {
                action: None,
                workspace: None,
            },
        ) => true,
        (
            Q::Action { action_id } | Q::Create { action_id, .. } | Q::Reconcile { action_id, .. },
            R::Action {
                action: Some(action),
                workspace: Some(w),
            },
        ) => {
            let identity = match action {
                A::Create {
                    action_id: id,
                    repository_id,
                } => {
                    id == action_id
                        && *repository_id == w.repository_id
                        && w.creation_action_id == *id
                }
                A::Reconcile {
                    action_id: id,
                    workspace_id,
                } => id == action_id && *workspace_id == w.id,
            };
            identity
                && match (request, action) {
                    (Q::Action { .. }, _) => true,
                    (
                        Q::Create {
                            repository_id: expected,
                            ..
                        },
                        A::Create { repository_id, .. },
                    ) => expected == repository_id,
                    (
                        Q::Reconcile {
                            workspace_id: expected,
                            ..
                        },
                        A::Reconcile { workspace_id, .. },
                    ) => expected == workspace_id,
                    _ => false,
                }
        }
        _ => false,
    }
}

fn owned_reply<T>(
    expected: &str,
    owner: String,
    result: Result<T, RepositoryError>,
) -> Result<OwnedReply<T>, RequestFailure> {
    if owner != expected {
        return Err(unavailable("Reply came from a different supervisor owner"));
    }
    Ok(OwnedReply { owner, result })
}
fn response_failure(response: Response) -> RequestFailure {
    match response {
        Response::Transport { error } => match (error.code, error.delivery) {
            (TransportCode::OwnerChanged | TransportCode::Incompatible, Delivery::Rejected)
            | (TransportCode::Busy, Delivery::NotSent) => RequestFailure::Transport { error },
            _ => unavailable("Invalid transport evidence from supervisor"),
        },
        Response::Incompatible { message } => {
            RequestFailure::transport(TransportCode::Incompatible, Delivery::Rejected, message)
        }
        _ => unavailable("Unexpected reply kind"),
    }
}
fn exchange_target(
    state: &Path,
    request: &Request,
    effects: &mut impl ClientEffects,
) -> Result<Response, RequestFailure> {
    let frame = encode_frame(request).map_err(|e| RequestFailure::Rejected {
        error: RepositoryError::new(
            ErrorCode::Capacity,
            format!("Invalid targeted request: {e}"),
        ),
    })?;
    let connect = |error: io::Error| {
        RequestFailure::transport(
            TransportCode::NotConnected,
            Delivery::NotSent,
            format!(
                "Cannot connect to the observed supervisor: {error}. Refresh status; no action was sent."
            ),
        )
    };
    effects.check_private(state, true).map_err(connect)?;
    let socket = socket_path(state);
    effects.check_private(&socket, false).map_err(connect)?;
    let mut stream = effects.connect(&socket).map_err(connect)?;
    configure(&stream).map_err(connect)?;
    stream.write_all(&frame).map_err(unavailable)?;
    read_frame_with_clock(&mut stream, || effects.now()).map_err(unavailable)
}

// Client policy and protocol stay above this seam; adapters supply only local I/O outcomes.
trait ClientEffects {
    type Stream: FrameStream;
    fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()>;
    fn connect(&mut self, socket: &Path) -> io::Result<Self::Stream>;
    fn now(&mut self) -> Instant;
}

struct LocalClient;
impl ClientEffects for LocalClient {
    type Stream = UnixStream;
    fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        check_private(path, directory)
    }
    fn connect(&mut self, socket: &Path) -> io::Result<Self::Stream> {
        UnixStream::connect(socket)
    }
    fn now(&mut self) -> Instant {
        Instant::now()
    }
}

// Discovery policy remains here; the local implementation supplies OS outcomes.
trait Discovery {
    fn prepare(&mut self, state: &Path) -> io::Result<()>;
    fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot>;
    fn owner_present(&mut self, state: &Path) -> io::Result<bool>;
    fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()>;
    fn elapsed(&self) -> Duration;
    fn wait(&mut self, duration: Duration);
}

struct LocalDiscovery(Instant);
impl LocalDiscovery {
    fn new() -> Self {
        Self(Instant::now())
    }
}
impl Discovery for LocalDiscovery {
    fn prepare(&mut self, state: &Path) -> io::Result<()> {
        prepare_state(state)
    }
    fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot> {
        SupervisorClient::new(state.to_owned()).snapshot(after)
    }
    fn owner_present(&mut self, state: &Path) -> io::Result<bool> {
        owner_present(state)
    }
    fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()> {
        let mut child = supervisor_command(executable, state).spawn().map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Cannot start supervisor {}: {e}. Rebuild or reinstall the application.",
                    executable.display()
                ),
            )
        })?;
        // Reap on every exit path without tying the supervisor lifetime to this client.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
    fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
    fn wait(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

fn supervisor_command(executable: &Path, state: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .arg(state)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command
}

fn discover(state: &Path, executable: &Path, effects: &mut impl Discovery) -> io::Result<()> {
    effects.prepare(state)?;
    match effects.snapshot(state, None) {
        Ok(_) => return Ok(()),
        Err(e)
            if !matches!(
                e.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Err(e);
        }
        Err(_) => {}
    }
    if !effects.owner_present(state)? {
        effects.start(executable, state)?;
    }
    let deadline = effects.elapsed() + Duration::from_secs(5);
    loop {
        match effects.snapshot(state, None) {
            Ok(_) => {
                return Ok(());
            }
            Err(e)
                if !matches!(
                    e.kind(),
                    io::ErrorKind::NotFound
                        | io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::UnexpectedEof
                ) =>
            {
                return Err(e);
            }
            Err(_) => {}
        }
        if effects.elapsed() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Supervisor did not become available. An owner may still hold this state; check the installed supervisor and state permissions, then reopen. Do not remove the owner lock.",
            ));
        }
        effects.wait(Duration::from_millis(25));
    }
}

pub fn serve_connection(core: &Core, mut stream: UnixStream) -> io::Result<()> {
    serve_stream(core, &mut stream)
}

fn serve_stream(core: &Core, stream: &mut impl FrameStream) -> io::Result<()> {
    configure(stream)?;
    let request: Request = read_frame(stream)?;
    let response = dispatch_request(request, &core.owner, |request| {
        if let Some(request) = request.workspace {
            Response::Workspace {
                owner: core.owner.clone(),
                result: core.workspace_request(request),
            }
        } else if let Some(request) = request.repository {
            Response::Repository {
                owner: core.owner.clone(),
                result: core.repository_request(request),
            }
        } else {
            Response::Snapshot {
                snapshot: core.snapshot(request.after),
            }
        }
    })?;
    write_frame(stream, &response)
}

// The gate owns authorization; a controlled route records whether domain dispatch occurred.
fn dispatch_request(
    request: Request,
    owner: &str,
    route: impl FnOnce(Request) -> Response,
) -> io::Result<Response> {
    if request.protocol != PROTOCOL_VERSION
        || request.application_version != env!("CARGO_PKG_VERSION")
    {
        return Ok(Response::Incompatible {
            message: format!(
                "Incompatible desktop/supervisor versions (client protocol {} / app {}, supervisor protocol {} / app {}). Install matching desktop and supervisor builds; close the old owner through its matching desktop before reopening.",
                request.protocol,
                request.application_version,
                PROTOCOL_VERSION,
                env!("CARGO_PKG_VERSION")
            ),
        });
    }
    if request.workspace.is_some() && request.repository.is_some() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Only one targeted request is permitted",
        ));
    }
    let targeted = request.workspace.is_some() || request.repository.is_some();
    if (targeted && request.after.is_some()) || (!targeted && request.expected_owner.is_some()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid status/target context",
        ));
    }
    if targeted && request.expected_owner.as_deref() != Some(owner) {
        return Ok(Response::Transport {
            error: transport_failure(
                TransportCode::OwnerChanged,
                Delivery::Rejected,
                "The supervisor owner differs from the caller observation. Refresh status; this attempt was rejected before domain dispatch.",
            ),
        });
    }
    Ok(route(request))
}

trait FrameStream: Read + Write {
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
    fn wait_readable(&self, timeout: Duration) -> io::Result<()>;
}
impl FrameStream for UnixStream {
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        UnixStream::set_read_timeout(self, timeout)
    }
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        UnixStream::set_write_timeout(self, timeout)
    }
    fn wait_readable(&self, timeout: Duration) -> io::Result<()> {
        // Changing SO_RCVTIMEO after peer close fails on macOS even while a
        // complete reply remains buffered. Poll preserves the frame deadline
        // without changing socket options between fragments.
        let milliseconds = i32::try_from(
            timeout.as_millis() + u128::from(!timeout.subsec_nanos().is_multiple_of(1_000_000)),
        )
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Frame timeout exceeds poll budget",
            )
        })?;
        let mut descriptor = libc::pollfd {
            fd: self.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        // SAFETY: descriptor is initialized, points to one live pollfd, and
        // the borrowed UnixStream owns its fd throughout this bounded call.
        match unsafe { libc::poll(&mut descriptor, 1, milliseconds) } {
            -1 => Err(io::Error::last_os_error()),
            0 => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Supervisor frame deadline exceeded",
            )),
            _ => Ok(()), // Read buffered bytes or obtain the socket's EOF/error.
        }
    }
}

fn configure(stream: &impl FrameStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))
}

fn read_frame<T: DeserializeOwned>(stream: &mut impl FrameStream) -> io::Result<T> {
    read_frame_with_clock(stream, Instant::now)
}

fn read_frame_with_clock<T: DeserializeOwned>(
    stream: &mut impl FrameStream,
    mut now: impl FnMut() -> Instant,
) -> io::Result<T> {
    let deadline = now() + IO_TIMEOUT;
    let mut reader = BufReader::new(stream);
    let mut frame = Vec::new();
    // read_until alone would allow an unbounded peer-controlled allocation.
    loop {
        let remaining = deadline
            .checked_duration_since(now())
            .filter(|remaining| !remaining.is_zero())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Supervisor frame deadline exceeded",
                )
            })?;
        reader.get_ref().wait_readable(remaining)?;
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "incomplete supervisor frame",
            ));
        }
        let length = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |end| end + 1);
        if frame.len() + length > MAX_FRAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "supervisor frame exceeds limit",
            ));
        }
        let complete = available[length - 1] == b'\n';
        frame.extend_from_slice(&available[..length]);
        reader.consume(length);
        if complete {
            return serde_json::from_slice(&frame).map_err(io::Error::other);
        }
    }
}

fn encode_frame(value: &impl Serialize) -> io::Result<Vec<u8>> {
    let mut frame = serde_json::to_vec(value)?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "supervisor frame exceeds limit",
        ));
    }
    Ok(frame)
}

fn write_frame(stream: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    stream.write_all(&encode_frame(value)?)
}

#[cfg(test)]
mod tests;
