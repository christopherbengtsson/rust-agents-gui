//! Controlled storage/filesystem/Git outcomes; no real effects in application units.
use super::*;
use crate::storage::{Delivery, Error};
use futures_util::future::BoxFuture;
use serde_json::{Value, json};
use std::sync::{
    Condvar,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Default)]
struct Data {
    registrations: BTreeMap<ActionId, Registration>,
    repositories: BTreeMap<RepositoryId, RepositoryRecord>,
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    actions: BTreeMap<ActionId, WorkspaceAction>,
    folders: BTreeMap<RepositoryId, String>,
    cursors: BTreeMap<WorkspaceId, u64>,
    claims: usize,
    claim_conflicts: usize,
    lost_claim: bool,
    fail_save: bool,
    writes: Vec<&'static str>,
}
#[derive(Default)]
struct Store(Mutex<Data>);
fn values<T: Serialize>(values: impl IntoIterator<Item = T>) -> Vec<Value> {
    values
        .into_iter()
        .map(|v| serde_json::to_value(v).unwrap())
        .collect()
}
impl Metadata for Store {
    fn read(&self, read: Read) -> BoxFuture<'_, std::result::Result<Delivery, Error>> {
        Box::pin(async move {
            let data = self.0.lock().unwrap();
            let (mut records, paginated) = match read {
                Read::Registration(id) => (values(data.registrations.get(&id)), false),
                Read::Repository(id) => (values(data.repositories.get(&id)), false),
                Read::CommonDirectory(common) => (
                    values(
                        data.repositories
                            .values()
                            .filter(|r| r.common_dir == common),
                    ),
                    false,
                ),
                Read::Repositories { after } => (
                    values(
                        data.repositories
                            .values()
                            .filter(|r| after.is_none_or(|a| r.id > a)),
                    ),
                    true,
                ),
                Read::PendingRegistrations { after } => (
                    values(data.registrations.values().filter(|r| {
                        r.state == RegistrationState::Pending
                            && after.is_none_or(|a| r.action_id > a)
                    })),
                    true,
                ),
                Read::Workspace(id) => (values(data.workspaces.get(&id)), false),
                Read::WorkspaceAction(id) => (
                    values(data.actions.get(&id).and_then(|a| {
                        data.workspaces
                            .values()
                            .find(|w| match a {
                                WorkspaceAction::Create { action_id, .. } => {
                                    w.creation_action_id == *action_id
                                }
                                WorkspaceAction::Reconcile { workspace_id, .. } => {
                                    w.id == *workspace_id
                                }
                            })
                            .map(|w| (a, w))
                    })),
                    false,
                ),
                Read::Workspaces { repository, after } => (
                    values(data.workspaces.values().filter(|w| {
                        w.repository_id == repository && after.is_none_or(|a| w.id > a)
                    })),
                    true,
                ),
                Read::PendingWorkspaces { after } => (
                    values(data.workspaces.values().filter(|w| {
                        w.status == CreationStatus::Queued && after.is_none_or(|a| w.id > a)
                    })),
                    true,
                ),
                Read::Folder {
                    repository,
                    candidate,
                } => (
                    values(
                        data.folders
                            .iter()
                            .filter(|(id, folder)| **id == repository || **folder == candidate)
                            .map(|(id, folder)| (folder, id.0.to_string())),
                    ),
                    false,
                ),
                Read::Name {
                    repository,
                    candidate,
                } => (
                    values(data.workspaces.values().filter(|w| {
                        w.repository_id == repository && w.name.as_deref() == Some(&candidate)
                    })),
                    false,
                ),
                Read::AllocationCursor(id) => (
                    vec![json!(data.cursors.get(&id).copied().unwrap_or(0))],
                    false,
                ),
                Read::CleanupBlocked(repository) => (
                    vec![json!(data.workspaces.values().any(|w| w.repository_id
                        == repository
                        && w.cleanup_unconfirmed))],
                    false,
                ),
                _ => panic!("Unrelated provider read"),
            };
            let next = if paginated && records.len() > 32 {
                records.truncate(32);
                records.last().map(|r| {
                    r.get("id")
                        .or_else(|| r.get("action_id"))
                        .unwrap()
                        .as_str()
                        .unwrap()
                        .to_string()
                })
            } else {
                None
            };
            Ok(Delivery::controlled(Ok(records), next))
        })
    }
    fn write(&self, write: Write) -> BoxFuture<'_, std::result::Result<Delivery, Error>> {
        Box::pin(async move {
            let mut data = self.0.lock().unwrap();
            match write {
                Write::RegistrationIntent(request) => {
                    data.writes.push("registration-intent");
                    data.registrations.insert(
                        request.action_id,
                        Registration {
                            create_initial_workspace: request.create_initial_workspace,
                            initial_workspace_id: None,
                            action_id: request.action_id,
                            path: request.path,
                            state: RegistrationState::Pending,
                        },
                    );
                }
                Write::RegistrationFinish {
                    action,
                    repository,
                    initial,
                } => {
                    data.writes.push("registration-finish");
                    if let Some(repository) = repository {
                        data.repositories.insert(repository.id, repository);
                    }
                    if let Some(w) = initial {
                        data.actions.insert(
                            w.creation_action_id,
                            WorkspaceAction::Create {
                                action_id: w.creation_action_id,
                                repository_id: w.repository_id,
                            },
                        );
                        data.workspaces.insert(w.id, w);
                    }
                    data.registrations.insert(action.action_id, action);
                }
                Write::WorkspaceSave { workspace, action } => {
                    data.writes.push("workspace-save");
                    if data.fail_save {
                        return Ok(Delivery::controlled(Err(Error::Fenced), None));
                    }
                    if let Some(action) = action {
                        data.actions.insert(action.id(), action);
                    }
                    data.workspaces.insert(workspace.id, workspace);
                }
                Write::Reservation(r) => {
                    data.claims += 1;
                    data.writes.push("claim");
                    if data.claim_conflicts > 0 {
                        data.claim_conflicts -= 1;
                        return Ok(Delivery::controlled(Err(Error::CandidateConflict), None));
                    }
                    data.folders.insert(r.workspace.repository_id, r.folder);
                    data.workspaces.insert(r.workspace.id, r.workspace);
                    if data.lost_claim {
                        data.lost_claim = false;
                        return Ok(Delivery::controlled(
                            Err(Error::Write {
                                original: Box::new(Error::Closed),
                                retirement: None,
                                uncertain: true,
                            }),
                            None,
                        ));
                    }
                }
                Write::AllocationCursor {
                    workspace_id,
                    cursor,
                } => {
                    data.writes.push("cursor");
                    data.cursors.insert(workspace_id, cursor);
                }
                _ => panic!("Unrelated provider write"),
            }
            // revision() accepts snapshots, allowing the unit seam to supply a
            // committed outcome without implementing transaction policy.
            Ok(Delivery::controlled(Ok(Vec::new()), None))
        })
    }
}
#[derive(Default)]
struct Block {
    released: Mutex<bool>,
    changed: Condvar,
}
impl Block {
    fn wait(&self) {
        let mut done = self.released.lock().unwrap();
        while !*done {
            done = self.changed.wait(done).unwrap();
        }
    }
    fn release(&self) {
        *self.released.lock().unwrap() = true;
        self.changed.notify_all();
    }
}
struct Effects {
    calls: Arc<AtomicUsize>,
    probes: Arc<AtomicUsize>,
    blocks: Arc<Mutex<BTreeMap<WorkspaceId, Arc<Block>>>>,
    started: tokio::sync::mpsc::UnboundedSender<WorkspaceId>,
    validation_block: Option<Arc<Block>>,
    occupied: bool,
}
impl coordinator::Effects for Effects {
    fn inspect(&self, path: &str) -> Result<ValidatedRepository> {
        self.probes.fetch_add(1, Ordering::SeqCst);
        if let Some(block) = &self.validation_block {
            block.wait();
        }
        Ok(ValidatedRepository {
            name: "app".into(),
            path: path.into(),
            common_dir: format!("{path}/.git"),
        })
    }
    fn candidate(
        &self,
        _: &RepositoryRecord,
        _: &Path,
        _: &Path,
        _: &str,
        _: &str,
        _: bool,
    ) -> Result<bool> {
        Ok(self.occupied)
    }
    fn creation(&self) -> Box<dyn CreationEffects + Send> {
        Box::new(Creation {
            calls: self.calls.clone(),
            blocks: self.blocks.clone(),
            started: self.started.clone(),
            artifacts: Artifacts {
                branch: BranchArtifact::Absent,
                folder: FolderArtifact::Absent,
            },
        })
    }
}
struct Creation {
    calls: Arc<AtomicUsize>,
    blocks: Arc<Mutex<BTreeMap<WorkspaceId, Arc<Block>>>>,
    started: tokio::sync::mpsc::UnboundedSender<WorkspaceId>,
    artifacts: Artifacts,
}
impl CreationEffects for Creation {
    fn quiescent(&mut self, _: &Workspace) -> Result<bool> {
        Ok(true)
    }
    fn base(&mut self, _: &RepositoryRecord) -> Result<PinnedBase> {
        Ok(PinnedBase {
            reference: "refs/heads/main".into(),
            commit: "abc".into(),
        })
    }
    fn inspect(&mut self, _: &RepositoryRecord, _: &Workspace) -> Result<Artifacts> {
        Ok(self.artifacts.clone())
    }
    fn reserve(&mut self, w: &Workspace) -> Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        let _ = self.started.send(w.id);
        let block = self.blocks.lock().unwrap().get(&w.id).cloned();
        if let Some(block) = block {
            block.wait();
        }
        self.artifacts.folder = FolderArtifact::Reserved;
        Ok(())
    }
    fn branch(&mut self, _: &RepositoryRecord, _: &Workspace) -> Result<()> {
        self.artifacts.branch = BranchArtifact::Owned;
        Ok(())
    }
    fn worktree(&mut self, _: &RepositoryRecord, _: &Workspace) -> Result<()> {
        self.artifacts.folder = FolderArtifact::Worktree;
        Ok(())
    }
}
fn repository(id: u128) -> RepositoryRecord {
    RepositoryRecord {
        id: RepositoryId(Uuid::from_u128(id)),
        name: "app".into(),
        path: format!("/repo-{id}"),
        common_dir: format!("/repo-{id}/.git"),
    }
}
fn queued(repo: RepositoryId, action: u128) -> Workspace {
    allocate(repo, ActionId(Uuid::from_u128(action)), Path::new("/state")).unwrap()
}
fn dependencies() -> (
    Arc<Store>,
    Arc<Effects>,
    tokio::sync::mpsc::UnboundedReceiver<WorkspaceId>,
) {
    let (started, receive) = tokio::sync::mpsc::unbounded_channel();
    (
        Arc::default(),
        Arc::new(Effects {
            calls: Arc::default(),
            probes: Arc::default(),
            blocks: Arc::default(),
            started,
            validation_block: None,
            occupied: false,
        }),
        receive,
    )
}
fn owner(store: Arc<Store>, effects: Arc<Effects>) -> Workspaces {
    Workspaces::with_dependencies("/state".into(), store, effects)
}
async fn ready(service: &Workspaces, id: WorkspaceId) {
    loop {
        if let WorkspaceReply::Workspace { workspace: Some(w) } = service
            .workspace_request(WorkspaceRequest::Get { id })
            .await
            .unwrap()
        {
            if w.status == CreationStatus::Ready {
                break;
            }
            assert!(
                !matches!(
                    w.status,
                    CreationStatus::Failed { .. } | CreationStatus::Uncertain { .. }
                ),
                "{w:?}"
            );
        }
        tokio::task::yield_now().await;
    }
}

#[tokio::test]
async fn independent_repository_effects_progress_while_another_is_blocked() {
    let (store, effects, mut started) = dependencies();
    let a = repository(1);
    let b = repository(2);
    let wa = queued(a.id, 11);
    let wb = queued(b.id, 12);
    let block = Arc::new(Block::default());
    effects.blocks.lock().unwrap().insert(wa.id, block.clone());
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(a.id, a);
        d.repositories.insert(b.id, b);
        d.workspaces.insert(wa.id, wa.clone());
    }
    let service = owner(store.clone(), effects.clone());
    assert_eq!(started.recv().await, Some(wa.id));
    store.0.lock().unwrap().workspaces.insert(wb.id, wb.clone());
    service.notify();
    assert_eq!(started.recv().await, Some(wb.id));
    ready(&service, wb.id).await;
    block.release();
    ready(&service, wa.id).await;
    service.finish().await.unwrap();
    assert_eq!(effects.calls.load(Ordering::SeqCst), 2);
}
#[tokio::test]
async fn actual_common_directory_serializes_waiters_and_retains_the_key() {
    let (store, effects, mut started) = dependencies();
    let a = repository(1);
    let mut b = repository(2);
    b.common_dir = a.common_dir.clone();
    let wa = queued(a.id, 11);
    let wb = queued(b.id, 12);
    let block = Arc::new(Block::default());
    effects.blocks.lock().unwrap().insert(wa.id, block.clone());
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(a.id, a);
        d.repositories.insert(b.id, b);
        d.workspaces.insert(wa.id, wa.clone());
    }
    let service = owner(store.clone(), effects.clone());
    assert_eq!(started.recv().await, Some(wa.id));
    store.0.lock().unwrap().workspaces.insert(wb.id, wb.clone());
    service.notify();
    loop {
        if service.state().unwrap().activities.len() == 2 {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(started.try_recv().is_err());
    block.release();
    assert_eq!(started.recv().await, Some(wb.id));
    ready(&service, wa.id).await;
    ready(&service, wb.id).await;
    service.finish().await.unwrap();
}
#[tokio::test]
async fn duplicate_ids_observe_exact_payload_and_cross_kind_ids_reject() {
    let (store, effects, _) = dependencies();
    let service = owner(store.clone(), effects);
    let request = RegisterRepository {
        create_initial_workspace: false,
        action_id: ActionId(Uuid::from_u128(10)),
        path: "/app".into(),
    };
    let first = service.register(request.clone()).await.unwrap();
    assert_eq!(
        first.action_id,
        service.register(request.clone()).await.unwrap().action_id
    );
    let mut changed = request.clone();
    changed.path = "/other".into();
    assert_eq!(
        service.register(changed).await.unwrap_err().code,
        ErrorCode::Conflict
    );
    assert!(
        service
            .admit_workspace(WorkspaceAction::Create {
                action_id: request.action_id,
                repository_id: repository(1).id
            })
            .await
            .is_err()
    );
    let id = ActionId(Uuid::from_u128(20));
    let repo = repository(1);
    store
        .0
        .lock()
        .unwrap()
        .repositories
        .insert(repo.id, repo.clone());
    let action = WorkspaceAction::Create {
        action_id: id,
        repository_id: repo.id,
    };
    service.admit_workspace(action.clone()).await.unwrap();
    service.admit_workspace(action).await.unwrap();
    let mut changed = request;
    changed.action_id = id;
    assert_eq!(
        service.register(changed).await.unwrap_err().code,
        ErrorCode::Conflict
    );
    service.finish().await.ok();
}
#[tokio::test]
async fn allocation_lost_ack_reconciles_the_same_saved_location_without_another_claim() {
    let (store, effects, _) = dependencies();
    let repo = repository(1);
    let w = queued(repo.id, 11);
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo);
        d.workspaces.insert(w.id, w.clone());
        d.lost_claim = true;
    }
    let service = owner(store.clone(), effects);
    ready(&service, w.id).await;
    service.finish().await.unwrap();
    assert_eq!(store.0.lock().unwrap().claims, 1);
}
#[tokio::test]
async fn allocation_turn_persists_cursor_and_does_not_cap_history_or_numeric_tiers() {
    let (store, effects, _) = dependencies();
    let repo = repository(1);
    let w = queued(repo.id, 11);
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo);
        d.workspaces.insert(w.id, w.clone());
        d.claim_conflicts = 33;
        d.cursors.insert(w.id, 1_000_000);
    }
    let service = owner(store.clone(), effects);
    ready(&service, w.id).await;
    service.finish().await.unwrap();
    let d = store.0.lock().unwrap();
    assert_eq!(d.claims, 34);
    assert!(d.writes.contains(&"cursor"));
    assert!(d.cursors[&w.id] >= 1_000_032);
}
#[tokio::test]
async fn immutable_reserved_location_skips_allocation_on_recovery() {
    let (store, effects, _) = dependencies();
    let repo = repository(1);
    let mut w = queued(repo.id, 11);
    w.name = Some("marta".into());
    w.branch = Some("workspace/marta".into());
    w.path = Some("/state/app/marta/checkout".into());
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo);
        d.workspaces.insert(w.id, w.clone());
    }
    let service = owner(store.clone(), effects);
    ready(&service, w.id).await;
    service.finish().await.unwrap();
    let d = store.0.lock().unwrap();
    assert_eq!(d.claims, 0);
    assert_eq!(d.workspaces[&w.id].path, w.path);
}
#[tokio::test]
async fn registration_and_first_creation_are_one_finish_and_duplicate_alias_does_not_create_again()
{
    let (store, effects, _) = dependencies();
    let service = owner(store.clone(), effects);
    let request = RegisterRepository {
        create_initial_workspace: true,
        action_id: ActionId(Uuid::from_u128(10)),
        path: "/app".into(),
    };
    service.register(request.clone()).await.unwrap();
    let initial = loop {
        let action = service.action(request.action_id).await.unwrap().unwrap();
        if let Some(id) = action.initial_workspace_id {
            break id;
        }
        tokio::task::yield_now().await;
    };
    ready(&service, initial).await;
    let mut alias = request;
    alias.action_id = ActionId(Uuid::from_u128(20));
    service.register(alias.clone()).await.unwrap();
    loop {
        if service
            .action(alias.action_id)
            .await
            .unwrap()
            .unwrap()
            .state
            != RegistrationState::Pending
        {
            break;
        }
        tokio::task::yield_now().await;
    }
    service.finish().await.unwrap();
    let d = store.0.lock().unwrap();
    assert_eq!(d.repositories.len(), 1);
    assert_eq!(d.workspaces.len(), 1);
    assert_eq!(
        d.writes
            .iter()
            .filter(|w| **w == "registration-finish")
            .count(),
        2
    );
}
#[tokio::test]
async fn failed_intent_never_dispatches_creation_and_status_does_not_replay() {
    let (store, effects, _) = dependencies();
    let repo = repository(1);
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo.clone());
        d.fail_save = true;
    }
    let service = owner(store.clone(), effects.clone());
    assert!(
        service
            .admit_workspace(WorkspaceAction::Create {
                action_id: ActionId(Uuid::from_u128(11)),
                repository_id: repo.id
            })
            .await
            .is_err()
    );
    assert!(store.0.lock().unwrap().workspaces.is_empty());
    assert_eq!(effects.calls.load(Ordering::SeqCst), 0);
    service.finish().await.unwrap();
}
#[tokio::test]
async fn quit_fences_next_mutation_and_waits_for_physical_work_then_repeats_same_result() {
    let (store, effects, mut started) = dependencies();
    let repo = repository(1);
    let w = queued(repo.id, 11);
    let block = Arc::new(Block::default());
    effects.blocks.lock().unwrap().insert(w.id, block.clone());
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo);
        d.workspaces.insert(w.id, w.clone());
    }
    let service = owner(store, effects);
    assert_eq!(started.recv().await, Some(w.id));
    service.fence().await;
    assert!(
        service
            .register(RegisterRepository {
                create_initial_workspace: false,
                action_id: ActionId(Uuid::from_u128(22)),
                path: "/other".into()
            })
            .await
            .is_err()
    );
    let completion = service.finish();
    tokio::pin!(completion);
    assert!(futures_util::poll!(&mut completion).is_pending());
    block.release();
    let first = completion.await;
    assert!(first.is_ok(), "{first:?}");
    assert_eq!(first, service.finish().await);
    assert!(service.state().unwrap().activities.is_empty());
}
#[tokio::test]
async fn navigation_is_paged_and_running_restore_is_uncertain_without_requeue() {
    let (store, effects, _) = dependencies();
    let repo = repository(1);
    let mut w = queued(repo.id, 11);
    w.status = CreationStatus::Running {
        stage: CreationStage::CreatingBranch,
    };
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo.clone());
        for n in 0..70 {
            let mut item = w.clone();
            item.id = WorkspaceId(Uuid::from_u128(n + 100));
            d.workspaces.insert(item.id, item);
        }
    }
    let service = owner(store, effects.clone());
    let WorkspaceReply::Workspaces { page } = service
        .workspace_request(WorkspaceRequest::List {
            repository_id: repo.id,
            after: None,
        })
        .await
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(page.workspaces.len(), 32);
    assert!(page.next.is_some());
    assert!(
        page.workspaces
            .iter()
            .all(|w| matches!(w.status, CreationStatus::Uncertain { .. }))
    );
    assert_eq!(effects.calls.load(Ordering::SeqCst), 0);
    service.finish().await.unwrap();
}
#[test]
fn utf8_path_budget_and_invalid_locations_reject_without_retargeting() {
    assert!(check_registration_path_budget(&"é".repeat(2049)).is_err());
    for path in ["relative", "/a/../b", "/a//b", "/a/./b"] {
        assert!(validate_workspace_path(path).is_err());
    }
}
pub(crate) fn controlled_completion(
    work: impl std::future::Future<Output = Result<()>> + Send + 'static,
) -> Workspaces {
    let (store, _, _) = dependencies();
    Workspaces {
        shared: Arc::new(Mutex::new(OwnerState::new("/state".into()))),
        storage: None,
        decision: Arc::new(tokio::sync::Mutex::new(())),
        store,
        wake: Mutex::new(None),
        worker: tokio::sync::Mutex::new(Some(tokio::spawn(work))),
        finish_result: Mutex::new(None),
    }
}

fn idle_owner(store: Arc<Store>) -> Workspaces {
    Workspaces {
        shared: Arc::new(Mutex::new(OwnerState::new("/state".into()))),
        storage: None,
        decision: Arc::new(tokio::sync::Mutex::new(())),
        store,
        wake: Mutex::new(None),
        worker: tokio::sync::Mutex::new(None),
        finish_result: Mutex::new(None),
    }
}
#[tokio::test]
async fn pending_capacity_counts_active_identity_once_and_preserves_recovery_admission() {
    let store = Arc::new(Store::default());
    let repo = repository(1);
    let interrupted = queued(repo.id, 99);
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo.clone());
        for n in 1..=8 {
            let w = queued(repo.id, n);
            d.workspaces.insert(w.id, w);
        }
        let mut w = interrupted.clone();
        w.status = CreationStatus::Running {
            stage: CreationStage::CreatingBranch,
        };
        d.workspaces.insert(w.id, w);
    }
    let service = idle_owner(store.clone());
    assert_eq!(
        service
            .admit_workspace(WorkspaceAction::Create {
                action_id: ActionId(Uuid::from_u128(100)),
                repository_id: repo.id
            })
            .await
            .unwrap_err()
            .code,
        ErrorCode::Capacity
    );
    service
        .admit_workspace(WorkspaceAction::Reconcile {
            action_id: ActionId(Uuid::from_u128(101)),
            workspace_id: interrupted.id,
        })
        .await
        .unwrap();
    assert_eq!(
        store.0.lock().unwrap().workspaces[&interrupted.id].creation_action_id,
        interrupted.creation_action_id
    );
    assert_eq!(
        store.0.lock().unwrap().workspaces[&interrupted.id].status,
        CreationStatus::Queued
    );
}
#[tokio::test]
async fn invalid_registration_and_cross_kind_payloads_do_not_write() {
    let store = Arc::new(Store::default());
    let service = idle_owner(store.clone());
    for path in [
        "relative".into(),
        "/bad\0path".into(),
        "/".to_owned() + &"é".repeat(2049),
    ] {
        assert!(
            service
                .register(RegisterRepository {
                    create_initial_workspace: true,
                    action_id: ActionId(Uuid::new_v4()),
                    path
                })
                .await
                .is_err()
        );
    }
    assert!(store.0.lock().unwrap().writes.is_empty());
}
#[tokio::test]
async fn changed_workspace_payload_rejects_and_completed_status_survives_unrelated_fault() {
    let store = Arc::new(Store::default());
    let a = repository(1);
    let b = repository(2);
    let service = idle_owner(store.clone());
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(a.id, a.clone());
        d.repositories.insert(b.id, b.clone());
    }
    let id = ActionId(Uuid::from_u128(11));
    service
        .admit_workspace(WorkspaceAction::Create {
            action_id: id,
            repository_id: a.id,
        })
        .await
        .unwrap();
    assert_eq!(
        service
            .admit_workspace(WorkspaceAction::Create {
                action_id: id,
                repository_id: b.id
            })
            .await
            .unwrap_err()
            .code,
        ErrorCode::Conflict
    );
    let saved = Registration {
        create_initial_workspace: false,
        initial_workspace_id: None,
        action_id: ActionId(Uuid::from_u128(22)),
        path: "/old".into(),
        state: RegistrationState::Completed {
            repository_id: a.id,
        },
    };
    store
        .0
        .lock()
        .unwrap()
        .registrations
        .insert(saved.action_id, saved.clone());
    service.state().unwrap().fault = Some(RepositoryError::storage("unrelated failure"));
    assert_eq!(service.action(saved.action_id).await.unwrap(), Some(saved));
}
#[tokio::test]
async fn byte_bounded_navigation_uses_last_delivered_cursor() {
    let store = Arc::new(Store::default());
    {
        let mut d = store.0.lock().unwrap();
        for n in 1..=20 {
            let mut r = repository(n);
            r.path = "/".to_owned() + &"\u{1}".repeat(2000);
            d.repositories.insert(r.id, r);
        }
    }
    let service = idle_owner(store);
    let first = service.repositories(None, false).await.unwrap();
    assert!(first.repositories.len() < 20);
    assert_eq!(first.next, first.repositories.last().map(|r| r.id));
    assert!(serde_json::to_vec(&first).unwrap().len() <= PAGE_BYTES);
    let second = service.repositories(first.next, false).await.unwrap();
    assert!(second.repositories[0].id > first.next.unwrap());
}
#[tokio::test]
async fn requester_loss_keeps_admitted_creation_owned_until_completion() {
    let (store, effects, mut started) = dependencies();
    let repo = repository(1);
    let w = queued(repo.id, 11);
    let block = Arc::new(Block::default());
    effects.blocks.lock().unwrap().insert(w.id, block.clone());
    {
        let mut d = store.0.lock().unwrap();
        d.repositories.insert(repo.id, repo);
        d.workspaces.insert(w.id, w.clone());
    }
    let service = owner(store, effects);
    assert_eq!(started.recv().await, Some(w.id));
    let request = service.workspace_request(WorkspaceRequest::Get { id: w.id });
    tokio::pin!(request);
    assert!(futures_util::poll!(&mut request).is_ready());
    // No retained request future is needed to finish the already admitted effect.
    block.release();
    ready(&service, w.id).await;
    service.finish().await.unwrap();
}
#[tokio::test]
async fn quit_during_read_only_registration_settles_without_starting_first_creation() {
    let (store, mut effects, _) = dependencies();
    let block = Arc::new(Block::default());
    Arc::get_mut(&mut effects).unwrap().validation_block = Some(block.clone());
    let service = owner(store.clone(), effects.clone());
    let request = RegisterRepository {
        create_initial_workspace: true,
        action_id: ActionId(Uuid::from_u128(10)),
        path: "/app".into(),
    };
    service.register(request.clone()).await.unwrap();
    loop {
        if effects.probes.load(Ordering::SeqCst) > 0 {
            break;
        }
        tokio::task::yield_now().await;
    }
    service.fence().await;
    block.release();
    service.finish().await.unwrap();
    let d = store.0.lock().unwrap();
    assert!(d.repositories.is_empty());
    assert!(d.workspaces.is_empty());
    assert!(matches!(
        d.registrations[&request.action_id].state,
        RegistrationState::Failed { .. }
    ));
}

struct BlockedAdmission {
    at_write: bool,
}
impl Metadata for BlockedAdmission {
    fn read(&self, _: Read) -> BoxFuture<'_, std::result::Result<Delivery, Error>> {
        Box::pin(async move {
            if !self.at_write {
                std::future::pending::<()>().await;
            }
            Ok(Delivery::controlled(Ok(Vec::new()), None))
        })
    }
    fn write(&self, _: Write) -> BoxFuture<'_, std::result::Result<Delivery, Error>> {
        Box::pin(std::future::pending())
    }
}
#[tokio::test]
async fn cancellation_unwinds_provisional_reads_and_fences_unsettled_admission_writes() {
    for at_write in [false, true] {
        let mut service = idle_owner(Arc::default());
        service.store = Arc::new(BlockedAdmission { at_write });
        let id = ActionId(Uuid::new_v4());
        let mut request = Box::pin(service.register(RegisterRepository {
            create_initial_workspace: false,
            action_id: id,
            path: "/app".into(),
        }));
        assert!(futures_util::poll!(&mut request).is_pending());
        assert!(service.state().unwrap().activities.contains_key(&id));
        drop(request);
        let state = service.state().unwrap();
        assert!(state.activities.is_empty());
        assert_eq!(state.fenced, at_write);
        if at_write {
            assert!(
                state
                    .fault
                    .as_ref()
                    .unwrap()
                    .message
                    .contains(&id.0.to_string())
            );
        }
    }
}
