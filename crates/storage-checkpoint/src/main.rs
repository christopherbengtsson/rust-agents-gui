//! Bounded actual-storage qualification, explicitly outside application units.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let Some(root) = std::env::args_os().nth(1) else {
        eprintln!("Usage: qualify-storage <new disposable root>");
        std::process::exit(2);
    };
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(45),
        storage_checkpoint::qualify(std::path::Path::new(&root)),
    )
    .await;
    match result {
        Ok(Ok(report)) => println!(
            "{}",
            serde_json::to_string_pretty(&report).expect("qualification report serializes")
        ),
        failure => {
            eprintln!("Storage qualification failed: {failure:?}; retained state: {root:?}");
            std::process::exit(1);
        }
    }
}
