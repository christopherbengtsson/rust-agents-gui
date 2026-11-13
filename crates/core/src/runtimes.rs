//! Application-wide Codex setup. Only this owner admits and reconciles setup effects.
mod process;
mod protocol;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{
        Arc, Mutex,
        mpsc::{self, SyncSender},
    },
    thread::JoinHandle,
};
use uuid::Uuid;

pub const SUPPORTED_VERSION: &str = "0.167.2";
pub const MAX_EXECUTABLE_BYTES: usize = 4096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidRequest,
    StaleTarget,
    Busy,
    Storage,
    PersistenceUncertain,
    Connection,
    MissingExecutable,
    IncompatibleVersion,
    Protocol,
    Authentication,
    Configuration,
    Timeout,
    OutputLimit,
    Process,
    CleanupUnconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeError {
    pub code: ErrorCode,
    pub message: String,
}
impl RuntimeError {
    pub(crate) fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
    pub(crate) fn storage() -> Self {
        Self::new(
            ErrorCode::Storage,
            "Cannot read or commit runtime preferences. Refresh saved status; preserve application state.",
        )
    }
    pub(crate) fn cleanup() -> Self {
        Self::new(
            ErrorCode::CleanupUnconfirmed,
            "Owned Codex cleanup is unconfirmed. Further setup work is blocked. Close lingering runtime work externally; if ownership remains uncertain, restart the Mac and refresh. No saved PID will be signaled.",
        )
    }
}
impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for RuntimeError {}
pub(crate) type Result<T> = std::result::Result<T, RuntimeError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Runtime {
    Codex,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub owner: String,
    pub runtime: Runtime,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Configuration {
    pub revision: u64,
    pub executable: Option<String>,
}
impl Default for Configuration {
    fn default() -> Self {
        Self {
            revision: 1,
            executable: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum RuntimeRequest {
    Observe {
        target: Target,
    },
    Action {
        target: Target,
        action_id: Uuid,
    },
    Save {
        target: Target,
        action_id: Uuid,
        revision: u64,
        executable: Option<String>,
    },
    Check {
        target: Target,
        action_id: Uuid,
        revision: u64,
    },
}
impl RuntimeRequest {
    fn target(&self) -> &Target {
        match self {
            Self::Observe { target }
            | Self::Action { target, .. }
            | Self::Save { target, .. }
            | Self::Check { target, .. } => target,
        }
    }
    fn id(&self) -> Option<Uuid> {
        match self {
            Self::Observe { .. } => None,
            Self::Action { action_id, .. }
            | Self::Save { action_id, .. }
            | Self::Check { action_id, .. } => Some(*action_id),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Pending,
    Saving,
    Discovering,
    Version,
    Initializing,
    Account,
    Configuration,
    Cleaning,
    Completed,
    Failed,
    Uncertain,
}
impl Phase {
    pub fn terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Uncertain)
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub action_id: Uuid,
    pub request: RuntimeRequest,
    pub phase: Phase,
    pub error: Option<RuntimeError>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub check_id: Uuid,
    pub revision: u64,
    pub timestamp_ms: u64,
    pub executable: Option<String>,
    pub version: Option<String>,
    pub authentication: String,
    pub provider: Option<String>,
    pub billing: String,
    pub passed: bool,
    pub error: Option<RuntimeError>,
    pub cleanup_confirmed: bool,
    pub exit_successful: Option<bool>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLookup {
    pub action_id: Uuid,
    pub complete: bool,
    pub action: Option<Action>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReply {
    pub target: Target,
    pub configuration: Option<Configuration>,
    pub busy: bool,
    pub active: Option<Action>,
    pub lookup: Option<ActionLookup>,
    pub diagnostic: Option<Diagnostic>,
    pub blocked: Option<RuntimeError>,
}

pub struct Runtimes {
    shared: Arc<Mutex<State>>,
    sender: Option<SyncSender<RuntimeRequest>>,
    worker: Option<JoinHandle<()>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
}
struct State {
    reply: RuntimeReply,
    actions: std::collections::VecDeque<Action>,
}

pub(crate) fn validate_preference(value: Option<&str>) -> Result<()> {
    if value.is_some_and(|p| {
        p.len() > MAX_EXECUTABLE_BYTES
            || !Path::new(p).is_absolute()
            || p.contains(['\0', '\n', '\r'])
    }) {
        return Err(RuntimeError::new(
            ErrorCode::InvalidRequest,
            "Use an absolute Codex executable path of at most 4096 bytes, or reset to automatic discovery.",
        ));
    }
    Ok(())
}
impl State {
    fn ready(target: Target, configuration: Configuration) -> Self {
        Self {
            reply: RuntimeReply {
                target,
                configuration: Some(configuration),
                busy: false,
                active: None,
                lookup: None,
                diagnostic: None,
                blocked: None,
            },
            actions: Default::default(),
        }
    }
    fn remember(&mut self, action: Action) {
        self.actions.retain(|a| a.action_id != action.action_id);
        if self.actions.len() == 32 {
            self.actions.pop_front();
        }
        self.actions.push_back(action);
    }
    fn admit(&mut self, request: &RuntimeRequest) -> Result<bool> {
        if request.target() != &self.reply.target {
            return Err(RuntimeError::new(
                ErrorCode::StaleTarget,
                "The supervisor owner changed. Refresh saved runtime status before another action.",
            ));
        }
        let reconcile_storage = self.reply.blocked.as_ref().is_some_and(|e| {
            matches!(e.code, ErrorCode::Storage | ErrorCode::PersistenceUncertain)
        });
        if matches!(request, RuntimeRequest::Observe { .. }) {
            if reconcile_storage && !self.reply.busy {
                self.reply.busy = true;
                return Ok(true);
            }
            return Ok(false);
        }
        let id = request
            .id()
            .expect("non-observation request has an action ID");
        if let Some(action) = self.actions.iter().find(|a| a.action_id == id)
            && !(reconcile_storage && matches!(request, RuntimeRequest::Action { .. }))
        {
            if !matches!(request, RuntimeRequest::Action { .. }) && action.request != *request {
                return Err(RuntimeError::new(
                    ErrorCode::InvalidRequest,
                    "This action ID already belongs to another runtime request.",
                ));
            }
            self.reply.lookup = Some(ActionLookup {
                action_id: id,
                complete: true,
                action: Some(action.clone()),
            });
            return Ok(false);
        }
        if matches!(request, RuntimeRequest::Action { .. }) {
            if !reconcile_storage
                && self
                    .reply
                    .lookup
                    .as_ref()
                    .is_some_and(|l| l.action_id == id && l.complete)
            {
                return Ok(false);
            }
            if self.reply.busy {
                return Ok(false);
            }
            self.reply.lookup = Some(ActionLookup {
                action_id: id,
                complete: false,
                action: None,
            });
        } else {
            if let RuntimeRequest::Save { executable, .. } = request {
                validate_preference(executable.as_deref())?;
            }
            if self.reply.busy {
                // Status is unresolved, and no second job is admitted. In
                // particular an evicted repeated identity is not freshly rejected.
                self.reply.lookup = Some(ActionLookup {
                    action_id: id,
                    complete: false,
                    action: None,
                });
                return Ok(false);
            }
            // The worker resolves durable identity before fresh admission rules.
            // Configuration/cleanup fences may reject new work, never hide an
            // already admitted action's recorded result.
            let action = Action {
                action_id: id,
                request: request.clone(),
                phase: Phase::Pending,
                error: None,
            };
            self.remember(action.clone());
            self.reply.active = Some(action);
            self.reply.lookup = None;
        }
        self.reply.busy = true;
        Ok(true)
    }
}

pub(crate) trait RuntimeStore {
    fn configuration(&mut self) -> Result<Configuration>;
    fn action(&mut self, id: Uuid) -> Result<Option<Action>>;
    fn blocked(&mut self) -> Result<bool>;
    fn intent(&mut self, action: &Action, boot: &str) -> Result<()>;
    fn save(&mut self, action: &Action, configuration: &Configuration) -> Result<()>;
    fn finish(&mut self, action: &Action, cleanup_confirmed: bool) -> Result<()>;
}

impl Runtimes {
    pub fn open(directory: &Path, owner: String) -> Self {
        let mut state = State::ready(
            Target {
                owner,
                runtime: Runtime::Codex,
            },
            Configuration::default(),
        );
        state.reply.configuration = None;
        state.reply.busy = true;
        let shared = Arc::new(Mutex::new(state));
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker_state = shared.clone();
        let directory = directory.to_owned();
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let worker_stop = stop.clone();
        let worker = std::thread::spawn(move || {
            let boot = process::boot_identity();
            let mut store = match crate::storage::runtime::open(&directory, &boot) {
                Ok(store) => store,
                Err(error) => {
                    update(&worker_state, |s| {
                        s.reply.busy = false;
                        s.reply.blocked = Some(error);
                    });
                    return;
                }
            };
            refresh_store(&worker_state, &mut store);
            let mut runner = process::Runner::new(directory, worker_stop.clone());
            let mut disconnected = false;
            let mut pending_cleanup: Option<Action> = None;
            loop {
                if !disconnected {
                    match receiver.recv_timeout(std::time::Duration::from_millis(50)) {
                        Ok(request) => execute(
                            &worker_state,
                            &mut store,
                            request,
                            &boot,
                            |configuration, id, progress| runner.check(configuration, id, progress),
                        ),
                        Err(mpsc::RecvTimeoutError::Disconnected) => disconnected = true,
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                    }
                }
                if !runner.cleanup_confirmed() && runner.reconcile_cleanup() {
                    pending_cleanup = worker_state
                        .lock()
                        .ok()
                        .and_then(|s| s.reply.active.clone());
                    if let Some(action) = &mut pending_cleanup {
                        action.phase = Phase::Failed;
                        action.error = Some(RuntimeError::new(
                            ErrorCode::Process,
                            "The previous setup check stopped; owned cleanup is now confirmed. Run a new explicit check when ready.",
                        ));
                    }
                }
                if let Some(action) = &pending_cleanup
                    && store.finish(action, true).is_ok()
                {
                    publish_action(&worker_state, action);
                    update(&worker_state, |s| {
                        if let Some(d) = &mut s.reply.diagnostic {
                            d.cleanup_confirmed = true;
                            d.error = action.error.clone();
                        }
                    });
                    refresh_store(&worker_state, &mut store);
                    pending_cleanup = None;
                }
                if disconnected {
                    if runner.cleanup_confirmed() && pending_cleanup.is_none() {
                        break;
                    }
                    // Shutdown can return uncertainty while this worker continues
                    // owning/reconciling its handles; it never abandons them.
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        });
        Self {
            shared,
            sender: Some(sender),
            worker: Some(worker),
            stop,
        }
    }
    pub fn request(&self, request: RuntimeRequest) -> Result<RuntimeReply> {
        let lookup_id = request.id();
        let mut state = self.shared.lock().map_err(|_| RuntimeError::storage())?;
        if state.admit(&request)?
            && self
                .sender
                .as_ref()
                .is_none_or(|s| s.try_send(request).is_err())
        {
            state.reply.busy = false;
            state.reply.blocked = Some(RuntimeError::new(
                ErrorCode::Process,
                "Runtime worker is unavailable. Preserve state and restart the application.",
            ));
        }
        let mut reply = state.reply.clone();
        reply.lookup = lookup_id.map(|id| {
            state
                .reply
                .lookup
                .clone()
                .filter(|l| l.action_id == id)
                .unwrap_or(ActionLookup {
                    action_id: id,
                    complete: false,
                    action: None,
                })
        });
        Ok(reply)
    }
    /// Call off presentation paths. A failure leaves this owner and its handles intact.
    pub fn shutdown(&mut self) -> Result<()> {
        use std::sync::atomic::Ordering;
        self.stop.store(true, Ordering::SeqCst);
        self.sender.take();
        await_worker(&mut WorkerCompletion {
            worker: &mut self.worker,
            start: std::time::Instant::now(),
        })?;
        let state = self.shared.lock().map_err(|_| RuntimeError::cleanup())?;
        if let Some(error) = &state.reply.blocked {
            return Err(error.clone());
        }
        Ok(())
    }
}

trait Completion {
    fn finished(&self) -> bool;
    fn elapsed(&self) -> std::time::Duration;
    fn tick(&mut self);
    fn join(&mut self) -> Result<()>;
}
fn await_worker(completion: &mut impl Completion) -> Result<()> {
    while !completion.finished() {
        if completion.elapsed() >= std::time::Duration::from_secs(4) {
            return Err(RuntimeError::cleanup());
        }
        completion.tick();
    }
    completion.join()
}
struct WorkerCompletion<'a> {
    worker: &'a mut Option<JoinHandle<()>>,
    start: std::time::Instant,
}
impl Completion for WorkerCompletion<'_> {
    fn finished(&self) -> bool {
        self.worker.as_ref().is_none_or(JoinHandle::is_finished)
    }
    fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
    fn tick(&mut self) {
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    fn join(&mut self) -> Result<()> {
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| RuntimeError::cleanup())?;
        }
        Ok(())
    }
}

fn update(shared: &Mutex<State>, change: impl FnOnce(&mut State)) {
    if let Ok(mut state) = shared.lock() {
        change(&mut state);
    }
}
fn refresh_store(shared: &Mutex<State>, store: &mut dyn RuntimeStore) {
    let result = store
        .configuration()
        .and_then(|c| store.blocked().map(|b| (c, b)));
    update(shared, |s| {
        s.reply.busy = false;
        match result {
            Ok((c, b)) => {
                s.reply.configuration = Some(c);
                s.reply.blocked = b.then(RuntimeError::cleanup);
            }
            Err(error) => {
                s.reply.blocked = Some(error);
            }
        }
    });
}

fn execute(
    shared: &Mutex<State>,
    store: &mut dyn RuntimeStore,
    request: RuntimeRequest,
    boot: &str,
    check: impl FnOnce(&Configuration, Uuid, &mut dyn FnMut(Phase)) -> Diagnostic,
) {
    if matches!(request, RuntimeRequest::Observe { .. }) {
        refresh_store(shared, store);
        return;
    }
    let id = request.id().expect("worker receives targeted actions only");
    if matches!(request, RuntimeRequest::Action { .. }) {
        let result = store.action(id).map(|a| a.map(recovered_action));
        update(shared, |s| match result {
            Ok(action) => {
                if let Some(a) = &action {
                    s.remember(a.clone());
                }
                s.reply.lookup = Some(ActionLookup {
                    action_id: id,
                    complete: true,
                    action,
                });
            }
            Err(error) => {
                s.reply.blocked = Some(error);
            }
        });
        refresh_store(shared, store);
        return;
    }
    let mut action = Action {
        action_id: id,
        request: request.clone(),
        phase: Phase::Pending,
        error: None,
    };
    // A cache miss never grants replay: the worker checks durable identity first.
    let result = (|| {
        if let Some(old) = store.action(id)? {
            if old.request != request {
                return Err(RuntimeError::new(
                    ErrorCode::InvalidRequest,
                    "This action ID already belongs to another request.",
                ));
            }
            action = recovered_action(old);
            return Ok(());
        }
        if store.blocked()? {
            return Err(RuntimeError::cleanup());
        }
        let configuration = store.configuration()?;
        let revision = match &request {
            RuntimeRequest::Save { revision, .. } | RuntimeRequest::Check { revision, .. } => {
                *revision
            }
            _ => unreachable!(),
        };
        if configuration.revision != revision {
            return Err(RuntimeError::new(
                ErrorCode::StaleTarget,
                "The saved executable changed. Refresh saved status.",
            ));
        }
        store.intent(&action, boot)?;
        match &request {
            RuntimeRequest::Save { executable, .. } => {
                action.phase = Phase::Saving;
                publish_action(shared, &action);
                let revision = configuration
                    .revision
                    .checked_add(1)
                    .filter(|v| *v <= 9_007_199_254_740_991)
                    .ok_or_else(RuntimeError::storage)?;
                let next = Configuration {
                    revision,
                    executable: executable.clone(),
                };
                action.phase = Phase::Completed;
                store.save(&action, &next)?;
                update(shared, |s| {
                    s.reply.configuration = Some(next);
                    s.reply.diagnostic = None;
                });
            }
            RuntimeRequest::Check { .. } => {
                let mut progress = |phase| {
                    action.phase = phase;
                    publish_action(shared, &action);
                };
                let diagnostic = check(&configuration, id, &mut progress);
                action.phase = if !diagnostic.cleanup_confirmed {
                    Phase::Uncertain
                } else if diagnostic.passed {
                    Phase::Completed
                } else {
                    Phase::Failed
                };
                action.error = diagnostic.error.clone();
                // Failed final persistence leaves the pre-effect recovery record in place.
                let persisted = store.finish(&action, diagnostic.cleanup_confirmed);
                update(shared, |s| s.reply.diagnostic = Some(diagnostic));
                persisted?;
            }
            _ => unreachable!(),
        }
        Ok(())
    })();
    if let Err(error) = result {
        action.phase = if matches!(
            error.code,
            ErrorCode::PersistenceUncertain | ErrorCode::CleanupUnconfirmed
        ) {
            Phase::Uncertain
        } else {
            Phase::Failed
        };
        action.error = Some(error.clone());
        // Reopen/read on the storage side settles a lost commit, never a repeated write.
        if let Ok(Some(saved)) = store.action(id) {
            if saved.request != request { /* Never substitute another request's recorded outcome. */
            } else if saved.phase.terminal() {
                action = saved;
            } else if matches!(request, RuntimeRequest::Save { .. }) {
                // The atomic preference/result transaction did not commit. A
                // confirmed read of its Pending intent is not a crash diagnosis.
                action.phase = Phase::Failed;
                if error.code == ErrorCode::PersistenceUncertain {
                    action.error = Some(RuntimeError::new(
                        ErrorCode::Storage,
                        "Authoritative read confirmed the Save did not commit. Previous saved preferences remain; edit and save explicitly.",
                    ));
                }
                if let Err(failure) = store.finish(&action, true) {
                    action.phase = Phase::Uncertain;
                    action.error = Some(failure);
                }
            }
        }
        update(shared, |s| {
            s.reply.diagnostic = None;
            if matches!(
                error.code,
                ErrorCode::PersistenceUncertain | ErrorCode::CleanupUnconfirmed
            ) {
                s.reply.blocked = Some(error);
            }
        });
    }
    publish_action(shared, &action);
    refresh_store(shared, store);
}
fn publish_action(shared: &Mutex<State>, action: &Action) {
    update(shared, |s| {
        s.remember(action.clone());
        s.reply.active = Some(action.clone());
    });
}

fn recovered_action(mut action: Action) -> Action {
    if !action.phase.terminal() {
        action.phase = Phase::Uncertain;
        action.error = Some(RuntimeError::new(
            ErrorCode::PersistenceUncertain,
            "This setup action has no recorded final result. Saved configuration is authoritative; no action was replayed.",
        ));
    }
    action
}

// A failed explicit shutdown retains its worker on the Runtimes value. When the
// Core itself is destroyed, transfer that still-owned worker to module recovery
// ownership rather than detach it or perform an unbounded destructor join. The
// admitted supervisor has one runtime owner; completed retained workers are joined.
static RECOVERY_WORKERS: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());
impl Drop for Runtimes {
    fn drop(&mut self) {
        if self.shutdown().is_err()
            && let Some(worker) = self.worker.take()
        {
            let mut retained = RECOVERY_WORKERS
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut unfinished = Vec::new();
            for previous in retained.drain(..) {
                if previous.is_finished() {
                    let _ = previous.join();
                } else {
                    unfinished.push(previous);
                }
            }
            unfinished.push(worker);
            *retained = unfinished;
        }
    }
}
