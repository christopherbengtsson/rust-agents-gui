//! UI-independent application core hosted by native Tauri.

#[cfg(feature = "codegen")]
pub mod codegen;
pub mod connection;
mod git;
mod lifetime;
pub mod native;
pub mod observations;
mod private_state;
mod process_effects;
pub mod runtimes;
mod storage;
pub mod supervisor;
pub mod workspaces;

use serde::{Deserialize, Serialize};

/// The desktop identity command returns this scalar wire type.
pub type DesktopPid = u32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum CoreState {
    Starting,
    Recovering,
    Ready,
    Failed,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct CoreStatus {
    pub state: CoreState,
    pub native_pid: u32,
    pub application_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct SnapshotCursor {
    pub incarnation: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct CoreSnapshot {
    pub status: CoreStatus,
    pub cursor: SnapshotCursor,
    /// A full snapshot replaces observations from another incarnation or cursor.
    pub reset: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub provider_login_pending: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub diagnostic: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuitStatus {
    pub complete: bool,
    pub diagnostic: Option<String>,
}

pub struct Core {
    keepalive: std::sync::Mutex<Option<std::sync::Arc<Self>>>,
    lifetime: std::sync::Arc<lifetime::Lifetime>,
    exit_requested: std::sync::atomic::AtomicBool,
    incarnation: String,
    pid: u32,
    providers: Option<std::sync::Arc<runtimes::ProviderConnections>>,
    workspaces: Option<workspaces::Workspaces>,
    storage: Option<storage::Storage>,
    processes: Option<process_effects::Processes>,
    journal: std::sync::Arc<observations::Journal>,
}

impl Default for Core {
    fn default() -> Self {
        Self::new()
    }
}

impl Core {
    pub fn new() -> Self {
        Self {
            keepalive: std::sync::Mutex::new(None),
            lifetime: std::sync::Arc::default(),
            exit_requested: std::sync::atomic::AtomicBool::new(false),
            incarnation: format!(
                "{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system clock before epoch")
                    .as_nanos()
            ),
            pid: std::process::id(),
            providers: None,
            workspaces: None,
            storage: None,
            processes: None,
            journal: std::sync::Arc::new(observations::Journal::default()),
        }
    }

    /// Startup and any partial cleanup are retained independently of the requesting host.
    pub async fn start(
        directory: std::path::PathBuf,
        lease: supervisor::StateLease,
    ) -> std::sync::Arc<Self> {
        Self::start_native(
            directory,
            std::sync::Arc::new(lifetime::Lifetime::starting(lease)),
        )
        .await
    }
    async fn start_native(
        directory: std::path::PathBuf,
        lifetime: std::sync::Arc<lifetime::Lifetime>,
    ) -> std::sync::Arc<Self> {
        let workspace_directory = directory.clone();
        let processes = process_effects::Processes::new();
        let workspace_processes = processes.clone();
        let provider_processes = processes.clone();
        Self::start_with(
            lifetime,
            Some(processes),
            move || async move {
                let directory = tokio::task::spawn_blocking(move || {
                    supervisor::check_private(&workspace_directory, true)?;
                    std::fs::canonicalize(workspace_directory)
                })
                .await
                .map_err(workspaces::RepositoryError::storage)?
                .map_err(workspaces::RepositoryError::storage)?;
                let mut opening = storage::Storage::start(directory.clone());
                let storage = opening
                    .wait()
                    .await
                    .map_err(workspaces::RepositoryError::storage)?;
                Ok(workspaces::Workspaces::open(
                    directory,
                    storage,
                    workspace_processes,
                ))
            },
            move |journal, storage| async move {
                runtimes::ProviderConnections::open(
                    journal,
                    provider_processes,
                    storage.ok_or_else(|| {
                        workspaces::RepositoryError::storage("Shared storage is unavailable")
                    })?,
                )
                .await
            },
        )
        .await
    }
    async fn start_with<F, P>(
        lifetime: std::sync::Arc<lifetime::Lifetime>,
        processes: Option<process_effects::Processes>,
        open_workspace: impl FnOnce() -> F + Send + 'static,
        open_provider: impl FnOnce(std::sync::Arc<observations::Journal>, Option<storage::Storage>) -> P
        + Send
        + 'static,
    ) -> std::sync::Arc<Self>
    where
        P: std::future::Future<
                Output = Result<runtimes::ProviderConnections, workspaces::RepositoryError>,
            > + Send
            + 'static,
        F: std::future::Future<
                Output = Result<workspaces::Workspaces, workspaces::RepositoryError>,
            > + Send
            + 'static,
    {
        let task = tokio::spawn(async move {
            let mut core = Self {
                lifetime,
                processes,
                ..Self::new()
            };
            let opened = tokio::spawn(open_workspace()).await;
            let failure = match opened {
                Ok(Ok(workspaces)) => {
                    core.lifetime.recovering();
                    core.journal = workspaces.journal();
                    core.storage = workspaces.storage.clone();
                    core.workspaces = Some(workspaces);
                    let journal = core.journal.clone();
                    let storage = core.storage.clone();
                    match tokio::spawn(open_provider(journal, storage)).await {
                        Ok(Ok(providers)) => {
                            core.providers = Some(std::sync::Arc::new(providers));
                            None
                        }
                        Ok(Err(error)) => Some(error.message),
                        Err(error) => Some(format!("Provider startup worker failed: {error}")),
                    }
                }
                Ok(Err(error)) => Some(error.message),
                Err(error) => Some(format!("Workspace startup worker failed: {error}")),
            };
            let core = std::sync::Arc::new(core);
            *core.keepalive.lock().unwrap() = Some(core.clone());
            if let Some(error) = failure {
                core.lifetime.fence();
                core.drain(vec![error]).await;
            } else {
                core.lifetime.ready();
            }
            core
        });
        task.await
            .expect("startup coordinator must retain its core")
    }
    fn fence_native(&self) {
        self.lifetime.fence();
        if let Some(providers) = &self.providers {
            providers.fence();
        }
    }
    fn provider_request(
        &self,
        request: runtimes::ProviderRequest,
    ) -> Result<runtimes::ProviderReply, workspaces::RepositoryError> {
        let effect = matches!(
            request,
            runtimes::ProviderRequest::Observe { refresh: true, .. }
                | runtimes::ProviderRequest::SelectExecutable { .. }
                | runtimes::ProviderRequest::StartBrowserLogin { .. }
                | runtimes::ProviderRequest::ReopenBrowser { .. }
        );
        let _start = effect.then(|| self.lifetime.admit()).transpose()?;
        self.providers
            .as_ref()
            .ok_or_else(|| {
                workspaces::RepositoryError::new(
                    workspaces::ErrorCode::Connection,
                    "Provider setup is unavailable",
                )
            })?
            .request(request)
    }
    pub fn begin_quit(self: &std::sync::Arc<Self>) -> QuitStatus {
        if self.lifetime.fence() {
            if let Some(providers) = &self.providers {
                providers.fence();
            }
            let core = self.clone();
            tokio::spawn(async move {
                core.drain(Vec::new()).await;
            });
        }
        self.quit_status()
    }
    async fn drain(&self, mut errors: Vec<String>) {
        if let Some(workspaces) = &self.workspaces {
            workspaces.fence().await;
        }
        self.lifetime.transferred().await;
        use futures_util::FutureExt;
        if let Some(workspaces) = &self.workspaces {
            match std::panic::AssertUnwindSafe(workspaces.finish())
                .catch_unwind()
                .await
            {
                Ok(Ok(())) => {}
                Ok(Err(error)) => errors.push(format!("Workspaces: {error}")),
                Err(_) => errors.push("Workspace completion panicked; ownership retained".into()),
            }
        }
        if let Some(providers) = &self.providers {
            match std::panic::AssertUnwindSafe(providers.finish())
                .catch_unwind()
                .await
            {
                Ok(Ok(())) => {}
                Ok(Err(error)) => errors.push(format!("Providers: {error}")),
                Err(_) => errors.push("Provider completion panicked; ownership retained".into()),
            }
        }
        if let Some(processes) = &self.processes
            && let Err(error) = processes.finish().await
        {
            errors.push(format!("Processes: {error}"));
        }
        if let Some(storage) = &self.storage {
            let report = storage.close().await;
            if let Some(checkpoint) = &report.checkpoint {
                eprintln!(
                    "Storage close checkpoint: {}",
                    serde_json::to_string(checkpoint).expect("checkpoint serializes")
                );
            }
            errors.extend(
                report
                    .faults
                    .iter()
                    .map(|fault| format!("Storage {:?}: {}", fault.action, fault.error)),
            );
        }
        self.lifetime.settle(errors);
        self.journal.wake();
    }
    pub fn quit_status(&self) -> QuitStatus {
        let mut report = self.lifetime.report();
        if !report.complete
            && let Some(diagnostic) = self
                .processes
                .as_ref()
                .and_then(process_effects::Processes::diagnostic)
        {
            report.diagnostic = Some(match report.diagnostic {
                Some(prior) => format!("{prior}; {diagnostic}"),
                None => diagnostic,
            });
        }
        report
    }
    pub fn release_lease(&self) {
        if self.quit_status().complete {
            self.lifetime.release();
            self.keepalive
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();
        }
    }
    pub async fn repository_request(
        &self,
        request: workspaces::RepositoryRequest,
    ) -> Result<workspaces::RepositoryReply, workspaces::RepositoryError> {
        let _start = matches!(
            request,
            workspaces::RepositoryRequest::Register { .. }
                | workspaces::RepositoryRequest::List { refresh: true, .. }
                | workspaces::RepositoryRequest::Get { refresh: true, .. }
        )
        .then(|| self.lifetime.admit())
        .transpose()?;
        self.workspaces
            .as_ref()
            .ok_or_else(|| {
                workspaces::RepositoryError::storage(
                    "Repository storage has not been opened by the execution incarnation",
                )
            })?
            .request(request)
            .await
    }

    pub async fn workspace_request(
        &self,
        request: workspaces::WorkspaceRequest,
    ) -> Result<workspaces::WorkspaceReply, workspaces::RepositoryError> {
        let _start = (!matches!(
            request,
            workspaces::WorkspaceRequest::Action { .. }
                | workspaces::WorkspaceRequest::Get { .. }
                | workspaces::WorkspaceRequest::List { .. }
        ))
        .then(|| self.lifetime.admit())
        .transpose()?;
        self.workspaces
            .as_ref()
            .ok_or_else(|| {
                workspaces::RepositoryError::storage(
                    "Workspace storage has not been opened by the execution incarnation",
                )
            })?
            .workspace_request(request)
            .await
    }

    pub fn status(&self) -> CoreStatus {
        CoreStatus {
            state: self.lifetime.phase(),
            native_pid: self.pid,
            application_version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

impl Core {
    pub fn snapshot(&self, after: Option<SnapshotCursor>) -> CoreSnapshot {
        let cursor = SnapshotCursor {
            incarnation: self.incarnation.clone(),
            revision: self.workspaces.as_ref().map_or(1, |w| w.revision()),
        };
        CoreSnapshot {
            diagnostic: self.quit_status().diagnostic,
            provider_login_pending: self
                .providers
                .as_ref()
                .filter(|providers| providers.has_pending_login())
                .map(|_| true),
            status: self.status(),
            reset: after.as_ref() != Some(&cursor),
            cursor,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core() -> Core {
        Core {
            keepalive: std::sync::Mutex::new(None),
            lifetime: std::sync::Arc::default(),
            exit_requested: std::sync::atomic::AtomicBool::new(false),
            incarnation: "test-incarnation".into(),
            pid: 42,
            workspaces: None,
            storage: None,
            processes: None,
            providers: None,
            journal: std::sync::Arc::new(observations::Journal::default()),
        }
    }

    #[test]
    fn partial_startup_drains_acquired_workspace_and_retains_failed_lease() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                use std::sync::{
                    Arc,
                    atomic::{AtomicBool, Ordering},
                };
                let released = Arc::new(AtomicBool::new(false));
                let completed = Arc::new(AtomicBool::new(false));
                let marker = completed.clone();
                let lifetime = Arc::new(lifetime::Lifetime::starting(
                    supervisor::StateLease::controlled(released.clone()),
                ));
                let core = Core::start_with(
                    lifetime,
                    None,
                    move || async move {
                        Ok(workspaces::tests::controlled_completion(async move {
                            marker.store(true, Ordering::SeqCst);
                            Ok(())
                        }))
                    },
                    |_, _| async {
                        Err(workspaces::RepositoryError::storage(
                            "controlled provider startup failure",
                        ))
                    },
                )
                .await;
                assert!(completed.load(Ordering::SeqCst));
                assert_eq!(core.status().state, CoreState::Failed);
                assert!(
                    core.quit_status()
                        .diagnostic
                        .unwrap()
                        .contains("controlled provider startup failure")
                );
                core.release_lease();
                assert!(!released.load(Ordering::SeqCst));
                // Test-only teardown of the deliberately retained failed incarnation.
                core.keepalive.lock().unwrap().take();
            });
    }
    #[test]
    fn quit_deadline_and_lost_request_keep_the_same_drain_and_lease() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap()
            .block_on(async {
                use std::sync::{
                    Arc,
                    atomic::{AtomicBool, Ordering},
                };
                let released = Arc::new(AtomicBool::new(false));
                let lifetime = Arc::new(lifetime::Lifetime::starting(
                    supervisor::StateLease::controlled(released.clone()),
                ));
                lifetime.ready();
                let (finish, finished) = tokio::sync::oneshot::channel();
                let core = Arc::new(Core {
                    lifetime,
                    workspaces: Some(workspaces::tests::controlled_completion(async move {
                        finished.await.unwrap();
                        Ok(())
                    })),
                    ..Core::new()
                });
                let guard = core.lifetime.admit().unwrap();
                core.begin_quit();
                assert_eq!(core.status().state, CoreState::Stopping);
                assert!(core.lifetime.admit().is_err());
                core.release_lease();
                assert!(!released.load(Ordering::SeqCst));
                drop(guard);
                assert!(
                    tokio::time::timeout(std::time::Duration::from_secs(1), async {
                        while !core.quit_status().complete {
                            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                        }
                    })
                    .await
                    .is_err()
                );
                assert!(!core.begin_quit().complete);
                finish.send(()).unwrap();
                while !core.quit_status().complete {
                    tokio::task::yield_now().await;
                }
                assert!(!released.load(Ordering::SeqCst));
                core.release_lease();
                assert!(released.load(Ordering::SeqCst));
            });
    }
    #[test]
    fn failed_workspace_completion_does_not_skip_provider_cleanup() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let providers = std::sync::Arc::new(runtimes::ProviderConnections::test_owner());
                let core = std::sync::Arc::new(Core {
                    providers: Some(providers.clone()),
                    workspaces: Some(workspaces::tests::controlled_completion(async {
                        panic!("controlled workspace completion panic")
                    })),
                    ..Core::new()
                });
                core.begin_quit();
                while core.quit_status().diagnostic.is_none() {
                    tokio::task::yield_now().await;
                }
                assert!(providers.is_fenced());
                assert_eq!(core.status().state, CoreState::Failed);
                let status = core.begin_quit();
                assert!(!status.complete);
                assert!(status.diagnostic.unwrap().contains("Workspaces"));
            });
    }
    #[test]
    fn initial_status_reports_the_execution_owner_without_starting_work() {
        let status = core().status();
        assert_eq!(status.state, CoreState::Ready);
        assert_eq!(status.native_pid, 42);
        assert_eq!(status.application_version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn snapshot_catches_up_or_resets_observations_from_another_owner() {
        let core = core();
        let first = core.snapshot(None);
        assert!(first.reset);
        let caught_up = core.snapshot(Some(first.cursor.clone()));
        assert!(!caught_up.reset);
        assert_eq!(caught_up.cursor, first.cursor);
        for cursor in [
            SnapshotCursor {
                incarnation: "previous-incarnation".into(),
                revision: 1,
            },
            SnapshotCursor {
                incarnation: first.cursor.incarnation.clone(),
                revision: 0,
            },
            SnapshotCursor {
                incarnation: first.cursor.incarnation.clone(),
                revision: 999,
            },
        ] {
            let snapshot = core.snapshot(Some(cursor));
            assert!(snapshot.reset);
            assert_eq!(snapshot.cursor, first.cursor);
        }
    }
}

/// Explicit actual-effects entry points; never selected by application unit gates.
#[cfg(feature = "qualification")]
pub mod qualification {
    pub async fn storage(root: &std::path::Path) -> Result<serde_json::Value, String> {
        crate::storage::qualification::run(root)
            .await
            .map_err(|error| error.to_string())
    }
    pub async fn workspaces(root: &std::path::Path) -> Result<serde_json::Value, String> {
        crate::workspaces::qualification::run(root)
            .await
            .map_err(|error| error.to_string())
    }
}

/// Explicit disposable process qualification; never called by the product or units.
#[cfg(feature = "qualification")]
pub async fn qualify_process_effects(
    root: &std::path::Path,
    fixture: &std::path::Path,
) -> Result<serde_json::Value, String> {
    process_effects::qualification::run(root, fixture)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(feature = "qualification")]
#[doc(hidden)]
pub async fn qualify_process_parent_loss(
    root: &std::path::Path,
    fixture: &std::path::Path,
) -> std::io::Result<()> {
    process_effects::qualification::parent_loss_owner(root, fixture).await
}
