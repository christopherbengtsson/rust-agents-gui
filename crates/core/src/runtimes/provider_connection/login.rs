//! Login policy belongs to ProviderConnections; this file is its private lifecycle.
use super::*;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LoginPhase {
    Starting,
    Pending,
    Cancelling,
    Verifying,
    Completed,
    Failed,
    Cancelled,
    CleanupUncertain,
}
impl LoginPhase {
    pub(super) fn active(self) -> bool {
        !matches!(self, Self::Completed | Self::Failed | Self::Cancelled)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum LoginOutcome {
    Completed,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct LoginAttempt {
    pub id: uuid::Uuid,
    pub phase: LoginPhase,
    pub outcome: Option<LoginOutcome>,
    pub browser_open_failed: bool,
    pub diagnostic: Option<String>,
}
impl LoginAttempt {
    pub(super) fn interrupted(id: uuid::Uuid) -> Self {
        Self { id, phase: LoginPhase::CleanupUncertain, outcome: None, browser_open_failed: false,
            diagnostic: Some("The previous supervisor lost ownership before recording physical cleanup. This credential scope remains fenced; restarting or retrying cannot establish that its resources stopped.".into()) }
    }
}
pub(super) struct LoginControl {
    cancel: CancellationToken,
    reopen: mpsc::Sender<()>,
    receiver: Option<mpsc::Receiver<()>>,
}

type LoginNotice = Arc<dyn Fn(codex::LoginEvent) + Send + Sync>;
// Internal effect seam: admission, persistence, verification and cancellation
// decisions remain in ProviderConnections under both production and unit outcomes.
trait BrowserEffects: Send + Sync {
    fn authenticate(
        &self,
        cancel: CancellationToken,
        reopen: mpsc::Receiver<()>,
        notice: LoginNotice,
    ) -> impl Future<Output = Result<LoginOutcome, SetupFailure>> + Send;
    fn verify(
        &self,
        cancel: CancellationToken,
        notice: LoginNotice,
    ) -> impl Future<Output = Result<SetupEvidence, SetupFailure>> + Send;
}
struct CodexBrowser(codex::Launch);
impl BrowserEffects for CodexBrowser {
    async fn authenticate(
        &self,
        cancel: CancellationToken,
        reopen: mpsc::Receiver<()>,
        notice: LoginNotice,
    ) -> Result<LoginOutcome, SetupFailure> {
        codex::browser_login(self.0.clone(), &cancel, reopen, |event| notice(event)).await
    }
    async fn verify(
        &self,
        cancel: CancellationToken,
        notice: LoginNotice,
    ) -> Result<SetupEvidence, SetupFailure> {
        codex::probe(self.0.clone(), &cancel, || {
            notice(codex::LoginEvent::CleanupUncertain)
        })
        .await
    }
}

impl ProviderConnections {
    pub(crate) fn has_pending_login(&self) -> bool {
        self.records
            .lock()
            .expect("provider records poisoned")
            .values()
            .any(|entry| {
                entry
                    .snapshot
                    .login
                    .as_ref()
                    .is_some_and(|login| login.phase.active())
            })
    }
    pub(super) fn reconcile_login(
        &self,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
    ) -> Result<ProviderReply, RepositoryError> {
        let mut records = self.records.lock().expect("provider records poisoned");
        let entry = records
            .get_mut(&target)
            .ok_or_else(|| invalid_target("Unknown provider configuration"))?;
        // If admission never happened, changing the revision fences a delayed
        // start carrying the old revision. Absence is now authoritative.
        if entry
            .snapshot
            .login
            .as_ref()
            .is_none_or(|login| login.id != attempt)
            && !matches!(
                entry.snapshot.phase,
                SetupPhase::Checking | SetupPhase::CleanupUncertain
            )
        {
            entry.snapshot.revision = entry
                .snapshot
                .revision
                .checked_add(1)
                .ok_or_else(|| invalid_target("Provider revision exhausted"))?;
        }
        let observation = entry.snapshot.clone();
        drop(records);
        self.changed(target);
        Ok(ProviderReply::Observation { observation })
    }
    pub(super) fn admit_login(
        &self,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
        revision: u32,
    ) -> Result<ProviderObservation, RepositoryError> {
        if self.cancel.is_cancelled() {
            return Err(invalid_target("Provider connections are shutting down"));
        }
        let mut records = self.records.lock().expect("provider records poisoned");
        let entry = records
            .get(&target)
            .ok_or_else(|| invalid_target("Unknown provider configuration"))?;
        if entry
            .snapshot
            .login
            .as_ref()
            .is_some_and(|login| login.id == attempt)
        {
            return Ok(entry.snapshot.clone());
        }
        if entry.snapshot.revision != revision || revision > u32::MAX - 32 {
            return Err(invalid_target(
                "Setup changed. Retrieve its recorded status before signing in.",
            ));
        }
        if records.values().any(|other| {
            other.scope == entry.scope
                && (matches!(
                    other.snapshot.phase,
                    SetupPhase::Checking | SetupPhase::CleanupUncertain
                ) || other
                    .snapshot
                    .login
                    .as_ref()
                    .is_some_and(|login| login.phase.active()))
        }) {
            return Err(invalid_target(
                "This credential scope already has an active check, sign-in or unconfirmed cleanup. Retrieve its recorded status.",
            ));
        }
        if records
            .values()
            .filter(|entry| {
                entry.snapshot.phase == SetupPhase::Checking
                    || entry
                        .snapshot
                        .login
                        .as_ref()
                        .is_some_and(|login| login.phase.active())
            })
            .count()
            >= 8
        {
            return Err(RepositoryError::new(
                ErrorCode::Capacity,
                "Provider process capacity is occupied. Retrieve status before trying again.",
            ));
        }
        let entry = records.get_mut(&target).expect("validated target retained");
        let (reopen, receiver) = mpsc::channel(1);
        entry.control = Some(LoginControl {
            cancel: self.cancel.child_token(),
            reopen,
            receiver: Some(receiver),
        });
        entry.snapshot.login = Some(Box::new(LoginAttempt {
            id: attempt,
            phase: LoginPhase::Starting,
            outcome: None,
            browser_open_failed: false,
            diagnostic: None,
        }));
        entry.snapshot.revision += 1;
        let result = entry.snapshot.clone();
        drop(records);
        self.changed(target);
        Ok(result)
    }
    pub(super) fn changed(&self, target_id: uuid::Uuid) {
        self.journal.publish(vec![Target::Provider { target_id }]);
        self.journal.wake();
    }
    pub(super) fn update_login(
        &self,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
        update: impl FnOnce(&mut LoginAttempt),
    ) {
        let mut records = self.records.lock().expect("provider records poisoned");
        let Some(entry) = records.get_mut(&target) else {
            return;
        };
        let Some(login) = entry
            .snapshot
            .login
            .as_mut()
            .filter(|login| login.id == attempt)
        else {
            return;
        };
        update(login);
        entry.snapshot.revision = entry.snapshot.revision.saturating_add(1);
        drop(records);
        self.changed(target);
    }
    pub(super) fn start_browser(
        self: &Arc<Self>,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
        revision: u32,
    ) -> Result<ProviderReply, RepositoryError> {
        self.ensure_reaper();
        let mut tasks = self.tasks.lock().expect("provider tasks poisoned");
        if !self.failure.lock().unwrap().is_empty() {
            return Err(invalid_target("Provider worker cleanup is fenced"));
        }
        let launch = self
            .records
            .lock()
            .expect("provider records poisoned")
            .get(&target)
            .and_then(|entry| entry.launch.clone())
            .ok_or_else(|| invalid_target("Check and validate Codex before signing in"))?;
        let observation = self.admit_login(target, attempt, revision)?;
        let control = {
            let mut records = self.records.lock().expect("provider records poisoned");
            let entry = records.get_mut(&target).expect("admitted target retained");
            entry.control.as_mut().and_then(|control| {
                control
                    .receiver
                    .take()
                    .map(|receiver| (control.cancel.clone(), receiver))
            })
        };
        if let Some((cancel, reopen)) = control {
            let owner = self.clone();
            tasks.spawn(async move {
                use futures_util::FutureExt;
                if std::panic::AssertUnwindSafe(owner.execute_browser(
                    target,
                    attempt,
                    cancel,
                    reopen,
                    CodexBrowser(launch),
                ))
                .catch_unwind()
                .await
                .is_err()
                {
                    owner.worker_panicked(target, Some(attempt), None);
                }
            });
        }

        self.task_changed.notify_one();
        Ok(ProviderReply::Observation { observation })
    }
    async fn execute_browser(
        self: &Arc<Self>,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
        cancel: CancellationToken,
        reopen: mpsc::Receiver<()>,
        effects: impl BrowserEffects,
    ) {
        if let Err(failure) = self.persist_attempt(attempt, true).await {
            self.finish_login(target, attempt, LoginOutcome::Failed, Err(failure));
            return;
        }
        let notice_owner = self.clone();
        let notice: LoginNotice = Arc::new(move |event| {
            notice_owner.update_login(target, attempt, |login| match event {
                codex::LoginEvent::Pending => { if login.phase == LoginPhase::Starting { login.phase = LoginPhase::Pending; } },
                codex::LoginEvent::BrowserOpened(success) => login.browser_open_failed = !success,
                codex::LoginEvent::CleanupUncertain => { login.phase = LoginPhase::CleanupUncertain; login.diagnostic = Some("Owned login cleanup is unconfirmed; this credential scope remains blocked.".into()); },
            });
        });
        let result = effects
            .authenticate(cancel.clone(), reopen, notice.clone())
            .await;
        let (outcome, failure) = match result {
            Ok(outcome) => (outcome, None),
            Err(failure) => (
                if cancel.is_cancelled() {
                    LoginOutcome::Cancelled
                } else {
                    LoginOutcome::Failed
                },
                Some(failure),
            ),
        };
        self.update_login(target, attempt, |login| {
            login.phase = LoginPhase::Verifying;
            login.outcome = Some(outcome);
            login.diagnostic = failure.map(|failure| failure.diagnostic);
        });
        let evidence = if self.cancel.is_cancelled() {
            Err(SetupFailure::at(
                SetupProblem::StatusReadFailed,
                "Verification skipped during owned shutdown",
            ))
        } else {
            effects.verify(self.cancel.clone(), notice).await
        };
        if let Err(failure) = self.persist_attempt(attempt, false).await {
            self.update_login(target, attempt, |login| {
                login.phase = LoginPhase::CleanupUncertain;
                login.diagnostic = Some(failure.diagnostic);
            });
            return;
        }
        self.finish_login(target, attempt, outcome, evidence);
    }
    async fn persist_attempt(
        &self,
        attempt: uuid::Uuid,
        pending: bool,
    ) -> Result<(), SetupFailure> {
        if let Some(store) = self.persistence.clone() {
            tokio::task::spawn_blocking(move || store.record(attempt, pending))
                .await
                .map_err(|_| {
                    SetupFailure::at(
                        SetupProblem::CleanupUncertain,
                        "Login intent storage worker failed",
                    )
                })?
                .map_err(|_| {
                    SetupFailure::at(
                        SetupProblem::CleanupUncertain,
                        "Login intent/cleanup could not be durably recorded",
                    )
                })?;
        }
        Ok(())
    }
    pub(super) fn finish_login(
        &self,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
        outcome: LoginOutcome,
        evidence: Result<SetupEvidence, SetupFailure>,
    ) {
        let mut records = self.records.lock().expect("provider records poisoned");
        let Some(entry) = records.get_mut(&target) else {
            return;
        };
        let Some(login) = entry
            .snapshot
            .login
            .as_mut()
            .filter(|login| login.id == attempt)
        else {
            return;
        };
        login.outcome = Some(outcome);
        login.phase = match outcome {
            LoginOutcome::Completed => LoginPhase::Completed,
            LoginOutcome::Failed => LoginPhase::Failed,
            LoginOutcome::Cancelled => LoginPhase::Cancelled,
        };
        match evidence {
            Ok(evidence) => {
                entry.snapshot.problem = (evidence.authentication == Authentication::Unavailable)
                    .then_some(SetupProblem::AuthenticationUnavailable);
                entry.snapshot.evidence = Some(evidence);
                entry.snapshot.diagnostic = None;
            }
            Err(failure) => {
                entry.snapshot.problem = Some(failure.problem);
                entry.snapshot.diagnostic = Some(failure.diagnostic);
            }
        }
        entry.snapshot.phase = SetupPhase::Observed;
        entry.snapshot.revision = entry.snapshot.revision.saturating_add(1);
        entry.control = None;
        drop(records);
        self.changed(target);
    }
    pub(super) fn cancel_login(
        &self,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
    ) -> Result<ProviderReply, RepositoryError> {
        {
            let mut records = self.records.lock().expect("provider records poisoned");
            let entry = records
                .get_mut(&target)
                .ok_or_else(|| invalid_target("Unknown provider configuration"))?;
            let login = entry
                .snapshot
                .login
                .as_mut()
                .filter(|login| login.id == attempt)
                .ok_or_else(|| {
                    invalid_target("The sign-in attempt changed. Retrieve its recorded status.")
                })?;
            if let Some(control) = &entry.control {
                if matches!(login.phase, LoginPhase::Starting | LoginPhase::Pending) {
                    login.phase = LoginPhase::Cancelling;
                    entry.snapshot.revision = entry.snapshot.revision.saturating_add(1);
                }
                control.cancel.cancel();
            }
        }
        self.changed(target);
        Ok(ProviderReply::Observation {
            observation: self.read(target)?,
        })
    }
    pub(super) fn reopen_browser(
        &self,
        target: uuid::Uuid,
        attempt: uuid::Uuid,
    ) -> Result<ProviderReply, RepositoryError> {
        if self.cancel.is_cancelled() {
            return Err(invalid_target("Owned shutdown is in progress"));
        }
        let records = self.records.lock().expect("provider records poisoned");
        let entry = records
            .get(&target)
            .ok_or_else(|| invalid_target("Unknown provider configuration"))?;
        if !entry
            .snapshot
            .login
            .as_ref()
            .is_some_and(|login| login.id == attempt && login.phase == LoginPhase::Pending)
        {
            return Err(invalid_target(
                "This browser attempt is no longer pending. Retrieve its recorded status.",
            ));
        }
        entry
            .control
            .as_ref()
            .ok_or_else(|| invalid_target("Browser attachment unavailable"))?
            .reopen
            .try_send(())
            .map_err(|_| {
                invalid_target(
                    "A browser-open action is already pending. Retrieve its recorded status.",
                )
            })?;
        Ok(ProviderReply::Observation {
            observation: entry.snapshot.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct MemoryStore {
        calls: Arc<Mutex<Vec<&'static str>>>,
        fail: Option<bool>,
    }
    impl crate::storage::ProviderLoginPersistence for MemoryStore {
        fn record(&self, _: uuid::Uuid, pending: bool) -> Result<(), RepositoryError> {
            self.calls
                .lock()
                .unwrap()
                .push(if pending { "intent" } else { "cleanup" });
            if self.fail == Some(pending) {
                Err(RepositoryError::storage("controlled storage failure"))
            } else {
                Ok(())
            }
        }
    }
    struct Evidence {
        calls: Arc<Mutex<Vec<&'static str>>>,
        cancelled: bool,
    }
    impl BrowserEffects for Evidence {
        async fn authenticate(
            &self,
            cancel: CancellationToken,
            _: mpsc::Receiver<()>,
            notice: LoginNotice,
        ) -> Result<LoginOutcome, SetupFailure> {
            self.calls.lock().unwrap().push("authenticate");
            notice(codex::LoginEvent::Pending);
            if self.cancelled {
                assert!(cancel.is_cancelled());
                Err(SetupFailure::at(
                    SetupProblem::StatusReadFailed,
                    "Version setup interrupted by cancellation",
                ))
            } else {
                Ok(LoginOutcome::Completed)
            }
        }
        async fn verify(
            &self,
            _: CancellationToken,
            _: LoginNotice,
        ) -> Result<SetupEvidence, SetupFailure> {
            self.calls.lock().unwrap().push("verify");
            Err(SetupFailure::at(
                SetupProblem::StatusReadFailed,
                "controlled verification failure",
            ))
        }
    }
    fn run(future: impl Future<Output = ()>) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap()
            .block_on(future);
    }
    #[test]
    fn durable_intent_precedes_effects_and_cleanup_failure_preserves_scope_fence() {
        run(async {
            for fail in [None, Some(true), Some(false)] {
                let calls = Arc::new(Mutex::new(Vec::new()));
                let mut raw = ProviderConnections::test_owner();
                raw.persistence = Some(Arc::new(MemoryStore {
                    calls: calls.clone(),
                    fail,
                }));
                let owner = Arc::new(raw);
                let target = owner.test_target("codex");
                let attempt = uuid::Uuid::new_v4();
                owner.admit_login(target, attempt, 0).unwrap();
                let (cancel, receive) = {
                    let mut records = owner.records.lock().unwrap();
                    let control = records.get_mut(&target).unwrap().control.as_mut().unwrap();
                    (control.cancel.clone(), control.receiver.take().unwrap())
                };
                owner
                    .execute_browser(
                        target,
                        attempt,
                        cancel,
                        receive,
                        Evidence {
                            calls: calls.clone(),
                            cancelled: false,
                        },
                    )
                    .await;
                let snapshot = owner.read(target).unwrap();
                if fail == Some(true) {
                    assert_eq!(*calls.lock().unwrap(), ["intent"]);
                    assert_eq!(snapshot.login.unwrap().phase, LoginPhase::Failed);
                } else {
                    assert_eq!(
                        *calls.lock().unwrap(),
                        ["intent", "authenticate", "verify", "cleanup"]
                    );
                    assert_eq!(
                        snapshot.login.unwrap().phase,
                        if fail == Some(false) {
                            LoginPhase::CleanupUncertain
                        } else {
                            LoginPhase::Completed
                        }
                    );
                    if fail == Some(false) {
                        assert!(
                            owner
                                .admit_login(target, uuid::Uuid::new_v4(), snapshot.revision)
                                .is_err()
                        );
                    } else {
                        assert_eq!(snapshot.problem, Some(SetupProblem::StatusReadFailed));
                    }
                }
            }
        });
    }
    #[test]
    fn shutdown_cancels_late_preparation_and_skips_verification_without_replacing_work() {
        run(async {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let mut raw = ProviderConnections::test_owner();
            raw.persistence = Some(Arc::new(MemoryStore {
                calls: calls.clone(),
                fail: None,
            }));
            let owner = Arc::new(raw);
            let target = owner.test_target("codex");
            let attempt = uuid::Uuid::new_v4();
            owner.admit_login(target, attempt, 0).unwrap();
            let (cancel, receive) = {
                let mut records = owner.records.lock().unwrap();
                let control = records.get_mut(&target).unwrap().control.as_mut().unwrap();
                (control.cancel.clone(), control.receiver.take().unwrap())
            };
            owner.cancel.cancel();
            owner
                .execute_browser(
                    target,
                    attempt,
                    cancel,
                    receive,
                    Evidence {
                        calls: calls.clone(),
                        cancelled: true,
                    },
                )
                .await;
            assert_eq!(
                *calls.lock().unwrap(),
                ["intent", "authenticate", "cleanup"]
            );
            assert_eq!(
                owner.read(target).unwrap().login.unwrap().phase,
                LoginPhase::Cancelled
            );
            owner.finish().await.unwrap();
        });
    }
}
