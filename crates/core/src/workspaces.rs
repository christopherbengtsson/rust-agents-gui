//! Repository policy. Read-only validation and durable registration belong here.
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, VecDeque},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
};
use uuid::Uuid;

#[path = "workspaces/records.rs"]
mod records;
pub use records::*;
pub(crate) use records::{RepositoryRecord, Result, identity, validate_workspace_path};
#[path = "workspaces/creation.rs"]
mod creation;
#[path = "workspaces/inspection.rs"]
mod inspection;
pub(crate) use inspection::{CONTENT_BYTES, FileEvidence, InspectionEffects};
pub use inspection::{
    ChangePage, ChangedFile, ChangesObservation, ComparisonId, DiffRow, FileId, FilePreview,
    InspectionPhase, PreviewObservation, PreviewTarget, RowKind, UnsupportedContent,
};
#[path = "workspaces/naming.rs"]
pub(crate) mod naming;
pub use creation::*;

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

#[path = "workspaces/coordinator.rs"]
mod coordinator;
#[path = "workspaces/persistence.rs"]
mod persistence;
#[cfg(feature = "qualification")]
#[path = "workspaces/qualification.rs"]
pub(crate) mod qualification;
use crate::storage::{Read, Write};
use persistence::{Metadata, one, page, save};

pub(crate) fn check_registration_path_budget(path: &str) -> Result<()> {
    if path.len() > 4096 {
        return Err(RepositoryError::new(
            ErrorCode::Capacity,
            "Registration path exceeds 4096 UTF-8 bytes",
        ));
    }
    Ok(())
}
pub(crate) struct ValidatedRepository {
    pub name: String,
    pub path: String,
    pub common_dir: String,
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
const PENDING_LIMIT: usize = 8;

#[derive(Debug, Clone)]
struct Activity {
    generation: u64,
    workspace: Option<WorkspaceId>,
    running: bool,
}
/// A provisional decision is visible before storage is asked. Cancellation after
/// write admission fences uncertainty; successful intent transfers to the coordinator.
struct DecisionActivity<'a> {
    owner: &'a Workspaces,
    id: ActionId,
    owned: bool,
    writing: bool,
}
impl DecisionActivity<'_> {
    fn transfer(mut self, workspace: Option<WorkspaceId>) {
        if self.owned
            && let Ok(mut state) = self.owner.state()
            && let Some(activity) = state.activities.get_mut(&self.id)
        {
            activity.workspace = workspace;
        }
        self.owned = false;
        self.owner.notify();
    }
}
impl Drop for DecisionActivity<'_> {
    fn drop(&mut self) {
        if self.owned
            && let Ok(mut state) = self.owner.state()
        {
            let activity = state.activities.remove(&self.id);
            if self.writing {
                state.fenced = true;
                let error = RepositoryError::storage(format!(
                    "Action {} lost admission settlement; reconcile before effects",
                    self.id.0
                ));
                if let Some(workspace) = activity.and_then(|a| a.workspace) {
                    state.workspace_faults.insert(workspace, error.clone());
                }
                state.fault = Some(error);
            }
        }
        self.owner.notify();
    }
}
struct OwnerState {
    journal: Arc<crate::observations::Journal>,
    inspections: inspection::Inspections,
    directory: PathBuf,
    cache: VecDeque<(RepositoryId, Availability)>,
    probes: VecDeque<RepositoryRecord>,
    fault: Option<RepositoryError>,
    cleanup_faults: BTreeMap<String, RepositoryError>,
    fenced: bool,
    revision: u64,
    generation: u64,
    activities: BTreeMap<ActionId, Activity>,
    workspace_faults: BTreeMap<WorkspaceId, RepositoryError>,
}
impl OwnerState {
    fn new(directory: PathBuf) -> Self {
        Self {
            journal: Arc::default(),
            inspections: Default::default(),
            directory,
            cache: VecDeque::new(),
            probes: VecDeque::new(),
            fault: None,
            cleanup_faults: BTreeMap::new(),
            fenced: false,
            revision: 1,
            generation: 1,
            activities: BTreeMap::new(),
            workspace_faults: BTreeMap::new(),
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
            RepositoryError::new(ErrorCode::Capacity, "Observation revision exhausted")
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
        if self.cache.len() == CACHE_SIZE {
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
        if self.probes.len() == CACHE_SIZE {
            self.probes.pop_front();
        }
        self.probes.push_back(record.clone());
        Ok(Availability::Checking)
    }
    fn workspace_observation(&self, mut workspace: Workspace) -> Workspace {
        if matches!(workspace.status, CreationStatus::Running { .. })
            && !self
                .activities
                .values()
                .any(|a| a.workspace == Some(workspace.id))
        {
            workspace.status = CreationStatus::Uncertain {
                error: RepositoryError::new(
                    ErrorCode::Unavailable,
                    "Creation was interrupted; reconcile this exact saved workspace explicitly",
                ),
            };
        }
        if let Some(error) = self.workspace_faults.get(&workspace.id) {
            workspace.status = CreationStatus::Uncertain {
                error: error.clone(),
            };
        }
        workspace
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
    let inner = shared.lock().map_err(|poison| {
        let mut state = poison.into_inner();
        state.fenced = true;
        let identity = format!(
            "owner panic: registration {:?}, workspace {:?}",
            state.activities, state.generation
        );
        let error = RepositoryError::new(
            ErrorCode::CleanupUnconfirmed,
            "Workspace decision owner panicked; cleanup remains fenced",
        );
        state.cleanup_faults.insert(identity, error.clone());
        error
    })?;
    let journal = inner.journal.clone();
    Ok(OwnerGuard {
        inner: Some(inner),
        journal,
    })
}

/// One retained coordinator on the supplied runtime. Requester lifetimes never own effects.
pub struct Workspaces {
    shared: Arc<Mutex<OwnerState>>,
    pub(crate) storage: Option<crate::storage::Storage>,
    decision: Arc<tokio::sync::Mutex<()>>,
    store: Arc<dyn Metadata>,
    wake: Mutex<Option<tokio::sync::mpsc::Sender<()>>>,
    worker: tokio::sync::Mutex<Option<tokio::task::JoinHandle<Result<()>>>>,
    finish_result: Mutex<Option<Result<()>>>,
}
impl Workspaces {
    pub(crate) fn open(directory: PathBuf, storage: crate::storage::Storage) -> Self {
        let mut owner = Self::with_store(directory, Arc::new(storage.clone()));
        owner.storage = Some(storage);
        owner
    }
    fn with_store(directory: PathBuf, store: Arc<dyn Metadata>) -> Self {
        Self::with_dependencies(directory, store, Arc::new(coordinator::LocalEffects))
    }
    fn with_dependencies(
        directory: PathBuf,
        store: Arc<dyn Metadata>,
        effects: Arc<dyn coordinator::Effects>,
    ) -> Self {
        let shared = Arc::new(Mutex::new(OwnerState::new(directory)));
        let decision = Arc::new(tokio::sync::Mutex::new(()));
        let (wake, receiver) = tokio::sync::mpsc::channel(1);
        let worker = tokio::spawn(coordinator::run(
            shared.clone(),
            decision.clone(),
            store.clone(),
            receiver,
            effects,
        ));
        Self {
            shared,
            storage: None,
            decision,
            store,
            wake: Mutex::new(Some(wake)),
            worker: tokio::sync::Mutex::new(Some(worker)),
            finish_result: Mutex::new(None),
        }
    }
    fn state(&self) -> Result<OwnerGuard<'_>> {
        lock_state(&self.shared)
    }
    fn begin_decision(&self, id: ActionId) -> Result<DecisionActivity<'_>> {
        let mut state = self.state()?;
        let owned = !state.fenced && !state.activities.contains_key(&id);
        if owned {
            let generation = state.generation;
            state.activities.insert(
                id,
                Activity {
                    generation,
                    workspace: None,
                    running: false,
                },
            );
        }
        Ok(DecisionActivity {
            owner: self,
            id,
            owned,
            writing: false,
        })
    }
    pub(crate) fn journal(&self) -> Arc<crate::observations::Journal> {
        self.shared
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .journal
            .clone()
    }
    pub fn revision(&self) -> u64 {
        self.shared.lock().map_or(1, |s| s.revision)
    }
    fn notify(&self) {
        if let Some(wake) = &*self.wake.lock().unwrap_or_else(|e| e.into_inner()) {
            let _ = wake.try_send(());
        }
    }
    pub(crate) async fn fence(&self) {
        let _gate = self.decision.lock().await;
        let mut state = self.shared.lock().unwrap_or_else(|e| e.into_inner());
        state.fenced = true;
        state.activities.retain(|_, activity| activity.running);
        // Already physically admitted work may settle, but a pre-effect continuation
        // must revalidate this generation before starting another mutation.
        if let Some(next) = state.generation.checked_add(1) {
            state.generation = next;
        } else {
            state.fault = Some(RepositoryError::storage("Workspace generation exhausted"));
        }
    }
    pub(crate) async fn finish(&self) -> Result<()> {
        self.fence().await;
        self.wake.lock().unwrap_or_else(|e| e.into_inner()).take();
        let mut worker = self.worker.lock().await;
        if let Some(task) = worker.as_mut() {
            let result = task.await.map_err(RepositoryError::storage).and_then(|r| r);
            *self.finish_result.lock().unwrap() = Some(result.clone());
            worker.take();
            result
        } else {
            self.finish_result.lock().unwrap().clone().unwrap_or(Ok(()))
        }
    }
    pub async fn request(&self, request: RepositoryRequest) -> Result<RepositoryReply> {
        let result = match request {
            RepositoryRequest::Register { request } => self
                .register(request)
                .await
                .map(|registration| RepositoryReply::Registration { registration }),
            RepositoryRequest::Action { action_id } => self
                .action(action_id)
                .await
                .map(|registration| RepositoryReply::Action { registration }),
            RepositoryRequest::List { after, refresh } => self
                .repositories(after, refresh)
                .await
                .map(|page| RepositoryReply::Repositories { page }),
            RepositoryRequest::Get { id, refresh } => self
                .repository(id, refresh)
                .await
                .map(|repository| RepositoryReply::Repository { repository }),
        };
        self.notify();
        result
    }
    async fn pending_capacity(&self) -> Result<()> {
        // Both indexed pages exceed the resource budget before requiring a second page.
        let (registrations, rnext) = page::<Registration>(
            self.store.as_ref(),
            Read::PendingRegistrations { after: None },
        )
        .await?;
        let (workspaces, wnext) =
            page::<Workspace>(self.store.as_ref(), Read::PendingWorkspaces { after: None }).await?;
        let mut pending: std::collections::BTreeSet<_> =
            registrations.iter().map(|r| r.action_id).collect();
        pending.extend(workspaces.iter().map(|w| w.creation_action_id));
        pending.extend(self.state()?.activities.keys().copied());
        if rnext.is_some() || wnext.is_some() || pending.len() > PENDING_LIMIT {
            return Err(RepositoryError::new(
                ErrorCode::Capacity,
                "Eight operations are pending; retrieve status before admitting more",
            ));
        }
        Ok(())
    }
    pub async fn register(&self, request: RegisterRepository) -> Result<Registration> {
        let _gate = self.decision.lock().await;
        let mut activity = self.begin_decision(request.action_id)?;
        if let Some(saved) =
            one::<Registration>(self.store.as_ref(), Read::Registration(request.action_id)).await?
        {
            if saved.path != request.path
                || saved.create_initial_workspace != request.create_initial_workspace
            {
                return Err(conflict());
            }
            return Ok(saved);
        }
        if self.state()?.fenced {
            return Err(fenced());
        }
        if one::<(WorkspaceAction, Workspace)>(
            self.store.as_ref(),
            Read::WorkspaceAction(request.action_id),
        )
        .await?
        .is_some()
        {
            return Err(conflict());
        }
        check_registration_path_budget(&request.path)?;
        if !Path::new(&request.path).is_absolute() || request.path.contains('\0') {
            return Err(RepositoryError::new(
                ErrorCode::InvalidPath,
                "Choose an absolute repository path",
            ));
        }
        self.pending_capacity().await?;
        let id = request.action_id;
        self.state()?.publish(vec![
            crate::observations::Target::Registration { action_id: id },
            crate::observations::Target::Repositories,
        ]);
        activity.writing = true;
        let write = save(self.store.as_ref(), Write::RegistrationIntent(request)).await;
        self.notify();
        // Reads capture writer settlement even when acknowledgment failed.
        let saved = one::<Registration>(self.store.as_ref(), Read::Registration(id)).await?;
        activity.writing = false;
        match saved {
            Some(saved) => {
                activity.transfer(None);
                Ok(saved)
            }
            None => {
                write?;
                Err(RepositoryError::storage("Registration intent is absent"))
            }
        }
    }
    pub async fn action(&self, id: ActionId) -> Result<Option<Registration>> {
        let saved = one::<Registration>(self.store.as_ref(), Read::Registration(id)).await?;
        if saved
            .as_ref()
            .is_some_and(|a| a.state == RegistrationState::Pending)
            && let Some(error) = &self.state()?.fault
        {
            return Err(error.clone());
        }
        Ok(saved)
    }
    pub async fn repositories(
        &self,
        after: Option<RepositoryId>,
        refresh: bool,
    ) -> Result<RepositoryPage> {
        let (records, next) =
            page::<RepositoryRecord>(self.store.as_ref(), Read::Repositories { after }).await?;
        let mut state = self.state()?;
        let mut result = RepositoryPage {
            repositories: Vec::new(),
            next: None,
        };
        for record in records {
            let availability = state.queue(&record, refresh)?;
            result.repositories.push(observe(&record, availability));
            if serde_json::to_vec(&result)
                .map_err(RepositoryError::storage)?
                .len()
                > PAGE_BYTES
            {
                result.repositories.pop();
                result.next = result.repositories.last().map(|r| r.id);
                if result.next.is_none() {
                    return Err(RepositoryError::new(
                        ErrorCode::Capacity,
                        "Repository exceeds observation budget",
                    ));
                }
                drop(state);
                self.notify();
                return Ok(result);
            }
        }
        result.next = next
            .map(|s| {
                Uuid::parse_str(&s)
                    .map(RepositoryId)
                    .map_err(RepositoryError::storage)
            })
            .transpose()?;
        drop(state);
        self.notify();
        Ok(result)
    }
    pub async fn repository(&self, id: RepositoryId, refresh: bool) -> Result<Option<Repository>> {
        let record = one::<RepositoryRecord>(self.store.as_ref(), Read::Repository(id)).await?;
        let result = record
            .map(|r| self.state()?.queue(&r, refresh).map(|a| observe(&r, a)))
            .transpose();
        self.notify();
        result
    }
    pub async fn workspace_request(&self, request: WorkspaceRequest) -> Result<WorkspaceReply> {
        let result = self.workspace_inner(request).await;
        self.notify();
        result
    }
    async fn workspace_inner(&self, request: WorkspaceRequest) -> Result<WorkspaceReply> {
        if matches!(
            request,
            WorkspaceRequest::Changes { .. } | WorkspaceRequest::ChangePreview { .. }
        ) {
            return self.inspection_request(request).await;
        }
        match request {
            WorkspaceRequest::Create {
                action_id,
                repository_id,
            } => {
                self.admit_workspace(WorkspaceAction::Create {
                    action_id,
                    repository_id,
                })
                .await
            }
            WorkspaceRequest::Reconcile {
                action_id,
                workspace_id,
            } => {
                self.admit_workspace(WorkspaceAction::Reconcile {
                    action_id,
                    workspace_id,
                })
                .await
            }
            WorkspaceRequest::Action { action_id } => {
                let saved = one::<(WorkspaceAction, Workspace)>(
                    self.store.as_ref(),
                    Read::WorkspaceAction(action_id),
                )
                .await?;
                let (action, workspace) = saved.map_or((None, None), |(a, w)| (Some(a), Some(w)));
                Ok(WorkspaceReply::Action {
                    action,
                    workspace: workspace
                        .map(|w| self.state().map(|s| s.workspace_observation(w)))
                        .transpose()?,
                })
            }
            WorkspaceRequest::Get { id } => {
                let workspace = one::<Workspace>(self.store.as_ref(), Read::Workspace(id)).await?;
                Ok(WorkspaceReply::Workspace {
                    workspace: workspace
                        .map(|w| self.state().map(|s| s.workspace_observation(w)))
                        .transpose()?,
                })
            }
            WorkspaceRequest::List {
                repository_id,
                after,
            } => {
                let (records, next) = page::<Workspace>(
                    self.store.as_ref(),
                    Read::Workspaces {
                        repository: repository_id,
                        after,
                    },
                )
                .await?;
                let state = self.state()?;
                let mut result = WorkspacePage {
                    workspaces: Vec::new(),
                    next: None,
                };
                for workspace in records {
                    result
                        .workspaces
                        .push(state.workspace_observation(workspace));
                    if serde_json::to_vec(&result)
                        .map_err(RepositoryError::storage)?
                        .len()
                        > PAGE_BYTES
                    {
                        result.workspaces.pop();
                        result.next = result.workspaces.last().map(|w| w.id);
                        if result.next.is_none() {
                            return Err(RepositoryError::new(
                                ErrorCode::Capacity,
                                "Workspace exceeds observation budget",
                            ));
                        }
                        return Ok(WorkspaceReply::Workspaces { page: result });
                    }
                }
                result.next = next
                    .map(|s| {
                        Uuid::parse_str(&s)
                            .map(WorkspaceId)
                            .map_err(RepositoryError::storage)
                    })
                    .transpose()?;
                Ok(WorkspaceReply::Workspaces { page: result })
            }
            _ => unreachable!(),
        }
    }
    async fn admit_workspace(&self, action: WorkspaceAction) -> Result<WorkspaceReply> {
        let _gate = self.decision.lock().await;
        let mut activity = self.begin_decision(action.id())?;
        if let Some((saved, workspace)) = one::<(WorkspaceAction, Workspace)>(
            self.store.as_ref(),
            Read::WorkspaceAction(action.id()),
        )
        .await?
        {
            if saved != action {
                return Err(conflict());
            }
            return Ok(WorkspaceReply::Action {
                action: Some(saved),
                workspace: Some(self.state()?.workspace_observation(workspace)),
            });
        }
        if self.state()?.fenced {
            return Err(fenced());
        }
        if one::<Registration>(self.store.as_ref(), Read::Registration(action.id()))
            .await?
            .is_some()
        {
            return Err(conflict());
        }
        let workspace = match action {
            WorkspaceAction::Create {
                action_id,
                repository_id,
            } => {
                self.pending_capacity().await?;
                if one::<RepositoryRecord>(self.store.as_ref(), Read::Repository(repository_id))
                    .await?
                    .is_none()
                {
                    return Err(RepositoryError::new(
                        ErrorCode::NotFound,
                        "Repository not found",
                    ));
                }
                if one::<bool>(self.store.as_ref(), Read::CleanupBlocked(repository_id))
                    .await?
                    .unwrap_or(false)
                {
                    return Err(RepositoryError::new(
                        ErrorCode::CleanupUnconfirmed,
                        "Reconcile unconfirmed repository cleanup first",
                    ));
                }
                allocate(repository_id, action_id, &self.state()?.directory)?
            }
            WorkspaceAction::Reconcile { workspace_id, .. } => {
                if self.state()?.activities.len() > PENDING_LIMIT {
                    return Err(RepositoryError::new(
                        ErrorCode::Capacity,
                        "Workspace execution capacity is occupied",
                    ));
                }
                let mut w = one::<Workspace>(self.store.as_ref(), Read::Workspace(workspace_id))
                    .await?
                    .ok_or_else(|| {
                        RepositoryError::new(ErrorCode::NotFound, "Workspace not found")
                    })?;
                let state = self.state()?;
                if state
                    .activities
                    .values()
                    .any(|a| a.workspace == Some(workspace_id))
                    || w.status == CreationStatus::Ready
                    || (w.status == CreationStatus::Queued
                        && !state.workspace_faults.contains_key(&workspace_id))
                {
                    return Err(conflict());
                }
                w.status = CreationStatus::Queued;
                w
            }
        };
        self.state()?
            .workspace_changed(workspace.id, workspace.repository_id);
        if activity.owned
            && let Some(record) = self.state()?.activities.get_mut(&action.id())
        {
            record.workspace = Some(workspace.id);
        }
        activity.writing = true;
        let written = save(
            self.store.as_ref(),
            Write::WorkspaceSave {
                workspace: workspace.clone(),
                action: Some(action.clone()),
            },
        )
        .await;
        let saved = one::<(WorkspaceAction, Workspace)>(
            self.store.as_ref(),
            Read::WorkspaceAction(action.id()),
        )
        .await?;
        activity.writing = false;
        let Some((saved_action, workspace)) = saved else {
            written?;
            return Err(RepositoryError::storage("Workspace intent is absent"));
        };
        self.state()?.workspace_faults.remove(&workspace.id);
        activity.transfer(Some(workspace.id));
        Ok(WorkspaceReply::Action {
            action: Some(saved_action),
            workspace: Some(self.state()?.workspace_observation(workspace)),
        })
    }
}
fn conflict() -> RepositoryError {
    RepositoryError::new(
        ErrorCode::Conflict,
        "Action identity, payload or workspace state conflicts; retrieve its exact status",
    )
}
fn fenced() -> RepositoryError {
    RepositoryError::storage("Workspace admission is fenced")
}
#[cfg(test)]
#[path = "workspaces/tests.rs"]
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
