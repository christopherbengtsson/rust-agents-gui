//! UI-independent application interface hosted by the local supervisor.

pub mod connection;
pub mod supervisor;

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
        }
    }

    pub fn status(&self) -> SupervisorStatus {
        SupervisorStatus {
            state: SupervisorState::Ready,
            supervisor_pid: std::process::id(),
            application_version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

impl Core {
    pub fn snapshot(&self, after: Option<SnapshotCursor>) -> SupervisorSnapshot {
        let cursor = SnapshotCursor {
            owner: self.owner.clone(),
            revision: 1,
        };
        SupervisorSnapshot {
            status: self.status(),
            reset: after.as_ref() != Some(&cursor),
            cursor,
        }
    }
}
