//! UI-independent application interface hosted by the local supervisor.

pub mod connection;
mod git;
pub mod runtimes;
mod storage;
pub mod supervisor;
pub mod workspaces;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisorState {
    Ready,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorStatus {
    pub state: SupervisorState,
    pub supervisor_pid: u32,
    pub application_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotCursor {
    pub owner: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorSnapshot {
    pub status: SupervisorStatus,
    pub cursor: SnapshotCursor,
    /// A full snapshot replaces observations from another owner or cursor.
    pub reset: bool,
}

pub struct Core {
    owner: String,
    pid: u32,
    workspaces: Option<workspaces::Workspaces>,
    runtimes: Option<runtimes::Runtimes>,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    pub fn new() -> Self {
        Self {
            owner: format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system clock before epoch")
                    .as_nanos()
            ),
            pid: std::process::id(),
            workspaces: None,
            runtimes: None,
        }
    }

    /// Called only by the admitted supervisor owner, never by presentation clients.
    pub fn with_state(directory: &std::path::Path) -> Result<Self, workspaces::RepositoryError> {
        let mut core = Self::new();
        core.workspaces = Some(workspaces::Workspaces::open(directory)?);
        core.runtimes = Some(runtimes::Runtimes::open(directory, core.owner.clone()));
        Ok(core)
    }

    pub fn runtime_request(
        &self,
        request: runtimes::RuntimeRequest,
    ) -> Result<runtimes::RuntimeReply, runtimes::RuntimeError> {
        self.runtimes
            .as_ref()
            .ok_or_else(runtimes::RuntimeError::storage)?
            .request(request)
    }

    /// Host lifecycle seam. A failed bounded stop retains runtime ownership and
    /// must not be reported as successful supervisor shutdown.
    pub fn shutdown_runtimes(&mut self) -> Result<(), runtimes::RuntimeError> {
        self.runtimes
            .as_mut()
            .map_or(Ok(()), runtimes::Runtimes::shutdown)
    }

    pub fn repository_request(
        &self,
        request: workspaces::RepositoryRequest,
    ) -> Result<workspaces::RepositoryReply, workspaces::RepositoryError> {
        self.workspaces
            .as_ref()
            .ok_or_else(|| {
                workspaces::RepositoryError::storage(
                    "Repository storage has not been opened by the execution owner",
                )
            })?
            .request(request)
    }

    pub fn workspace_request(
        &self,
        request: workspaces::WorkspaceRequest,
    ) -> Result<workspaces::WorkspaceReply, workspaces::RepositoryError> {
        self.workspaces
            .as_ref()
            .ok_or_else(|| {
                workspaces::RepositoryError::storage(
                    "Workspace storage has not been opened by the execution owner",
                )
            })?
            .workspace_request(request)
    }

    pub fn status(&self) -> SupervisorStatus {
        SupervisorStatus {
            state: SupervisorState::Ready,
            supervisor_pid: self.pid,
            application_version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

impl Core {
    pub fn snapshot(&self, after: Option<SnapshotCursor>) -> SupervisorSnapshot {
        let cursor = SnapshotCursor {
            owner: self.owner.clone(),
            revision: self.workspaces.as_ref().map_or(1, |w| w.revision()),
        };
        SupervisorSnapshot {
            status: self.status(),
            reset: after.as_ref() != Some(&cursor),
            cursor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> Core {
        Core {
            owner: "test-owner".into(),
            pid: 42,
            workspaces: None,
            runtimes: None,
        }
    }

    #[test]
    fn initial_status_reports_the_execution_owner_without_starting_work() {
        let status = core().status();
        assert_eq!(status.state, SupervisorState::Ready);
        assert_eq!(status.supervisor_pid, 42);
        assert_eq!(status.application_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn snapshot_catches_up_or_resets_observations_from_another_owner() {
        let core = core();
        let first = core.snapshot(None);
        assert!(first.reset);
        let caught_up = core.snapshot(Some(first.cursor.clone()));
        assert!(!caught_up.reset);
        assert_eq!(caught_up.cursor, first.cursor);
        for cursor in [
            SnapshotCursor {
                owner: "previous-owner".into(),
                revision: 1,
            },
            SnapshotCursor {
                owner: first.cursor.owner.clone(),
                revision: 0,
            },
            SnapshotCursor {
                owner: first.cursor.owner.clone(),
                revision: 999,
            },
        ] {
            let snapshot = core.snapshot(Some(cursor));
            assert!(snapshot.reset);
            assert_eq!(snapshot.cursor, first.cursor);
        }
    }
}
