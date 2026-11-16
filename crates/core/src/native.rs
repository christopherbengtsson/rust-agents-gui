//! One native startup/request/delivery/shutdown incarnation. No discovery or action replay.
use crate::connection::{ConnectionSlots, Delivery, OwnedReply, RequestFailure, TransportCode};
use crate::workspaces::{
    RepositoryError, RepositoryReply, RepositoryRequest, WorkspaceReply, WorkspaceRequest,
};
use crate::{Core, CoreSnapshot, CoreState, CoreStatus, QuitStatus, SnapshotCursor};
use futures_util::FutureExt;
use std::{
    future::Future,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::sync::Notify;

pub struct NativeHost {
    incarnation: String,
    runtime: tokio::runtime::Handle,
    state: Mutex<State>,
    changed: Notify,
    startup: Mutex<Option<tokio::task::JoinHandle<()>>>,
    shutdown: Mutex<Option<tokio::task::JoinHandle<()>>>,
    shutdown_join: tokio::sync::Mutex<()>,
    tasks: Arc<crate::connection::tasks::Tasks>,
    ordinary: ConnectionSlots,
    control: ConnectionSlots,
    pub(crate) feed: crate::connection::subscription::FeedOwner,
}
/// The native result retains its admission until Tauri serializes/drops it.
/// The generated wire contract remains OwnedReply; this is an internal resource guard.
#[derive(serde::Serialize)]
pub struct NativeReply<T> {
    #[serde(flatten)]
    reply: OwnedReply<T>,
    #[serde(skip)]
    _permit: tokio::sync::OwnedSemaphorePermit,
}
type DomainResult<T> = std::pin::Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send>>;
struct State {
    initialized: bool,
    startup_lifetime: Option<Arc<crate::lifetime::Lifetime>>,
    core: Option<Arc<Core>>,
    quitting: bool,
    finished: bool,
    error: Option<String>,
    cleanup_error: Option<String>,
}
fn failure(code: TransportCode, message: impl Into<String>) -> RequestFailure {
    RequestFailure::transport(
        code,
        if code == TransportCode::IncarnationChanged {
            Delivery::Rejected
        } else {
            Delivery::NotSent
        },
        message,
    )
}
fn within_budget(value: &impl serde::Serialize) -> bool {
    struct Budget(usize);
    impl std::io::Write for Budget {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_sub(bytes.len())
                .ok_or_else(|| std::io::Error::other("Core wire budget exceeded"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Budget(64 * 1024), value).is_ok()
}
impl NativeHost {
    /// Must be called inside the host's driven runtime; it never constructs a runtime.
    pub fn start(directory: PathBuf) -> Arc<Self> {
        Self::start_with(move |host| async move {
            let lease = tokio::task::spawn_blocking({
                let directory = directory.clone();
                move || crate::supervisor::StateLease::acquire(&directory)
            })
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
            let lifetime = Arc::new(crate::lifetime::Lifetime::starting(lease));
            host.state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .startup_lifetime = Some(lifetime.clone());
            Ok(Core::start_native(directory, lifetime).await)
        })
    }
    fn start_with<F: Future<Output = Result<Arc<Core>, String>> + Send + 'static>(
        open: impl FnOnce(Arc<Self>) -> F + Send + 'static,
    ) -> Arc<Self> {
        let host = Arc::new(Self {
            incarnation: uuid::Uuid::new_v4().to_string(),
            runtime: tokio::runtime::Handle::current(),
            state: Mutex::new(State {
                initialized: false,
                startup_lifetime: None,
                core: None,
                quitting: false,
                finished: false,
                error: None,
                cleanup_error: None,
            }),
            changed: Notify::new(),
            startup: Mutex::new(None),
            shutdown: Mutex::new(None),
            shutdown_join: tokio::sync::Mutex::new(()),
            tasks: Arc::default(),
            ordinary: ConnectionSlots::new(64),
            control: ConnectionSlots::new(8),
            feed: Default::default(),
        });
        let owned = host.clone();
        *host.startup.lock().unwrap() = Some(tokio::spawn(async move {
            let result = std::panic::AssertUnwindSafe(open(owned.clone()))
                .catch_unwind()
                .await;
            let mut state = owned.state.lock().unwrap_or_else(|e| e.into_inner());
            match result {
                Ok(Ok(core)) => state.core = Some(core),
                Ok(Err(error)) => state.error = Some(error),
                Err(_) => {
                    state.error =
                        Some("Native startup panicked; ownership remains unconfirmed".into())
                }
            }
            tracing::info!(incarnation = %owned.incarnation, phase = ?state.core.as_ref().map(|core| core.status().state), "native startup settled");
            state.initialized = true;
            drop(state);
            owned.changed.notify_waiters();
        }));
        host
    }
    async fn initialized(&self) {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self
                .state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .initialized
            {
                return;
            }
            changed.await;
        }
    }
    pub fn snapshot(&self) -> CoreSnapshot {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        let mut snapshot = state
            .core
            .as_ref()
            .map(|core| core.snapshot(None))
            .unwrap_or_else(|| CoreSnapshot {
                status: CoreStatus {
                    state: if state.initialized {
                        CoreState::Failed
                    } else {
                        state
                            .startup_lifetime
                            .as_ref()
                            .map_or(CoreState::Starting, |l| l.phase())
                    },
                    native_pid: std::process::id(),
                    application_version: env!("CARGO_PKG_VERSION").into(),
                },
                cursor: SnapshotCursor {
                    incarnation: self.incarnation.clone(),
                    revision: 1,
                },
                reset: true,
                provider_login_pending: None,
                diagnostic: None,
            });
        snapshot.cursor.incarnation = self.incarnation.clone();
        snapshot.diagnostic = state
            .error
            .clone()
            .or_else(|| self.tasks.diagnostic())
            .or_else(|| {
                state
                    .core
                    .as_ref()
                    .and_then(|core| core.quit_status().diagnostic)
            });
        if snapshot.diagnostic.is_some() {
            snapshot.status.state = CoreState::Failed;
        }
        if state.quitting {
            snapshot.status.state = if state.finished {
                if snapshot.diagnostic.is_some() {
                    CoreState::Failed
                } else {
                    CoreState::Stopped
                }
            } else {
                CoreState::Stopping
            };
        }
        snapshot.diagnostic = snapshot.diagnostic.map(|message| {
            RepositoryError::new(crate::workspaces::ErrorCode::Connection, message).message
        });
        snapshot
    }
    pub(crate) fn core(&self, expected: &str) -> Result<Arc<Core>, RequestFailure> {
        if expected != self.incarnation {
            return Err(failure(
                TransportCode::IncarnationChanged,
                "Native core incarnation changed; retrieve status without replaying actions",
            ));
        }
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(error) = self.tasks.diagnostic() {
            return Err(failure(
                TransportCode::Busy,
                format!("Native request failure fences admission: {error}"),
            ));
        }
        if state.quitting {
            return Err(failure(
                TransportCode::Busy,
                "Native core is stopping; no replacement work is admitted",
            ));
        }
        state.core.clone().ok_or_else(|| {
            failure(
                TransportCode::NotConnected,
                state
                    .error
                    .clone()
                    .unwrap_or_else(|| "Native core is starting; check status".into()),
            )
        })
    }
    async fn dispatch<Q, T: serde::Serialize + Send + 'static>(
        &self,
        expected: String,
        request: Q,
        control: bool,
        work: impl FnOnce(Arc<Core>, Q) -> DomainResult<T> + Send + 'static,
    ) -> Result<NativeReply<T>, RequestFailure>
    where
        Q: serde::Serialize + Send + 'static,
    {
        if !within_budget(&request) {
            return Err(failure(
                TransportCode::Busy,
                "Core input exceeds 64 KiB; request was not admitted",
            ));
        }
        let core = self.core(&expected)?;
        let permit = if control {
            &self.control
        } else {
            &self.ordinary
        }
        .try_acquire()
        .ok_or_else(|| {
            failure(
                TransportCode::Busy,
                "Native request capacity is busy; retrieve status",
            )
        })?;
        tracing::info!(incarnation = %expected, control, "native request admitted");
        let receive = self.tasks.spawn(async move {
            let result = work(core,request).await;
            tracing::info!(incarnation = %expected, succeeded = result.is_ok(), "native request settled");
            let reply = OwnedReply { incarnation: expected, result };
            if !within_budget(&reply) {
                Err(RequestFailure::transport(TransportCode::ReplyUnavailable,Delivery::Unknown,"Core result exceeded the response budget; retrieve recorded status"))
            } else { Ok(NativeReply { reply, _permit: permit }) }
        }).map_err(|e| failure(TransportCode::Busy,e.to_string()))?;
        receive.await.map_err(|e| {
            RequestFailure::transport(
                TransportCode::ReplyUnavailable,
                Delivery::Unknown,
                e.to_string(),
            )
        })?
    }
    pub async fn repository_request(
        &self,
        expected: String,
        request: RepositoryRequest,
    ) -> Result<NativeReply<RepositoryReply>, RequestFailure> {
        let action = match &request {
            RepositoryRequest::Register { request } => Some(request.action_id.0),
            RepositoryRequest::Action { action_id } => Some(action_id.0),
            _ => None,
        };
        tracing::info!(incarnation = %expected, ?action, "repository request");
        let control = matches!(request, RepositoryRequest::Action { .. });
        self.dispatch(expected, request, control, |core, request| {
            Box::pin(async move { core.repository_request(request).await })
        })
        .await
    }
    pub async fn workspace_request(
        &self,
        expected: String,
        request: WorkspaceRequest,
    ) -> Result<NativeReply<WorkspaceReply>, RequestFailure> {
        let action = match &request {
            WorkspaceRequest::Create { action_id, .. }
            | WorkspaceRequest::Reconcile { action_id, .. }
            | WorkspaceRequest::Action { action_id } => Some(action_id.0),
            _ => None,
        };
        tracing::info!(incarnation = %expected, ?action, "workspace request");
        let control = matches!(request, WorkspaceRequest::Action { .. });
        self.dispatch(expected, request, control, |core, request| {
            Box::pin(async move { core.workspace_request(request).await })
        })
        .await
    }
    pub async fn provider_request(
        &self,
        expected: String,
        request: crate::runtimes::ProviderRequest,
    ) -> Result<NativeReply<crate::runtimes::ProviderReply>, RequestFailure> {
        use crate::runtimes::ProviderRequest;
        let action = match &request {
            ProviderRequest::StartBrowserLogin { attempt_id, .. }
            | ProviderRequest::CancelLogin { attempt_id, .. }
            | ProviderRequest::ReopenBrowser { attempt_id, .. }
            | ProviderRequest::ReconcileLogin { attempt_id, .. } => Some(*attempt_id),
            ProviderRequest::SelectExecutable { action_id, .. }
            | ProviderRequest::ReconcileSelection { action_id, .. } => Some(*action_id),
            _ => None,
        };
        tracing::info!(incarnation = %expected, ?action, "provider request");
        let control = matches!(
            request,
            crate::runtimes::ProviderRequest::CancelLogin { .. }
                | crate::runtimes::ProviderRequest::ReconcileLogin { .. }
                | crate::runtimes::ProviderRequest::ReconcileSelection { .. }
        );
        self.dispatch(expected, request, control, |core, request| {
            Box::pin(async move { core.provider_request(request) })
        })
        .await
    }
    /// Called synchronously by ExitRequested. The continuing task owns the same drain.
    pub fn begin_quit(self: &Arc<Self>) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.quitting {
            return;
        }
        state.quitting = true;
        tracing::info!(incarnation = %self.incarnation, "native shutdown fenced");
        if let Some(core) = &state.core {
            core.fence_native();
        }
        drop(state);
        let host = self.clone();
        *self.shutdown.lock().unwrap_or_else(|e| e.into_inner()) = Some(self.runtime.spawn(async move {
            host.initialized().await;
            let startup = host
                .startup
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();
            let mut errors = Vec::new();
            {
                let state = host.state.lock().unwrap_or_else(|e| e.into_inner());
                if state.core.is_none() && state.startup_lifetime.is_some() {
                    errors.push("Startup resources remain unconfirmed; lease retained".into());
                }
            }
            if let Some(task) = startup
                && let Err(error) = task.await
            {
                errors.push(error.to_string());
            }
            let core = host
                .state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .core
                .clone();
            if let Some(core) = &core {
                core.fence_native();
            }
            if let Err(error) = host.tasks.close_and_wait().await {
                errors.push(format!("Native request cleanup: {error}"));
            }
            if let Some(core) = &core {
                if core.lifetime.phase() == CoreState::Stopping
                    && std::panic::AssertUnwindSafe(core.drain(errors.clone()))
                        .catch_unwind()
                        .await
                        .is_err()
                {
                    core.lifetime
                        .cleanup_failed("Native domain drain panicked; lease retained".into());
                }
                if let Some(error) = core.quit_status().diagnostic {
                    errors.push(error);
                }
            }
            if let Err(error) = host.stop_delivery().await {
                errors.push(format!("Native delivery cleanup: {error:?}"));
            }
            if errors.is_empty() && core.as_ref().is_some_and(|core| !core.quit_status().complete) {
                errors.push("Native core cleanup remains unconfirmed; lease retained".into());
            }
            let mut state = host.state.lock().unwrap_or_else(|e| e.into_inner());
            if !errors.is_empty() {
                state.cleanup_error = Some(errors.join("; "));
                state.error = state.cleanup_error.clone();
            }
            tracing::info!(incarnation = %host.incarnation, failures = errors.len(), "native shutdown settled");
            state.finished = true;
            drop(state);
            host.changed.notify_waiters();
        }));
    }
    /// Final native-adapter barrier: call only after its resources have joined.
    /// A failed domain/request/delivery drain cannot release the state lease.
    pub fn release_lease(&self) {
        if !self.quit_status().complete {
            return;
        }
        if let Some(core) = &self.state.lock().unwrap_or_else(|e| e.into_inner()).core {
            core.release_lease();
        }
    }
    pub fn quit_status(&self) -> QuitStatus {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        QuitStatus {
            complete: state.finished && state.cleanup_error.is_none(),
            diagnostic: state.cleanup_error.clone().or_else(|| state.error.clone()),
        }
    }
    pub async fn finished(&self) {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self
                .state
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .finished
            {
                break;
            }
            changed.await;
        }
        let _join = self.shutdown_join.lock().await;
        std::future::poll_fn(|cx| {
            let mut owned = self.shutdown.lock().unwrap_or_else(|e| e.into_inner());
            let Some(worker) = owned.as_mut() else {
                return std::task::Poll::Ready(());
            };
            match std::pin::Pin::new(worker).poll(cx) {
                std::task::Poll::Pending => std::task::Poll::Pending,
                std::task::Poll::Ready(result) => {
                    owned.take();
                    if let Err(error) = result {
                        self.state
                            .lock()
                            .unwrap_or_else(|e| e.into_inner())
                            .cleanup_error =
                            Some(format!("Native shutdown worker failed: {error}"));
                    }
                    std::task::Poll::Ready(())
                }
            }
        })
        .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn quit_during_startup_retains_one_drain_and_never_publishes_ready() {
        let (release, pending) = tokio::sync::oneshot::channel();
        let host = NativeHost::start_with(|_| async move {
            pending.await.unwrap();
            Ok(std::sync::Arc::new(crate::Core::new()))
        });
        assert_eq!(host.snapshot().status.state, crate::CoreState::Starting);
        host.begin_quit();
        host.begin_quit();
        assert_eq!(host.snapshot().status.state, crate::CoreState::Stopping);
        assert!(!host.quit_status().complete);
        release.send(()).unwrap();
        host.finished().await;
        assert!(host.quit_status().complete);
        assert_eq!(host.snapshot().status.state, crate::CoreState::Stopped);
    }
    #[tokio::test]
    async fn stale_incarnation_rejects_before_domain_dispatch() {
        let host =
            NativeHost::start_with(|_| async { Ok(std::sync::Arc::new(crate::Core::new())) });
        host.initialized().await;
        let result = host
            .repository_request(
                "retired".into(),
                crate::workspaces::RepositoryRequest::List {
                    refresh: false,
                    after: None,
                },
            )
            .await;
        assert!(
            matches!(result, Err(crate::connection::RequestFailure::Transport { error }) if error.code == crate::connection::TransportCode::IncarnationChanged)
        );
        host.begin_quit();
        host.finished().await;
    }
    #[tokio::test]
    async fn abandoned_request_keeps_capacity_and_control_remains_available() {
        let host = NativeHost::start_with(|_| async { Ok(Arc::new(Core::new())) });
        host.initialized().await;
        let incarnation = host.incarnation.clone();
        let (send, receive) = tokio::sync::oneshot::channel();
        let worker_host = host.clone();
        let waiter = tokio::spawn(async move {
            worker_host
                .dispatch(incarnation, (), false, |_, ()| {
                    Box::pin(async move {
                        receive.await.unwrap();
                        Ok(())
                    })
                })
                .await
        });
        tokio::task::yield_now().await;
        waiter.abort();
        let permits: Vec<_> = (0..63)
            .map(|_| host.ordinary.try_acquire().unwrap())
            .collect();
        assert!(host.ordinary.try_acquire().is_none());
        assert!(host.control.try_acquire().is_some());
        host.begin_quit();
        assert!(!host.quit_status().complete);
        send.send(()).unwrap();
        host.finished().await;
        assert!(host.quit_status().complete);
        drop(permits);
    }
    #[tokio::test]
    async fn direct_feed_requires_exact_ack_and_replacement_joins_failed_delivery() {
        let host = NativeHost::start_with(|_| async { Ok(Arc::new(Core::new())) });
        host.initialized().await;
        let (send, mut receive) = tokio::sync::mpsc::unbounded_channel();
        host.replace_subscription(
            host.incarnation.clone(),
            "first".into(),
            None,
            move |event| {
                send.send(event).unwrap();
                Ok(())
            },
        )
        .await
        .unwrap();
        let crate::connection::FeedEvent::Batch { batch } = receive.recv().await.unwrap() else {
            panic!("first batch");
        };
        let core = host.core(&host.incarnation).unwrap();
        core.journal
            .publish(vec![crate::observations::Target::Repositories]);
        core.journal.wake();
        tokio::task::yield_now().await;
        assert!(receive.try_recv().is_err());
        let mut wrong = batch.acknowledgement.clone();
        wrong.cursor.incarnation = "retired".into();
        assert!(host.acknowledge_subscription(wrong).is_err());
        host.acknowledge_subscription(batch.acknowledgement)
            .unwrap();
        assert!(matches!(
            receive.recv().await,
            Some(crate::connection::FeedEvent::Batch { .. })
        ));
        host.replace_subscription(host.incarnation.clone(), "panic".into(), None, |_| {
            panic!("controlled delivery panic")
        })
        .await
        .unwrap();
        tokio::task::yield_now().await;
        assert!(host.stop_subscription("panic").await.is_err());
        assert!(
            host.replace_subscription(
                host.incarnation.clone(),
                "replacement".into(),
                None,
                |_| Ok(())
            )
            .await
            .is_err()
        );
        host.begin_quit();
        host.finished().await;
        assert!(!host.quit_status().complete);
    }
    #[tokio::test]
    async fn owner_exclusion_before_lease_can_quit_without_hiding_startup_error() {
        let host =
            NativeHost::start_with(|_| async { Err("Another native core owns this state".into()) });
        host.initialized().await;
        assert_eq!(host.snapshot().status.state, CoreState::Failed);
        host.begin_quit();
        host.finished().await;
        assert!(host.quit_status().complete);
        assert!(
            host.snapshot()
                .diagnostic
                .unwrap()
                .contains("Another native core")
        );
    }
    #[tokio::test]
    async fn serialized_result_keeps_its_request_admission_until_dropped() {
        let host = NativeHost::start_with(|_| async { Ok(Arc::new(Core::new())) });
        host.initialized().await;
        let reply = host
            .dispatch(host.incarnation.clone(), (), false, |_, ()| {
                Box::pin(async { Ok(()) })
            })
            .await
            .unwrap();
        let permits: Vec<_> = (0..63)
            .map(|_| host.ordinary.try_acquire().unwrap())
            .collect();
        assert!(host.ordinary.try_acquire().is_none());
        let value = serde_json::to_value(&reply).unwrap();
        assert_eq!(value["incarnation"], host.incarnation);
        assert!(value.get("_permit").is_none());
        assert!(host.ordinary.try_acquire().is_none());
        drop(reply);
        assert!(host.ordinary.try_acquire().is_some());
        drop(permits);
        host.begin_quit();
        host.finished().await;
    }
    #[test]
    fn synchronous_quit_callback_uses_the_retained_host_runtime() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let host = runtime.block_on(async {
            let host = NativeHost::start_with(|_| async { Ok(Arc::new(Core::new())) });
            host.initialized().await;
            host
        });
        host.begin_quit();
        runtime.block_on(host.finished());
        assert!(host.quit_status().complete);
    }
    #[tokio::test]
    async fn lease_stays_owned_until_native_host_resources_finish() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let released = Arc::new(AtomicBool::new(false));
        let lifetime = Arc::new(crate::lifetime::Lifetime::starting(
            crate::supervisor::StateLease::controlled(released.clone()),
        ));
        lifetime.ready();
        let host = NativeHost::start_with(move |_| async move {
            Ok(Arc::new(Core {
                lifetime,
                ..Core::new()
            }))
        });
        host.initialized().await;
        host.begin_quit();
        host.finished().await;
        assert!(host.quit_status().complete);
        assert!(!released.load(Ordering::SeqCst));
    }
}
