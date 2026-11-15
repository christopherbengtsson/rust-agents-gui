use super::codex;
use crate::{
    observations::{Journal, Target},
    workspaces::{ErrorCode, RepositoryError},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    future::Future,
    io,
    sync::{Arc, Mutex},
};
use tokio::task::JoinSet;
mod login;
mod selection;
pub use login::{LoginAttempt, LoginOutcome, LoginPhase};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct ProviderTarget {
    pub id: uuid::Uuid,
    pub provider: String,
    pub executable: Option<String>,
    pub selected_executable: Option<String>,
    pub discovery_source: Option<String>,
    pub codex_home: String,
    pub working_directory: String,
    pub minimum_supported_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum Authentication {
    Chatgpt,
    ApiKey,
    NotRequired,
    Unavailable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum BillingRoute {
    Chatgpt,
    ApiKey,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct SetupEvidence {
    pub version: String,
    pub authentication: Authentication,
    pub billing: BillingRoute,
    pub account: Option<String>,
    pub plan: Option<String>,
    pub model_provider: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SetupProblem {
    MissingExecutable,
    IncompatibleProtocol,
    AuthenticationUnavailable,
    StatusReadFailed,
    CleanupUncertain,
    SelectionFailed,
}

#[derive(Debug)]
pub(crate) struct SetupFailure {
    pub problem: SetupProblem,
    pub diagnostic: String,
    source: Option<io::Error>,
}
impl SetupFailure {
    pub(crate) fn at(problem: SetupProblem, detail: impl Into<String>) -> Self {
        Self {
            problem,
            diagnostic: detail.into().chars().take(512).collect(),
            source: None,
        }
    }
    pub(crate) fn io(stage: &str, error: io::Error) -> Self {
        let diagnostic = format!(
            "{stage}: {:?}; OS error {:?}",
            error.kind(),
            error.raw_os_error()
        );
        Self {
            problem: SetupProblem::StatusReadFailed,
            diagnostic,
            source: Some(error),
        }
    }
}
impl From<SetupProblem> for SetupFailure {
    fn from(problem: SetupProblem) -> Self {
        Self::at(problem, "Setup evidence could not be interpreted")
    }
}
impl std::fmt::Display for SetupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.diagnostic)
    }
}
impl std::error::Error for SetupFailure {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source as &dyn std::error::Error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SetupPhase {
    Unknown,
    Checking,
    Observed,
    CleanupUncertain,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct ProviderObservation {
    pub selection: Option<Box<ExecutableSelection>>,
    pub target: ProviderTarget,
    pub revision: u32,
    pub phase: SetupPhase,
    pub evidence: Option<SetupEvidence>,
    pub problem: Option<SetupProblem>,
    pub diagnostic: Option<String>,
    pub login: Option<Box<LoginAttempt>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct ExecutableSelection {
    pub id: uuid::Uuid,
    pub path: String,
    pub phase: SelectionPhase,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum SelectionPhase {
    Checking,
    Remembered,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum ProviderRequest {
    Target,
    Observe {
        target_id: uuid::Uuid,
        refresh: bool,
    },
    StartBrowserLogin {
        target_id: uuid::Uuid,
        attempt_id: uuid::Uuid,
        expected_revision: u32,
    },
    CancelLogin {
        target_id: uuid::Uuid,
        attempt_id: uuid::Uuid,
    },
    ReopenBrowser {
        target_id: uuid::Uuid,
        attempt_id: uuid::Uuid,
    },
    ReconcileLogin {
        target_id: uuid::Uuid,
        attempt_id: uuid::Uuid,
    },
    SelectExecutable {
        target_id: uuid::Uuid,
        action_id: uuid::Uuid,
        expected_revision: u32,
        path: String,
    },
    ReconcileSelection {
        target_id: uuid::Uuid,
        action_id: uuid::Uuid,
    },
    BeginShutdown,
    ShutdownStatus,
    FinishShutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderReply {
    Target {
        target: ProviderTarget,
    },
    Observation {
        observation: ProviderObservation,
    },
    Shutdown {
        complete: bool,
        diagnostic: Option<String>,
    },
}

struct Entry {
    snapshot: ProviderObservation,
    scope: String,
    control: Option<login::LoginControl>,
    launch: Option<codex::Launch>,
}
pub struct ProviderConnections {
    records: Mutex<BTreeMap<uuid::Uuid, Entry>>,
    tasks: Mutex<JoinSet<()>>,
    journal: Arc<Journal>,
    launch: Option<codex::Launch>,
    cancel: tokio_util::sync::CancellationToken,
    persistence: Option<Arc<dyn crate::storage::ProviderLoginPersistence>>,
    selection_store: Option<Arc<dyn crate::storage::ProviderSelectionPersistence>>,
    shutdown: Mutex<Option<Result<(), String>>>,
    exit_requested: std::sync::atomic::AtomicBool,
}
impl ProviderConnections {
    pub(crate) fn open(
        journal: Arc<Journal>,
        state: &std::path::Path,
    ) -> Result<Self, RepositoryError> {
        let (launch, mut target) = codex::Launch::capture();
        let selection_store = Arc::new(crate::storage::ProviderSelectionStore::open(
            state,
            launch.credential_scope(),
        )?);
        let selected = selection_store.selected()?;
        if let Some((_, path)) = &selected {
            target.selected_executable = Some(path.clone());
        }
        let persistence = Arc::new(crate::storage::ProviderLoginStore::open(
            state,
            launch.credential_scope(),
        )?);
        let recovered = persistence.pending()?;
        Ok(Self {
            records: Mutex::new(BTreeMap::from([(
                target.id,
                Entry {
                    snapshot: ProviderObservation {
                        selection: selected.map(|(id, path)| {
                            Box::new(ExecutableSelection {
                                id,
                                path,
                                phase: SelectionPhase::Remembered,
                            })
                        }),
                        target,
                        revision: 0,
                        phase: SetupPhase::Unknown,
                        evidence: None,
                        problem: None,
                        diagnostic: None,
                        login: recovered.map(LoginAttempt::interrupted).map(Box::new),
                    },
                    scope: launch.credential_scope(),
                    control: None,
                    launch: None,
                },
            )])),
            tasks: Mutex::new(JoinSet::new()),
            journal,
            launch: Some(launch),
            cancel: tokio_util::sync::CancellationToken::new(),
            persistence: Some(persistence),
            selection_store: Some(selection_store),
            shutdown: Mutex::new(None),
            exit_requested: std::sync::atomic::AtomicBool::new(false),
        })
    }
    pub(crate) fn request(
        self: &Arc<Self>,
        request: ProviderRequest,
    ) -> Result<ProviderReply, RepositoryError> {
        match request {
            ProviderRequest::ReconcileSelection {
                target_id,
                action_id,
            } => {
                let mut records = self.records.lock().expect("provider records poisoned");
                let entry = records
                    .get_mut(&target_id)
                    .ok_or_else(|| invalid_target("Unknown provider configuration"))?;
                if entry
                    .snapshot
                    .selection
                    .as_ref()
                    .is_none_or(|selection| selection.id != action_id)
                    && !matches!(
                        entry.snapshot.phase,
                        SetupPhase::Checking | SetupPhase::CleanupUncertain
                    )
                    && !entry
                        .snapshot
                        .login
                        .as_ref()
                        .is_some_and(|login| login.phase.active())
                {
                    entry.snapshot.revision = entry
                        .snapshot
                        .revision
                        .checked_add(1)
                        .ok_or_else(|| invalid_target("Provider revision exhausted"))?;
                }
                let observation = entry.snapshot.clone();
                drop(records);
                self.changed(target_id);
                Ok(ProviderReply::Observation { observation })
            }
            ProviderRequest::SelectExecutable {
                target_id,
                action_id,
                expected_revision,
                path,
            } => {
                if !selection::valid_path(&path) {
                    return Err(invalid_target(
                        "Choose an absolute executable path without control characters (maximum 4096 bytes).",
                    ));
                }
                let owner = self.clone();
                let selected = path.clone();
                let observation = self.check_with(
                    target_id,
                    true,
                    Some((action_id, expected_revision, path)),
                    move || async move { owner.discover_verify(target_id, Some(selected)).await },
                )?;
                Ok(ProviderReply::Observation { observation })
            }
            ProviderRequest::StartBrowserLogin {
                target_id,
                attempt_id,
                expected_revision,
            } => self.start_browser(target_id, attempt_id, expected_revision),
            ProviderRequest::CancelLogin {
                target_id,
                attempt_id,
            } => self.cancel_login(target_id, attempt_id),
            ProviderRequest::ReopenBrowser {
                target_id,
                attempt_id,
            } => self.reopen_browser(target_id, attempt_id),
            ProviderRequest::ReconcileLogin {
                target_id,
                attempt_id,
            } => self.reconcile_login(target_id, attempt_id),
            ProviderRequest::BeginShutdown => {
                self.begin_shutdown();
                Ok(self.shutdown_status())
            }
            ProviderRequest::ShutdownStatus => Ok(self.shutdown_status()),
            ProviderRequest::FinishShutdown => {
                if !self.shutdown_complete() {
                    return Err(invalid_target("Owned shutdown is not complete"));
                }
                self.exit_requested
                    .store(true, std::sync::atomic::Ordering::SeqCst);
                Ok(self.shutdown_status())
            }
            ProviderRequest::Target => {
                let records = self.records.lock().expect("provider records poisoned");
                let target = records
                    .values()
                    .find(|entry| entry.snapshot.target.provider == "codex")
                    .ok_or_else(|| invalid_target("Codex setup is unavailable"))?
                    .snapshot
                    .target
                    .clone();
                Ok(ProviderReply::Target { target })
            }
            ProviderRequest::Observe { target_id, refresh } => {
                // Baseline/reconnect/status retrieval has no process effects.
                // A committed tab activation explicitly requests refresh:true.
                if !refresh {
                    return Ok(ProviderReply::Observation {
                        observation: self.read(target_id)?,
                    });
                }
                let owner = self.clone();
                let observation = self.observe_with(target_id, refresh, move || async move {
                    owner.discover_verify(target_id, None).await
                })?;
                Ok(ProviderReply::Observation { observation })
            }
        }
    }
    fn read(&self, id: uuid::Uuid) -> Result<ProviderObservation, RepositoryError> {
        self.records
            .lock()
            .expect("provider records poisoned")
            .get(&id)
            .map(|entry| entry.snapshot.clone())
            .ok_or_else(|| {
                invalid_target(
                    "Provider configuration changed. Refresh setup using the current supervisor.",
                )
            })
    }
    fn observe_with<F: Future<Output = Result<SetupEvidence, SetupFailure>> + Send + 'static>(
        self: &Arc<Self>,
        id: uuid::Uuid,
        refresh: bool,
        probe: impl FnOnce() -> F,
    ) -> Result<ProviderObservation, RepositoryError> {
        self.check_with(id, refresh, None, probe)
    }
    fn check_with<F: Future<Output = Result<SetupEvidence, SetupFailure>> + Send + 'static>(
        self: &Arc<Self>,
        id: uuid::Uuid,
        refresh: bool,
        selection: Option<(uuid::Uuid, u32, String)>,
        probe: impl FnOnce() -> F,
    ) -> Result<ProviderObservation, RepositoryError> {
        let mut tasks = self.tasks.lock().expect("provider tasks poisoned");
        let mut records = self.records.lock().expect("provider records poisoned");
        if let Some((action, expected, path)) = &selection {
            let entry = records
                .get(&id)
                .ok_or_else(|| invalid_target("Unknown provider configuration"))?;
            if let Some(previous) = entry
                .snapshot
                .selection
                .as_ref()
                .filter(|previous| previous.id == *action)
            {
                if previous.path != *path {
                    return Err(invalid_target(
                        "Selection identity already belongs to another path",
                    ));
                }
                return Ok(entry.snapshot.clone());
            }
            if entry.snapshot.revision != *expected {
                return Err(invalid_target(
                    "Setup changed. Retrieve current status before selecting Codex.",
                ));
            }
        }
        let scope = records
            .get(&id)
            .ok_or_else(|| invalid_target("Unknown provider configuration"))?
            .scope
            .clone();
        if records.values().any(|entry| {
            entry.scope == scope
                && (matches!(
                    entry.snapshot.phase,
                    SetupPhase::Checking | SetupPhase::CleanupUncertain
                ) || entry
                    .snapshot
                    .login
                    .as_ref()
                    .is_some_and(|login| login.phase.active()))
        }) {
            if selection.is_some() {
                return Err(invalid_target(
                    "An active check, sign-in or cleanup fence blocks executable selection.",
                ));
            }
            return records
                .get(&id)
                .map(|entry| entry.snapshot.clone())
                .ok_or_else(|| invalid_target("Unknown provider configuration"));
        }
        let entry = records
            .get_mut(&id)
            .ok_or_else(|| invalid_target("Unknown provider configuration. Refresh setup."))?;
        if matches!(
            entry.snapshot.phase,
            SetupPhase::Checking | SetupPhase::CleanupUncertain
        ) || entry
            .snapshot
            .login
            .as_ref()
            .is_some_and(|login| login.phase.active())
            || (!refresh && entry.snapshot.phase != SetupPhase::Unknown)
        {
            return Ok(entry.snapshot.clone());
        }
        if self.cancel.is_cancelled() {
            return Err(invalid_target("Provider setup is shutting down"));
        }
        let revision = entry
            .snapshot
            .revision
            .checked_add(1)
            .filter(|revision| *revision < u32::MAX)
            .ok_or_else(|| {
                invalid_target("Provider revision exhausted; restart the supervisor.")
            })?;
        entry.snapshot.revision = revision;
        entry.snapshot.phase = SetupPhase::Checking;
        entry.launch = None;
        if let Some((id, _, path)) = selection {
            entry.snapshot.selection = Some(Box::new(ExecutableSelection {
                id,
                path,
                phase: SelectionPhase::Checking,
            }));
        }
        let snapshot = entry.snapshot.clone();
        while let Some(result) = tasks.try_join_next() {
            // Probe tasks contain no unchecked external panics. A panic is an
            // invariant failure; keep failed work visibly fenced rather than replay.
            if result.is_err() {
                return Err(invalid_target(
                    "Provider worker failed; restart the supervisor after owned cleanup.",
                ));
            }
        }
        let owner = self.clone();
        let future = probe();
        tasks.spawn(async move {
            owner.complete(id, revision, future.await);
        });
        self.journal
            .publish(vec![Target::Provider { target_id: id }]);
        drop(records);
        self.journal.wake();
        Ok(snapshot)
    }
    fn complete(
        &self,
        id: uuid::Uuid,
        revision: u32,
        outcome: Result<SetupEvidence, SetupFailure>,
    ) {
        let mut records = self.records.lock().expect("provider records poisoned");
        let Some(entry) = records
            .get_mut(&id)
            .filter(|entry| entry.snapshot.revision == revision)
        else {
            return;
        };
        match outcome {
            Ok(evidence) => {
                entry.snapshot.diagnostic = None;
                entry.snapshot.problem = if evidence.authentication == Authentication::Unavailable {
                    Some(SetupProblem::AuthenticationUnavailable)
                } else {
                    None
                };
                entry.snapshot.evidence = Some(evidence);
            }
            Err(failure) => {
                entry.snapshot.problem = Some(failure.problem);
                entry.snapshot.diagnostic = Some(failure.diagnostic);
            }
        }
        if let Some(selection) = entry
            .snapshot
            .selection
            .as_mut()
            .filter(|selection| selection.phase == SelectionPhase::Checking)
        {
            selection.phase = SelectionPhase::Failed;
        }
        entry.snapshot.phase = SetupPhase::Observed;
        entry.snapshot.revision += 1; // start reserves this final revision.
        self.journal
            .publish(vec![Target::Provider { target_id: id }]);
        drop(records);
        self.journal.wake();
    }
    fn cleanup_uncertain(&self, id: uuid::Uuid, revision: u32) {
        let mut records = self.records.lock().expect("provider records poisoned");
        let Some(entry) = records
            .get_mut(&id)
            .filter(|entry| entry.snapshot.revision == revision)
        else {
            return;
        };
        entry.snapshot.phase = SetupPhase::CleanupUncertain;
        entry.snapshot.problem = Some(SetupProblem::CleanupUncertain);
        entry.snapshot.diagnostic=Some("Complete owned process-group/pipe cleanup was not verified within EOF/TERM/KILL deadlines; the group leader remains retained".into());
        self.journal
            .publish(vec![Target::Provider { target_id: id }]);
        drop(records);
        self.journal.wake();
    }
    async fn join(&self) -> io::Result<()> {
        std::future::poll_fn(|cx| {
            let mut tasks = self.tasks.lock().expect("provider tasks poisoned");
            loop {
                match tasks.poll_join_next(cx) {
                    std::task::Poll::Ready(Some(Ok(()))) => {}
                    std::task::Poll::Ready(Some(Err(error))) => {
                        return std::task::Poll::Ready(Err(io::Error::other(error.to_string())));
                    }
                    std::task::Poll::Ready(None) => return std::task::Poll::Ready(Ok(())),
                    std::task::Poll::Pending => return std::task::Poll::Pending,
                }
            }
        })
        .await
    }
    pub(crate) async fn finish(&self) -> io::Result<()> {
        self.cancel.cancel();
        self.join().await?;
        if self
            .records
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
        {
            return Err(io::Error::other(
                "A retained sign-in has unconfirmed cleanup",
            ));
        }
        if let Some(launch) = self.launch.clone() {
            tokio::task::spawn_blocking(move || launch.remove_neutral_directory())
                .await
                .map_err(|error| io::Error::other(error.to_string()))??;
        }
        Ok(())
    }
    #[cfg(test)]
    fn test_owner() -> Self {
        Self {
            records: Mutex::default(),
            tasks: Mutex::default(),
            journal: Arc::default(),
            launch: None,
            cancel: tokio_util::sync::CancellationToken::new(),
            persistence: None,
            selection_store: None,
            shutdown: Mutex::new(None),
            exit_requested: std::sync::atomic::AtomicBool::new(false),
        }
    }
    #[cfg(test)]
    fn test_target(&self, provider: &str) -> uuid::Uuid {
        let id = uuid::Uuid::new_v4();
        let target = ProviderTarget {
            id,
            provider: provider.into(),
            executable: Some("fixture".into()),
            selected_executable: None,
            discovery_source: None,
            codex_home: "/fixture/home".into(),
            working_directory: "/fixture/neutral".into(),
            minimum_supported_version: "0.169.0".into(),
        };
        self.records.lock().unwrap().insert(
            id,
            Entry {
                snapshot: ProviderObservation {
                    selection: None,
                    target,
                    revision: 0,
                    phase: SetupPhase::Unknown,
                    evidence: None,
                    problem: None,
                    diagnostic: None,
                    login: None,
                },
                scope: id.to_string(),
                control: None,
                launch: None,
            },
        );
        id
    }
}
fn invalid_target(message: &str) -> RepositoryError {
    RepositoryError::new(ErrorCode::Connection, message)
}

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<ProviderRequest>();
    exports.add::<ExecutableSelection>();
    exports.add::<SelectionPhase>();
    exports.add::<ProviderReply>();
    exports.add::<ProviderTarget>();
    exports.add::<ProviderObservation>();
    exports.add::<SetupEvidence>();
    exports.add::<SetupPhase>();
    exports.add::<SetupProblem>();
    exports.add::<Authentication>();
    exports.add::<BillingRoute>();
    exports.add::<LoginAttempt>();
    exports.add::<LoginPhase>();
    exports.add::<LoginOutcome>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn login_admission_is_exact_and_fences_checks_and_competing_attempts() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let a = owner.test_target("codex");
            let b = owner.test_target("synthetic");
            let id = uuid::Uuid::new_v4();
            let first = owner.admit_login(a, id, 0).unwrap();
            assert_eq!(first.login.as_ref().unwrap().phase, LoginPhase::Starting);
            assert_eq!(owner.admit_login(a, id, 0).unwrap(), first);
            assert!(
                owner
                    .admit_login(a, uuid::Uuid::new_v4(), first.revision)
                    .is_err()
            );
            owner
                .observe_with(a, true, || async { panic!("login must exclude a probe") })
                .unwrap();
            assert!(owner.admit_login(b, uuid::Uuid::new_v4(), 0).is_ok());
            assert_eq!(owner.read(a).unwrap(), first);
        });
    }

    #[test]
    fn same_scope_readiness_cleanup_fences_other_checks_but_unrelated_targets_progress() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let a = owner.test_target("codex");
            let b = owner.test_target("same-scope");
            let c = owner.test_target("unrelated");
            let scope = owner.records.lock().unwrap().get(&a).unwrap().scope.clone();
            owner.records.lock().unwrap().get_mut(&b).unwrap().scope = scope;
            let (send, receive) = tokio::sync::oneshot::channel();
            let observation = owner
                .observe_with(a, true, || async { receive.await.unwrap() })
                .unwrap();
            owner.cleanup_uncertain(a, observation.revision);
            let value = owner
                .observe_with(b, true, || async {
                    panic!("same-scope cleanup must exclude this check")
                })
                .unwrap();
            assert_eq!(value.phase, SetupPhase::Unknown);
            owner
                .observe_with(c, true, || async { Ok(test_evidence()) })
                .unwrap();
            send.send(Ok(test_evidence())).unwrap();
            owner.join().await.unwrap();
            assert_eq!(owner.read(c).unwrap().phase, SetupPhase::Observed);
        });
    }

    #[test]
    fn reconciliation_fences_a_late_start_and_uncertain_cleanup_is_scope_local() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let a = owner.test_target("codex");
            let b = owner.test_target("synthetic");
            let c = owner.test_target("synthetic-shared");
            let scope = owner.records.lock().unwrap().get(&a).unwrap().scope.clone();
            owner.records.lock().unwrap().get_mut(&c).unwrap().scope = scope;
            let id = uuid::Uuid::new_v4();
            owner.reconcile_login(a, id).unwrap();
            assert!(owner.admit_login(a, id, 0).is_err());
            owner.admit_login(a, id, 1).unwrap();
            owner.update_login(a, id, |login| login.phase = LoginPhase::CleanupUncertain);
            assert!(owner.admit_login(c, uuid::Uuid::new_v4(), 0).is_err());
            assert!(owner.admit_login(b, uuid::Uuid::new_v4(), 0).is_ok());
            owner.finish_login(b, id, LoginOutcome::Completed, Ok(test_evidence()));
            assert_eq!(
                owner.read(b).unwrap().login.unwrap().phase,
                LoginPhase::Starting
            );
            assert_eq!(
                owner.read(a).unwrap().login.unwrap().phase,
                LoginPhase::CleanupUncertain
            );
            assert!(owner.finish().await.is_err());
        });
    }

    #[test]
    fn cancelled_login_retains_last_good_on_failed_verification_and_rejects_stale_cancel() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let a = owner.test_target("codex");
            owner
                .records
                .lock()
                .unwrap()
                .get_mut(&a)
                .unwrap()
                .snapshot
                .evidence = Some(test_evidence());
            let id = uuid::Uuid::new_v4();
            owner.admit_login(a, id, 0).unwrap();
            assert!(owner.cancel_login(a, uuid::Uuid::new_v4()).is_err());
            owner.cancel_login(a, id).unwrap();
            owner.cancel_login(a, id).unwrap();
            assert_eq!(
                owner.read(a).unwrap().login.unwrap().phase,
                LoginPhase::Cancelling
            );
            owner.finish_login(
                a,
                id,
                LoginOutcome::Cancelled,
                Err(SetupProblem::StatusReadFailed.into()),
            );
            let observation = owner.read(a).unwrap();
            assert_eq!(observation.evidence, Some(test_evidence()));
            assert_eq!(observation.problem, Some(SetupProblem::StatusReadFailed));
            assert_eq!(observation.login.unwrap().phase, LoginPhase::Cancelled);
            owner.finish().await.unwrap();
        });
    }

    #[test]
    fn diagnostics_retain_source_without_exporting_its_untrusted_message() {
        let failure = SetupFailure::io(
            "app-server spawn",
            io::Error::new(io::ErrorKind::PermissionDenied, "do-not-export-secret"),
        );
        assert!(failure.diagnostic.contains("PermissionDenied"));
        assert!(failure.diagnostic.contains("app-server spawn"));
        assert!(!failure.diagnostic.contains("do-not-export-secret"));
        assert!(std::error::Error::source(&failure).is_some());
    }

    #[test]
    fn independent_targets_coalesce_reads_and_reject_late_or_cross_target_evidence() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let a = owner.test_target("codex");
            let b = owner.test_target("synthetic");
            let (send, receive) = tokio::sync::oneshot::channel();
            let first = owner
                .observe_with(a, false, || async { receive.await.unwrap() })
                .unwrap();
            assert_eq!(first.phase, SetupPhase::Checking);
            owner
                .observe_with(a, true, || async { panic!("duplicate probe") })
                .unwrap();
            assert_eq!(owner.read(b).unwrap().phase, SetupPhase::Unknown);
            assert!(owner.read(uuid::Uuid::from_u128(99)).is_err());
            send.send(Err(SetupProblem::MissingExecutable.into()))
                .unwrap();
            owner.join().await.unwrap();
            assert_eq!(
                owner.read(a).unwrap().problem,
                Some(SetupProblem::MissingExecutable)
            );
            assert_eq!(owner.read(b).unwrap().problem, None);
            let before = owner.read(a).unwrap();
            owner.complete(a, before.revision.wrapping_sub(1), Ok(test_evidence()));
            assert_eq!(owner.read(a).unwrap(), before);
        });
    }

    fn test_evidence() -> SetupEvidence {
        SetupEvidence {
            version: "0.169.0".into(),
            authentication: Authentication::ApiKey,
            billing: BillingRoute::ApiKey,
            account: None,
            plan: None,
            model_provider: Some("openai".into()),
            model: None,
        }
    }

    #[test]
    fn failed_refresh_retains_last_good_and_cleanup_uncertainty_fences_replacement() {
        run(async {
            let owner = Arc::new(ProviderConnections::test_owner());
            let a = owner.test_target("codex");
            owner
                .observe_with(a, false, || async { Ok(test_evidence()) })
                .unwrap();
            owner.join().await.unwrap();
            owner
                .observe_with(a, true, || async {
                    Err(SetupProblem::StatusReadFailed.into())
                })
                .unwrap();
            owner.join().await.unwrap();
            assert!(owner.read(a).unwrap().evidence.is_some());
            assert_eq!(
                owner.read(a).unwrap().problem,
                Some(SetupProblem::StatusReadFailed)
            );
            let (send, receive) = tokio::sync::oneshot::channel();
            let snapshot = owner
                .observe_with(a, true, || async { receive.await.unwrap() })
                .unwrap();
            owner.cleanup_uncertain(a, snapshot.revision);
            owner
                .observe_with(a, true, || async {
                    panic!("cleanup must fence replacement")
                })
                .unwrap();
            assert_eq!(owner.read(a).unwrap().phase, SetupPhase::CleanupUncertain);
            send.send(Err(SetupProblem::StatusReadFailed.into()))
                .unwrap();
            owner.join().await.unwrap();
            assert_eq!(owner.read(a).unwrap().phase, SetupPhase::Observed);
        });
    }
    fn run(future: impl Future<Output = ()>) {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap()
            .block_on(future);
    }
}
