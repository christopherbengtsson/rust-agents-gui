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
fn observation_subscribe(
    expected_owner: String,
    subscription_id: String,
    after: Option<workspace_core::observations::FeedCursor>,
    on_event: tauri::ipc::Channel<workspace_core::connection::FeedEvent>,
    connection: tauri::State<'_, std::sync::Arc<Mutex<Option<SupervisorConnection>>>>,
) -> Result<(), RequestFailure> {
    connected(&connection)?.start_subscription(
        expected_owner,
        subscription_id,
        after,
        move |event| {
            on_event
                .send(event)
                .map_err(|error| std::io::Error::other(error.to_string()))
        },
    )
}

#[tauri::command]
fn observation_acknowledge(
    acknowledgement: workspace_core::connection::FeedAck,
    connection: tauri::State<'_, std::sync::Arc<Mutex<Option<SupervisorConnection>>>>,
) -> Result<(), RequestFailure> {
    connected(&connection)?.acknowledge_subscription(acknowledgement)
}

#[tauri::command]
async fn observation_stop(
    subscription_id: String,
    connection: tauri::State<'_, std::sync::Arc<Mutex<Option<SupervisorConnection>>>>,
) -> Result<(), RequestFailure> {
    let client = connected(&connection)?;
    tauri::async_runtime::spawn_blocking(move || client.stop_subscription(&subscription_id))
        .await
        .map_err(|error| repository_connection_error(error.to_string()))?
}

#[tauri::command]
fn desktop_pid() -> u32 {
    std::process::id()
}

fn main() {
    let connection = std::sync::Arc::new(Mutex::new(None::<SupervisorConnection>));
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(connection.clone())
        .invoke_handler(tauri::generate_handler![
            supervisor_status,
            desktop_pid,
            repository_request,
            workspace_request,
            observation_subscribe,
            observation_acknowledge,
            observation_stop
        ])
        .run(tauri::generate_context!())
        .expect("failed to run desktop");
    // The event loop has finished; shutdown/join never blocks rendering.
    if let Ok(client) = connected(&connection) {
        let _ = client.stop_observations();
    }
}
