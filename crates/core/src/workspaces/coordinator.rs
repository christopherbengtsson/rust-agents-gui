//! Workspace-owned execution and completion. No requester owns a physical effect.
use super::*;
use futures_util::FutureExt;
use std::sync::Weak;
use tokio::sync::{Mutex as AsyncMutex, mpsc, oneshot};

pub(super) trait Effects: Send + Sync {
    fn inspect(&self, path: &str) -> Result<ValidatedRepository>;
    fn candidate(
        &self,
        repository: &RepositoryRecord,
        root: &Path,
        directory: &Path,
        folder: &str,
        name: &str,
        folder_saved: bool,
    ) -> Result<naming::Occupancy>;
    fn creation(&self) -> Box<dyn CreationEffects + Send>;
}
pub(super) struct LocalEffects;
impl Effects for LocalEffects {
    fn inspect(&self, path: &str) -> Result<ValidatedRepository> {
        crate::git::inspect(path)
    }
    fn candidate(
        &self,
        repository: &RepositoryRecord,
        root: &Path,
        directory: &Path,
        folder: &str,
        name: &str,
        folder_saved: bool,
    ) -> Result<naming::Occupancy> {
        crate::git::candidate_occupied(repository, root, directory, folder, name, folder_saved)
    }
    fn creation(&self) -> Box<dyn CreationEffects + Send> {
        Box::new(crate::git::LocalCreation)
    }
}

#[derive(Clone)]
struct WorkspaceExecution {
    shared: Arc<Mutex<OwnerState>>,
    decision: Gate,
    store: Arc<dyn Metadata>,
    effects: Arc<dyn Effects>,
    action: ActionId,
}
type Gate = Arc<AsyncMutex<()>>;
#[derive(Default)]
struct RepositoryGates(BTreeMap<String, Weak<AsyncMutex<()>>>);
impl RepositoryGates {
    fn gate(&mut self, common: &str) -> Gate {
        self.0.retain(|_, gate| gate.strong_count() > 0);
        if let Some(gate) = self.0.get(common).and_then(Weak::upgrade) {
            return gate;
        }
        let gate = Arc::new(AsyncMutex::new(()));
        self.0.insert(common.into(), Arc::downgrade(&gate));
        gate
    }
}
fn validate(state: &OwnerState, id: ActionId, generation: u64) -> Result<()> {
    if state.fenced
        || state.generation != generation
        || !state
            .activities
            .get(&id)
            .is_some_and(|a| a.generation == generation)
    {
        return Err(interrupted());
    }
    Ok(())
}
fn interrupted() -> RepositoryError {
    RepositoryError::new(
        ErrorCode::Unavailable,
        "Workspace start was fenced before its next effect; reconcile the saved action explicitly",
    )
}
fn retain(state: &mut OwnerState, id: ActionId, workspace: Option<WorkspaceId>) -> Result<u64> {
    if state.fenced {
        return Err(fenced());
    }
    let generation = state.generation;
    if let Some(activity) = state.activities.get_mut(&id) {
        if activity.running || activity.generation != generation || activity.workspace != workspace
        {
            return Err(conflict());
        }
        activity.running = true;
        return Ok(generation);
    }
    if workspace.is_some_and(|id| state.activities.values().any(|a| a.workspace == Some(id))) {
        return Err(conflict());
    }
    state.activities.insert(
        id,
        Activity {
            generation,
            workspace,
            running: true,
        },
    );
    Ok(generation)
}
enum CompletedWork {
    Activity {
        id: ActionId,
        generation: u64,
        workspace: Option<WorkspaceId>,
    },
    ReadOnly,
}
struct Completion {
    work: CompletedWork,
    result: Result<()>,
}

pub(super) async fn run(
    shared: Arc<Mutex<OwnerState>>,
    decision: Gate,
    store: Arc<dyn Metadata>,
    mut wake: mpsc::Receiver<()>,
    effects: Arc<dyn Effects>,
) -> Result<()> {
    let mut tasks = tokio::task::JoinSet::new();
    let mut gates = RepositoryGates::default();
    let (mut registration_after, mut workspace_after) = (None, None);
    let mut inspection_active = false;
    loop {
        // Drain completions every turn, even under continuously queued work.
        while let Some(joined) = tasks.try_join_next() {
            settle(&shared, joined, &mut inspection_active);
        }
        let fenced = lock_state(&shared)?.fenced;
        let mut dispatched = false;
        if !fenced && tasks.len() < PENDING_LIMIT {
            let _gate = decision.lock().await;
            if !lock_state(&shared)?.fenced {
                let result = async {
                    let (registrations, next) = page::<Registration>(
                        store.as_ref(),
                        Read::PendingRegistrations {
                            after: registration_after,
                        },
                    )
                    .await?;
                    registration_after = parse_action_cursor(next)?;
                    for action in registrations {
                        if tasks.len() == PENDING_LIMIT {
                            break;
                        }
                        let generation = {
                            let mut state = lock_state(&shared)?;
                            if state
                                .activities
                                .get(&action.action_id)
                                .is_some_and(|a| a.running)
                            {
                                continue;
                            }
                            retain(&mut state, action.action_id, None)?
                        };
                        let (state, gate, storage, effects) = (
                            shared.clone(),
                            decision.clone(),
                            store.clone(),
                            effects.clone(),
                        );
                        let id = action.action_id;
                        tasks.spawn(async move {
                            let result = std::panic::AssertUnwindSafe(registration(
                                state, gate, storage, action, generation, effects,
                            ))
                            .catch_unwind()
                            .await
                            .unwrap_or_else(|_| Err(panicked()));
                            Completion {
                                work: CompletedWork::Activity {
                                    id,
                                    generation,
                                    workspace: None,
                                },
                                result,
                            }
                        });
                        dispatched = true;
                    }
                    let (workspaces, next) = page::<Workspace>(
                        store.as_ref(),
                        Read::PendingWorkspaces {
                            after: workspace_after,
                        },
                    )
                    .await?;
                    workspace_after = parse_workspace_cursor(next)?;
                    for workspace in workspaces {
                        if tasks.len() == PENDING_LIMIT {
                            break;
                        }
                        let id = {
                            let state = lock_state(&shared)?;
                            state
                                .activities
                                .iter()
                                .find(|(_, a)| a.workspace == Some(workspace.id))
                                .map_or(workspace.creation_action_id, |(id, _)| *id)
                        };
                        let generation = {
                            let mut state = lock_state(&shared)?;
                            if state.workspace_faults.contains_key(&workspace.id)
                                || state.activities.get(&id).is_some_and(|a| a.running)
                            {
                                continue;
                            }
                            retain(&mut state, id, Some(workspace.id))?
                        };
                        // Register activity before awaiting another owner. Failed preparation
                        // is retired by this same coordinator, never left provisional.
                        let repository = one::<RepositoryRecord>(
                            store.as_ref(),
                            Read::Repository(workspace.repository_id),
                        )
                        .await;
                        let repository = match repository {
                            Ok(Some(r)) => r,
                            other => {
                                let result = other.and_then(|_| {
                                    Err(RepositoryError::storage("Workspace repository missing"))
                                });
                                settle(
                                    &shared,
                                    Ok(Completion {
                                        work: CompletedWork::Activity {
                                            id,
                                            generation,
                                            workspace: Some(workspace.id),
                                        },
                                        result,
                                    }),
                                    &mut inspection_active,
                                );
                                continue;
                            }
                        };
                        let repository_gate = gates.gate(&repository.common_dir);
                        let (state, gate, storage, effects) = (
                            shared.clone(),
                            decision.clone(),
                            store.clone(),
                            effects.clone(),
                        );
                        let workspace_id = workspace.id;
                        tasks.spawn(async move {
                            let result = std::panic::AssertUnwindSafe(creation(
                                WorkspaceExecution {
                                    shared: state,
                                    decision: gate,
                                    store: storage,
                                    effects,
                                    action: id,
                                },
                                workspace,
                                repository,
                                repository_gate,
                                generation,
                            ))
                            .catch_unwind()
                            .await
                            .unwrap_or_else(|_| Err(panicked()));
                            Completion {
                                work: CompletedWork::Activity {
                                    id,
                                    generation,
                                    workspace: Some(workspace_id),
                                },
                                result,
                            }
                        });
                        dispatched = true;
                    }
                    Ok::<(), RepositoryError>(())
                }
                .await;
                if let Err(error) = result {
                    let mut state = lock_state(&shared)?;
                    state.fenced = true;
                    state.fault = Some(error);
                }
            }
        }
        // Read-only inspection keeps its existing bounded queue and one physical
        // slot; it no longer blocks independent registration/creation work.
        if !fenced && !inspection_active {
            let _gate = decision.lock().await;
            let mut state = lock_state(&shared)?;
            if !state.fenced && state.inspections.has_pending() {
                inspection_active = true;
                let shared = shared.clone();
                tasks.spawn_blocking(move || {
                    let result = catch_effect(|| {
                        inspection::process_inspection(&shared, &mut crate::git::LocalInspection);
                        Ok(())
                    });
                    Completion {
                        work: CompletedWork::ReadOnly,
                        result,
                    }
                });
                dispatched = true;
            } else if !state.fenced
                && let Some(record) = state.probes.pop_front()
            {
                inspection_active = true;
                let shared = shared.clone();
                let effects = effects.clone();
                tasks.spawn_blocking(move || {
                    let result = probe(&shared, record, effects.as_ref());
                    Completion {
                        work: CompletedWork::ReadOnly,
                        result,
                    }
                });
                dispatched = true;
            }
        }
        if wake.is_closed() && tasks.is_empty() {
            break;
        }
        if dispatched {
            tokio::task::yield_now().await;
            continue;
        }
        tokio::select! {
            joined = tasks.join_next(), if !tasks.is_empty() => { if let Some(joined) = joined { settle(&shared, joined, &mut inspection_active); } }
            message = wake.recv(), if !wake.is_closed() => { let _ = message; }
            else => { break; }
        }
    }
    let state = lock_state(&shared)?;
    let mut errors: Vec<String> = state.fault.iter().map(|e| e.message.clone()).collect();
    errors.extend(
        state
            .workspace_faults
            .iter()
            .map(|(id, e)| format!("Workspace {}: {}", id.0, e.message)),
    );
    errors.extend(state.cleanup_faults.values().map(|e| e.message.clone()));
    if errors.is_empty() {
        Ok(())
    } else {
        Err(RepositoryError::storage(errors.join("; ")))
    }
}
fn parse_action_cursor(next: Option<String>) -> Result<Option<ActionId>> {
    next.map(|s| {
        Uuid::parse_str(&s)
            .map(ActionId)
            .map_err(RepositoryError::storage)
    })
    .transpose()
}
fn parse_workspace_cursor(next: Option<String>) -> Result<Option<WorkspaceId>> {
    next.map(|s| {
        Uuid::parse_str(&s)
            .map(WorkspaceId)
            .map_err(RepositoryError::storage)
    })
    .transpose()
}
fn panicked() -> RepositoryError {
    RepositoryError::new(
        ErrorCode::CleanupUnconfirmed,
        "Owned workspace task panicked; physical effects require reconciliation",
    )
}
fn settle(
    shared: &Mutex<OwnerState>,
    joined: std::result::Result<Completion, tokio::task::JoinError>,
    inspection_active: &mut bool,
) {
    let mut state = shared.lock().unwrap_or_else(|e| e.into_inner());
    match joined {
        Ok(completion) => {
            let (action, workspace) = match completion.work {
                CompletedWork::ReadOnly => {
                    *inspection_active = false;
                    (None, None)
                }
                CompletedWork::Activity {
                    id,
                    generation,
                    workspace,
                } => {
                    if !state
                        .activities
                        .get(&id)
                        .is_some_and(|a| a.generation == generation)
                    {
                        return;
                    }
                    state.activities.remove(&id);
                    (Some(id), workspace)
                }
            };
            if let Err(error) = completion.result {
                if let Some(id) = workspace {
                    state.workspace_faults.insert(id, error.clone());
                }
                if error.code == ErrorCode::CleanupUnconfirmed {
                    state
                        .cleanup_faults
                        .insert(format!("work {action:?}"), error.clone());
                }
                state.fault = Some(error);
                state.fenced = true;
            }
            let _ = state.change();
            let mut targets = vec![
                crate::observations::Target::Repositories,
                crate::observations::Target::Workspaces {
                    repository_id: None,
                },
            ];
            if let Some(action_id) = action {
                if let Some(workspace_id) = workspace {
                    targets.push(crate::observations::Target::Workspace { workspace_id });
                    targets.push(crate::observations::Target::WorkspaceAction { action_id });
                } else {
                    targets.push(crate::observations::Target::Registration { action_id });
                }
            }
            state.publish(targets);
        }
        Err(error) => {
            state.fault = Some(RepositoryError::storage(error));
            state.fenced = true;
        }
    }
    let journal = state.journal.clone();
    drop(state);
    journal.wake();
}

fn probe(
    shared: &Mutex<OwnerState>,
    record: RepositoryRecord,
    effects: &dyn Effects,
) -> Result<()> {
    let result = catch_effect(|| effects.inspect(&record.path));
    if let Err(error) = &result
        && error.code == ErrorCode::CleanupUnconfirmed
    {
        return Err(error.clone());
    }
    let availability = match result {
        Ok(actual) if actual.path == record.path && actual.common_dir == record.common_dir => {
            Availability::Available
        }
        Ok(_) => Availability::Unavailable {
            error: RepositoryError::new(
                ErrorCode::Unavailable,
                "Registered location identifies a different checkout; restore its original location",
            ),
        },
        Err(error) => Availability::Unavailable { error },
    };
    lock_state(shared)?.cache(record.id, availability)
}
async fn registration(
    shared: Arc<Mutex<OwnerState>>,
    decision: Gate,
    store: Arc<dyn Metadata>,
    mut action: Registration,
    generation: u64,
    effects: Arc<dyn Effects>,
) -> Result<()> {
    let path = action.path.clone();
    let validation = tokio::task::spawn_blocking(move || catch_effect(|| effects.inspect(&path)))
        .await
        .map_err(RepositoryError::storage)?;
    let cleanup_fault = validation
        .as_ref()
        .err()
        .filter(|e| e.code == ErrorCode::CleanupUnconfirmed)
        .cloned();
    let _gate = decision.lock().await;
    let admission = { validate(&*lock_state(&shared)?, action.action_id, generation) };
    if let Err(error) = admission {
        action.state = RegistrationState::Failed {
            error: cleanup_fault.clone().unwrap_or(error),
        };
        save(
            store.as_ref(),
            Write::RegistrationFinish {
                action,
                repository: None,
                initial: None,
            },
        )
        .await?;
        return cleanup_fault.map_or(Ok(()), Err);
    }
    let mut initial = None;
    let repository = match validation {
        Ok(validated) => {
            let existing = one::<RepositoryRecord>(
                store.as_ref(),
                Read::CommonDirectory(validated.common_dir.clone()),
            )
            .await?;
            let repository = match existing {
                Some(repository) => repository,
                None => {
                    let id = RepositoryId(Uuid::new_v4());
                    if action.create_initial_workspace {
                        initial = Some(allocate(
                            id,
                            ActionId(Uuid::new_v4()),
                            &lock_state(&shared)?.directory,
                        )?);
                        action.initial_workspace_id = initial.as_ref().map(|w| w.id);
                    }
                    RepositoryRecord {
                        id,
                        name: validated.name,
                        path: validated.path,
                        common_dir: validated.common_dir,
                    }
                }
            };
            action.state = RegistrationState::Completed {
                repository_id: repository.id,
            };
            Some(repository)
        }
        Err(error) => {
            action.state = RegistrationState::Failed { error };
            None
        }
    };
    let written = save(
        store.as_ref(),
        Write::RegistrationFinish {
            action: action.clone(),
            repository,
            initial,
        },
    )
    .await;
    let recorded =
        one::<Registration>(store.as_ref(), Read::Registration(action.action_id)).await?;
    if recorded.as_ref() != Some(&action) {
        written?;
        return Err(RepositoryError::storage(
            "Registration outcome was not committed",
        ));
    }
    let mut state = lock_state(&shared)?;
    state.publish(vec![
        crate::observations::Target::Registration {
            action_id: action.action_id,
        },
        crate::observations::Target::Repositories,
        crate::observations::Target::Workspaces {
            repository_id: None,
        },
    ]);
    state.change()?;
    cleanup_fault.map_or(Ok(()), Err)
}

const ATTEMPTS_PER_TURN: usize = 32;
async fn creation(
    execution: WorkspaceExecution,
    workspace: Workspace,
    repository: RepositoryRecord,
    repository_gate: Gate,
    generation: u64,
) -> Result<()> {
    let WorkspaceExecution {
        shared,
        decision,
        store,
        ..
    } = execution.clone();
    let workspace_id = workspace.id;
    let action_id = execution.action;
    let result = creation_inner(
        execution,
        workspace,
        repository,
        repository_gate,
        generation,
    )
    .await;
    if let Err(error) = &result
        && error.code == ErrorCode::Unavailable
    {
        let _gate = decision.lock().await;
        if !lock_state(&shared)?
            .activities
            .get(&action_id)
            .is_some_and(|a| a.generation == generation)
        {
            return Err(conflict());
        }
        let mut saved = one::<Workspace>(store.as_ref(), Read::Workspace(workspace_id))
            .await?
            .ok_or_else(|| RepositoryError::storage("Interrupted workspace is missing"))?;
        saved.status = CreationStatus::Uncertain {
            error: error.clone(),
        };
        save(
            store.as_ref(),
            Write::WorkspaceSave {
                workspace: saved,
                action: None,
            },
        )
        .await?;
        return Ok(());
    }
    result
}
async fn creation_inner(
    execution: WorkspaceExecution,
    mut workspace: Workspace,
    repository: RepositoryRecord,
    repository_gate: Gate,
    generation: u64,
) -> Result<()> {
    let WorkspaceExecution {
        shared,
        decision,
        store,
        effects: _,
        action,
    } = execution.clone();
    while workspace.name.is_none() {
        workspace =
            match allocate_turn(&execution, workspace.clone(), &repository, generation).await {
                Ok(workspace) => workspace,
                Err(error)
                    if matches!(
                        error.code,
                        ErrorCode::Git | ErrorCode::Conflict | ErrorCode::InvalidPath
                    ) =>
                {
                    workspace.status = CreationStatus::Failed { error };
                    save(
                        store.as_ref(),
                        Write::WorkspaceSave {
                            workspace,
                            action: None,
                        },
                    )
                    .await?;
                    return Ok(());
                }
                Err(error) => return Err(error),
            };
        if workspace.name.is_none() {
            tokio::task::yield_now().await;
        }
    }
    // The Arc is retained by waiters as well as the running physical operation.
    let repository_guard = repository_gate.lock_owned().await;
    {
        let _gate = decision.lock().await;
        validate(&*lock_state(&shared)?, action, generation)?;
    }
    if !workspace.cleanup_unconfirmed
        && one::<bool>(store.as_ref(), Read::CleanupBlocked(repository.id))
            .await?
            .unwrap_or(false)
    {
        workspace.status = CreationStatus::Uncertain {
            error: RepositoryError::new(
                ErrorCode::CleanupUnconfirmed,
                "Another workspace has unconfirmed repository cleanup; reconcile it first",
            ),
        };
        save(
            store.as_ref(),
            Write::WorkspaceSave {
                workspace,
                action: None,
            },
        )
        .await?;
        return Ok(());
    }
    drive_owned(
        execution,
        workspace,
        repository,
        generation,
        repository_guard,
    )
    .await
}
async fn allocate_turn(
    execution: &WorkspaceExecution,
    mut workspace: Workspace,
    repository: &RepositoryRecord,
    generation: u64,
) -> Result<Workspace> {
    let shared = execution.shared.as_ref();
    let decision = execution.decision.as_ref();
    let store = execution.store.as_ref();
    let id = execution.action;
    let root = lock_state(shared)?.directory.join("workspaces");
    let mut cursor =
        one::<crate::storage::CandidateCursor>(store, Read::AllocationCursor(workspace.id))
            .await?
            .unwrap_or_default();
    for _ in 0..ATTEMPTS_PER_TURN {
        {
            let _gate = decision.lock().await;
            validate(&*lock_state(shared)?, id, generation)?;
        }
        let folder_candidate = naming::numbered(
            &naming::slug(&repository.name),
            cursor.folder.checked_add(1).ok_or_else(|| {
                RepositoryError::new(ErrorCode::Capacity, "Folder cursor exhausted")
            })?,
        );
        let (folders, _) = page::<(String, String)>(
            store,
            Read::Folder {
                repository: repository.id,
                candidate: folder_candidate.clone(),
            },
        )
        .await?;
        let existing = folders
            .iter()
            .find(|(_, owner)| *owner == repository.id.0.to_string())
            .map(|(folder, _)| folder.clone());
        let folder = existing.clone().unwrap_or(folder_candidate);
        if existing.is_none() && !folders.is_empty() {
            cursor.folder = advance(cursor.folder)?;
            continue;
        }
        let name = naming::candidate(cursor.name, workspace.id.0.as_u128())?;
        if one::<Workspace>(
            store,
            Read::Name {
                repository: repository.id,
                candidate: name.clone(),
            },
        )
        .await?
        .is_some()
        {
            cursor.name = advance(cursor.name)?;
            continue;
        }
        let directory = root.join(&folder);
        let (repo, dir, root_probe, name_probe, folder_probe) = (
            repository.clone(),
            directory.clone(),
            root.clone(),
            name.clone(),
            folder.clone(),
        );
        let effects = execution.effects.clone();
        let occupied = tokio::task::spawn_blocking(move || {
            catch_effect(|| {
                effects.candidate(
                    &repo,
                    &root_probe,
                    &dir,
                    &folder_probe,
                    &name_probe,
                    existing.is_some(),
                )
            })
        })
        .await
        .map_err(RepositoryError::storage)??;
        match occupied {
            naming::Occupancy::RepositoryFolder => {
                cursor.folder = advance(cursor.folder)?;
                continue;
            }
            naming::Occupancy::WorkspaceName => {
                cursor.name = advance(cursor.name)?;
                continue;
            }
            naming::Occupancy::Available => {}
        }
        let mut candidate = workspace.clone();
        candidate.name = Some(name.clone());
        candidate.branch = Some(format!("workspace/{name}"));
        candidate.path = Some(
            directory
                .join(&name)
                .join("checkout")
                .to_str()
                .ok_or_else(|| {
                    RepositoryError::new(ErrorCode::InvalidPath, "Private path must be UTF-8")
                })?
                .into(),
        );
        check_workspace_budget(&candidate)?;
        let _gate = decision.lock().await;
        validate(&*lock_state(shared)?, id, generation)?;
        let outcome = store
            .write(Write::Reservation(crate::storage::Reservation {
                workspace: candidate.clone(),
                folder,
                continuation: cursor,
            }))
            .await;
        let outcome = outcome.and_then(|delivery| delivery.revision());
        match outcome {
            Ok(_) => {
                workspace = candidate;
                break;
            }
            Err(error) => {
                // Always settle/read the same action before another candidate, even
                // for a normal rolled-back conflict. Unknown cleanup fences this read.
                let saved = one::<Workspace>(store, Read::Workspace(workspace.id))
                    .await?
                    .ok_or_else(|| RepositoryError::storage("Pending workspace disappeared"))?;
                if saved.name.is_some() {
                    workspace = saved;
                    break;
                }
                if !matches!(error, crate::storage::Error::CandidateConflict) {
                    return Err(RepositoryError::storage(error));
                }
                // The next indexed reads identify which namespace changed.
                // Never consume an unrelated name or folder candidate here.
            }
        }
    }
    if workspace.name.is_none() {
        save(
            store,
            Write::AllocationCursor {
                workspace_id: workspace.id,
                cursor,
            },
        )
        .await?;
    }
    Ok(workspace)
}
fn advance(cursor: u64) -> Result<u64> {
    cursor
        .checked_add(1)
        .filter(|n| *n <= i64::MAX as u64)
        .ok_or_else(|| RepositoryError::new(ErrorCode::Capacity, "Allocation cursor exhausted"))
}

struct Persist {
    workspace: Workspace,
    before_effect: bool,
    reply: oneshot::Sender<Result<()>>,
}
async fn drive_owned(
    execution: WorkspaceExecution,
    workspace: Workspace,
    repository: RepositoryRecord,
    generation: u64,
    repository_guard: tokio::sync::OwnedMutexGuard<()>,
) -> Result<()> {
    let WorkspaceExecution {
        shared,
        decision,
        store,
        effects,
        action,
    } = execution;
    let (send, mut receive) = mpsc::channel::<Persist>(1);
    // This worker keeps its physical completion handle, repository guard (in the
    // caller), and capacity until joined. There is no per-request runtime/block_on.
    let mut worker = tokio::task::spawn_blocking(move || {
        let _repository_guard = repository_guard;
        let mut effects = effects.creation();
        catch_effect(|| {
            creation::drive(
                workspace,
                &repository,
                effects.as_mut(),
                |workspace, before_effect| {
                    let (reply, result) = oneshot::channel();
                    send.blocking_send(Persist {
                        workspace: workspace.clone(),
                        before_effect,
                        reply,
                    })
                    .map_err(RepositoryError::storage)?;
                    result.blocking_recv().map_err(RepositoryError::storage)?
                },
            )
        })
    });
    loop {
        tokio::select! {
            result = &mut worker => { return creation_outcome(result.map_err(RepositoryError::storage)?); }
            request = receive.recv() => {
                let Some(request) = request else { return creation_outcome(worker.await.map_err(RepositoryError::storage)?); };
                let result = std::panic::AssertUnwindSafe(async {
                    let _gate = decision.lock().await;
                    if request.before_effect { validate(&*lock_state(&shared)?, action, generation)?; }
                    let result = save(store.as_ref(), Write::WorkspaceSave { workspace: request.workspace.clone(), action: None }).await;
                    if let Err(error) = result {
                        let saved = one::<Workspace>(store.as_ref(), Read::Workspace(request.workspace.id)).await?;
                        if saved.as_ref() != Some(&request.workspace) { return Err(error); }
                    }
                    let mut state = lock_state(&shared)?;
                    state.workspace_changed(request.workspace.id, request.workspace.repository_id); state.change()
                }).catch_unwind().await.unwrap_or_else(|_| Err(panicked()));
                let _ = request.reply.send(result);
            }
        }
    }
}

fn creation_outcome(result: Result<Workspace>) -> Result<()> {
    let workspace = result?;
    if workspace.cleanup_unconfirmed {
        if let CreationStatus::Uncertain { error } = workspace.status {
            return Err(error);
        }
        return Err(panicked());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_completion_cannot_retire_new_activity_or_publish_old_fault() {
        let shared = Mutex::new(OwnerState::new("/state".into()));
        let id = ActionId(Uuid::from_u128(1));
        {
            let mut state = shared.lock().unwrap();
            state.generation = 2;
            retain(&mut state, id, None).unwrap();
        }
        let before = shared.lock().unwrap().journal.watermark();
        settle(
            &shared,
            Ok(Completion {
                work: CompletedWork::Activity {
                    id,
                    generation: 1,
                    workspace: None,
                },
                result: Err(panicked()),
            }),
            &mut false,
        );
        let state = shared.lock().unwrap();
        assert_eq!(state.activities[&id].generation, 2);
        assert!(state.fault.is_none());
        assert_eq!(state.journal.watermark(), before);
    }
    #[tokio::test]
    async fn keyed_gate_survives_all_waiters_and_retires_only_after_physical_release() {
        let mut gates = RepositoryGates::default();
        let gate = gates.gate("/common");
        let held = gate.clone().lock_owned().await;
        let waiter = gates.gate("/common");
        drop(gate);
        assert!(Arc::ptr_eq(&waiter, &gates.gate("/common")));
        assert!(waiter.try_lock().is_err());
        drop(held);
        drop(waiter);
        gates.gate("/other");
        assert!(!gates.0.contains_key("/common"));
    }
    #[test]
    fn cursor_exhaustion_is_checked_and_never_wraps_to_an_old_candidate() {
        assert!(advance(u64::MAX).is_err());
        assert!(advance(u64::try_from(i64::MAX).unwrap()).is_err());
    }
}
