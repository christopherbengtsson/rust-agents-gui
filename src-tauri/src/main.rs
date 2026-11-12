use workspace_core::{SupervisorStatus, connection::SupervisorClient};

#[tauri::command]
async fn supervisor_status() -> Result<SupervisorStatus, String> {
    let socket = std::env::var_os("AGENT_WORKSPACE_SOCKET").ok_or(
        "Supervisor endpoint is missing. Start the bootstrap with pnpm dev or pnpm start.",
    )?;
    tauri::async_runtime::spawn_blocking(move || SupervisorClient::new(socket.into()).status())
        .await
        .map_err(|error| format!("Status task failed: {error}"))?
        .map_err(|error| format!("Cannot read supervisor status: {error}. Check the launcher terminal, then refresh status."))
}

#[tauri::command]
fn desktop_pid() -> u32 {
    std::process::id()
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![supervisor_status, desktop_pid])
        .run(tauri::generate_context!())
        .expect("failed to run desktop");
}
