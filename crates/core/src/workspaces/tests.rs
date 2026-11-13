use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct Store {
    actions: BTreeMap<ActionId, Registration>,
    workspaces: BTreeMap<WorkspaceId, Workspace>,
    workspace_actions: BTreeMap<ActionId, (WorkspaceAction, WorkspaceId)>,
    repositories: BTreeMap<RepositoryId, RepositoryRecord>,
    calls: Vec<&'static str>,
    fail_intent: bool,
    finish_fault: Option<bool>,
}
impl RepositoryStore for Store {
    fn action(&mut self, id: ActionId) -> Result<Option<Registration>> {
        self.calls.push("read_action");
        Ok(self.actions.get(&id).cloned())
    }
    fn intent(&mut self, req: &RegisterRepository) -> Result<Registration> {
        self.calls.push("intent");
        if self.fail_intent {
            return Err(RepositoryError::storage("disk full"));
        }
        let action = Registration {
            create_initial_workspace: req.create_initial_workspace,
            initial_workspace_id: None,
            action_id: req.action_id,
            path: req.path.clone(),
            state: RegistrationState::Pending,
        };
        self.actions.insert(req.action_id, action.clone());
        Ok(action)
    }
    fn pending(&mut self) -> Result<Option<Registration>> {
        Ok(self
            .actions
            .values()
            .find(|a| a.state == RegistrationState::Pending)
            .cloned())
    }
    fn pending_count(&mut self) -> Result<usize> {
        Ok(self
            .actions
            .values()
            .filter(|a| a.state == RegistrationState::Pending)
            .count())
    }
    fn find_common(&mut self, common: &str) -> Result<Option<RepositoryRecord>> {
        Ok(self
            .repositories
            .values()
            .find(|r| r.common_dir == common)
            .cloned())
    }
    fn finish(
        &mut self,
        a: &Registration,
        r: Option<&RepositoryRecord>,
        state: &RegistrationState,
        initial: Option<&Workspace>,
    ) -> Result<Registration> {
        self.calls.push("finish");
        if self.finish_fault == Some(false) {
            return Err(RepositoryError::storage("commit failed"));
        }
        if let Some(w) = initial {
            self.save_workspace(
                w,
                Some(&WorkspaceAction::Create {
                    action_id: w.creation_action_id,
                    repository_id: w.repository_id,
                }),
            )?;
        }
        if let Some(r) = r {
            self.repositories.insert(r.id, r.clone());
        }
        let a = Registration {
            state: state.clone(),
            ..a.clone()
        };
        self.actions.insert(a.action_id, a.clone());
        if self.finish_fault == Some(true) {
            return Err(RepositoryError::storage("commit reply lost"));
        }
        Ok(a)
    }
    fn repository(&mut self, id: RepositoryId) -> Result<Option<RepositoryRecord>> {
        Ok(self.repositories.get(&id).cloned())
    }
    fn list(&mut self, after: Option<RepositoryId>, limit: usize) -> Result<Vec<RepositoryRecord>> {
        Ok(self
            .repositories
            .values()
            .filter(|r| after.is_none_or(|id| r.id > id))
            .take(limit)
            .cloned()
            .collect())
    }
}

fn validation(path: &str, common: &str) -> Result<ValidatedRepository> {
    Ok(ValidatedRepository {
        name: "one".into(),
        path: path.into(),
        common_dir: common.into(),
    })
}

#[test]
fn conflicting_ids_invalid_paths_and_failed_intent_admit_no_repository_effect() {
    let mut store = Store::default();
    let req = request();
    admit(&mut store, &req).unwrap();
    assert_eq!(
        admit(
            &mut store,
            &RegisterRepository {
                path: "/other".into(),
                ..req.clone()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Conflict
    );
    let mut store = Store::default();
    for path in ["relative", "", "/nul\0path"] {
        assert_eq!(
            admit(
                &mut store,
                &RegisterRepository {
                    path: path.into(),
                    ..req.clone()
                }
            )
            .unwrap_err()
            .code,
            ErrorCode::InvalidPath
        );
    }
    store.fail_intent = true;
    assert_eq!(
        admit(&mut store, &req).unwrap_err().code,
        ErrorCode::Storage
    );
    assert!(store.pending().unwrap().is_none());
    assert!(store.repositories.is_empty());
    assert!(!store.calls.contains(&"finish"));
}

#[test]
fn registration_path_budget_counts_utf8_bytes_before_recording_intent() {
    for path in [
        format!("/{}", "a".repeat(4095)),
        format!("/{}a", "é".repeat(2047)),
    ] {
        assert_eq!(path.len(), 4096);
        let mut store = Store::default();
        let req = RegisterRepository { path, ..request() };
        assert_eq!(
            admit(&mut store, &req).unwrap().state,
            RegistrationState::Pending
        );
        assert_eq!(store.calls, ["read_action", "intent"]);
    }
    for path in [
        format!("/{}", "a".repeat(4096)),
        format!("/{}", "é".repeat(2048)),
    ] {
        assert_eq!(path.len(), 4097);
        let mut store = Store::default();
        let req = RegisterRepository { path, ..request() };
        assert_eq!(
            admit(&mut store, &req).unwrap_err().code,
            ErrorCode::Capacity
        );
        assert_eq!(store.calls, ["read_action"]);
        assert!(store.actions.is_empty());
    }
}

#[test]
fn invalid_validation_records_an_actionable_failure_without_a_saved_repository() {
    let mut store = Store::default();
    let action = admit(&mut store, &request()).unwrap();
    let outcome = finish_registration(
        &mut store,
        &action,
        Err(RepositoryError::new(
            ErrorCode::InvalidPath,
            "Folder unavailable; restore its location",
        )),
        Path::new("/private/state"),
    )
    .unwrap();
    assert!(
        matches!(outcome.state, RegistrationState::Failed { error } if error.code == ErrorCode::InvalidPath)
    );
    assert!(store.repositories.is_empty());
    assert!(store.pending().unwrap().is_none());
}

#[test]
fn separate_clones_keep_separate_identity_even_with_the_same_name() {
    let mut store = Store::default();
    for (number, path) in [(1, "/first"), (2, "/clone")] {
        let action = admit(
            &mut store,
            &RegisterRepository {
                create_initial_workspace: false,
                action_id: ActionId(Uuid::from_u128(number)),
                path: path.into(),
            },
        )
        .unwrap();
        finish_registration(
            &mut store,
            &action,
            validation(path, &format!("{path}/.git")),
            Path::new("/private/state"),
        )
        .unwrap();
    }
    assert_eq!(store.repositories.len(), 2);
}

#[test]
fn uncertain_commit_is_resolved_from_committed_action_status_and_failure_retains_intent() {
    for committed in [true, false] {
        let mut store = Store::default();
        let action = admit(&mut store, &request()).unwrap();
        store.finish_fault = Some(committed);
        let result = finish_registration(
            &mut store,
            &action,
            validation("/projects/one", "/projects/one/.git"),
            Path::new("/private/state"),
        );
        if committed {
            assert!(matches!(
                result.unwrap().state,
                RegistrationState::Completed { .. }
            ));
            assert_eq!(store.repositories.len(), 1);
        } else {
            assert_eq!(result.unwrap_err().code, ErrorCode::Storage);
            assert_eq!(
                store.pending().unwrap().unwrap().action_id,
                action.action_id
            );
            assert!(store.repositories.is_empty());
        }
    }
}

fn state(store: Store) -> Mutex<OwnerState> {
    Mutex::new(OwnerState {
        directory: "/unused-private-state".into(),
        store: Ok(Box::new(store)),
        cache: VecDeque::new(),
        probes: VecDeque::new(),
        fault: None,
        revision: 1,
        active_workspace: None,
        workspace_fault: None,
    })
}

#[test]
fn restored_pending_intent_runs_read_only_validation_while_status_remains_readable() {
    let mut store = Store::default();
    admit(&mut store, &request()).unwrap();
    let shared = state(store);
    assert!(process_one(&shared, |path| {
        assert_eq!(path, "/projects/one");
        let mut state = shared
            .try_lock()
            .expect("Git must not hold the observation lock");
        assert_eq!(
            state.store().unwrap().pending().unwrap().unwrap().action_id,
            request().action_id
        );
        assert!(state.page(None, false).unwrap().repositories.is_empty());
        validation(path, "/projects/one/.git")
    }));
    assert!(!process_one(&shared, |_| panic!(
        "completed registration must not repeat validation"
    )));
    assert!(matches!(
        shared
            .lock()
            .unwrap()
            .store()
            .unwrap()
            .action(request().action_id)
            .unwrap()
            .unwrap()
            .state,
        RegistrationState::Completed { .. }
    ));
}

fn record(number: u128) -> RepositoryRecord {
    RepositoryRecord {
        id: RepositoryId(Uuid::from_u128(number)),
        name: "one".into(),
        path: format!("/project/{number}"),
        common_dir: format!("/project/{number}/.git"),
    }
}

#[test]
fn unavailable_or_replaced_paths_preserve_saved_identity_and_can_recover() {
    let mut store = Store::default();
    let record = record(1);
    store.repositories.insert(record.id, record.clone());
    let shared = state(store);
    shared.lock().unwrap().page(None, true).unwrap();
    process_one(&shared, |_| {
        Err(RepositoryError::new(
            ErrorCode::InvalidPath,
            "Restore the missing location",
        ))
    });
    assert!(matches!(
        shared
            .lock()
            .unwrap()
            .page(None, false)
            .unwrap()
            .repositories[0]
            .availability,
        Availability::Unavailable { .. }
    ));
    shared.lock().unwrap().page(None, true).unwrap();
    process_one(&shared, |_| validation(&record.path, "/replacement/.git"));
    assert!(matches!(
        shared
            .lock()
            .unwrap()
            .page(None, false)
            .unwrap()
            .repositories[0]
            .availability,
        Availability::Unavailable { .. }
    ));
    shared.lock().unwrap().page(None, true).unwrap();
    process_one(&shared, |_| validation(&record.path, &record.common_dir));
    let mut state = shared.lock().unwrap();
    let page = state.page(None, false).unwrap();
    assert_eq!(page.repositories[0].id, record.id);
    assert_eq!(page.repositories[0].availability, Availability::Available);
    assert_eq!(
        state.store().unwrap().repository(record.id).unwrap(),
        Some(record)
    );
}

#[test]
fn bounded_pages_and_pending_admission_do_not_restrict_durable_repository_count() {
    let mut store = Store::default();
    for number in 1..=100 {
        let r = record(number);
        store.repositories.insert(r.id, r);
    }
    for number in 1..=8 {
        admit(
            &mut store,
            &RegisterRepository {
                create_initial_workspace: false,
                action_id: ActionId(Uuid::from_u128(number)),
                path: "/one".into(),
            },
        )
        .unwrap();
    }
    assert_eq!(
        admit(
            &mut store,
            &RegisterRepository {
                create_initial_workspace: false,
                action_id: ActionId(Uuid::from_u128(9)),
                path: "/nine".into()
            }
        )
        .unwrap_err()
        .code,
        ErrorCode::Capacity
    );
    assert_eq!(
        admit(
            &mut store,
            &RegisterRepository {
                create_initial_workspace: false,
                action_id: ActionId(Uuid::from_u128(1)),
                path: "/one".into()
            }
        )
        .unwrap()
        .state,
        RegistrationState::Pending
    );
    let shared = state(store);
    let mut state = shared.lock().unwrap();
    let page = state.page(None, false).unwrap();
    assert_eq!(page.repositories.len(), 32);
    assert_eq!(page.next, Some(RepositoryId(Uuid::from_u128(32))));
    let next = state.page(page.next, false).unwrap();
    assert_eq!(next.repositories[0].id, RepositoryId(Uuid::from_u128(33)));
    for number in [64, 96] {
        state
            .page(Some(RepositoryId(Uuid::from_u128(number))), false)
            .unwrap();
    }
    assert!(state.cache.len() <= CACHE_SIZE && state.probes.len() <= CACHE_SIZE);
    assert_eq!(state.store().unwrap().list(None, 1000).unwrap().len(), 100);
}

#[test]
fn escaped_path_serialization_reduces_page_size_and_retains_next_cursor() {
    let mut store = Store::default();
    for number in 1..=20 {
        let mut r = record(number);
        r.path = format!("/{}", "\t".repeat(4095));
        store.repositories.insert(r.id, r);
    }
    let shared = state(store);
    let page = shared.lock().unwrap().page(None, false).unwrap();
    assert!(page.repositories.len() < 20);
    assert!(page.next.is_some());
    assert!(serde_json::to_vec(&page).unwrap().len() < 64 * 1024);
}
fn request() -> RegisterRepository {
    RegisterRepository {
        create_initial_workspace: false,
        action_id: ActionId(Uuid::from_u128(1)),
        path: "/projects/one".into(),
    }
}

#[test]
fn registration_records_intent_and_replays_its_status_without_another_effect() {
    let mut store = Store::default();
    let req = request();
    assert_eq!(
        admit(&mut store, &req).unwrap().state,
        RegistrationState::Pending
    );
    assert_eq!(
        admit(&mut store, &req).unwrap().state,
        RegistrationState::Pending
    );
    assert_eq!(store.calls, ["read_action", "intent", "read_action"]);
    assert!(store.repositories.is_empty());
}

#[test]
fn validated_aliases_and_linked_worktrees_share_identity_and_first_location() {
    let mut store = Store::default();
    let first = admit(&mut store, &request()).unwrap();
    let result = finish_registration(
        &mut store,
        &first,
        Ok(ValidatedRepository {
            name: "one".into(),
            path: "/projects/one".into(),
            common_dir: "/projects/one/.git".into(),
        }),
        Path::new("/private/state"),
    )
    .unwrap();
    let other = RegisterRepository {
        create_initial_workspace: false,
        action_id: ActionId(Uuid::from_u128(2)),
        path: "/linked/one".into(),
    };
    let next = admit(&mut store, &other).unwrap();
    let alias = finish_registration(
        &mut store,
        &next,
        Ok(ValidatedRepository {
            name: "linked".into(),
            path: other.path.clone(),
            common_dir: "/projects/one/.git".into(),
        }),
        Path::new("/private/state"),
    )
    .unwrap();
    assert_eq!(alias.state, result.state);
    assert_eq!(store.repositories.len(), 1);
    assert_eq!(
        store.repositories.values().next().unwrap().path,
        "/projects/one"
    );
    assert_eq!(admit(&mut store, &request()).unwrap(), result);
}

impl CreationStore for Store {
    fn workspace(&mut self, id: WorkspaceId) -> Result<Option<Workspace>> {
        Ok(self.workspaces.get(&id).cloned())
    }
    fn workspace_action(&mut self, id: ActionId) -> Result<Option<(WorkspaceAction, WorkspaceId)>> {
        Ok(self.workspace_actions.get(&id).cloned())
    }
    fn save_workspace(&mut self, w: &Workspace, a: Option<&WorkspaceAction>) -> Result<()> {
        if self.fail_intent {
            return Err(RepositoryError::storage("disk full"));
        }
        self.workspaces.insert(w.id, w.clone());
        if let Some(a) = a {
            self.workspace_actions.insert(a.id(), (a.clone(), w.id));
        }
        Ok(())
    }
    fn workspace_pending(&mut self) -> Result<Option<Workspace>> {
        Ok(self
            .workspaces
            .values()
            .find(|w| w.status == CreationStatus::Queued)
            .cloned())
    }
    fn workspace_cleanup_blocked(&mut self, r: RepositoryId) -> Result<bool> {
        Ok(self
            .workspaces
            .values()
            .any(|w| w.repository_id == r && w.cleanup_unconfirmed))
    }
    fn workspace_count_pending(&mut self) -> Result<usize> {
        Ok(self
            .workspaces
            .values()
            .filter(|w| {
                matches!(
                    w.status,
                    CreationStatus::Queued | CreationStatus::Running { .. }
                )
            })
            .count())
    }
    fn workspace_list(
        &mut self,
        r: RepositoryId,
        after: Option<WorkspaceId>,
        limit: usize,
    ) -> Result<Vec<Workspace>> {
        Ok(self
            .workspaces
            .values()
            .filter(|w| w.repository_id == r && after.is_none_or(|id| w.id > id))
            .take(limit)
            .cloned()
            .collect())
    }
}

#[test]
fn initial_creation_is_linked_once_and_duplicate_registration_has_no_new_workspace() {
    let mut store = Store::default();
    let req = RegisterRepository {
        create_initial_workspace: true,
        ..request()
    };
    let pending = admit(&mut store, &req).unwrap();
    let completed = finish_registration(
        &mut store,
        &pending,
        validation("/repo", "/repo/.git"),
        Path::new("/state"),
    )
    .unwrap();
    assert!(completed.initial_workspace_id.is_some());
    assert_eq!(store.workspaces.len(), 1);
    assert_eq!(admit(&mut store, &req).unwrap(), completed);
    let duplicate = RegisterRepository {
        action_id: ActionId(Uuid::new_v4()),
        ..req
    };
    let pending = admit(&mut store, &duplicate).unwrap();
    let reused = finish_registration(
        &mut store,
        &pending,
        validation("/repo", "/repo/.git"),
        Path::new("/state"),
    )
    .unwrap();
    assert_eq!(reused.initial_workspace_id, None);
    assert_eq!(store.workspaces.len(), 1);
}
#[test]
fn workspace_action_replay_conflicts_and_explicit_recovery_retain_exact_identity() {
    let mut store = Store::default();
    let req = request();
    let pending = admit(&mut store, &req).unwrap();
    let registration = finish_registration(
        &mut store,
        &pending,
        validation("/repo", "/repo/.git"),
        Path::new("/state"),
    )
    .unwrap();
    let RegistrationState::Completed { repository_id } = registration.state else {
        panic!()
    };
    let action = WorkspaceAction::Create {
        action_id: ActionId(Uuid::new_v4()),
        repository_id,
    };
    let w = admit_workspace(&mut store, Path::new("/state"), &action, None).unwrap();
    assert_eq!(
        admit_workspace(&mut store, Path::new("/other"), &action, None).unwrap(),
        w
    );
    assert_eq!(store.workspaces.len(), 1);
    assert!(
        admit_workspace(
            &mut store,
            Path::new("/state"),
            &WorkspaceAction::Reconcile {
                action_id: action.id(),
                workspace_id: w.id
            },
            None
        )
        .is_err()
    );
    let mut interrupted = w.clone();
    interrupted.status = CreationStatus::Running {
        stage: CreationStage::CreatingBranch,
    };
    store.save_workspace(&interrupted, None).unwrap();
    assert_eq!(
        admit_workspace(&mut store, Path::new("/state"), &action, None)
            .unwrap()
            .status,
        interrupted.status
    );
    let recovered = admit_workspace(
        &mut store,
        Path::new("/state"),
        &WorkspaceAction::Reconcile {
            action_id: ActionId(Uuid::new_v4()),
            workspace_id: w.id,
        },
        None,
    )
    .unwrap();
    assert_eq!(recovered.id, w.id);
    assert_eq!(recovered.path, w.path);
    assert_eq!(recovered.status, CreationStatus::Queued);
    assert!(
        admit_workspace(
            &mut store,
            Path::new("/state"),
            &WorkspaceAction::Create {
                action_id: req.action_id,
                repository_id
            },
            None
        )
        .is_err()
    );
}
#[test]
fn restored_inflight_workspace_is_uncertain_and_reads_do_not_queue_it() {
    let mut w = allocate(
        RepositoryId(Uuid::nil()),
        ActionId(Uuid::nil()),
        Path::new("/state"),
    )
    .unwrap();
    w.status = CreationStatus::Running {
        stage: CreationStage::CreatingWorktree,
    };
    let mut store = Store::default();
    store.save_workspace(&w, None).unwrap();
    let shared = state(store);
    let mut owner = shared.lock().unwrap();
    let restored: Workspace = serde_json::from_str(&serde_json::to_string(&w).unwrap()).unwrap();
    assert!(matches!(
        owner.workspace_observation(restored).status,
        CreationStatus::Uncertain { .. }
    ));
    assert!(
        owner
            .store()
            .unwrap()
            .workspace_pending()
            .unwrap()
            .is_none()
    );
}
#[test]
fn recovery_reserve_is_available_at_metadata_budget_and_navigation_pages_are_bounded() {
    let mut w = allocate(
        RepositoryId(Uuid::nil()),
        ActionId(Uuid::nil()),
        Path::new("/state"),
    )
    .unwrap();
    w.path = "\\".repeat(8500); // JSON escaping counts toward the metadata byte budget.
    check_workspace_budget(&w).unwrap();
    w.status = CreationStatus::Uncertain {
        error: RepositoryError::storage("x".repeat(10000)),
    };
    w.base = Some(PinnedBase {
        reference: "refs/remotes/origin/HEAD".into(),
        commit: "f".repeat(64),
    });
    assert!(
        serde_json::to_vec(&WorkspaceReply::Workspace {
            workspace: Some(w.clone())
        })
        .unwrap()
        .len()
            < 24 * 1024
    );
    w.path = "\\".repeat(12000);
    assert!(check_workspace_budget(&w).is_err());
}

#[test]
fn creation_keeps_pending_observations_readable_during_every_effect() {
    let repository = RepositoryRecord {
        id: RepositoryId(Uuid::new_v4()),
        name: "repo".into(),
        path: "/repo".into(),
        common_dir: "/repo/.git".into(),
    };
    let w = allocate(repository.id, ActionId(Uuid::new_v4()), Path::new("/state")).unwrap();
    let mut store = Store::default();
    store.repositories.insert(repository.id, repository);
    store.save_workspace(&w, None).unwrap();
    let service = Workspaces {
        shared: Arc::new(state(store)),
        wake: None,
        worker: None,
    };
    struct Reading<'a> {
        service: &'a Workspaces,
        artifacts: Artifacts,
    }
    impl Reading<'_> {
        fn status(&self, w: &Workspace, stage: CreationStage) {
            let WorkspaceReply::Workspace {
                workspace: Some(observed),
            } = self
                .service
                .workspace_request(WorkspaceRequest::Get { id: w.id })
                .unwrap()
            else {
                panic!()
            };
            assert_eq!(observed.status, CreationStatus::Running { stage });
        }
    }
    impl CreationEffects for Reading<'_> {
        fn quiescent(&mut self, _: &Workspace) -> Result<bool> {
            Ok(true)
        }
        fn base(&mut self, _: &RepositoryRecord) -> Result<PinnedBase> {
            Ok(PinnedBase {
                reference: "refs/heads/main".into(),
                commit: "a".repeat(40),
            })
        }
        fn inspect(&mut self, _: &RepositoryRecord, _: &Workspace) -> Result<Artifacts> {
            Ok(self.artifacts.clone())
        }
        fn reserve(&mut self, w: &Workspace) -> Result<()> {
            self.status(w, CreationStage::ReservingFolder);
            self.artifacts.folder = FolderArtifact::Reserved;
            Ok(())
        }
        fn branch(&mut self, _: &RepositoryRecord, w: &Workspace) -> Result<()> {
            self.status(w, CreationStage::CreatingBranch);
            self.artifacts.branch = BranchArtifact::Owned;
            Ok(())
        }
        fn worktree(&mut self, _: &RepositoryRecord, w: &Workspace) -> Result<()> {
            self.status(w, CreationStage::CreatingWorktree);
            self.artifacts.folder = FolderArtifact::Worktree;
            Ok(())
        }
    }
    let mut effects = Reading {
        service: &service,
        artifacts: Artifacts {
            branch: BranchArtifact::Absent,
            folder: FolderArtifact::Absent,
        },
    };
    assert!(process_creation(&service.shared, &mut effects));
    let WorkspaceReply::Workspace {
        workspace: Some(ready),
    } = service
        .workspace_request(WorkspaceRequest::Get { id: w.id })
        .unwrap()
    else {
        panic!()
    };
    assert_eq!(ready.status, CreationStatus::Ready);
}
#[test]
fn workspace_pages_use_byte_cursors_without_a_storage_cardinality_limit() {
    let repository_id = RepositoryId(Uuid::new_v4());
    let mut store = Store::default();
    for number in 1..=70 {
        let mut w = allocate(
            repository_id,
            ActionId(Uuid::from_u128(number)),
            Path::new("/state"),
        )
        .unwrap();
        w.id = WorkspaceId(Uuid::from_u128(number));
        w.path = "\\".repeat(7000);
        w.status = CreationStatus::Ready;
        store.save_workspace(&w, None).unwrap();
    }
    assert_eq!(store.workspace_count_pending().unwrap(), 0);
    let service = Workspaces {
        shared: Arc::new(state(store)),
        wake: None,
        worker: None,
    };
    let mut after = None;
    let mut seen = vec![];
    loop {
        let reply = service
            .workspace_request(WorkspaceRequest::List {
                repository_id,
                after,
            })
            .unwrap();
        assert!(serde_json::to_vec(&reply).unwrap().len() < 64 * 1024);
        let WorkspaceReply::Workspaces { page } = reply else {
            panic!()
        };
        assert!(page.workspaces.len() <= 32);
        seen.extend(page.workspaces.iter().map(|w| w.id));
        if page.next.is_none() {
            break;
        }
        after = page.next;
    }
    assert_eq!(seen.len(), 70);
    seen.dedup();
    assert_eq!(seen.len(), 70);
}

#[test]
fn failed_workspace_intent_blocks_admission_and_replay_does_not_clear_a_write_fault() {
    let repository = RepositoryRecord {
        id: RepositoryId(Uuid::new_v4()),
        name: "repo".into(),
        path: "/repo".into(),
        common_dir: "/repo/.git".into(),
    };
    let mut store = Store::default();
    store.repositories.insert(repository.id, repository.clone());
    store.fail_intent = true;
    let action = WorkspaceAction::Create {
        action_id: ActionId(Uuid::new_v4()),
        repository_id: repository.id,
    };
    assert!(admit_workspace(&mut store, Path::new("/state"), &action, None).is_err());
    assert!(store.workspaces.is_empty());
    assert!(store.workspace_actions.is_empty());
    store.fail_intent = false;
    let mut w = admit_workspace(&mut store, Path::new("/state"), &action, None).unwrap();
    w.status = CreationStatus::Failed {
        error: RepositoryError::storage("failure"),
    };
    store.save_workspace(&w, None).unwrap();
    let recovery = WorkspaceAction::Reconcile {
        action_id: ActionId(Uuid::new_v4()),
        workspace_id: w.id,
    };
    admit_workspace(&mut store, Path::new("/state"), &recovery, None).unwrap();
    let shared = Arc::new(state(store));
    shared.lock().unwrap().workspace_fault = Some((
        w.id,
        RepositoryError::storage("write failed before mutation"),
    ));
    let service = Workspaces {
        shared,
        wake: None,
        worker: None,
    };
    let WorkspaceAction::Reconcile {
        action_id,
        workspace_id,
    } = recovery
    else {
        panic!()
    };
    let WorkspaceReply::Action {
        workspace: Some(observed),
        ..
    } = service
        .workspace_request(WorkspaceRequest::Reconcile {
            action_id,
            workspace_id,
        })
        .unwrap()
    else {
        panic!()
    };
    assert!(matches!(observed.status, CreationStatus::Uncertain { .. }));
    assert!(service.shared.lock().unwrap().workspace_fault.is_some());
}
