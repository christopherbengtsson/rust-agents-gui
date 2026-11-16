//! Source-linked checkpoint: no alternate storage implementation or public product API.
#![allow(dead_code)] // Consumers convert in 03/04; compile the complete private owner here.
#[path = "../../core/src/storage.rs"]
mod storage;
#[allow(unused_imports)] // The macro re-export is consumed by core inspection, absent here.
#[path = "../../core/src/workspaces/records.rs"]
mod workspaces;

#[cfg(feature = "qualification")]
pub async fn qualify(root: &std::path::Path) -> Result<serde_json::Value, String> {
    storage::qualification::run(root)
        .await
        .map_err(|error| format!("{error:?}"))
}
