use super::*;
use std::collections::BTreeMap;

#[derive(Default)]
struct Store {
    actions: BTreeMap<ActionId, Registration>,
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
    ) -> Result<Registration> {
        self.calls.push("finish");
        if self.finish_fault == Some(false) {
            return Err(RepositoryError::storage("commit failed"));
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
                action_id: ActionId(Uuid::from_u128(number)),
                path: path.into(),
            },
        )
        .unwrap();
        finish_registration(
            &mut store,
            &action,
            validation(path, &format!("{path}/.git")),
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

fn state(store: Store) -> Mutex<WorkspaceState> {
    Mutex::new(WorkspaceState {
        directory: "/unused-private-state".into(),
        store: Ok(Box::new(store)),
        cache: VecDeque::new(),
        probes: VecDeque::new(),
        fault: None,
        revision: 1,
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
    )
    .unwrap();
    let other = RegisterRepository {
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
