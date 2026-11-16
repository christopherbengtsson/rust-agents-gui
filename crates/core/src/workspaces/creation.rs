//! Isolated workspace decisions. Effects and durable metadata are supplied at narrow seams.
use super::*;

pub(crate) fn choose_base<'a>(
    origin: Option<&'a str>,
    main: Option<&'a str>,
    master: Option<&'a str>,
) -> Result<(&'static str, &'a str)> {
    match (origin, main, master) {
        (Some(commit), _, _) => Ok(("refs/remotes/origin/HEAD", commit)),
        (None, Some(commit), None) => Ok(("refs/heads/main", commit)),
        (None, None, Some(commit)) => Ok(("refs/heads/master", commit)),
        _ => Err(RepositoryError::new(
            ErrorCode::Git,
            "Cannot identify a local default branch. Set origin/HEAD to an available local remote-tracking branch, or keep exactly one local main/master branch with a commit, then reconcile this workspace. No fetch was performed.",
        )),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum WorkspaceRequest {
    Changes {
        workspace_id: WorkspaceId,
        refresh: bool,
        comparison_id: Option<ComparisonId>,
        after: Option<FileId>,
    },
    ChangePreview {
        target: PreviewTarget,
        load: bool,
    },
    Create {
        action_id: ActionId,
        repository_id: RepositoryId,
    },
    Reconcile {
        action_id: ActionId,
        workspace_id: WorkspaceId,
    },
    Action {
        action_id: ActionId,
    },
    Get {
        id: WorkspaceId,
    },
    List {
        repository_id: RepositoryId,
        after: Option<WorkspaceId>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct WorkspacePage {
    pub workspaces: Vec<Workspace>,
    pub next: Option<WorkspaceId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkspaceReply {
    Changes {
        observation: ChangesObservation,
    },
    ChangePreview {
        observation: PreviewObservation,
    },
    Workspace {
        workspace: Option<Workspace>,
    },
    Action {
        action: Option<WorkspaceAction>,
        workspace: Option<Workspace>,
    },
    Workspaces {
        page: WorkspacePage,
    },
}

pub(crate) fn allocate(
    repository: RepositoryId,
    action: ActionId,
    _directory: &Path,
) -> Result<Workspace> {
    let id = WorkspaceId(Uuid::new_v4());
    let workspace = Workspace {
        id,
        repository_id: repository,
        creation_action_id: action,
        branch: None,
        name: None,
        path: None,

        base: None,
        artifacts: Artifacts {
            branch: BranchArtifact::Unknown,
            folder: FolderArtifact::Unknown,
        },
        status: CreationStatus::Queued,
        cleanup_unconfirmed: false,
    };
    check_workspace_budget(&workspace)?;
    Ok(workspace)
}
// Leave room in each single observation for bounded errors, pinned base and artifacts.
pub(crate) fn check_workspace_budget(workspace: &Workspace) -> Result<()> {
    if serde_json::to_vec(workspace)
        .map_err(RepositoryError::storage)?
        .len()
        > 20 * 1024
    {
        return Err(RepositoryError::new(
            ErrorCode::Capacity,
            "Workspace metadata exceeds its observation budget. Choose a shorter private application-state location; saved work is retained.",
        ));
    }
    Ok(())
}

pub(crate) trait CreationEffects {
    fn quiescent(&mut self, workspace: &Workspace) -> Result<bool>;
    fn base(&mut self, repository: &RepositoryRecord) -> Result<PinnedBase>;
    fn inspect(
        &mut self,
        repository: &RepositoryRecord,
        workspace: &Workspace,
    ) -> Result<Artifacts>;
    fn reserve(&mut self, workspace: &Workspace) -> Result<()>;
    fn branch(&mut self, repository: &RepositoryRecord, workspace: &Workspace) -> Result<()>;
    fn worktree(&mut self, repository: &RepositoryRecord, workspace: &Workspace) -> Result<()>;
}

pub(crate) fn drive(
    mut workspace: Workspace,
    repository: &RepositoryRecord,
    effects: &mut (impl CreationEffects + ?Sized),
    mut persist: impl FnMut(&Workspace, bool) -> Result<()>,
) -> Result<Workspace> {
    workspace.path()?;
    workspace.branch()?;
    if !effects.quiescent(&workspace)? {
        workspace.cleanup_unconfirmed = true;
        workspace.status = CreationStatus::Uncertain {
            error: RepositoryError::new(
                ErrorCode::CleanupUnconfirmed,
                "An earlier Git effect still owns this workspace's execution lease. Wait for it to finish, then reconcile. No creation was repeated and no saved PID was signaled.",
            ),
        };
        persist(&workspace, false)?;
        return Ok(workspace);
    }
    workspace.cleanup_unconfirmed = false;
    // Each fallible store boundary exits before any subsequent mutation.
    workspace.status = CreationStatus::Running {
        stage: CreationStage::ResolvingBase,
    };
    persist(&workspace, false)?;
    if workspace.base.is_none() {
        match effects.base(repository) {
            Ok(base) => {
                workspace.base = Some(base);
                persist(&workspace, false)?;
            }
            Err(error) => {
                workspace.cleanup_unconfirmed = error.code == ErrorCode::CleanupUnconfirmed;
                workspace.status = CreationStatus::Failed { error };
                persist(&workspace, false)?;
                return Ok(workspace);
            }
        }
    }
    workspace.artifacts = match effects.inspect(repository, &workspace) {
        Ok(artifacts) => artifacts,
        Err(error) => {
            workspace.cleanup_unconfirmed = error.code == ErrorCode::CleanupUnconfirmed;
            workspace.status = CreationStatus::Uncertain { error };
            persist(&workspace, false)?;
            return Ok(workspace);
        }
    };
    persist(&workspace, false)?;
    if workspace.artifacts.branch == BranchArtifact::Conflict
        || matches!(
            workspace.artifacts.folder,
            FolderArtifact::Conflict | FolderArtifact::Partial
        )
    {
        workspace.status = CreationStatus::Failed {
            error: RepositoryError::new(
                ErrorCode::Conflict,
                "Existing branch/folder artifacts cannot safely be continued. Inspect the displayed locations and restore access or move conflicting artifacts yourself, then reconcile. No work was overwritten or deleted.",
            ),
        };
        persist(&workspace, false)?;
        return Ok(workspace);
    }
    for stage in [
        CreationStage::ReservingFolder,
        CreationStage::CreatingBranch,
        CreationStage::CreatingWorktree,
    ] {
        let needed = match stage {
            CreationStage::ReservingFolder => workspace.artifacts.folder == FolderArtifact::Absent,
            CreationStage::CreatingBranch => workspace.artifacts.branch == BranchArtifact::Absent,
            CreationStage::CreatingWorktree => {
                workspace.artifacts.folder == FolderArtifact::Reserved
                    && workspace.artifacts.branch == BranchArtifact::Owned
            }
            CreationStage::AllocatingName | CreationStage::ResolvingBase => false,
        };
        if !needed {
            continue;
        }
        workspace.status = CreationStatus::Running { stage };
        persist(&workspace, true)?;
        let outcome = match stage {
            CreationStage::ReservingFolder => effects.reserve(&workspace),
            CreationStage::CreatingBranch => effects.branch(repository, &workspace),
            CreationStage::CreatingWorktree => effects.worktree(repository, &workspace),
            CreationStage::AllocatingName | CreationStage::ResolvingBase => unreachable!(),
        };
        if let Err(error) = outcome {
            workspace.cleanup_unconfirmed = error.code == ErrorCode::CleanupUnconfirmed;
            if !workspace.cleanup_unconfirmed
                && let Ok(artifacts) = effects.inspect(repository, &workspace)
            {
                workspace.artifacts = artifacts;
            }
            workspace.status = CreationStatus::Uncertain { error };
            persist(&workspace, false)?;
            return Ok(workspace);
        }
        workspace.artifacts = match effects.inspect(repository, &workspace) {
            Ok(artifacts) => artifacts,
            Err(error) => {
                workspace.cleanup_unconfirmed = error.code == ErrorCode::CleanupUnconfirmed;
                workspace.status = CreationStatus::Uncertain { error };
                persist(&workspace, false)?;
                return Ok(workspace);
            }
        };
        persist(&workspace, false)?;
    }
    workspace.status = if workspace.artifacts
        == (Artifacts {
            branch: BranchArtifact::Owned,
            folder: FolderArtifact::Worktree,
        }) {
        CreationStatus::Ready
    } else {
        CreationStatus::Uncertain {
            error: RepositoryError::new(
                ErrorCode::Git,
                "Creation could not be confirmed from the saved branch and worktree evidence. Reconcile to inspect this same workspace; no additional creation was requested.",
            ),
        }
    };
    persist(&workspace, false)?;
    Ok(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn automatic_base_uses_origin_then_only_an_unambiguous_local_default() {
        assert_eq!(
            choose_base(Some("abc"), Some("def"), Some("fed")).unwrap(),
            ("refs/remotes/origin/HEAD", "abc")
        );
        assert_eq!(
            choose_base(None, Some("def"), None).unwrap(),
            ("refs/heads/main", "def")
        );
        assert!(choose_base(None, Some("def"), Some("fed")).is_err());
        assert!(choose_base(None, None, None).is_err());
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;
    use std::cell::RefCell;
    struct Effects<'a> {
        calls: &'a RefCell<Vec<&'static str>>,
        artifacts: Artifacts,
        fail: Option<CreationStage>,
        quiescent: bool,
    }
    fn allocated(r: RepositoryId, a: ActionId, d: &Path) -> Result<Workspace> {
        let mut w = allocate(r, a, d)?;
        w.name = Some("marta".into());
        w.branch = Some("workspace/marta".into());
        w.path = Some("/state/repo/marta/checkout".into());
        Ok(w)
    }
    impl CreationEffects for Effects<'_> {
        fn quiescent(&mut self, _: &Workspace) -> Result<bool> {
            Ok(self.quiescent)
        }
        fn base(&mut self, _: &RepositoryRecord) -> Result<PinnedBase> {
            self.calls.borrow_mut().push("resolve");
            Ok(PinnedBase {
                reference: "refs/heads/main".into(),
                commit: "a".repeat(40),
            })
        }
        fn inspect(&mut self, _: &RepositoryRecord, _: &Workspace) -> Result<Artifacts> {
            Ok(self.artifacts.clone())
        }
        fn reserve(&mut self, _: &Workspace) -> Result<()> {
            self.calls.borrow_mut().push("reserve");
            self.artifacts.folder = FolderArtifact::Reserved;
            Ok(())
        }
        fn branch(&mut self, _: &RepositoryRecord, _: &Workspace) -> Result<()> {
            self.calls.borrow_mut().push("branch");
            self.artifacts.branch = BranchArtifact::Owned;
            if self.fail == Some(CreationStage::CreatingBranch) {
                Err(RepositoryError::new(
                    ErrorCode::Connection,
                    "branch reply lost",
                ))
            } else {
                Ok(())
            }
        }
        fn worktree(&mut self, _: &RepositoryRecord, _: &Workspace) -> Result<()> {
            self.calls.borrow_mut().push("worktree");
            self.artifacts.folder = FolderArtifact::Worktree;
            if self.fail == Some(CreationStage::CreatingWorktree) {
                Err(RepositoryError::new(
                    ErrorCode::Connection,
                    "worktree reply lost",
                ))
            } else {
                Ok(())
            }
        }
    }
    fn repo() -> RepositoryRecord {
        RepositoryRecord {
            id: RepositoryId(Uuid::nil()),
            name: "repo".into(),
            path: "/repo".into(),
            common_dir: "/repo/.git".into(),
        }
    }
    #[test]
    fn pinned_intent_and_each_stage_precede_their_effect_and_completion_is_durable() {
        let calls = RefCell::new(vec![]);
        let workspace = allocated(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
        let mut effects = Effects {
            calls: &calls,
            fail: None,
            quiescent: true,
            artifacts: Artifacts {
                branch: BranchArtifact::Absent,
                folder: FolderArtifact::Absent,
            },
        };
        let result = drive(workspace, &repo(), &mut effects, |w, _| {
            calls.borrow_mut().push(match w.status {
                CreationStatus::Running {
                    stage: CreationStage::ReservingFolder,
                } => "reserve_intent",
                CreationStatus::Running {
                    stage: CreationStage::CreatingBranch,
                } => "branch_intent",
                CreationStatus::Running {
                    stage: CreationStage::CreatingWorktree,
                } => "worktree_intent",
                CreationStatus::Ready => "ready",
                _ => "save",
            });
            Ok(())
        })
        .unwrap();
        assert_eq!(result.status, CreationStatus::Ready);
        let calls = calls.borrow();
        for (intent, effect) in [
            ("reserve_intent", "reserve"),
            ("branch_intent", "branch"),
            ("worktree_intent", "worktree"),
        ] {
            assert!(
                calls.iter().position(|c| *c == intent).unwrap()
                    < calls.iter().position(|c| *c == effect).unwrap()
            );
        }
        assert_eq!(calls.last(), Some(&"ready"));
    }
    #[test]
    fn failed_pinned_intent_prevents_every_mutation() {
        let calls = RefCell::new(vec![]);
        let workspace = allocated(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
        let mut effects = Effects {
            calls: &calls,
            fail: None,
            quiescent: true,
            artifacts: Artifacts {
                branch: BranchArtifact::Absent,
                folder: FolderArtifact::Absent,
            },
        };
        assert!(
            drive(workspace, &repo(), &mut effects, |_, _| Err(
                RepositoryError::storage("full")
            ))
            .is_err()
        );
        assert!(
            !calls
                .borrow()
                .iter()
                .any(|c| ["reserve", "branch", "worktree"].contains(c))
        );
    }
    #[test]
    fn lost_effect_replies_require_reconciliation_and_skip_existing_artifacts() {
        for stage in [
            CreationStage::CreatingBranch,
            CreationStage::CreatingWorktree,
        ] {
            let calls = RefCell::new(vec![]);
            let w = allocated(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
            let mut effects = Effects {
                calls: &calls,
                fail: Some(stage),
                quiescent: true,
                artifacts: Artifacts {
                    branch: BranchArtifact::Absent,
                    folder: FolderArtifact::Absent,
                },
            };
            let failed = drive(w, &repo(), &mut effects, |_, _| Ok(())).unwrap();
            assert!(matches!(failed.status, CreationStatus::Uncertain { .. }));
            assert_eq!(calls.borrow().iter().filter(|c| **c == "branch").count(), 1);
            effects.fail = None;
            let pinned = failed.base.clone();
            let restored: Workspace =
                serde_json::from_str(&serde_json::to_string(&failed).unwrap()).unwrap();
            let ready = drive(restored, &repo(), &mut effects, |_, _| Ok(())).unwrap();
            assert_eq!(ready.status, CreationStatus::Ready);
            assert_eq!(ready.base, pinned);
            for effect in ["branch", "worktree", "resolve"] {
                assert_eq!(
                    calls.borrow().iter().filter(|c| **c == effect).count(),
                    1,
                    "{effect}"
                );
            }
        }
    }
    #[test]
    fn unconfirmed_lease_blocks_effects_and_explicit_reconciliation_can_observe_release() {
        let calls = RefCell::new(vec![]);
        let w = allocated(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
        let mut effects = Effects {
            calls: &calls,
            fail: None,
            quiescent: false,
            artifacts: Artifacts {
                branch: BranchArtifact::Absent,
                folder: FolderArtifact::Absent,
            },
        };
        let uncertain = drive(w, &repo(), &mut effects, |_, _| Ok(())).unwrap();
        assert!(uncertain.cleanup_unconfirmed);
        assert!(calls.borrow().is_empty());
        effects.quiescent = true;
        assert_eq!(
            drive(uncertain, &repo(), &mut effects, |_, _| Ok(()))
                .unwrap()
                .status,
            CreationStatus::Ready
        );
    }
    #[test]
    fn conflicting_artifacts_and_each_failed_store_boundary_block_following_effects() {
        for artifact in [FolderArtifact::Conflict, FolderArtifact::Partial] {
            let calls = RefCell::new(vec![]);
            let w = allocated(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
            let mut effects = Effects {
                calls: &calls,
                fail: None,
                quiescent: true,
                artifacts: Artifacts {
                    branch: BranchArtifact::Owned,
                    folder: artifact,
                },
            };
            assert!(matches!(
                drive(w, &repo(), &mut effects, |_, _| Ok(()))
                    .unwrap()
                    .status,
                CreationStatus::Failed { .. }
            ));
            assert_eq!(*calls.borrow(), ["resolve"]);
        }
        for failed_stage in [
            CreationStage::ReservingFolder,
            CreationStage::CreatingBranch,
            CreationStage::CreatingWorktree,
        ] {
            let calls = RefCell::new(vec![]);
            let w = allocated(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
            let mut effects = Effects {
                calls: &calls,
                fail: None,
                quiescent: true,
                artifacts: Artifacts {
                    branch: BranchArtifact::Absent,
                    folder: FolderArtifact::Absent,
                },
            };
            assert!(
                drive(w, &repo(), &mut effects, |w, _| {
                    if w.status
                        == (CreationStatus::Running {
                            stage: failed_stage,
                        })
                    {
                        Err(RepositoryError::storage("full"))
                    } else {
                        Ok(())
                    }
                })
                .is_err()
            );
            let blocked = match failed_stage {
                CreationStage::ReservingFolder => "reserve",
                CreationStage::CreatingBranch => "branch",
                _ => "worktree",
            };
            assert!(!calls.borrow().contains(&blocked));
        }
    }
}

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<WorkspaceId>();
    exports.add::<PinnedBase>();
    exports.add::<CreationStage>();
    exports.add::<CreationStatus>();
    exports.add::<BranchArtifact>();
    exports.add::<FolderArtifact>();
    exports.add::<Artifacts>();
    exports.add::<Workspace>();
    exports.add::<WorkspaceAction>();
    exports.add::<WorkspaceRequest>();
    exports.add::<WorkspacePage>();
    exports.add::<WorkspaceReply>();
}
