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
            if let Some(error) = &self.reply.blocked {
                return Err(error.clone());
            }
            if self.reply.busy {
                return Err(RuntimeError::new(
                    ErrorCode::Busy,
                    "Wait for the current setup action and owned cleanup to settle.",
                ));
            }
            let configuration = self
                .reply
                .configuration
                .as_ref()
                .ok_or_else(RuntimeError::storage)?;
            let revision = match request {
                RuntimeRequest::Save {
                    revision,
                    executable,
                    ..
                } => {
                    validate_preference(executable.as_deref())?;
                    *revision
                }
                RuntimeRequest::Check { revision, .. } => *revision,
                _ => unreachable!(),
            };
            if revision != configuration.revision {
                return Err(RuntimeError::new(
                    ErrorCode::StaleTarget,
                    "The saved executable changed. Refresh its configuration before continuing.",
                ));
            }
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
            while let Ok(request) = receiver.recv() {
                execute(
                    &worker_state,
                    &mut store,
                    request,
                    &boot,
                    |configuration, id, progress| runner.check(configuration, id, progress),
                );
            }
            if !runner.cleanup_confirmed() {
                update(&worker_state, |s| {
                    s.reply.blocked = Some(RuntimeError::cleanup())
                });
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
        let lookup_id = if let RuntimeRequest::Action { action_id, .. } = &request {
            Some(*action_id)
        } else {
            None
        };
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
        let start = std::time::Instant::now();
        while self.worker.as_ref().is_some_and(|w| !w.is_finished()) {
            if start.elapsed() >= std::time::Duration::from_secs(4) {
                return Err(RuntimeError::cleanup());
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| RuntimeError::cleanup())?;
        }
        let state = self.shared.lock().map_err(|_| RuntimeError::cleanup())?;
        if let Some(error) = &state.reply.blocked {
            return Err(error.clone());
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
        let result = store.action(id);
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
            action = old;
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
            if saved.phase.terminal() {
                action = saved;
            } else if matches!(request, RuntimeRequest::Save { .. }) {
                let _ = store.finish(&action, true);
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

impl Drop for Runtimes {
    fn drop(&mut self) {
        // Explicit shutdown reports bounded failure while retaining this owner.
        // Destruction is the final fallback: request stop and observe completion,
        // rather than silently detaching a worker or abandoning its child handles.
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        self.sender.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
