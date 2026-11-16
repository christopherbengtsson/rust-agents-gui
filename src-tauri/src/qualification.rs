//! Explicit fixture-only WebView driver. Absent from normal desktop builds.
use std::io::Write;
use tauri::webview::{PageLoadEvent, PageLoadPayload};
#[tauri::command]
pub fn record(value: serde_json::Value) -> Result<(), String> {
    if std::env::var_os("AGENT_WORKSPACE_CODEX_FIXTURE").is_none() {
        return Err("Fixture not configured".into());
    }
    let path = std::env::var_os("AW_NATIVE_REPORT").ok_or("No fixture report")?;
    let line = serde_json::to_string(&value).map_err(|e| e.to_string())?;
    if line.len() > 64 * 1024 {
        return Err("Fixture record exceeds budget".into());
    }
    let mut output = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(output, "{line}").map_err(|e| e.to_string())
}
pub fn loaded(webview: &tauri::Webview, payload: &PageLoadPayload<'_>) {
    if webview.label() != "main"
        || payload.event() != PageLoadEvent::Finished
        || std::env::var_os("AGENT_WORKSPACE_CODEX_FIXTURE").is_none()
    {
        return;
    }
    if let Some(path) = std::env::var_os("AW_NATIVE_SCRIPT") {
        match std::fs::read_to_string(path) {
            Ok(script) => {
                if let Err(error) = webview.eval(script) {
                    eprintln!("Qualification injection failed: {error}");
                }
            }
            Err(error) => eprintln!("Qualification script unavailable: {error}"),
        }
    }
}

#[tauri::command]
pub async fn replace_webview(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Manager;
    if std::env::var_os("AGENT_WORKSPACE_CODEX_FIXTURE").is_none() {
        return Err("Fixture not configured".into());
    }
    // Keep a native window during replacement so last-window auto-exit is not
    // confused with an explicit Quit. It owns no domain execution.
    let holding = tauri::WebviewWindowBuilder::new(
        &app,
        "replacement-hold",
        tauri::WebviewUrl::External("about:blank".parse().expect("static fixture URL")),
    )
    .visible(false)
    .build()
    .map_err(|e| e.to_string())?;
    if let Some(window) = app.get_webview_window("main") {
        window.destroy().map_err(|e| e.to_string())?;
    }
    let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(3);
    while app.get_webview_window("main").is_some() {
        if tokio::time::Instant::now() >= deadline {
            return Err("Old WebView destruction has not completed".into());
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    tauri::WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
        .title("Agent Workspace — native fixture")
        .inner_size(900.0, 680.0)
        .build()
        .map_err(|e| e.to_string())?;
    holding.destroy().map_err(|e| e.to_string())?;
    Ok(())
}
