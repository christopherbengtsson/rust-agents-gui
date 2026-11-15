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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct ProviderTarget {
    pub id: uuid::Uuid,
    pub provider: String,
    pub executable: String,
    pub codex_home: String,
    pub working_directory: String,
    pub supported_version: String,
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
    pub target: ProviderTarget,
    pub revision: u32,
    pub phase: SetupPhase,
    pub evidence: Option<SetupEvidence>,
    pub problem: Option<SetupProblem>,
    pub diagnostic: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ProviderReply {
    Target { target: ProviderTarget },
    Observation { observation: ProviderObservation },
}

struct Entry {
    snapshot: ProviderObservation,
}
pub struct ProviderConnections {
    records: Mutex<BTreeMap<uuid::Uuid, Entry>>,
    tasks: Mutex<JoinSet<()>>,
    journal: Arc<Journal>,
    launch: Option<codex::Launch>,
    cancel: tokio_util::sync::CancellationToken,
}
impl ProviderConnections {
    pub(crate) fn open(journal: Arc<Journal>) -> Self {
        let (launch, target) = codex::Launch::capture();
        Self {
            records: Mutex::new(BTreeMap::from([(
                target.id,
                Entry {
                    snapshot: ProviderObservation {
                        target,
                        revision: 0,
                        phase: SetupPhase::Unknown,
                        evidence: None,
                        problem: None,
                        diagnostic: None,
                    },
                },
            )])),
            tasks: Mutex::new(JoinSet::new()),
            journal,
            launch: Some(launch),
            cancel: tokio_util::sync::CancellationToken::new(),
        }
    }
    pub(crate) fn request(
        self: &Arc<Self>,
        request: ProviderRequest,
    ) -> Result<ProviderReply, RepositoryError> {
        match request {
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
                let launch = self
                    .launch
                    .clone()
                    .ok_or_else(|| invalid_target("No provider adapter configured"))?;
                let owner = self.clone();
                let cancel = self.cancel.clone();
                let observation = self.observe_with(target_id, refresh, move || async move {
                    let generation = owner
                        .read(target_id)
                        .expect("admitted target retained")
                        .revision;
                    codex::probe(launch, &cancel, || {
                        owner.cleanup_uncertain(target_id, generation)
                    })
                    .await
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
        let mut tasks = self.tasks.lock().expect("provider tasks poisoned");
        let mut records = self.records.lock().expect("provider records poisoned");
        let entry = records
            .get_mut(&id)
            .ok_or_else(|| invalid_target("Unknown provider configuration. Refresh setup."))?;
        if matches!(
            entry.snapshot.phase,
            SetupPhase::Checking | SetupPhase::CleanupUncertain
        ) || (!refresh && entry.snapshot.phase != SetupPhase::Unknown)
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
        }
    }
    #[cfg(test)]
    fn test_target(&self, provider: &str) -> uuid::Uuid {
        let id = uuid::Uuid::new_v4();
        let target = ProviderTarget {
            id,
            provider: provider.into(),
            executable: "fixture".into(),
            codex_home: "/fixture/home".into(),
            working_directory: "/fixture/neutral".into(),
            supported_version: "0.169.0".into(),
        };
        self.records.lock().unwrap().insert(
            id,
            Entry {
                snapshot: ProviderObservation {
                    target,
                    revision: 0,
                    phase: SetupPhase::Unknown,
                    evidence: None,
                    problem: None,
                    diagnostic: None,
                },
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
    exports.add::<ProviderReply>();
    exports.add::<ProviderTarget>();
    exports.add::<ProviderObservation>();
    exports.add::<SetupEvidence>();
    exports.add::<SetupPhase>();
    exports.add::<SetupProblem>();
    exports.add::<Authentication>();
    exports.add::<BillingRoute>();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

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
