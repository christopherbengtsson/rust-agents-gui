//! UI-independent application interface hosted by the local supervisor.

#[cfg(feature = "codegen")]
pub mod codegen;
pub mod connection;
mod git;
mod lifetime;
pub mod observations;
mod private_state;
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
pub enum SupervisorState {
    Starting,
    Recovering,
    Ready,
    Failed,
    Stopping,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct SupervisorStatus {
    pub state: SupervisorState,
    pub supervisor_pid: u32,
    pub application_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct SnapshotCursor {
    pub owner: String,
    pub revision: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct SupervisorSnapshot {
    pub status: SupervisorStatus,
    pub cursor: SnapshotCursor,
    /// A full snapshot replaces observations from another owner or cursor.
    pub reset: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "codegen", ts(optional))]
    pub provider_login_pending: Option<bool>,
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
    owner: String,
    pid: u32,
    providers: Option<std::sync::Arc<runtimes::ProviderConnections>>,
    workspaces: Option<workspaces::Workspaces>,
    storage: Option<storage::Storage>,
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
            owner: format!(
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
            journal: std::sync::Arc::new(observations::Journal::default()),
        }
    }

    /// Startup and any partial cleanup are retained independently of the requesting host.
    pub async fn start(
        directory: std::path::PathBuf,
        lease: supervisor::StateLease,
    ) -> std::sync::Arc<Self> {
        let lifetime = std::sync::Arc::new(lifetime::Lifetime::starting(lease));
        let workspace_directory = directory.clone();
        Self::start_with(
            lifetime,
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
                Ok(workspaces::Workspaces::open(directory, storage))
            },
            move |journal, storage| async move {
                runtimes::ProviderConnections::open(
                    journal,
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
        self.lifetime.report()
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
                    "Repository storage has not been opened by the execution owner",
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
                    "Workspace storage has not been opened by the execution owner",
                )
            })?
            .workspace_request(request)
            .await
    }

    pub fn status(&self) -> SupervisorStatus {
        SupervisorStatus {
            state: self.lifetime.phase(),
            supervisor_pid: self.pid,
            application_version: env!("CARGO_PKG_VERSION").into(),
        }
    }
}

impl Core {
    pub fn snapshot(&self, after: Option<SnapshotCursor>) -> SupervisorSnapshot {
        let cursor = SnapshotCursor {
            owner: self.owner.clone(),
            revision: self.workspaces.as_ref().map_or(1, |w| w.revision()),
        };
        SupervisorSnapshot {
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
            owner: "test-owner".into(),
            pid: 42,
            workspaces: None,
            storage: None,
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
                assert_eq!(core.status().state, SupervisorState::Failed);
                assert!(
                    core.quit_status()
                        .diagnostic
                        .unwrap()
                        .contains("controlled provider startup failure")
                );
                core.release_lease();
                assert!(!released.load(Ordering::SeqCst));
                // Test-only teardown of the deliberately retained failed owner.
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
                assert_eq!(core.status().state, SupervisorState::Stopping);
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
                assert_eq!(core.status().state, SupervisorState::Failed);
                let status = core.begin_quit();
                assert!(!status.complete);
                assert!(status.diagnostic.unwrap().contains("Workspaces"));
            });
    }
    #[test]
    fn initial_status_reports_the_execution_owner_without_starting_work() {
        let status = core().status();
        assert_eq!(status.state, SupervisorState::Ready);
        assert_eq!(status.supervisor_pid, 42);
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
                owner: "previous-owner".into(),
                revision: 1,
            },
            SnapshotCursor {
                owner: first.cursor.owner.clone(),
                revision: 0,
            },
            SnapshotCursor {
                owner: first.cursor.owner.clone(),
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
