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

identity!(WorkspaceId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinnedBase {
    pub reference: String,
    pub commit: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CreationStage {
    ResolvingBase,
    ReservingFolder,
    CreatingBranch,
    CreatingWorktree,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CreationStatus {
    Queued,
    Running { stage: CreationStage },
    Ready,
    Failed { error: RepositoryError },
    Uncertain { error: RepositoryError },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BranchArtifact {
    Unknown,
    Absent,
    Owned,
    Conflict,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FolderArtifact {
    Unknown,
    Absent,
    Reserved,
    Worktree,
    Partial,
    Conflict,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifacts {
    pub branch: BranchArtifact,
    pub folder: FolderArtifact,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Workspace {
    pub id: WorkspaceId,
    pub repository_id: RepositoryId,
    pub creation_action_id: ActionId,
    pub name: String,
    pub branch: String,
    pub path: String,
    pub provider: String,
    pub base: Option<PinnedBase>,
    pub artifacts: Artifacts,
    pub status: CreationStatus,
    pub cleanup_unconfirmed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum WorkspaceAction {
    Create {
        action_id: ActionId,
        repository_id: RepositoryId,
    },
    Reconcile {
        action_id: ActionId,
        workspace_id: WorkspaceId,
    },
}
impl WorkspaceAction {
    pub(crate) fn id(&self) -> ActionId {
        match self {
            Self::Create { action_id, .. } | Self::Reconcile { action_id, .. } => *action_id,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum WorkspaceRequest {
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
pub struct WorkspacePage {
    pub workspaces: Vec<Workspace>,
    pub next: Option<WorkspaceId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkspaceReply {
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

pub(crate) trait CreationStore: Send {
    fn workspace(&mut self, id: WorkspaceId) -> Result<Option<Workspace>>;
    fn workspace_action(&mut self, id: ActionId) -> Result<Option<(WorkspaceAction, WorkspaceId)>>;
    fn save_workspace(
        &mut self,
        workspace: &Workspace,
        action: Option<&WorkspaceAction>,
    ) -> Result<()>;
    fn workspace_pending(&mut self) -> Result<Option<Workspace>>;
    fn workspace_count_pending(&mut self) -> Result<usize>;
    fn workspace_cleanup_blocked(&mut self, repository: RepositoryId) -> Result<bool>;
    fn workspace_list(
        &mut self,
        repository: RepositoryId,
        after: Option<WorkspaceId>,
        limit: usize,
    ) -> Result<Vec<Workspace>>;
}

pub(crate) fn allocate(
    repository: RepositoryId,
    action: ActionId,
    directory: &Path,
) -> Result<Workspace> {
    let id = WorkspaceId(Uuid::new_v4());
    let cities = [
        "stockholm",
        "kyoto",
        "lisbon",
        "lima",
        "oslo",
        "macau",
        "porto",
        "vienna",
    ];
    let name = format!(
        "{}-{}",
        cities[usize::from(id.0.as_bytes()[0]) % cities.len()],
        id.0
    );
    let path = directory
        .join("workspaces")
        .join(repository.0.to_string())
        .join(&name)
        .join("checkout");
    let path = path
        .to_str()
        .ok_or_else(|| {
            RepositoryError::new(
                ErrorCode::InvalidPath,
                "The private workspace directory is not UTF-8.",
            )
        })?
        .to_owned();
    let workspace = Workspace {
        id,
        repository_id: repository,
        creation_action_id: action,
        branch: format!("workspace/{name}"),
        name,
        path,
        provider: "codex".into(),
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
    effects: &mut impl CreationEffects,
    mut persist: impl FnMut(&Workspace) -> Result<()>,
) -> Result<Workspace> {
    if !effects.quiescent(&workspace)? {
        workspace.cleanup_unconfirmed = true;
        workspace.status = CreationStatus::Uncertain {
            error: RepositoryError::new(
                ErrorCode::CleanupUnconfirmed,
                "An earlier Git effect still owns this workspace's execution lease. Wait for it to finish, then reconcile. No creation was repeated and no saved PID was signaled.",
            ),
        };
        persist(&workspace)?;
        return Ok(workspace);
    }
    workspace.cleanup_unconfirmed = false;
    // Each fallible store boundary exits before any subsequent mutation.
    workspace.status = CreationStatus::Running {
        stage: CreationStage::ResolvingBase,
    };
    persist(&workspace)?;
    if workspace.base.is_none() {
        match effects.base(repository) {
            Ok(base) => {
                workspace.base = Some(base);
                persist(&workspace)?;
            }
            Err(error) => {
                workspace.status = CreationStatus::Failed { error };
                persist(&workspace)?;
                return Ok(workspace);
            }
        }
    }
    workspace.artifacts = match effects.inspect(repository, &workspace) {
        Ok(artifacts) => artifacts,
        Err(error) => {
            workspace.status = CreationStatus::Uncertain { error };
            persist(&workspace)?;
            return Ok(workspace);
        }
    };
    persist(&workspace)?;
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
        persist(&workspace)?;
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
            CreationStage::ResolvingBase => false,
        };
        if !needed {
            continue;
        }
        workspace.status = CreationStatus::Running { stage };
        persist(&workspace)?;
        let outcome = match stage {
            CreationStage::ReservingFolder => effects.reserve(&workspace),
            CreationStage::CreatingBranch => effects.branch(repository, &workspace),
            CreationStage::CreatingWorktree => effects.worktree(repository, &workspace),
            CreationStage::ResolvingBase => unreachable!(),
        };
        if let Err(error) = outcome {
            workspace.cleanup_unconfirmed = error.code == ErrorCode::CleanupUnconfirmed;
            if !workspace.cleanup_unconfirmed
                && let Ok(artifacts) = effects.inspect(repository, &workspace)
            {
                workspace.artifacts = artifacts;
            }
            workspace.status = CreationStatus::Uncertain { error };
            persist(&workspace)?;
            return Ok(workspace);
        }
        workspace.artifacts = match effects.inspect(repository, &workspace) {
            Ok(artifacts) => artifacts,
            Err(error) => {
                workspace.status = CreationStatus::Uncertain { error };
                persist(&workspace)?;
                return Ok(workspace);
            }
        };
        persist(&workspace)?;
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
    persist(&workspace)?;
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
        let workspace = allocate(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
        let mut effects = Effects {
            calls: &calls,
            fail: None,
            quiescent: true,
            artifacts: Artifacts {
                branch: BranchArtifact::Absent,
                folder: FolderArtifact::Absent,
            },
        };
        let result = drive(workspace, &repo(), &mut effects, |w| {
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
        let workspace = allocate(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
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
            drive(workspace, &repo(), &mut effects, |_| Err(
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
            let w = allocate(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
            let mut effects = Effects {
                calls: &calls,
                fail: Some(stage),
                quiescent: true,
                artifacts: Artifacts {
                    branch: BranchArtifact::Absent,
                    folder: FolderArtifact::Absent,
                },
            };
            let failed = drive(w, &repo(), &mut effects, |_| Ok(())).unwrap();
            assert!(matches!(failed.status, CreationStatus::Uncertain { .. }));
            assert_eq!(calls.borrow().iter().filter(|c| **c == "branch").count(), 1);
            effects.fail = None;
            let pinned = failed.base.clone();
            let restored: Workspace =
                serde_json::from_str(&serde_json::to_string(&failed).unwrap()).unwrap();
            let ready = drive(restored, &repo(), &mut effects, |_| Ok(())).unwrap();
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
        let w = allocate(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
        let mut effects = Effects {
            calls: &calls,
            fail: None,
            quiescent: false,
            artifacts: Artifacts {
                branch: BranchArtifact::Absent,
                folder: FolderArtifact::Absent,
            },
        };
        let uncertain = drive(w, &repo(), &mut effects, |_| Ok(())).unwrap();
        assert!(uncertain.cleanup_unconfirmed);
        assert!(calls.borrow().is_empty());
        effects.quiescent = true;
        assert_eq!(
            drive(uncertain, &repo(), &mut effects, |_| Ok(()))
                .unwrap()
                .status,
            CreationStatus::Ready
        );
    }
    #[test]
    fn conflicting_artifacts_and_each_failed_store_boundary_block_following_effects() {
        for artifact in [FolderArtifact::Conflict, FolderArtifact::Partial] {
            let calls = RefCell::new(vec![]);
            let w = allocate(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
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
                drive(w, &repo(), &mut effects, |_| Ok(())).unwrap().status,
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
            let w = allocate(repo().id, ActionId(Uuid::nil()), Path::new("/state")).unwrap();
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
                drive(w, &repo(), &mut effects, |w| {
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

pub(crate) fn admit_workspace(
    store: &mut dyn RepositoryStore,
    directory: &Path,
    action: &WorkspaceAction,
    active: Option<WorkspaceId>,
) -> Result<Workspace> {
    if let Some((recorded, workspace)) = store.workspace_action(action.id())? {
        if recorded != *action {
            return Err(RepositoryError::new(
                ErrorCode::Conflict,
                "This action ID belongs to a different workspace operation. Retrieve its recorded status.",
            ));
        }
        return store
            .workspace(workspace)?
            .ok_or_else(|| RepositoryError::storage("Workspace action has no saved workspace"));
    }
    if store.action(action.id())?.is_some() {
        return Err(RepositoryError::new(
            ErrorCode::Conflict,
            "This action ID belongs to repository registration.",
        ));
    }
    // Recovery reuses an existing identity and durable slot. Normal admission must
    // never strand interrupted records by consuming their recovery capacity.
    if matches!(action, WorkspaceAction::Create { .. })
        && store.pending_count()? + store.workspace_count_pending()? >= 8
    {
        return Err(RepositoryError::new(
            ErrorCode::Capacity,
            "Eight operations are pending. Retrieve their status before creating another workspace.",
        ));
    }
    let workspace = match action {
        WorkspaceAction::Create {
            action_id,
            repository_id,
        } => {
            if store.repository(*repository_id)?.is_none() {
                return Err(RepositoryError::new(
                    ErrorCode::NotFound,
                    "The requested repository is not registered.",
                ));
            }
            if store.workspace_cleanup_blocked(*repository_id)? {
                return Err(RepositoryError::new(
                    ErrorCode::CleanupUnconfirmed,
                    "A workspace in this repository has unconfirmed Git cleanup. Reconcile it before creating another.",
                ));
            }
            allocate(*repository_id, *action_id, directory)?
        }
        WorkspaceAction::Reconcile { workspace_id, .. } => {
            let mut workspace = store.workspace(*workspace_id)?.ok_or_else(|| {
                RepositoryError::new(
                    ErrorCode::NotFound,
                    "Workspace not found; its action has not been repeated.",
                )
            })?;
            if active == Some(*workspace_id)
                || matches!(
                    workspace.status,
                    CreationStatus::Queued | CreationStatus::Ready
                )
            {
                return Err(RepositoryError::new(
                    ErrorCode::Conflict,
                    "This workspace is already queued, active, or ready. Refresh its status.",
                ));
            }
            workspace.status = CreationStatus::Queued;
            workspace
        }
    };
    store.save_workspace(&workspace, Some(action))?;
    Ok(workspace)
}

impl OwnerState {
    pub(super) fn workspace_observation(&self, mut workspace: Workspace) -> Workspace {
        if matches!(workspace.status, CreationStatus::Running { .. })
            && self.active_workspace != Some(workspace.id)
        {
            workspace.status = CreationStatus::Uncertain {
                error: RepositoryError::new(
                    ErrorCode::Unavailable,
                    "Creation stopped before a durable outcome was recorded. Reconcile this workspace explicitly to inspect its saved branch and folder; reopening has not repeated creation.",
                ),
            };
        }
        if let Some((id, error)) = &self.workspace_fault
            && *id == workspace.id
        {
            workspace.status = CreationStatus::Uncertain {
                error: error.clone(),
            };
        }
        workspace
    }
}
impl Workspaces {
    pub fn workspace_request(&self, request: WorkspaceRequest) -> Result<WorkspaceReply> {
        let mut state = self.state()?;
        let reply = match request {
            WorkspaceRequest::Create {
                action_id,
                repository_id,
            } => {
                let directory = state.directory.clone();
                let active = state.active_workspace;
                let action = WorkspaceAction::Create {
                    action_id,
                    repository_id,
                };
                let workspace = admit_workspace(state.store()?, &directory, &action, active)?;
                WorkspaceReply::Action {
                    action: Some(action),
                    workspace: Some(state.workspace_observation(workspace)),
                }
            }
            WorkspaceRequest::Reconcile {
                action_id,
                workspace_id,
            } => {
                let directory = state.directory.clone();
                let active = state.active_workspace;
                let action = WorkspaceAction::Reconcile {
                    action_id,
                    workspace_id,
                };
                let new_request = state.store()?.workspace_action(action_id)?.is_none();
                // Only a new explicit recovery may re-admit a queued record whose write failed.
                if new_request
                    && state
                        .workspace_fault
                        .as_ref()
                        .is_some_and(|(id, _)| *id == workspace_id)
                    && let Some(mut w) = state.store()?.workspace(workspace_id)?
                {
                    w.status = CreationStatus::Uncertain {
                        error: RepositoryError::storage("Explicit recovery after a failed write"),
                    };
                    state.store()?.save_workspace(&w, None)?;
                }
                let workspace = admit_workspace(state.store()?, &directory, &action, active)?;
                if new_request {
                    state.workspace_fault = None;
                }
                WorkspaceReply::Action {
                    action: Some(action),
                    workspace: Some(state.workspace_observation(workspace)),
                }
            }
            WorkspaceRequest::Action { action_id } => {
                let action = state.store()?.workspace_action(action_id)?;
                let workspace = action
                    .as_ref()
                    .map(|(_, id)| state.store()?.workspace(*id))
                    .transpose()?
                    .flatten();
                WorkspaceReply::Action {
                    action: action.map(|(a, _)| a),
                    workspace: workspace.map(|w| state.workspace_observation(w)),
                }
            }
            WorkspaceRequest::Get { id } => {
                let workspace = state.store()?.workspace(id)?;
                WorkspaceReply::Workspace {
                    workspace: workspace.map(|w| state.workspace_observation(w)),
                }
            }
            WorkspaceRequest::List {
                repository_id,
                after,
            } => {
                let records = state
                    .store()?
                    .workspace_list(repository_id, after, PAGE_SIZE + 1)?;
                let mut page = WorkspacePage {
                    workspaces: vec![],
                    next: None,
                };
                let total = records.len();
                for workspace in records.into_iter().take(PAGE_SIZE) {
                    page.workspaces.push(state.workspace_observation(workspace));
                    if serde_json::to_vec(&page)
                        .map_err(RepositoryError::storage)?
                        .len()
                        > PAGE_BYTES
                    {
                        page.workspaces.pop();
                        if page.workspaces.is_empty() {
                            return Err(RepositoryError::new(
                                ErrorCode::Capacity,
                                "Saved workspace exceeds the navigation budget; its record is retained.",
                            ));
                        }
                        break;
                    }
                }
                if total > page.workspaces.len() {
                    page.next = page.workspaces.last().map(|w| w.id);
                }
                WorkspaceReply::Workspaces { page }
            }
        };
        drop(state);
        self.notify();
        Ok(reply)
    }
}

pub(super) fn process_creation(
    shared: &Mutex<OwnerState>,
    effects: &mut impl CreationEffects,
) -> bool {
    let work = (|| -> Result<Option<(Workspace, RepositoryRecord)>> {
        let mut state = shared.lock().map_err(RepositoryError::storage)?;
        let Some(workspace) = state.store()?.workspace_pending()? else {
            return Ok(None);
        };
        if state
            .workspace_fault
            .as_ref()
            .is_some_and(|(id, _)| *id == workspace.id)
        {
            return Ok(None);
        }
        let repository = state
            .store()?
            .repository(workspace.repository_id)?
            .ok_or_else(|| RepositoryError::storage("Saved workspace repository is missing"))?;
        state.active_workspace = Some(workspace.id);
        Ok(Some((workspace, repository)))
    })();
    let (workspace, repository) = match work {
        Ok(Some(work)) => work,
        _ => return false,
    };
    let id = workspace.id;
    let result = drive(workspace, &repository, effects, |workspace| {
        let mut state = shared.lock().map_err(RepositoryError::storage)?;
        let store = state.store()?;
        if let Err(error) = store.save_workspace(workspace, None)
            && store.workspace(workspace.id)?.as_ref() != Some(workspace)
        {
            return Err(error);
        }
        state.change()
    });
    if let Ok(mut state) = shared.lock() {
        state.active_workspace = None;
        state.workspace_fault = result.err().map(|e| (id, e));
    }
    // A failed persistence boundary must not spin or replay the queued record.
    result_is_success(shared, id)
}
fn result_is_success(shared: &Mutex<OwnerState>, id: WorkspaceId) -> bool {
    shared.lock().is_ok_and(|s| {
        !s.workspace_fault
            .as_ref()
            .is_some_and(|(key, _)| *key == id)
    })
}
