//! Repository policy. Read-only validation and durable registration belong here.
use serde::{Deserialize, Serialize};
use std::fmt;
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};
use uuid::Uuid;

macro_rules! identity {
    ($name:ident) => {
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
        )]
        #[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
        #[serde(transparent)]
        pub struct $name(
            #[cfg_attr(feature = "codegen", schemars(regex(pattern = r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")))]
            pub Uuid,
        );
    };
}
identity!(RepositoryId);
identity!(ActionId);
mod creation;
mod inspection;
pub(crate) use inspection::{CONTENT_BYTES, FileEvidence, InspectionEffects};
pub use inspection::{
    ChangePage, ChangedFile, ChangesObservation, ComparisonId, DiffRow, FileId, FilePreview,
    InspectionPhase, PreviewObservation, PreviewTarget, RowKind, UnsupportedContent,
};
pub(crate) mod naming;
pub use creation::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
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
    CleanupUnconfirmed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
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
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct RegisterRepository {
    #[serde(default)]
    pub create_initial_workspace: bool,
    pub action_id: ActionId,
    pub path: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RegistrationState {
    Pending,
    Completed { repository_id: RepositoryId },
    Failed { error: RepositoryError },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct Registration {
    #[serde(default)]
    pub create_initial_workspace: bool,
    #[serde(default)]
    pub initial_workspace_id: Option<WorkspaceId>,
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
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Availability {
    Checking,
    Available,
    Unavailable { error: RepositoryError },
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct Repository {
    pub id: RepositoryId,
    pub name: String,
    pub path: String,
    pub availability: Availability,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct RepositoryPage {
    pub repositories: Vec<Repository>,
    pub next: Option<RepositoryId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
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
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum RepositoryReply {
    Registration { registration: Registration },
    Action { registration: Option<Registration> },
    Repositories { page: RepositoryPage },
    Repository { repository: Option<Repository> },
}

pub(crate) type StoreClose = std::result::Result<(), (Box<dyn RepositoryStore>, RepositoryError)>;
pub(crate) trait RepositoryStore: CreationStore {
    fn close(self: Box<Self>) -> StoreClose {
        Ok(())
    }
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
        initial: Option<&Workspace>,
    ) -> Result<Registration>;
    fn repository(&mut self, id: RepositoryId) -> Result<Option<RepositoryRecord>>;
    fn list(&mut self, after: Option<RepositoryId>, limit: usize) -> Result<Vec<RepositoryRecord>>;
}

pub(crate) fn check_registration_path_budget(path: &str) -> Result<()> {
    if path.len() > 4096 {
        return Err(RepositoryError::new(
            ErrorCode::Capacity,
            "This path exceeds the registration input budget (4096 UTF-8 bytes). Choose a shorter repository path.",
        ));
    }
    Ok(())
}

fn admit(store: &mut dyn RepositoryStore, request: &RegisterRepository) -> Result<Registration> {
    if let Some(action) = store.action(request.action_id)? {
        if action.path != request.path
            || action.create_initial_workspace != request.create_initial_workspace
        {
            return Err(RepositoryError::new(
                ErrorCode::Conflict,
                "This action ID belongs to a different repository path. Retrieve its result; submit an edited path as a new action.",
            ));
        }
        return Ok(action);
    }
    if store.workspace_action(request.action_id)?.is_some() {
        return Err(RepositoryError::new(
            ErrorCode::Conflict,
            "This action ID belongs to a workspace operation.",
        ));
    }
    if !std::path::Path::new(&request.path).is_absolute() || request.path.contains('\0') {
        return Err(RepositoryError::new(
            ErrorCode::InvalidPath,
            "Enter an absolute repository folder path, or choose Browse.",
        ));
    }
    check_registration_path_budget(&request.path)?;
    if store.pending_count()? + store.workspace_count_pending()? >= 8 {
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
    directory: &Path,
) -> Result<Registration> {
    let mut initial = None;
    let mut action = action.clone();
    let (repository, state) = match validation {
        Err(error) => (None, RegistrationState::Failed { error }),
        Ok(validated) => {
            let existing = store.find_common(&validated.common_dir)?;
            if existing.is_none() && action.create_initial_workspace {
                // Allocate the continuation before the registration transaction, never after its reply.
                let repository_id = RepositoryId(Uuid::new_v4());
                initial = Some(allocate(
                    repository_id,
                    ActionId(Uuid::new_v4()),
                    directory,
                )?);
                action.initial_workspace_id = initial.as_ref().map(|w| w.id);
            }
            let repository = existing.unwrap_or_else(|| RepositoryRecord {
                id: initial
                    .as_ref()
                    .map_or_else(|| RepositoryId(Uuid::new_v4()), |w| w.repository_id),
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
                return store.finish(&action, None, &RegistrationState::Failed { error: RepositoryError::new(ErrorCode::Capacity, "This repository's location exceeds the navigation byte budget. Use a shorter location.") }, None);
            }
            let state = RegistrationState::Completed {
                repository_id: repository.id,
            };
            (Some(repository), state)
        }
    };
    match store.finish(&action, repository.as_ref(), &state, initial.as_ref()) {
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

struct OwnerState {
    journal: Arc<crate::observations::Journal>,
    inspections: inspection::Inspections,
    directory: PathBuf,
    store: Result<Box<dyn RepositoryStore>>,
    cache: VecDeque<(RepositoryId, Availability)>,
    probes: VecDeque<RepositoryRecord>,
    fault: Option<RepositoryError>,
    fenced: bool,
    storage_closed: bool,
    revision: u64,
    active_workspace: Option<WorkspaceId>,
    workspace_faults: BTreeMap<WorkspaceId, RepositoryError>,
}
impl OwnerState {
    fn open(
        directory: &Path,
        resolve: impl FnOnce(&Path) -> Result<PathBuf>,
        open: impl FnOnce(&Path) -> Result<Box<dyn RepositoryStore>>,
    ) -> Result<Self> {
        let directory = resolve(directory)?;
        validate_workspace_path(
            directory
                .to_str()
                .ok_or_else(|| RepositoryError::storage("Private state path must be UTF-8"))?,
        )?;
        let store = open(&directory);
        Ok(Self {
            inspections: inspection::Inspections::default(),
            directory,
            store,
            cache: VecDeque::new(),
            probes: VecDeque::new(),
            fault: None,
            fenced: false,
            storage_closed: false,
            revision: 1,
            journal: Arc::new(crate::observations::Journal::default()),
            active_workspace: None,
            workspace_faults: BTreeMap::new(),
        })
    }
    fn store(&mut self) -> Result<&mut (dyn RepositoryStore + 'static)> {
        if self.storage_closed {
            return Err(RepositoryError::storage("Workspace storage is closed"));
        }
        if self.store.is_err() {
            self.store = crate::storage::SqliteStore::open(&self.directory)
                .map(|s| Box::new(s) as Box<dyn RepositoryStore>);
        }
        match &mut self.store {
            Ok(store) => Ok(store.as_mut()),
            Err(error) => Err(error.clone()),
        }
    }
    fn publish(&mut self, targets: Vec<crate::observations::Target>) {
        self.journal.publish(targets);
    }
    fn workspace_changed(&mut self, workspace_id: WorkspaceId, repository_id: RepositoryId) {
        self.publish(vec![
            crate::observations::Target::Workspace { workspace_id },
            crate::observations::Target::Workspaces {
                repository_id: Some(repository_id),
            },
        ]);
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
        self.publish(vec![
            crate::observations::Target::Repositories,
            crate::observations::Target::Repository { repository_id: id },
        ]);
        self.change()
    }
    fn queue(&mut self, record: &RepositoryRecord, refresh: bool) -> Result<Availability> {
        if self.fenced {
            return self
                .availability(record.id)
                .ok_or_else(|| RepositoryError::storage("Workspace probes are fenced"));
        }
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

struct OwnerGuard<'a> {
    inner: Option<MutexGuard<'a, OwnerState>>,
    journal: Arc<crate::observations::Journal>,
}
impl std::ops::Deref for OwnerGuard<'_> {
    type Target = OwnerState;
    fn deref(&self) -> &OwnerState {
        self.inner.as_ref().expect("live owner guard")
    }
}
impl std::ops::DerefMut for OwnerGuard<'_> {
    fn deref_mut(&mut self) -> &mut OwnerState {
        self.inner.as_mut().expect("live owner guard")
    }
}
impl Drop for OwnerGuard<'_> {
    fn drop(&mut self) {
        drop(self.inner.take());
        self.journal.wake();
    }
}
fn lock_state(shared: &Mutex<OwnerState>) -> Result<OwnerGuard<'_>> {
    let inner = shared.lock().map_err(RepositoryError::storage)?;
    let journal = inner.journal.clone();
    Ok(OwnerGuard {
        inner: Some(inner),
        journal,
    })
}

/// One supervisor-owned worker; requests never create per-repository runtimes.
pub struct Workspaces {
    shared: Arc<Mutex<OwnerState>>,
    wake: Mutex<Option<tokio::sync::mpsc::Sender<()>>>,
    worker: tokio::sync::Mutex<Option<tokio::task::JoinHandle<Result<()>>>>,
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
    pub fn open(directory: &Path) -> Result<Self> {
        let state = OwnerState::open(
            directory,
            |path| {
                crate::supervisor::check_private(path, true).map_err(RepositoryError::storage)?;
                std::fs::canonicalize(path).map_err(|error| {
                    RepositoryError::storage(format!(
                        "Cannot resolve private application state: {error}"
                    ))
                })
            },
            |path| {
                crate::storage::SqliteStore::open(path)
                    .map(|s| Box::new(s) as Box<dyn RepositoryStore>)
            },
        )?;
        state.store.as_ref().map_err(Clone::clone)?;
        let shared = Arc::new(Mutex::new(state));
        let (wake, mut receiver) = tokio::sync::mpsc::channel(1);
        let worker_state = shared.clone();
        let worker = tokio::spawn(async move {
            let mut next = 0;
            let mut operations = tokio::task::JoinSet::new();
            loop {
                let state = worker_state.clone();
                operations.spawn_blocking(move || {
                    let worked = worker_cycle(&mut next, |kind| match kind {
                        0 => process_creation(&state, &mut crate::git::LocalCreation),
                        1 => process_one(&state, crate::git::inspect),
                        _ => {
                            inspection::process_inspection(&state, &mut crate::git::LocalInspection)
                        }
                    });
                    (worked, next)
                });
                match operations.join_next().await {
                    Some(Ok((worked, cursor))) => {
                        next = cursor;
                        if worked {
                            continue;
                        }
                    }
                    Some(Err(error)) => {
                        let failure =
                            RepositoryError::storage(format!("Workspace worker failed: {error}"));
                        let mut state = worker_state.lock().unwrap_or_else(|e| e.into_inner());
                        state.fenced = true;
                        state.fault = Some(failure.clone());
                        break;
                    }
                    None => unreachable!("one owned workspace cycle"),
                }
                if receiver.recv().await.is_none() {
                    break;
                }
            }
            tokio::task::spawn_blocking(move || {
                let mut state = worker_state.lock().unwrap_or_else(|e| e.into_inner());
                let mut errors = Vec::new();
                if let Some(error) = &state.fault {
                    errors.push(error.message.clone());
                }
                errors.extend(
                    state
                        .workspace_faults
                        .iter()
                        .map(|(id, error)| format!("Workspace {}: {}", id.0, error.message)),
                );
                state.storage_closed = true;
                let store = std::mem::replace(
                    &mut state.store,
                    Err(RepositoryError::storage("Workspace storage is closed")),
                );
                match store {
                    Ok(store) => {
                        if let Err((store, error)) = store.close() {
                            errors.push(error.message.clone());
                            state.store = Ok(store);
                        }
                    }
                    Err(error) => errors.push(error.message),
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(RepositoryError::storage(errors.join("; ")))
                }
            })
            .await
            .map_err(RepositoryError::storage)?
        });
        Ok(Self {
            shared,
            wake: Mutex::new(Some(wake)),
            worker: tokio::sync::Mutex::new(Some(worker)),
        })
    }
    pub(crate) fn fence(&self) {
        self.shared.lock().unwrap_or_else(|e| e.into_inner()).fenced = true;
    }
    pub(crate) async fn finish(&self) -> Result<()> {
        self.fence();
        self.wake.lock().unwrap_or_else(|e| e.into_inner()).take();
        let mut worker = self.worker.lock().await;
        if let Some(task) = worker.as_mut() {
            let result = task.await.map_err(RepositoryError::storage)?;
            worker.take();
            result
        } else {
            Ok(())
        }
    }
    fn state(&self) -> Result<OwnerGuard<'_>> {
        lock_state(&self.shared)
    }
    pub(crate) fn journal(&self) -> Arc<crate::observations::Journal> {
        self.shared
            .lock()
            .expect("workspace owner lock poisoned")
            .journal
            .clone()
    }
    fn notify(&self) {
        if let Some(wake) = &*self.wake.lock().unwrap_or_else(|e| e.into_inner()) {
            let _ = wake.try_send(());
        }
    }
    pub fn revision(&self) -> u64 {
        self.shared.lock().map_or(1, |s| s.revision)
    }
    pub fn register(&self, request: RegisterRepository) -> Result<Registration> {
        let mut state = self.state()?;
        if state.fenced {
            return Err(RepositoryError::storage("Workspace admission is fenced"));
        }
        let result = state.store().and_then(|store| admit(store, &request));
        state.publish(vec![
            crate::observations::Target::Registration {
                action_id: request.action_id,
            },
            crate::observations::Target::Repositories,
        ]);
        drop(state);
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

fn worker_cycle(next: &mut usize, mut run: impl FnMut(usize) -> bool) -> bool {
    for offset in 0..3 {
        let kind = (*next + offset) % 3;
        if run(kind) {
            *next = (kind + 1) % 3;
            return true;
        }
    }
    false
}

fn process_one(
    shared: &Mutex<OwnerState>,
    inspect: impl FnOnce(&str) -> Result<ValidatedRepository>,
) -> bool {
    enum Work {
        Register(Registration),
        Probe(RepositoryRecord),
    }
    let work = {
        let Ok(mut state) = lock_state(shared) else {
            return false;
        };
        match state.store().and_then(|store| store.pending()) {
            Ok(Some(action)) => Work::Register(action),
            Ok(None) => {
                if state.fault.is_some() {
                    state.publish(vec![crate::observations::Target::Repositories]);
                }
                match state.probes.pop_front() {
                    Some(record) => Work::Probe(record),
                    None => return false,
                }
            }
            Err(error) => {
                state.fault = Some(error);
                state.publish(vec![crate::observations::Target::Repositories]);
                return false;
            }
        }
    };
    // Never hold storage/observation locks during filesystem or Git execution.
    let identity = match &work {
        Work::Register(action) => format!("registration {}", action.action_id.0),
        Work::Probe(record) => format!("repository {}", record.id.0),
    };
    let validation = catch_effect(|| {
        inspect(match &work {
            Work::Register(action) => &action.path,
            Work::Probe(record) => &record.path,
        })
    });
    let Ok(mut state) = lock_state(shared) else {
        return false;
    };
    if let Err(error) = &validation
        && error.code == ErrorCode::CleanupUnconfirmed
    {
        state.fenced = true;
        state.fault = Some(RepositoryError::new(
            ErrorCode::CleanupUnconfirmed,
            format!("{identity}: {}", error.message),
        ));
    }
    let result = match work {
        Work::Register(action) => {
            state.publish(vec![
                crate::observations::Target::Registration {
                    action_id: action.action_id,
                },
                crate::observations::Target::Repositories,
                crate::observations::Target::Workspaces {
                    repository_id: None,
                },
            ]);
            let directory = state.directory.clone();
            state
                .store()
                .and_then(|store| finish_registration(store, &action, validation, &directory))
                .and_then(|registration| {
                    let mut targets = vec![crate::observations::Target::Registration {
                        action_id: registration.action_id,
                    }];
                    if let RegistrationState::Completed { repository_id } = registration.state {
                        targets.push(crate::observations::Target::Repository { repository_id });
                    }
                    if let Some(workspace_id) = registration.initial_workspace_id {
                        targets.push(crate::observations::Target::Workspace { workspace_id });
                    }
                    state.publish(targets);
                    state.change()
                })
        }
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
            if state
                .fault
                .as_ref()
                .is_some_and(|error| error.code != ErrorCode::CleanupUnconfirmed)
                && state.fault.take().is_some()
            {
                state.publish(vec![crate::observations::Target::Repositories]);
            }
            true
        }
        Err(error) => {
            state.fault = Some(error);
            state.publish(vec![crate::observations::Target::Repositories]);
            false
        }
    }
}

#[cfg(test)]
pub(crate) mod tests;

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<RepositoryId>();
    exports.add::<ActionId>();
    exports.add::<ErrorCode>();
    exports.add::<RepositoryError>();
    exports.add::<RegisterRepository>();
    exports.add::<RegistrationState>();
    exports.add::<Registration>();
    exports.add::<Availability>();
    exports.add::<Repository>();
    exports.add::<RepositoryPage>();
    exports.add::<RepositoryRequest>();
    exports.add::<RepositoryReply>();
    creation::export_contracts(exports);
    inspection::export_contracts(exports);
}

/// A panic means uncertain effects, never an invitation to retry.
fn catch_effect<T>(work: impl FnOnce() -> Result<T>) -> Result<T> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)).unwrap_or_else(|_| {
        Err(RepositoryError::new(
            ErrorCode::CleanupUnconfirmed,
            "Owned effect panicked; reconcile physical cleanup",
        ))
    })
}
