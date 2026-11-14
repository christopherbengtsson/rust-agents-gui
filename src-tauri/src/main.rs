use std::{path::PathBuf, sync::Mutex};
use tauri::Manager;
use workspace_core::connection::{Delivery, OwnedReply, RequestFailure, TransportCode};
use workspace_core::workspaces::{RepositoryReply, RepositoryRequest};
use workspace_core::{SupervisorSnapshot, connection::SupervisorConnection};

#[tauri::command]
async fn repository_request(
    expected_owner: String,
    request: RepositoryRequest,
    connection: tauri::State<'_, std::sync::Arc<Mutex<Option<SupervisorConnection>>>>,
) -> Result<OwnedReply<RepositoryReply>, RequestFailure> {
    let admitted = connected(&connection)?.prepare_repository_request(expected_owner, request)?;
    tauri::async_runtime::spawn_blocking(move || admitted.run())
        .await
        .map_err(|e| {
            RequestFailure::transport(
                TransportCode::ReplyUnavailable,
                Delivery::Unknown,
                format!("Repository request task failed: {e}. Refresh recorded action status."),
            )
        })?
}

#[tauri::command]
async fn workspace_request(
    expected_owner: String,
    request: workspace_core::workspaces::WorkspaceRequest,
    connection: tauri::State<'_, std::sync::Arc<Mutex<Option<SupervisorConnection>>>>,
) -> Result<OwnedReply<workspace_core::workspaces::WorkspaceReply>, RequestFailure> {
    let admitted = connected(&connection)?.prepare_workspace_request(expected_owner, request)?;
    tauri::async_runtime::spawn_blocking(move || admitted.run())
        .await
        .map_err(|e| {
            RequestFailure::transport(
                TransportCode::ReplyUnavailable,
                Delivery::Unknown,
                format!("Workspace request task failed: {e}. Refresh recorded action status."),
            )
        })?
}

fn connected(
    connection: &Mutex<Option<SupervisorConnection>>,
) -> Result<SupervisorConnection, RequestFailure> {
    connection
        .lock()
        .map_err(|e| repository_connection_error(e.to_string()))?
        .clone()
        .ok_or_else(|| repository_connection_error("Connect to the supervisor first".into()))
}

fn repository_connection_error(message: String) -> RequestFailure {
    RequestFailure::transport(TransportCode::NotConnected, Delivery::NotSent, message)
}

#[tauri::command]
async fn supervisor_status(
    app: tauri::AppHandle,
    connection: tauri::State<'_, std::sync::Arc<Mutex<Option<SupervisorConnection>>>>,
) -> Result<SupervisorSnapshot, RequestFailure> {
    let state = std::env::var_os("AGENT_WORKSPACE_STATE_DIR")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| app.path().data_dir().map(|p| p.join("AgentWorkspace")))
        .map_err(|e| repository_connection_error(e.to_string()))?;
    let executable = std::env::var_os("AGENT_WORKSPACE_SUPERVISOR")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(|| {
            app.path()
                .resolve("workspace-supervisor", tauri::path::BaseDirectory::Resource)
        })
        .map_err(|e| repository_connection_error(e.to_string()))?;
    let client = {
        let mut connection = connection
            .lock()
            .map_err(|e| repository_connection_error(e.to_string()))?;
        connection
            .get_or_insert_with(|| SupervisorConnection::new(state, executable))
            .clone()
    };
    let admitted = client.prepare_snapshot()?;
    tauri::async_runtime::spawn_blocking(move || {
        admitted
            .run()
            .map_err(|e| repository_connection_error(e.to_string()))
    })
    .await
    .map_err(|e| {
        RequestFailure::transport(
            TransportCode::ReplyUnavailable,
            Delivery::Unknown,
            format!("Status task failed: {e}"),
        )
    })?
}

#[tauri::command]
fn desktop_pid() -> u32 {
    std::process::id()
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(std::sync::Arc::new(Mutex::new(
            None::<SupervisorConnection>,
        )))
        .invoke_handler(tauri::generate_handler![
            supervisor_status,
            desktop_pid,
            repository_request,
            workspace_request
        ])
        .run(tauri::generate_context!())
        .expect("failed to run desktop");
}
