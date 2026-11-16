//! Explicit actual-effects qualification. Never an application unit test.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Some(root) = std::env::args_os().nth(1) else {
        eprintln!("Usage: qualify-workspaces <new disposable root>");
        std::process::exit(2);
    };
    match workspace_checkpoint::qualify(std::path::Path::new(&root)).await {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
        Err(error) => {
            eprintln!("Qualification failed: {error}; retained root {root:?}");
            std::process::exit(1);
        }
    }
}
