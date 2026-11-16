//! Explicit actual-process qualification, excluded from application units.
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let arguments: Vec<_> = std::env::args_os().collect();
    if arguments.len() == 4 && arguments[1] == "--parent-loss-owner" {
        workspace_core::qualify_process_parent_loss(
            std::path::Path::new(&arguments[2]),
            std::path::Path::new(&arguments[3]),
        )
        .await
        .unwrap();
        return;
    }
    if arguments.len() != 3 {
        eprintln!("Usage: qualify-process-effects <new private root> <fixture.py>");
        std::process::exit(2);
    }
    match workspace_core::qualify_process_effects(
        std::path::Path::new(&arguments[1]),
        std::path::Path::new(&arguments[2]),
    )
    .await
    {
        Ok(report) => println!("{}", serde_json::to_string_pretty(&report).unwrap()),
        Err(error) => {
            eprintln!("Process qualification failed: {error}");
            std::process::exit(1);
        }
    }
}
