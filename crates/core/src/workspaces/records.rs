//! Domain records shared by policy and private storage.
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    path::{Path, PathBuf},
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
pub(crate) use identity;
identity!(RepositoryId);
identity!(ActionId);
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
identity!(WorkspaceId);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct PinnedBase {
    pub reference: String,
    pub commit: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CreationStage {
    AllocatingName,
    ResolvingBase,
    ReservingFolder,
    CreatingBranch,
    CreatingWorktree,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum CreationStatus {
    Queued,
    Running { stage: CreationStage },
    Ready,
    Failed { error: RepositoryError },
    Uncertain { error: RepositoryError },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum BranchArtifact {
    Unknown,
    Absent,
    Owned,
    Conflict,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
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
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct Artifacts {
    pub branch: BranchArtifact,
    pub folder: FolderArtifact,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct Workspace {
    pub id: WorkspaceId,
    pub repository_id: RepositoryId,
    pub creation_action_id: ActionId,
    pub name: Option<String>,
    pub branch: Option<String>,
    #[serde(deserialize_with = "deserialize_workspace_path")]
    pub path: Option<String>,
    #[serde(default)]
    pub legacy: bool,
    pub base: Option<PinnedBase>,
    pub artifacts: Artifacts,
    pub status: CreationStatus,
    pub cleanup_unconfirmed: bool,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
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
impl Workspace {
    pub(crate) fn path(&self) -> Result<&str> {
        let path = self
            .path
            .as_deref()
            .ok_or_else(|| RepositoryError::storage("Workspace name has not been reserved"))?;
        validate_workspace_path(path)?;
        Ok(path)
    }
    pub(crate) fn branch(&self) -> Result<&str> {
        self.branch
            .as_deref()
            .ok_or_else(|| RepositoryError::storage("Workspace name has not been reserved"))
    }
}

pub(crate) fn validate_workspace_path(value: &str) -> Result<()> {
    use std::path::Component;
    let path = Path::new(value);
    if !path.is_absolute()
        || !path
            .components()
            .all(|part| matches!(part, Component::RootDir | Component::Normal(_)))
        || path.components().collect::<PathBuf>().as_os_str() != path.as_os_str()
    {
        return Err(RepositoryError::storage(
            "Incompatible workspace path: expected a normalized absolute location. Use fresh private development state; saved records and artifacts have not been moved or deleted.",
        ));
    }
    Ok(())
}

fn deserialize_workspace_path<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> std::result::Result<Option<String>, D::Error> {
    let path = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = &path {
        validate_workspace_path(value).map_err(serde::de::Error::custom)?;
    }
    Ok(path)
}
impl WorkspaceAction {
    pub(crate) fn id(&self) -> ActionId {
        match self {
            Self::Create { action_id, .. } | Self::Reconcile { action_id, .. } => *action_id,
        }
    }
}
