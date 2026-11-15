//! Selection and discovery policy shares the provider owner's admission and fencing.
use super::*;
use std::path::Path;

pub(super) fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && path.len() <= 4096
        && Path::new(path).is_absolute()
        && !path.chars().any(char::is_control)
}
impl ProviderConnections {
    pub(super) async fn discover_verify(
        self: &Arc<Self>,
        target: uuid::Uuid,
        selection: Option<String>,
    ) -> Result<SetupEvidence, SetupFailure> {
        let snapshot = self.read(target).map_err(|_| {
            SetupFailure::at(SetupProblem::StatusReadFailed, "Provider target changed")
        })?;
        let generation = snapshot.revision;
        let path = selection
            .as_deref()
            .or(snapshot.target.selected_executable.as_deref());
        let template = self.launch.clone().ok_or_else(|| {
            SetupFailure::at(
                SetupProblem::StatusReadFailed,
                "No provider adapter configured",
            )
        })?;
        let uncertain = || self.cleanup_uncertain(target, generation);
        let resolved = template
            .discover(path.map(Path::new), &self.cancel, &uncertain)
            .await;
        let (launch, source) = match resolved {
            Ok(found) => found,
            Err(error) => {
                self.resolution(target, generation, None, None);
                return Err(error);
            }
        };
        self.resolution(
            target,
            generation,
            Some(launch.executable()),
            Some(source.into()),
        );
        let (validated, result) =
            codex::probe_checked(launch.clone(), &self.cancel, uncertain).await;
        self.accept_validation(
            target,
            generation,
            selection,
            validated,
            Some(launch),
            result,
        )
        .await
    }
    fn resolution(
        &self,
        target: uuid::Uuid,
        generation: u32,
        executable: Option<String>,
        source: Option<String>,
    ) {
        let mut records = self.records.lock().expect("provider records poisoned");
        if let Some(entry) = records
            .get_mut(&target)
            .filter(|entry| entry.snapshot.revision == generation)
        {
            entry.snapshot.target.executable = executable;
            entry.snapshot.target.discovery_source = source;
        }
    }
    async fn accept_validation(
        &self,
        target: uuid::Uuid,
        generation: u32,
        selected: Option<String>,
        validated: bool,
        launch: Option<codex::Launch>,
        result: Result<SetupEvidence, SetupFailure>,
    ) -> Result<SetupEvidence, SetupFailure> {
        if validated {
            if let Some(path) = &selected {
                let action = self
                    .read(target)
                    .ok()
                    .filter(|snapshot| snapshot.revision == generation)
                    .and_then(|snapshot| snapshot.selection)
                    .map(|selection| selection.id)
                    .ok_or_else(|| {
                        SetupFailure::at(
                            SetupProblem::SelectionFailed,
                            "Selection changed before it could be remembered",
                        )
                    })?;
                if let Some(store) = self.selection_store.clone() {
                    let path = path.clone();
                    tokio::task::spawn_blocking(move || store.remember(action, &path)).await
                        .map_err(|_| SetupFailure::at(SetupProblem::SelectionFailed, "Executable preference worker failed"))?
                        .map_err(|_| SetupFailure::at(SetupProblem::SelectionFailed, "The executable location could not be remembered. Locate Codex again to retry."))?;
                }
            }
            let mut records = self.records.lock().expect("provider records poisoned");
            if let Some(entry) = records
                .get_mut(&target)
                .filter(|entry| entry.snapshot.revision == generation)
            {
                entry.launch = launch;
                if let Some(path) = selected {
                    entry.snapshot.target.selected_executable = Some(path);
                    if let Some(selection) = &mut entry.snapshot.selection {
                        selection.phase = SelectionPhase::Remembered;
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    fn run(f: impl Future<Output = ()>) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f);
    }
    struct Store {
        calls: AtomicUsize,
        fail: bool,
    }
    impl crate::storage::ProviderSelectionPersistence for Store {
        fn remember(&self, _: uuid::Uuid, path: &str) -> Result<(), RepositoryError> {
            assert_eq!(path, "/chosen/codex");
            self.calls.fetch_add(1, Ordering::SeqCst);
            if self.fail {
                Err(RepositoryError::storage("controlled"))
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn selection_admission_is_exact_duplicate_safe_and_excludes_login() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let target = owner.test_target("codex");
            let id = uuid::Uuid::new_v4();
            let (send, receive) = tokio::sync::oneshot::channel();
            let first = owner
                .check_with(
                    target,
                    true,
                    Some((id, 0, "/chosen/codex".into())),
                    || async { receive.await.unwrap() },
                )
                .unwrap();
            let duplicate = owner
                .check_with(
                    target,
                    true,
                    Some((id, 0, "/chosen/codex".into())),
                    || async { panic!("duplicate") },
                )
                .unwrap();
            assert_eq!(first, duplicate);
            assert!(
                owner
                    .admit_login(target, uuid::Uuid::new_v4(), first.revision)
                    .is_err()
            );
            assert!(
                owner
                    .check_with(target, true, Some((id, 0, "/different".into())), || async {
                        panic!("changed payload")
                    })
                    .is_err()
            );
            send.send(Err(SetupProblem::IncompatibleProtocol.into()))
                .unwrap();
            owner.join().await.unwrap();
            assert_eq!(
                owner.read(target).unwrap().selection.unwrap().phase,
                SelectionPhase::Failed
            );
        });
    }
    #[test]
    fn absent_selection_reconciliation_fences_a_delayed_intent() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let target = owner.test_target("codex");
            let id = uuid::Uuid::new_v4();
            owner
                .request(ProviderRequest::ReconcileSelection {
                    target_id: target,
                    action_id: id,
                })
                .unwrap();
            assert!(
                owner
                    .check_with(
                        target,
                        true,
                        Some((id, 0, "/chosen/codex".into())),
                        || async { panic!("late selection") }
                    )
                    .is_err()
            );
        });
    }
    #[test]
    fn validated_selection_is_remembered_even_when_authentication_read_fails() {
        run(async {
            let mut owner = ProviderConnections::test_owner();
            let store = Arc::new(Store {
                calls: AtomicUsize::new(0),
                fail: false,
            });
            owner.selection_store = Some(store.clone());
            let owner = Arc::new(owner);
            let target = owner.test_target("codex");
            let worker = owner.clone();
            owner
                .check_with(
                    target,
                    true,
                    Some((uuid::Uuid::new_v4(), 0, "/chosen/codex".into())),
                    move || async move {
                        worker
                            .accept_validation(
                                target,
                                1,
                                Some("/chosen/codex".into()),
                                true,
                                None,
                                Err(SetupProblem::StatusReadFailed.into()),
                            )
                            .await
                    },
                )
                .unwrap();
            owner.join().await.unwrap();
            let snapshot = owner.read(target).unwrap();
            assert_eq!(store.calls.load(Ordering::SeqCst), 1);
            assert_eq!(
                snapshot.target.selected_executable.as_deref(),
                Some("/chosen/codex")
            );
            assert_eq!(
                snapshot.selection.unwrap().phase,
                SelectionPhase::Remembered
            );
            assert_eq!(snapshot.problem, Some(SetupProblem::StatusReadFailed));
        });
    }
    #[test]
    fn invalid_or_unpersisted_selection_does_not_replace_the_saved_preference() {
        for valid in [false, true] {
            run(async {
                let mut owner = ProviderConnections::test_owner();
                let store = Arc::new(Store {
                    calls: AtomicUsize::new(0),
                    fail: true,
                });
                owner.selection_store = Some(store.clone());
                let owner = Arc::new(owner);
                let target = owner.test_target("codex");
                owner
                    .records
                    .lock()
                    .unwrap()
                    .get_mut(&target)
                    .unwrap()
                    .snapshot
                    .target
                    .selected_executable = Some("/previous/codex".into());
                let worker = owner.clone();
                owner
                    .check_with(
                        target,
                        true,
                        Some((uuid::Uuid::new_v4(), 0, "/chosen/codex".into())),
                        move || async move {
                            worker
                                .accept_validation(
                                    target,
                                    1,
                                    Some("/chosen/codex".into()),
                                    valid,
                                    None,
                                    Err(SetupProblem::IncompatibleProtocol.into()),
                                )
                                .await
                        },
                    )
                    .unwrap();
                owner.join().await.unwrap();
                let snapshot = owner.read(target).unwrap();
                assert_eq!(
                    snapshot.target.selected_executable.as_deref(),
                    Some("/previous/codex")
                );
                assert_eq!(snapshot.selection.unwrap().phase, SelectionPhase::Failed);
                assert_eq!(store.calls.load(Ordering::SeqCst), usize::from(valid));
            });
        }
    }
}
