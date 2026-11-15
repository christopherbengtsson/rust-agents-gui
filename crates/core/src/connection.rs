//! Versioned, bounded observations; transport loss never replays actions.
use crate::workspaces::{ErrorCode, RepositoryError, RepositoryReply, RepositoryRequest};
use crate::{
    Core, SnapshotCursor, SupervisorSnapshot, SupervisorStatus,
    supervisor::{check_private, owner_present, prepare_state, socket_path},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    future::Future,
    io,
    os::unix::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};
mod admission;
mod framed_exchange;
mod subscription;
mod tasks;
pub use admission::{Admitted, ConnectionSlots};
use framed_exchange::FramedExchange;
pub use subscription::{FeedAck, FeedBatch, FeedEvent};
use tasks::Tasks;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::UnixStream,
    time::Instant,
};
use tokio_util::sync::CancellationToken;

const IO_TIMEOUT: Duration = Duration::from_secs(2);
pub const PROTOCOL_VERSION: u32 = 9;

/// Evidence about this exchange only, never the outcome of an earlier attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Delivery {
    NotSent,
    Rejected,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum TransportCode {
    Busy,
    NotConnected,
    OwnerChanged,
    ReplyUnavailable,
    Incompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct TransportFailure {
    pub code: TransportCode,
    pub delivery: Delivery,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
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
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    subscribe: Option<subscription::Subscribe>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Response {
    Feed {
        batch: FeedBatch,
    },
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
    subscription: ConnectionSlots,
    feed: Arc<subscription::FeedOwner>,
    tasks: Arc<Tasks>,
    blocking: ConnectionSlots,
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
            subscription: ConnectionSlots::new(1),
            feed: Arc::new(subscription::FeedOwner::default()),
            tasks: Arc::new(Tasks::default()),
            blocking: ConnectionSlots::new(5),
        }
    }
    fn client(&self) -> SupervisorClient {
        SupervisorClient {
            state: self.state.clone(),
            tasks: self.tasks.clone(),
            blocking: self.blocking.clone(),
        }
    }
    /// Nonwaiting admission occurs before task creation; run transfers to the completion owner.
    pub fn prepare_repository_request(
        &self,
        expected_owner: String,
        request: RepositoryRequest,
    ) -> Result<Admitted<OwnedReply<RepositoryReply>>, RequestFailure> {
        let lane = if matches!(request, RepositoryRequest::Register { .. }) {
            &self.intents
        } else {
            &self.observations
        };
        let permit = Self::admit(lane)?;
        self.require_connection(&expected_owner)?;
        let client = self.client();
        Ok(Admitted::new(permit, self.tasks.clone(), async move {
            client.repository_request(&expected_owner, request).await
        }))
    }
    pub fn prepare_workspace_request(
        &self,
        expected_owner: String,
        request: crate::workspaces::WorkspaceRequest,
    ) -> Result<Admitted<OwnedReply<crate::workspaces::WorkspaceReply>>, RequestFailure> {
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
        let client = self.client();
        Ok(Admitted::new(permit, self.tasks.clone(), async move {
            client.workspace_request(&expected_owner, request).await
        }))
    }
    pub fn prepare_snapshot(&self) -> Result<Admitted<SupervisorSnapshot>, RequestFailure> {
        let permit = Self::admit(&self.status)?;
        let connection = self.clone();
        Ok(Admitted::new(permit, self.tasks.clone(), async move {
            let mut effects = LocalDiscovery::new(connection.client());
            connection
                .snapshot_admitted_with(&mut effects)
                .await
                .map_err(|e| {
                    RequestFailure::transport(
                        TransportCode::NotConnected,
                        Delivery::NotSent,
                        e.to_string(),
                    )
                })
        }))
    }
    /// Only desktop-owned resources are stopped; supervisor-owned domain work continues.
    pub async fn stop_observations(&self) -> Result<(), RequestFailure> {
        self.tasks.cancel.cancel();
        let feed = self.feed.stop_all().await;
        let requests = self.tasks.close_and_wait().await.map_err(unavailable);
        feed.and(requests)
    }
    fn admit(lane: &ConnectionSlots) -> Result<admission::ConnectionPermit, RequestFailure> {
        lane.try_acquire().ok_or_else(|| RequestFailure::transport(
            TransportCode::Busy, Delivery::NotSent,
            "Desktop connection lane is busy. Wait for physical requests to finish before retrying explicitly.",
        ))
    }

    pub async fn repository_request(
        &self,
        expected_owner: &str,
        request: RepositoryRequest,
    ) -> Result<OwnedReply<RepositoryReply>, RequestFailure> {
        self.prepare_repository_request(expected_owner.into(), request)?
            .run()
            .await
    }
    pub async fn workspace_request(
        &self,
        expected_owner: &str,
        request: crate::workspaces::WorkspaceRequest,
    ) -> Result<OwnedReply<crate::workspaces::WorkspaceReply>, RequestFailure> {
        self.prepare_workspace_request(expected_owner.into(), request)?
            .run()
            .await
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

    pub async fn snapshot(&self) -> Result<SupervisorSnapshot, RequestFailure> {
        self.prepare_snapshot()?.run().await
    }
    #[cfg(test)]
    async fn snapshot_with(&self, effects: &mut impl Discovery) -> io::Result<SupervisorSnapshot> {
        let _permit = self
            .status
            .try_acquire()
            .ok_or_else(|| io::Error::new(io::ErrorKind::WouldBlock, "Status lane is busy"))?;
        self.snapshot_admitted_with(effects).await
    }
    async fn snapshot_admitted_with(
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
            discover(&self.state, &self.executable, effects).await?;
            // Retain discovery success even if the subsequent status exchange fails.
            self.observed
                .lock()
                .map_err(|e| io::Error::other(e.to_string()))?
                .initialized = true;
        }
        let snapshot = effects.snapshot(&self.state, cursor).await
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
    tasks: Arc<Tasks>,
    blocking: ConnectionSlots,
}
impl SupervisorClient {
    pub async fn workspace_request(
        &self,
        expected_owner: &str,
        request: crate::workspaces::WorkspaceRequest,
    ) -> Result<OwnedReply<crate::workspaces::WorkspaceReply>, RequestFailure> {
        self.workspace_request_with(expected_owner, request, async |state, request| {
            exchange_target(state, request, &mut self.effects()).await
        })
        .await
    }
    async fn workspace_request_with(
        &self,
        expected_owner: &str,
        request: crate::workspaces::WorkspaceRequest,
        exchange: impl AsyncFnOnce(&Path, &Request) -> Result<Response, RequestFailure>,
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
                subscribe: None,
            },
        )
        .await?;
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
    pub async fn repository_request(
        &self,
        expected_owner: &str,
        request: RepositoryRequest,
    ) -> Result<OwnedReply<RepositoryReply>, RequestFailure> {
        self.repository_request_with(expected_owner, request, async |state, request| {
            exchange_target(state, request, &mut self.effects()).await
        })
        .await
    }
    async fn repository_request_with(
        &self,
        expected_owner: &str,
        request: RepositoryRequest,
        exchange: impl AsyncFnOnce(&Path, &Request) -> Result<Response, RequestFailure>,
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
                subscribe: None,
            },
        )
        .await?;
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
        Self {
            state,
            tasks: Arc::new(Tasks::default()),
            blocking: ConnectionSlots::new(5),
        }
    }

    /// Opening the desktop may start an absent owner. Subsequent observations never start one.
    pub async fn connect_or_start(state: PathBuf, executable: &Path) -> io::Result<Self> {
        let client = Self::new(state);
        discover(
            &client.state,
            executable,
            &mut LocalDiscovery::new(Self::new(client.state.clone())),
        )
        .await?;
        Ok(client)
    }
    fn effects(&self) -> LocalClient {
        LocalClient {
            tasks: self.tasks.clone(),
            blocking: self.blocking.clone(),
        }
    }

    pub async fn status(&self) -> io::Result<SupervisorStatus> {
        Ok(self.snapshot(None).await?.status)
    }
    pub async fn snapshot(&self, after: Option<SnapshotCursor>) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with_version(after, PROTOCOL_VERSION, env!("CARGO_PKG_VERSION"))
            .await
    }
    pub async fn snapshot_with_version(
        &self,
        after: Option<SnapshotCursor>,
        protocol: u32,
        application_version: &str,
    ) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with(after, protocol, application_version, &mut self.effects())
            .await
    }

    async fn snapshot_with(
        &self,
        after: Option<SnapshotCursor>,
        protocol: u32,
        application_version: &str,
        effects: &mut impl ClientEffects,
    ) -> io::Result<SupervisorSnapshot> {
        let cancel = self.tasks.cancel.clone();
        let request = Request {
            protocol,
            subscribe: None,
            application_version: application_version.into(),
            after,
            expected_owner: None,
            repository: None,
            workspace: None,
        };
        let payload = encode_payload(&request)?;
        let stream = open_stream(&self.state, effects, &cancel).await?;
        let mut exchange = FramedExchange::new(stream);
        exchange
            .write_frame(payload, Instant::now() + IO_TIMEOUT, &cancel)
            .await?;
        match read_json(&mut exchange, Instant::now() + IO_TIMEOUT, &cancel).await? {
            Response::Snapshot { snapshot } => {
                validate_snapshot(&snapshot)?;
                Ok(snapshot)
            }
            Response::Incompatible { message } => {
                Err(io::Error::new(io::ErrorKind::InvalidData, message))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unexpected reply to status request",
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
async fn exchange_target(
    state: &Path,
    request: &Request,
    effects: &mut impl ClientEffects,
) -> Result<Response, RequestFailure> {
    let payload = encode_payload(request).map_err(|e| RequestFailure::Rejected {
        error: RepositoryError::new(
            ErrorCode::Capacity,
            format!("Invalid targeted request: {e}"),
        ),
    })?;
    let cancel = effects.cancellation();
    let stream = open_stream(state, effects, &cancel).await.map_err(|error| RequestFailure::transport(
        TransportCode::NotConnected, Delivery::NotSent, format!("Cannot connect to the observed supervisor: {error}. Refresh status; no action was sent.")))?;
    let mut exchange = FramedExchange::new(stream);
    // Once write starts, partial transmission is uncertain, including timeout/cancellation.
    exchange
        .write_frame(payload, Instant::now() + IO_TIMEOUT, &cancel)
        .await
        .map_err(unavailable)?;
    read_json(&mut exchange, Instant::now() + IO_TIMEOUT, &cancel)
        .await
        .map_err(unavailable)
}
trait ClientEffects {
    type Stream: AsyncRead + AsyncWrite + Unpin + 'static;
    async fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()>;
    async fn connect(&mut self, socket: &Path) -> io::Result<Self::Stream>;
    fn cancellation(&self) -> CancellationToken;
}
struct LocalClient {
    tasks: Arc<Tasks>,
    blocking: ConnectionSlots,
}
impl ClientEffects for LocalClient {
    type Stream = UnixStream;
    async fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        let path = path.to_owned();
        self.tasks
            .blocking(&self.blocking, move || check_private(&path, directory))
            .await
    }
    async fn connect(&mut self, socket: &Path) -> io::Result<Self::Stream> {
        tokio::time::timeout(IO_TIMEOUT, UnixStream::connect(socket))
            .await
            .map_err(|_| io::Error::from(io::ErrorKind::TimedOut))?
    }
    fn cancellation(&self) -> CancellationToken {
        self.tasks.cancel.clone()
    }
}
async fn open_stream(
    state: &Path,
    effects: &mut impl ClientEffects,
    cancel: &CancellationToken,
) -> io::Result<impl AsyncRead + AsyncWrite + Unpin + 'static> {
    // Blocking checks always finish under their owner, even during cancellation.
    effects.check_private(state, true).await?;
    let socket = socket_path(state);
    effects.check_private(&socket, false).await?;
    cancel
        .run_until_cancelled(effects.connect(&socket))
        .await
        .ok_or_else(|| io::Error::from(io::ErrorKind::Interrupted))?
}

// Discovery policy remains here; the local implementation supplies OS outcomes.
trait Discovery {
    async fn prepare(&mut self, state: &Path) -> io::Result<()>;
    async fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot>;
    async fn owner_present(&mut self, state: &Path) -> io::Result<bool>;
    async fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()>;
    fn elapsed(&self) -> Duration;
    async fn wait(&mut self, duration: Duration);
}

struct LocalDiscovery {
    started: Instant,
    client: SupervisorClient,
}
impl LocalDiscovery {
    fn new(client: SupervisorClient) -> Self {
        Self {
            started: Instant::now(),
            client,
        }
    }
}
impl Discovery for LocalDiscovery {
    async fn prepare(&mut self, state: &Path) -> io::Result<()> {
        let state = state.to_owned();
        self.client
            .tasks
            .blocking(&self.client.blocking, move || prepare_state(&state))
            .await
    }
    async fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot> {
        debug_assert_eq!(state, self.client.state);
        self.client.snapshot(after).await
    }
    async fn owner_present(&mut self, state: &Path) -> io::Result<bool> {
        let state = state.to_owned();
        self.client
            .tasks
            .blocking(&self.client.blocking, move || owner_present(&state))
            .await
    }
    async fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()> {
        let executable = executable.to_owned();
        let state = state.to_owned();
        self.client.tasks.blocking(&self.client.blocking, move || {
            let mut child = supervisor_command(&executable, &state).spawn().map_err(|e| io::Error::new(e.kind(), format!("Cannot start supervisor {}: {e}. Rebuild or reinstall the application.", executable.display())))?;
            // Intentional process reaper: this independent supervisor outlives the desktop.
            std::thread::spawn(move || { let _ = child.wait(); });
            Ok(())
        }).await
    }
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }
    async fn wait(&mut self, duration: Duration) {
        let _ = self
            .client
            .tasks
            .cancel
            .run_until_cancelled(tokio::time::sleep(duration))
            .await;
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

async fn discover(state: &Path, executable: &Path, effects: &mut impl Discovery) -> io::Result<()> {
    effects.prepare(state).await?;
    match effects.snapshot(state, None).await {
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
    if !effects.owner_present(state).await? {
        effects.start(executable, state).await?;
    }
    let deadline = effects.elapsed() + Duration::from_secs(5);
    loop {
        match effects.snapshot(state, None).await {
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
        effects.wait(Duration::from_millis(25)).await;
    }
}

/// Per-server completion owner and bounded blocking dispatch, independent of Tauri.
pub struct ConnectionServer {
    core: Arc<Core>,
    tasks: Arc<Tasks>,
    blocking: ConnectionSlots,
}
impl ConnectionServer {
    pub fn new(core: Arc<Core>) -> Self {
        Self {
            core,
            tasks: Arc::new(Tasks::default()),
            blocking: ConnectionSlots::new(8),
        }
    }
    pub async fn serve(&self, stream: UnixStream, cancel: &CancellationToken) -> io::Result<()> {
        let core = self.core.clone();
        serve_stream(
            stream,
            &core.owner.clone(),
            core.journal.clone(),
            cancel,
            async |request| {
                self.tasks
                    .blocking(&self.blocking, move || Ok(route_request(&core, request)))
                    .await
            },
        )
        .await
    }
    pub async fn finish(&self) -> io::Result<()> {
        self.tasks.close_and_wait().await
    }
}
fn route_request(core: &Core, request: Request) -> Response {
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
}
async fn serve_stream<S: AsyncRead + AsyncWrite + Unpin + 'static>(
    stream: S,
    owner: &str,
    journal: Arc<crate::observations::Journal>,
    cancel: &CancellationToken,
    route: impl AsyncFnOnce(Request) -> io::Result<Response>,
) -> io::Result<()> {
    let mut exchange = FramedExchange::new(stream);
    let request: Request = read_json(&mut exchange, Instant::now() + IO_TIMEOUT, cancel).await?;
    if let Some(rejection) = reject_request(&request, owner)? {
        return write_json(
            &mut exchange,
            &rejection,
            Instant::now() + IO_TIMEOUT,
            cancel,
        )
        .await;
    }
    let mut request = request;
    if let Some(subscribe) = request.subscribe.take() {
        return subscription::serve(owner, &journal, subscribe, &mut exchange, cancel).await;
    }
    // No cancellation race drops a started blocking dispatch; its owner joins it.
    let response = route(request).await?;
    write_json(
        &mut exchange,
        &response,
        Instant::now() + IO_TIMEOUT,
        cancel,
    )
    .await
}

// The gate owns authorization; a controlled route records whether domain dispatch occurred.
fn reject_request(request: &Request, owner: &str) -> io::Result<Option<Response>> {
    if request.protocol != PROTOCOL_VERSION
        || request.application_version != env!("CARGO_PKG_VERSION")
    {
        return Ok(Some(Response::Incompatible {
            message: format!(
                "Incompatible desktop/supervisor versions (client protocol {} / app {}, supervisor protocol {} / app {}). Install matching desktop and supervisor builds; close the old owner through its matching desktop before reopening.",
                request.protocol,
                request.application_version,
                PROTOCOL_VERSION,
                env!("CARGO_PKG_VERSION")
            ),
        }));
    }
    if usize::from(request.workspace.is_some())
        + usize::from(request.repository.is_some())
        + usize::from(request.subscribe.is_some())
        > 1
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Only one targeted request is permitted",
        ));
    }
    let targeted =
        request.workspace.is_some() || request.repository.is_some() || request.subscribe.is_some();
    if (targeted && request.after.is_some()) || (!targeted && request.expected_owner.is_some()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid status/target context",
        ));
    }
    if targeted && request.expected_owner.as_deref() != Some(owner) {
        return Ok(Some(Response::Transport {
            error: transport_failure(
                TransportCode::OwnerChanged,
                Delivery::Rejected,
                "The supervisor owner differs from the caller observation. Refresh status; this attempt was rejected before domain dispatch.",
            ),
        }));
    }
    Ok(None)
}

fn encode_payload(value: &impl Serialize) -> io::Result<bytes::Bytes> {
    let payload = serde_json::to_vec(value)?;
    if payload.len() > framed_exchange::MAX_PAYLOAD {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "supervisor frame exceeds limit",
        ));
    }
    Ok(payload.into())
}
async fn read_json<T: DeserializeOwned, S: AsyncRead + AsyncWrite + Unpin + 'static>(
    exchange: &mut FramedExchange<S>,
    deadline: Instant,
    cancel: &CancellationToken,
) -> io::Result<T> {
    let payload = exchange
        .read_frame(deadline, cancel)
        .await.map_err(|error| {
            if error.kind() == io::ErrorKind::InvalidData {
                io::Error::new(error.kind(), format!("Incompatible or malformed supervisor framing: {error}. Install matching desktop and supervisor builds."))
            } else { error }
        })?
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::UnexpectedEof, "Supervisor connection closed")
        })?;
    serde_json::from_slice(&payload).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}
async fn write_json<S: AsyncRead + AsyncWrite + Unpin + 'static>(
    exchange: &mut FramedExchange<S>,
    value: &impl Serialize,
    deadline: Instant,
    cancel: &CancellationToken,
) -> io::Result<()> {
    exchange
        .write_frame(encode_payload(value)?, deadline, cancel)
        .await
}

#[cfg(test)]
mod tests;

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<Delivery>();
    exports.add::<TransportCode>();
    exports.add::<TransportFailure>();
    exports.add::<RequestFailure>();
    exports.owned_replies();
    subscription::export_contracts(exports);
}
