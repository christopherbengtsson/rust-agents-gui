//! Temporary source-linked 02–04 integration checkpoint. No alternate implementation.
#![allow(dead_code, unused_imports)]
#[path = "../../core/src/git.rs"]
mod git;
#[path = "../../core/src/observations.rs"]
mod observations;
#[path = "../../core/src/private_state.rs"]
mod private_state;
#[path = "../../core/src/storage.rs"]
mod storage;
#[path = "../../core/src/workspaces.rs"]
mod workspaces;
mod supervisor {
    pub(crate) use crate::private_state::check_private;
}

#[cfg(feature = "qualification")]
pub async fn qualify(root: &std::path::Path) -> Result<serde_json::Value, String> {
    workspaces::qualification::run(root)
        .await
        .map_err(|e| e.to_string())
}
