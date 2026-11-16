use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tauri::{Emitter, Manager};
use workspace_core::{
    connection::RequestFailure,
    native::{NativeHost, NativeReply},
};
type Host = Arc<NativeHost>;
struct ExitSignals(std::sync::Mutex<Option<tauri::async_runtime::JoinHandle<()>>>);
#[cfg(feature = "qualification")]
mod qualification;

#[tauri::command]
fn core_status(host: tauri::State<'_, Host>) -> workspace_core::CoreSnapshot {
    host.snapshot()
}
#[tauri::command]
fn desktop_pid() -> workspace_core::DesktopPid {
    std::process::id()
}
#[tauri::command]
async fn repository_request(
    expected_incarnation: String,
    request: workspace_core::workspaces::RepositoryRequest,
    host: tauri::State<'_, Host>,
) -> Result<NativeReply<workspace_core::workspaces::RepositoryReply>, RequestFailure> {
    host.repository_request(expected_incarnation, request).await
}
#[tauri::command]
async fn workspace_request(
    expected_incarnation: String,
    request: workspace_core::workspaces::WorkspaceRequest,
    host: tauri::State<'_, Host>,
) -> Result<NativeReply<workspace_core::workspaces::WorkspaceReply>, RequestFailure> {
    host.workspace_request(expected_incarnation, request).await
}
#[tauri::command]
async fn provider_request(
    expected_incarnation: String,
    request: workspace_core::runtimes::ProviderRequest,
    host: tauri::State<'_, Host>,
) -> Result<NativeReply<workspace_core::runtimes::ProviderReply>, RequestFailure> {
    host.provider_request(expected_incarnation, request).await
}
#[tauri::command]
async fn observation_subscribe(
    expected_incarnation: String,
    subscription_id: String,
    after: Option<workspace_core::observations::FeedCursor>,
    on_event: tauri::ipc::Channel<workspace_core::connection::FeedEvent>,
    host: tauri::State<'_, Host>,
) -> Result<(), RequestFailure> {
    host.replace_subscription(expected_incarnation, subscription_id, after, move |event| {
        on_event
            .send(event)
            .map_err(|error| std::io::Error::other(error.to_string()))
    })
    .await
}
#[tauri::command]
fn observation_acknowledge(
    acknowledgement: workspace_core::connection::FeedAck,
    host: tauri::State<'_, Host>,
) -> Result<(), RequestFailure> {
    host.acknowledge_subscription(acknowledgement)
}
#[tauri::command]
async fn observation_stop(
    subscription_id: String,
    host: tauri::State<'_, Host>,
) -> Result<(), RequestFailure> {
    host.stop_subscription(&subscription_id).await
}

// All ordinary exits, including launcher signals, use this completed-drain rule.
// Future restart/update callers must await host.finished(), check quit_status(),
// and only then request framework restart; framework restart bypasses ExitRequested.
fn main() {
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();
    let monitoring = Arc::new(AtomicBool::new(false));
    let completed = Arc::new(AtomicBool::new(false));
    tauri::Builder::default()
        .menu(|app| {
            let menu = tauri::menu::Menu::default(app)?;
            // In pinned Tauri 2.12.5's macOS default, the final application-menu
            // item is native Quit. Muda maps it directly to NSApp terminate:,
            // bypassing cancellable ExitRequested. Preserve the other defaults
            // and route Quit through Tauri's request-exit path instead.
            if let Some(tauri::menu::MenuItemKind::Submenu(application)) = menu.items()?.first() {
                let items = application.items()?;
                if let Some(position) = items.len().checked_sub(1) {
                    application.remove_at(position)?;
                    let quit = tauri::menu::MenuItem::with_id(
                        app,
                        "quit-owned",
                        "Quit Agent Workspace",
                        true,
                        Some("CmdOrCtrl+Q"),
                    )?;
                    application.insert(&quit, position)?;
                }
            }
            Ok(menu)
        })
        .on_menu_event(|app, event| {
            if event.id().as_ref() == "quit-owned" {
                app.exit(0);
            }
        })
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .on_page_load(|webview, payload| {
            #[cfg(feature = "qualification")]
            qualification::loaded(webview, payload);
            #[cfg(not(feature = "qualification"))]
            let _ = (webview, payload);
        })
        .setup(|app| {
            let directory = std::env::var_os("AGENT_WORKSPACE_STATE_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or(
                    app.path()
                        .data_dir()?
                        .join("AgentWorkspace/native-schema-4"),
                );
            let runtime = tauri::async_runtime::handle();
            let _entered = runtime.inner().enter();
            use tokio::signal::unix::{SignalKind, signal};
            let mut terminate = signal(SignalKind::terminate())?;
            let mut interrupt = signal(SignalKind::interrupt())?;
            let host = NativeHost::start(directory);
            #[cfg(feature = "qualification")]
            if std::env::var_os("AW_NATIVE_QUIT_ON_START").is_some() {
                qualification::record(serde_json::json!({ "stage": "quit-during-startup", "snapshot": host.snapshot() }))?;
                host.begin_quit();
                app.handle().exit(0);
            }
            app.manage(host);
            let app_handle = app.handle().clone();
            let signals = tauri::async_runtime::spawn(async move {
                loop {
                    tokio::select! { _ = terminate.recv() => {}, _ = interrupt.recv() => {} }
                    app_handle.exit(0);
                }
            });
            app.manage(ExitSignals(std::sync::Mutex::new(Some(signals))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            core_status,
            desktop_pid,
            repository_request,
            provider_request,
            workspace_request,
            observation_subscribe,
            observation_acknowledge,
            observation_stop,
            #[cfg(feature = "qualification")]
            qualification::record,
            #[cfg(feature = "qualification")]
            qualification::replace_webview
        ])
        .build(tauri::generate_context!())
        .expect("failed to build desktop")
        .run(move |app, event| {
            if let tauri::RunEvent::Reopen {
                has_visible_windows: false,
                ..
            } = event
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.show();
                let _ = window.set_focus();
            }
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                if completed.load(Ordering::Acquire) {
                    return;
                }
                api.prevent_exit();
                let host = app.state::<Host>().inner().clone();
                host.begin_quit();
                if monitoring.swap(true, Ordering::AcqRel) {
                    return;
                }
                let app = app.clone();
                let completed = completed.clone();
                let monitoring = monitoring.clone();
                let _ = app.emit("owned-shutdown", "Stopping native core activity…");
                tauri::async_runtime::spawn(async move {
                    if tokio::time::timeout(std::time::Duration::from_secs(10), host.finished()).await.is_err() {
                        let _ = app.emit("owned-shutdown", "Native shutdown is still pending. Ownership and cleanup handles are retained; keep the app open. Repeated Quit observes this same drain.");
                        host.finished().await;
                    }
                    if host.quit_status().complete {
                        let signals = app
                            .state::<ExitSignals>()
                            .0
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .take();
                        if let Some(signals) = signals {
                            signals.abort();
                            let _ = signals.await;
                        }
                        host.release_lease();
                        completed.store(true, Ordering::Release);
                        app.exit(0);
                    } else {
                        monitoring.store(false, Ordering::Release);
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let message = host.snapshot().diagnostic.unwrap_or_else(|| "Native cleanup remains unconfirmed; keep the app open.".into());
                        let _ = app.emit("owned-shutdown", message);
                    }
                });
            }
        });
}
