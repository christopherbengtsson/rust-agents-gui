//! Repository policy. Read-only validation and durable registration belong here.
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, MutexGuard,
        mpsc::{self, SyncSender},
    },
    thread::JoinHandle,
};
use uuid::Uuid;

macro_rules! identity {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
    };
}
identity!(RepositoryId);
identity!(ActionId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidPath,
    Git,
    Storage,
    Conflict,
    Capacity,
    NotFound,
    Unavailable,
    Connection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryError {
    pub code: ErrorCode,
    pub message: String,
}
impl RepositoryError {
    pub(crate) fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        let mut message = message.into();
        if message.len() > 2048 {
            let mut end = 2048;
            while !message.is_char_boundary(end) {
                end -= 1;
            }
            message.truncate(end);
            message.push('…');
        }
        Self { code, message }
    }
    pub(crate) fn storage(message: impl fmt::Display) -> Self {
        Self::new(
            ErrorCode::Storage,
            format!(
                "Application storage failed: {message}. Check the private state directory and free disk space, then refresh to reconcile this action."
            ),
        )
    }
}
impl fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for RepositoryError {}
pub(crate) type Result<T> = std::result::Result<T, RepositoryError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegisterRepository {
    pub action_id: ActionId,
    pub path: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RegistrationState {
    Pending,
    Completed { repository_id: RepositoryId },
    Failed { error: RepositoryError },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registration {
    pub action_id: ActionId,
    pub path: String,
    #[serde(flatten)]
    pub state: RegistrationState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RepositoryRecord {
    pub id: RepositoryId,
    pub name: String,
    pub path: String,
    pub common_dir: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Availability {
    Checking,
    Available,
    Unavailable { error: RepositoryError },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Repository {
    pub id: RepositoryId,
    pub name: String,
    pub path: String,
    pub availability: Availability,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryPage {
    pub repositories: Vec<Repository>,
    pub next: Option<RepositoryId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum RepositoryRequest {
    Register {
        request: RegisterRepository,
    },
    Action {
        action_id: ActionId,
    },
    List {
        after: Option<RepositoryId>,
        #[serde(default)]
        refresh: bool,
    },
    Get {
        id: RepositoryId,
        #[serde(default)]
        refresh: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RepositoryReply {
    Registration { registration: Registration },
    Action { registration: Option<Registration> },
    Repositories { page: RepositoryPage },
    Repository { repository: Option<Repository> },
}

pub(crate) trait RepositoryStore: Send {
    fn action(&mut self, id: ActionId) -> Result<Option<Registration>>;
    fn intent(&mut self, request: &RegisterRepository) -> Result<Registration>;
    fn pending(&mut self) -> Result<Option<Registration>>;
    fn pending_count(&mut self) -> Result<usize>;
    fn find_common(&mut self, common: &str) -> Result<Option<RepositoryRecord>>;
    fn finish(
        &mut self,
        action: &Registration,
        repository: Option<&RepositoryRecord>,
        state: &RegistrationState,
    ) -> Result<Registration>;
    fn repository(&mut self, id: RepositoryId) -> Result<Option<RepositoryRecord>>;
    fn list(&mut self, after: Option<RepositoryId>, limit: usize) -> Result<Vec<RepositoryRecord>>;
}

fn admit(store: &mut dyn RepositoryStore, request: &RegisterRepository) -> Result<Registration> {
    if let Some(action) = store.action(request.action_id)? {
        if action.path != request.path {
            return Err(RepositoryError::new(
                ErrorCode::Conflict,
                "This action ID belongs to a different repository path. Retrieve its result; submit an edited path as a new action.",
            ));
        }
        return Ok(action);
    }
    if !std::path::Path::new(&request.path).is_absolute() || request.path.contains('\0') {
        return Err(RepositoryError::new(
            ErrorCode::InvalidPath,
            "Enter an absolute repository folder path, or choose Browse.",
        ));
    }
    if request.path.len() > 4096 {
        return Err(RepositoryError::new(
            ErrorCode::Capacity,
            "This path exceeds the registration input budget (4096 UTF-8 bytes). Choose a shorter repository path.",
        ));
    }
    if store.pending_count()? >= 8 {
        return Err(RepositoryError::new(
            ErrorCode::Capacity,
            "Registration is busy with eight pending actions. Refresh their status before adding another repository.",
        ));
    }
    store.intent(request)
}

pub(crate) struct ValidatedRepository {
    pub name: String,
    pub path: String,
    pub common_dir: String,
}

fn finish_registration(
    store: &mut dyn RepositoryStore,
    action: &Registration,
    validation: Result<ValidatedRepository>,
) -> Result<Registration> {
    let (repository, state) = match validation {
        Err(error) => (None, RegistrationState::Failed { error }),
        Ok(validated) => {
            let existing = store.find_common(&validated.common_dir)?;
            let repository = existing.unwrap_or_else(|| RepositoryRecord {
                id: RepositoryId(Uuid::new_v4()),
                name: validated.name,
                path: validated.path,
                common_dir: validated.common_dir,
            });
            // Bound a single observation before it becomes a usable registration.
            let observation = observe(&repository, Availability::Checking);
            if serde_json::to_vec(&observation)
                .map_err(RepositoryError::storage)?
                .len()
                > 24 * 1024
            {
                return store.finish(action, None, &RegistrationState::Failed { error: RepositoryError::new(ErrorCode::Capacity, "This repository's location exceeds the navigation byte budget. Use a shorter location.") });
            }
            let state = RegistrationState::Completed {
                repository_id: repository.id,
            };
            (Some(repository), state)
        }
    };
    match store.finish(action, repository.as_ref(), &state) {
        Ok(recorded) => Ok(recorded),
        Err(error) => {
            // The store exposes only committed reads, including after commit errors.
            // Reconcile a lost commit reply instead of inserting another identity.
            match store.action(action.action_id)? {
                Some(recorded) if recorded.state != RegistrationState::Pending => Ok(recorded),
                _ => Err(error),
            }
        }
    }
}

fn observe(record: &RepositoryRecord, availability: Availability) -> Repository {
    Repository {
        id: record.id,
        name: record.name.clone(),
        path: record.path.clone(),
        availability,
    }
}

const PAGE_SIZE: usize = 32;
const PAGE_BYTES: usize = 48 * 1024;
const CACHE_SIZE: usize = 64;

struct WorkspaceState {
    directory: PathBuf,
    store: Result<Box<dyn RepositoryStore>>,
    cache: VecDeque<(RepositoryId, Availability)>,
    probes: VecDeque<RepositoryRecord>,
    fault: Option<RepositoryError>,
    revision: u64,
}
impl WorkspaceState {
    fn store(&mut self) -> Result<&mut (dyn RepositoryStore + 'static)> {
        if self.store.is_err() {
            self.store = crate::storage::SqliteStore::open(&self.directory)
                .map(|s| Box::new(s) as Box<dyn RepositoryStore>);
        }
        match &mut self.store {
            Ok(store) => Ok(store.as_mut()),
            Err(error) => Err(error.clone()),
        }
    }
    fn change(&mut self) -> Result<()> {
        self.revision = self.revision.checked_add(1).ok_or_else(|| {
            RepositoryError::new(
                ErrorCode::Capacity,
                "Observation revision exhausted. Reopen with a new supervisor owner.",
            )
        })?;
        Ok(())
    }
    fn availability(&self, id: RepositoryId) -> Option<Availability> {
        self.cache
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, value)| value.clone())
    }
    fn cache(&mut self, id: RepositoryId, value: Availability) -> Result<()> {
        if self.availability(id).as_ref() == Some(&value) {
            return Ok(());
        }
        self.cache.retain(|(key, _)| *key != id);
        if self.cache.len() >= CACHE_SIZE {
            self.cache.pop_front();
        }
        self.cache.push_back((id, value));
        self.change()
    }
    fn queue(&mut self, record: &RepositoryRecord, refresh: bool) -> Result<Availability> {
        if !refresh && let Some(value) = self.availability(record.id) {
            return Ok(value);
        }
        self.cache(record.id, Availability::Checking)?;
        self.probes.retain(|r| r.id != record.id);
        if self.probes.len() >= CACHE_SIZE {
            self.probes.pop_front();
        }
        self.probes.push_back(record.clone());
        Ok(Availability::Checking)
    }
    fn page(&mut self, after: Option<RepositoryId>, refresh: bool) -> Result<RepositoryPage> {
        let records = self.store()?.list(after, PAGE_SIZE + 1)?;
        let mut page = RepositoryPage {
            repositories: vec![],
            next: None,
        };
        let total = records.len();
        for record in records.into_iter().take(PAGE_SIZE) {
            let availability = self.queue(&record, refresh)?;
            page.repositories.push(observe(&record, availability));
            if serde_json::to_vec(&page)
                .map_err(RepositoryError::storage)?
                .len()
                > PAGE_BYTES
            {
                page.repositories.pop();
                if page.repositories.is_empty() {
                    return Err(RepositoryError::new(
                        ErrorCode::Capacity,
                        "A saved repository exceeds the navigation budget. Its record has been preserved; use a compatible build to inspect it.",
                    ));
                }
                break;
            }
        }
        if total > page.repositories.len() {
            page.next = page.repositories.last().map(|r| r.id);
        }
        Ok(page)
    }
}

/// One supervisor-owned worker; requests never create per-repository runtimes.
pub struct Workspaces {
    shared: Arc<Mutex<WorkspaceState>>,
    wake: Option<SyncSender<()>>,
    worker: Option<JoinHandle<()>>,
}
impl Workspaces {
    pub fn request(&self, request: RepositoryRequest) -> Result<RepositoryReply> {
        match request {
            RepositoryRequest::Register { request } => self
                .register(request)
                .map(|registration| RepositoryReply::Registration { registration }),
            RepositoryRequest::Action { action_id } => self
                .action(action_id)
                .map(|registration| RepositoryReply::Action { registration }),
            RepositoryRequest::List { after, refresh } => self
                .repositories(after, refresh)
                .map(|page| RepositoryReply::Repositories { page }),
            RepositoryRequest::Get { id, refresh } => self
                .repository(id, refresh)
                .map(|repository| RepositoryReply::Repository { repository }),
        }
    }
    pub fn open(directory: &Path) -> Self {
        let shared = Arc::new(Mutex::new(WorkspaceState {
            directory: directory.to_owned(),
            store: crate::storage::SqliteStore::open(directory)
                .map(|s| Box::new(s) as Box<dyn RepositoryStore>),
            cache: VecDeque::new(),
            probes: VecDeque::new(),
            fault: None,
            revision: 1,
        }));
        let (wake, receiver) = mpsc::sync_channel(1);
        let worker_state = shared.clone();
        let worker = std::thread::spawn(move || {
            loop {
                if matches!(receiver.try_recv(), Err(mpsc::TryRecvError::Disconnected)) {
                    break;
                }
                if process_one(&worker_state, crate::git::inspect) {
                    continue;
                }
                if receiver.recv().is_err() {
                    break;
                }
            }
        });
        Self {
            shared,
            wake: Some(wake),
            worker: Some(worker),
        }
    }
    fn state(&self) -> Result<MutexGuard<'_, WorkspaceState>> {
        self.shared.lock().map_err(RepositoryError::storage)
    }
    fn notify(&self) {
        if let Some(wake) = &self.wake {
            let _ = wake.try_send(());
        }
    }
    pub fn revision(&self) -> u64 {
        self.shared.lock().map_or(1, |s| s.revision)
    }
    pub fn register(&self, request: RegisterRepository) -> Result<Registration> {
        let result = admit(self.state()?.store()?, &request);
        // A failed insert may still have committed intent. Only the owner reconciles it.
        self.notify();
        result
    }
    pub fn action(&self, id: ActionId) -> Result<Option<Registration>> {
        let mut state = self.state()?;
        let action = state.store()?.action(id)?;
        let result = if action
            .as_ref()
            .is_some_and(|a| a.state == RegistrationState::Pending)
        {
            match &state.fault {
                Some(error) => Err(error.clone()),
                None => Ok(action),
            }
        } else {
            Ok(action)
        };
        drop(state);
        self.notify();
        result
    }
    pub fn repositories(
        &self,
        after: Option<RepositoryId>,
        refresh: bool,
    ) -> Result<RepositoryPage> {
        let result = self.state()?.page(after, refresh);
        self.notify();
        result
    }
    pub fn repository(&self, id: RepositoryId, refresh: bool) -> Result<Option<Repository>> {
        let mut state = self.state()?;
        let record = state.store()?.repository(id)?;
        let result = record
            .map(|record| state.queue(&record, refresh).map(|a| observe(&record, a)))
            .transpose();
        drop(state);
        self.notify();
        result
    }
}
impl Drop for Workspaces {
    fn drop(&mut self) {
        self.wake.take();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn process_one(
    shared: &Mutex<WorkspaceState>,
    inspect: impl FnOnce(&str) -> Result<ValidatedRepository>,
) -> bool {
    enum Work {
        Register(Registration),
        Probe(RepositoryRecord),
    }
    let work = {
        let Ok(mut state) = shared.lock() else {
            return false;
        };
        match state.store().and_then(|store| store.pending()) {
            Ok(Some(action)) => Work::Register(action),
            Ok(None) => {
                state.fault = None;
                match state.probes.pop_front() {
                    Some(record) => Work::Probe(record),
                    None => return false,
                }
            }
            Err(error) => {
                state.fault = Some(error);
                return false;
            }
        }
    };
    // Never hold storage/observation locks during filesystem or Git execution.
    let validation = inspect(match &work {
        Work::Register(action) => &action.path,
        Work::Probe(record) => &record.path,
    });
    let Ok(mut state) = shared.lock() else {
        return false;
    };
    let result = match work {
        Work::Register(action) => state
            .store()
            .and_then(|store| finish_registration(store, &action, validation))
            .and_then(|_| state.change()),
        Work::Probe(record) => {
            let availability = match validation {
                Ok(actual)
                    if actual.path == record.path && actual.common_dir == record.common_dir =>
                {
                    Availability::Available
                }
                Ok(_) => Availability::Unavailable {
                    error: RepositoryError::new(
                        ErrorCode::Unavailable,
                        "The registered location now identifies a different checkout or Git directory. Restore the original repository location; its saved identity is retained.",
                    ),
                },
                Err(mut error) => {
                    // Keep observations bounded even when Git supplies verbose diagnostics.
                    if error.message.len() > 1024 {
                        let mut end = 1024;
                        while !error.message.is_char_boundary(end) {
                            end -= 1;
                        }
                        error.message.truncate(end);
                    }
                    Availability::Unavailable { error }
                }
            };
            state.cache(record.id, availability)
        }
    };
    match result {
        Ok(()) => {
            state.fault = None;
            true
        }
        Err(error) => {
            state.fault = Some(error);
            false
        }
    }
}

#[cfg(test)]
mod tests;
