//! Root admission and one retained shutdown attempt; domain outcomes stay in domains.
use crate::SupervisorState;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

pub(crate) struct Lifetime {
    state: Mutex<State>,
    changed: Notify,
    settled: Notify,
}
struct State {
    phase: SupervisorState,
    starts: usize,
    attempted: bool,
    errors: Vec<String>,
    lease: Option<crate::supervisor::StateLease>,
}
pub(crate) struct StartGuard(Arc<Lifetime>);
impl Drop for StartGuard {
    fn drop(&mut self) {
        self.0
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .starts -= 1;
        self.0.changed.notify_one();
    }
}
impl Default for Lifetime {
    fn default() -> Self {
        Self {
            state: Mutex::new(State {
                phase: SupervisorState::Ready,
                starts: 0,
                attempted: false,
                errors: Vec::new(),
                lease: None,
            }),
            changed: Notify::new(),
            settled: Notify::new(),
        }
    }
}
impl Lifetime {
    pub(crate) fn starting(lease: crate::supervisor::StateLease) -> Self {
        let owner = Self::default();
        {
            let mut state = owner.state.lock().unwrap();
            state.phase = SupervisorState::Starting;
            state.lease = Some(lease);
        }
        owner
    }
    pub(crate) fn phase(&self) -> SupervisorState {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .phase
            .clone()
    }
    pub(crate) fn recovering(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).phase = SupervisorState::Recovering;
    }
    pub(crate) fn ready(&self) {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).phase = SupervisorState::Ready;
    }
    pub(crate) fn admit(
        self: &Arc<Self>,
    ) -> Result<StartGuard, crate::workspaces::RepositoryError> {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.phase != SupervisorState::Ready {
            return Err(crate::workspaces::RepositoryError::new(
                crate::workspaces::ErrorCode::Connection,
                "Core is fenced; retrieve status before starting work",
            ));
        }
        state.starts += 1;
        Ok(StartGuard(self.clone()))
    }
    pub(crate) fn fence(&self) -> bool {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.attempted {
            return false;
        }
        state.attempted = true;
        state.phase = SupervisorState::Stopping;
        true
    }
    pub(crate) async fn transferred(&self) {
        loop {
            let changed = self.changed.notified();
            if self.state.lock().unwrap_or_else(|e| e.into_inner()).starts == 0 {
                return;
            }
            changed.await;
        }
    }
    pub(crate) fn settle(&self, errors: Vec<String>) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.phase = if errors.is_empty() {
            SupervisorState::Stopped
        } else {
            SupervisorState::Failed
        };
        state.errors = errors;
        self.settled.notify_waiters();
    }
    pub(crate) async fn finished(&self) -> crate::QuitStatus {
        loop {
            let notified = self.settled.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let status = self.report();
            if status.complete || status.diagnostic.is_some() {
                return status;
            }
            notified.await;
        }
    }
    pub(crate) fn report(&self) -> crate::QuitStatus {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        crate::QuitStatus {
            complete: state.phase == SupervisorState::Stopped,
            diagnostic: (!state.errors.is_empty()).then(|| state.errors.join("; ")),
        }
    }
    pub(crate) fn cleanup_failed(&self, error: String) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.phase = SupervisorState::Failed;
        state.errors.push(error);
        self.settled.notify_waiters();
    }
    pub(crate) fn release(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        if state.phase == SupervisorState::Stopped {
            state.lease.take();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::Future;
    #[test]
    fn lease_is_last_and_failed_cleanup_retains_it() {
        use std::sync::atomic::{AtomicBool, Ordering};
        for fail in [false, true] {
            let released = Arc::new(AtomicBool::new(false));
            let owner =
                Lifetime::starting(crate::supervisor::StateLease::controlled(released.clone()));
            owner.ready();
            owner.fence();
            owner.release();
            assert!(!released.load(Ordering::SeqCst));
            owner.settle(if fail {
                vec!["final persistence failed".into()]
            } else {
                vec![]
            });
            owner.release();
            assert_eq!(released.load(Ordering::SeqCst), !fail);
        }
    }
    #[test]
    fn quit_fences_new_starts_but_waits_for_admitted_transfer() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let owner = Arc::new(Lifetime::default());
                let guard = owner.admit().unwrap();
                assert!(owner.fence());
                assert!(!owner.fence());
                assert!(owner.admit().is_err());
                let wait = owner.transferred();
                tokio::pin!(wait);
                std::future::poll_fn(|cx| {
                    assert!(wait.as_mut().poll(cx).is_pending());
                    std::task::Poll::Ready(())
                })
                .await;
                drop(guard);
                wait.await;
                owner.settle(vec!["workspace panic".into(), "provider cleanup".into()]);
                assert!(!owner.report().complete);
                assert_eq!(
                    owner.report().diagnostic.unwrap(),
                    "workspace panic; provider cleanup"
                );
                assert!(!owner.fence());
            });
    }
}
