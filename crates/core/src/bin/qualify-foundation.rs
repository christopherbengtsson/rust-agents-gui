//! Opt-in actual SQLx/workspace qualification; never an application unit.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Some(root) = std::env::args_os().nth(1) else {
        eprintln!("Usage: qualify-foundation <new disposable root>");
        std::process::exit(2);
    };
    let root = std::path::Path::new(&root);
    let result = async {
        std::fs::DirBuilder::new()
            .create(root)
            .map_err(|e| e.to_string())?;
        let storage = workspace_core::qualification::storage(&root.join("storage")).await?;
        let workspaces =
            workspace_core::qualification::workspaces(&root.join("workspaces")).await?;
        Ok::<_, String>(serde_json::json!({"storage":storage, "workspaces":workspaces}))
    }
    .await;
    match result {
        Ok(report) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("report serializes")
        ),
        Err(error) => {
            eprintln!("Foundation qualification failed: {error}; retained root {root:?}");
            std::process::exit(1);
        }
    }
}
