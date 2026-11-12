//! UI-independent application interface hosted by the local supervisor.

pub mod connection;

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

#[derive(Default)]
pub struct Core;

impl Core {
    pub fn new() -> Self {
        Self
    }

    pub fn status(&self) -> SupervisorStatus {
        SupervisorStatus {
            state: SupervisorState::Ready,
            supervisor_pid: std::process::id(),
            application_version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}
