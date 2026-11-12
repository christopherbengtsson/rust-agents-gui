use std::{path::PathBuf, sync::Mutex};
use tauri::Manager;
use workspace_core::{SupervisorSnapshot, connection::SupervisorConnection};

#[tauri::command]
async fn supervisor_status(
    app: tauri::AppHandle,
    connection: tauri::State<'_, std::sync::Arc<Mutex<Option<SupervisorConnection>>>>,
) -> Result<SupervisorSnapshot, String> {
    let state = std::env::var_os("AGENT_WORKSPACE_STATE_DIR")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| app.path().data_dir().map(|p| p.join("AgentWorkspace")))
        .map_err(|e| e.to_string())?;
    let executable = std::env::var_os("AGENT_WORKSPACE_SUPERVISOR")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| {
            app.path()
                .resolve("workspace-supervisor", tauri::path::BaseDirectory::Resource)
        })
        .map_err(|e| e.to_string())?;
    let connection = connection.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut connection = connection.lock().map_err(|e| e.to_string())?;
        connection
            .get_or_insert_with(|| SupervisorConnection::new(state, executable))
            .snapshot()
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|error| format!("Status task failed: {error}"))?
}

#[tauri::command]
fn desktop_pid() -> u32 {
    std::process::id()
}

fn main() {
    tauri::Builder::default()
        .manage(std::sync::Arc::new(Mutex::new(
            None::<SupervisorConnection>,
        )))
        .invoke_handler(tauri::generate_handler![supervisor_status, desktop_pid])
        .run(tauri::generate_context!())
        .expect("failed to run desktop");
}
