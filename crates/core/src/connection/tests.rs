use super::*;
use std::collections::VecDeque;
pub(super) mod support;
pub(super) use support::*;
pub(super) fn run<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .start_paused(true)
        .build()
        .unwrap()
        .block_on(future)
}
fn snapshot() -> SupervisorSnapshot {
    Core {
        owner: "owner-a".into(),
        pid: 42,
        workspaces: None,
        providers: None,
        journal: Arc::new(crate::observations::Journal::default()),
    }
    .snapshot(None)
}

#[test]
fn quit_before_first_snapshot_reconciles_existing_owner_or_proves_absence_without_starting() {
    run(async {
        for present in [false, true] {
            let connection =
                SupervisorConnection::new("/private/state".into(), "/missing/supervisor".into());
            let mut effects = DiscoveryScenario::new(vec![Err(io::ErrorKind::NotFound.into())]);
            effects.owner = Ok(present);
            let result = connection.quit_target_with(&mut effects).await;
            if present {
                assert!(result.is_err());
            } else {
                assert_eq!(result.unwrap(), None);
            }
            assert_eq!(effects.calls, ["snapshot", "owner"]);
            assert!(connection.prepare_snapshot().is_err());
        }
        let connection =
            SupervisorConnection::new("/private/state".into(), "/missing/supervisor".into());
        let mut effects = DiscoveryScenario::new(vec![Ok(snapshot())]);
        assert_eq!(
            connection.quit_target_with(&mut effects).await.unwrap(),
            Some("owner-a".into())
        );
        assert_eq!(effects.calls, ["snapshot"]);
        assert!(connection.require_connection("owner-a").is_ok());
    });
}

#[test]
fn quit_does_not_prove_absence_before_admitted_startup_physically_settles() {
    run(async {
        let connection =
            SupervisorConnection::new("/private/state".into(), "/missing/supervisor".into());
        let startup = connection.status.try_acquire().unwrap();
        let mut effects = DiscoveryScenario::new(vec![Err(io::ErrorKind::NotFound.into())]);
        let mut quit = std::pin::pin!(connection.quit_target_with(&mut effects));
        std::future::poll_fn(|cx| {
            assert!(quit.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(startup);
        assert_eq!(quit.await.unwrap(), None);
        assert!(connection.prepare_snapshot().is_err());
    });
}

struct DiscoveryScenario {
    observations: VecDeque<io::Result<SupervisorSnapshot>>,
    prepared: io::Result<()>,
    owner: io::Result<bool>,
    launch: io::Result<()>,
    calls: Vec<&'static str>,
    cursors: Vec<Option<SnapshotCursor>>,
    time: Duration,
    wait_step: Duration,
}
impl DiscoveryScenario {
    fn new(observations: Vec<io::Result<SupervisorSnapshot>>) -> Self {
        Self {
            observations: observations.into(),
            prepared: Ok(()),
            owner: Ok(false),
            launch: Ok(()),
            calls: vec![],
            cursors: vec![],
            time: Duration::ZERO,
            wait_step: Duration::from_millis(25),
        }
    }
}
impl Discovery for DiscoveryScenario {
    async fn prepare(&mut self, state: &Path) -> io::Result<()> {
        assert_eq!(state, Path::new("/private/state"));
        self.calls.push("prepare");
        std::mem::replace(&mut self.prepared, Ok(()))
    }
    async fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot> {
        assert_eq!(state, Path::new("/private/state"));
        self.calls.push("snapshot");
        self.cursors.push(after);
        self.observations
            .pop_front()
            .expect("unexpected observation/retry")
    }
    async fn owner_present(&mut self, _: &Path) -> io::Result<bool> {
        self.calls.push("owner");
        std::mem::replace(&mut self.owner, Ok(false))
    }
    async fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()> {
        assert_eq!(executable, Path::new("/app/supervisor"));
        assert_eq!(state, Path::new("/private/state"));
        self.calls.push("start");
        std::mem::replace(&mut self.launch, Ok(()))
    }
    fn elapsed(&self) -> Duration {
        self.time
    }
    async fn wait(&mut self, duration: Duration) {
        assert_eq!(duration, Duration::from_millis(25));
        self.calls.push("wait");
        self.time += self.wait_step;
    }
}
fn failure(kind: io::ErrorKind) -> io::Result<SupervisorSnapshot> {
    Err(io::Error::from(kind))
}
async fn discover_scenario(scenario: &mut DiscoveryScenario) -> io::Result<()> {
    discover(
        Path::new("/private/state"),
        Path::new("/app/supervisor"),
        scenario,
    )
    .await
}

#[test]
fn existing_owner_is_discovered_without_lock_probe_or_start() {
    run(async {
        let mut scenario = DiscoveryScenario::new(vec![Ok(snapshot())]);
        discover_scenario(&mut scenario).await.unwrap();
        assert_eq!(scenario.calls, ["prepare", "snapshot"]);
    });
}

#[test]
fn absent_owner_starts_once_and_waits_for_readiness() {
    run(async {
        let mut scenario = DiscoveryScenario::new(vec![
            failure(io::ErrorKind::NotFound),
            failure(io::ErrorKind::ConnectionRefused),
            failure(io::ErrorKind::UnexpectedEof),
            Ok(snapshot()),
        ]);
        discover_scenario(&mut scenario).await.unwrap();
        assert_eq!(
            scenario.calls,
            [
                "prepare", "snapshot", "owner", "start", "snapshot", "wait", "snapshot", "wait",
                "snapshot"
            ]
        );
    });
}

#[test]
fn concurrent_owner_startup_is_waited_for_without_starting_a_competitor() {
    run(async {
        let mut scenario = DiscoveryScenario::new(vec![
            failure(io::ErrorKind::ConnectionRefused),
            failure(io::ErrorKind::NotFound),
            Ok(snapshot()),
        ]);
        scenario.owner = Ok(true);
        discover_scenario(&mut scenario).await.unwrap();
        assert_eq!(
            scenario.calls,
            [
                "prepare", "snapshot", "owner", "snapshot", "wait", "snapshot"
            ]
        );
    });
}

#[test]
fn unsafe_state_or_incompatible_endpoint_never_starts_an_owner() {
    run(async {
        let mut scenario = DiscoveryScenario::new(vec![]);
        scenario.prepared = Err(io::ErrorKind::PermissionDenied.into());
        assert_eq!(
            discover_scenario(&mut scenario).await.unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(scenario.calls, ["prepare"]);
        for kind in [
            io::ErrorKind::PermissionDenied,
            io::ErrorKind::InvalidData,
            io::ErrorKind::TimedOut,
            io::ErrorKind::UnexpectedEof,
        ] {
            let mut scenario = DiscoveryScenario::new(vec![failure(kind)]);
            assert_eq!(
                discover_scenario(&mut scenario).await.unwrap_err().kind(),
                kind
            );
            assert_eq!(scenario.calls, ["prepare", "snapshot"]);
        }
    });
}

#[test]
fn lock_probe_and_spawn_failures_remain_failures_without_retry() {
    run(async {
        let mut scenario = DiscoveryScenario::new(vec![failure(io::ErrorKind::NotFound)]);
        scenario.owner = Err(io::ErrorKind::PermissionDenied.into());
        assert!(discover_scenario(&mut scenario).await.is_err());
        assert_eq!(scenario.calls, ["prepare", "snapshot", "owner"]);
        let mut scenario = DiscoveryScenario::new(vec![failure(io::ErrorKind::NotFound)]);
        scenario.launch = Err(io::ErrorKind::NotFound.into());
        assert!(discover_scenario(&mut scenario).await.is_err());
        assert_eq!(scenario.calls, ["prepare", "snapshot", "owner", "start"]);
    });
}

#[test]
fn startup_timeout_is_controlled_and_does_not_launch_again() {
    run(async {
        let mut scenario = DiscoveryScenario::new(vec![
            failure(io::ErrorKind::NotFound),
            failure(io::ErrorKind::NotFound),
            failure(io::ErrorKind::ConnectionRefused),
        ]);
        scenario.wait_step = Duration::from_secs(5);
        let error = discover_scenario(&mut scenario).await.unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(error.to_string().contains("Do not remove the owner lock"));
        assert_eq!(
            scenario
                .calls
                .iter()
                .filter(|&&call| call == "start")
                .count(),
            1
        );
        assert_eq!(
            scenario
                .calls
                .iter()
                .filter(|&&call| call == "wait")
                .count(),
            1
        );
    });
}

#[test]
fn non_transient_readiness_failure_is_not_retried() {
    run(async {
        let mut scenario = DiscoveryScenario::new(vec![
            failure(io::ErrorKind::NotFound),
            failure(io::ErrorKind::InvalidData),
        ]);
        assert_eq!(
            discover_scenario(&mut scenario).await.unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert!(!scenario.calls.contains(&"wait"));
    });
}

#[test]
fn refresh_after_disconnect_retrieves_cursor_without_restart_or_replay() {
    run(async {
        let first = snapshot();
        let mut recovered = first.clone();
        recovered.reset = false;
        let mut scenario = DiscoveryScenario::new(vec![
            Ok(first.clone()),
            Ok(first.clone()),
            failure(io::ErrorKind::ConnectionRefused),
            Ok(recovered),
        ]);
        let connection =
            SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        connection.snapshot_with(&mut scenario).await.unwrap();
        let error = connection.snapshot_with(&mut scenario).await.unwrap_err();
        assert!(error.to_string().contains("no actions have been replayed"));
        assert_eq!(
            connection
                .snapshot_with(&mut scenario)
                .await
                .unwrap()
                .cursor,
            first.cursor
        );
        assert_eq!(
            scenario.calls,
            ["prepare", "snapshot", "snapshot", "snapshot", "snapshot"]
        );
        assert_eq!(
            scenario.cursors,
            [None, None, Some(first.cursor.clone()), Some(first.cursor)]
        );
    });
}

#[test]
fn spawn_command_preserves_executable_and_single_state_argument() {
    run(async {
        let command = supervisor_command(
            Path::new("/app with spaces/supervisor"),
            Path::new("/private/state with spaces"),
        );
        assert_eq!(command.get_program(), "/app with spaces/supervisor");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["/private/state with spaces"]
        );
    });
}

#[test]
fn dropping_desktop_and_reopening_observes_the_same_owner_without_starting() {
    run(async {
        let before = snapshot();
        let mut scenario = DiscoveryScenario::new(vec![
            Ok(before.clone()),
            Ok(before.clone()),
            Ok(before.clone()),
            Ok(before.clone()),
        ]);
        let desktop = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        desktop.snapshot_with(&mut scenario).await.unwrap();
        drop(desktop);
        assert_eq!(scenario.calls, ["prepare", "snapshot", "snapshot"]);
        let reopened = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        assert_eq!(
            reopened.snapshot_with(&mut scenario).await.unwrap().cursor,
            before.cursor
        );
        assert_eq!(
            scenario.calls,
            [
                "prepare", "snapshot", "snapshot", "prepare", "snapshot", "snapshot"
            ]
        );
    });
}

#[test]
fn lost_owner_never_restarts_on_refresh_but_reopen_can_admit_a_new_owner() {
    run(async {
        let before = snapshot();
        let after = Core {
            owner: "owner-b".into(),
            pid: 43,
            workspaces: None,
            providers: None,
            journal: Arc::new(crate::observations::Journal::default()),
        }
        .snapshot(Some(before.cursor.clone()));
        let mut scenario = DiscoveryScenario::new(vec![
            Ok(before.clone()),
            Ok(before),
            failure(io::ErrorKind::ConnectionRefused),
            failure(io::ErrorKind::ConnectionRefused),
            Ok(after.clone()),
            Ok(after.clone()),
        ]);
        let desktop = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        desktop.snapshot_with(&mut scenario).await.unwrap();
        assert!(desktop.snapshot_with(&mut scenario).await.is_err());
        assert!(!scenario.calls.contains(&"start"));
        let reopened = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        let observed = reopened.snapshot_with(&mut scenario).await.unwrap();
        assert!(observed.reset);
        assert_eq!(observed.cursor, after.cursor);
        assert_eq!(
            scenario
                .calls
                .iter()
                .filter(|&&call| call == "start")
                .count(),
            1
        );
    });
}

#[test]
fn oversized_registration_is_a_definite_capacity_rejection_before_transport() {
    run(async {
        let client = SupervisorClient::new("/unused-private-state".into());
        for path in [
            format!("/{}", "a".repeat(4096)),
            format!("/{}", "é".repeat(2048)),
            format!("/{}", "a".repeat(65_536)),
        ] {
            let request = RepositoryRequest::Register {
                request: crate::workspaces::RegisterRepository {
                    create_initial_workspace: false,
                    action_id: crate::workspaces::ActionId(uuid::Uuid::from_u128(1)),
                    path,
                },
            };
            let error = client
                .repository_request_with("owner-a", request, async |_, _| {
                    panic!("oversized input must not access state, connect, or send")
                })
                .await
                .unwrap_err();
            let RequestFailure::Rejected { error } = error else {
                panic!("expected local rejection")
            };
            assert_eq!(error.code, ErrorCode::Capacity);
            assert!(error.message.contains("4096 UTF-8 bytes"));
        }
    });
}

#[test]
fn bounded_registration_uses_transport_and_preserves_uncertain_failures() {
    run(async {
        let client = SupervisorClient::new("/unused-private-state".into());
        for path in [
            format!("/{}", "a".repeat(4095)),
            format!("/{}a", "é".repeat(2047)),
            format!("/{}", "\u{1}".repeat(4095)),
        ] {
            let request = RepositoryRequest::Register {
                request: crate::workspaces::RegisterRepository {
                    create_initial_workspace: false,
                    action_id: crate::workspaces::ActionId(uuid::Uuid::from_u128(1)),
                    path,
                },
            };
            let mut calls = 0;
            for kind in [
                io::ErrorKind::BrokenPipe,
                io::ErrorKind::ConnectionReset,
                io::ErrorKind::TimedOut,
            ] {
                let error = client
                    .repository_request_with("owner-a", request.clone(), async |state, envelope| {
                        calls += 1;
                        assert_eq!(state, Path::new("/unused-private-state"));
                        let Some(RepositoryRequest::Register { request: sent }) =
                            &envelope.repository
                        else {
                            panic!("expected the targeted registration request")
                        };
                        let RepositoryRequest::Register { request: expected } = &request else {
                            unreachable!()
                        };
                        assert_eq!(sent, expected);
                        assert!(encode_payload(envelope).unwrap().len() <= 65_532);
                        Err(unavailable(io::Error::from(kind)))
                    })
                    .await
                    .unwrap_err();
                let RequestFailure::Transport { error } = error else {
                    panic!("expected transport failure")
                };
                assert_eq!(error.code, TransportCode::ReplyUnavailable);
                assert!(error.message.contains("recorded action status"));
            }
            assert_eq!(calls, 3);
        }
    });
}

#[test]
fn targeted_client_rejects_replacement_kind_and_target_without_another_exchange() {
    run(async {
        let client = SupervisorClient::new("/unused-private-state".into());
        let id = crate::workspaces::ActionId(uuid::Uuid::from_u128(1));
        for response in [
            Response::Repository {
                owner: "owner-b".into(),
                result: Ok(RepositoryReply::Action { registration: None }),
            },
            Response::Workspace {
                owner: "owner-a".into(),
                result: Ok(crate::workspaces::WorkspaceReply::Action {
                    action: None,
                    workspace: None,
                }),
            },
            Response::Repository {
                owner: "owner-a".into(),
                result: Ok(RepositoryReply::Action {
                    registration: Some(crate::workspaces::Registration {
                        action_id: crate::workspaces::ActionId(uuid::Uuid::from_u128(2)),
                        path: "/one".into(),
                        create_initial_workspace: false,
                        initial_workspace_id: None,
                        state: crate::workspaces::RegistrationState::Pending,
                    }),
                }),
            },
        ] {
            let failure = client
                .repository_request_with(
                    "owner-a",
                    RepositoryRequest::Action { action_id: id },
                    async |_, request| {
                        assert_eq!(request.expected_owner.as_deref(), Some("owner-a"));
                        Ok(response)
                    },
                )
                .await
                .unwrap_err();
            let RequestFailure::Transport { error } = failure else {
                panic!("must preserve uncertain delivery")
            };
            assert_eq!(error.code, TransportCode::ReplyUnavailable);
            assert_eq!(error.delivery, Delivery::Unknown);
        }
    });
}

#[test]
fn provider_transport_rejects_wrong_owner_configuration_and_ambiguous_requests_before_dispatch() {
    run(async {
        use crate::runtimes::*;
        let client = SupervisorClient::new("/unused-private-state".into());
        let id = uuid::Uuid::from_u128(1);
        for (owner, target_id) in [("owner-b", id), ("owner-a", uuid::Uuid::from_u128(2))] {
            let target = ProviderTarget {
                id: target_id,
                provider: "codex".into(),
                executable: Some("/fixture/codex".into()),
                selected_executable: None,
                discovery_source: None,
                codex_home: "/fixture/home".into(),
                working_directory: "/fixture/neutral".into(),
                minimum_supported_version: "0.169.0".into(),
            };
            let mut calls = 0;
            let result = client
                .provider_request_with(
                    "owner-a",
                    ProviderRequest::Observe {
                        target_id: id,
                        refresh: false,
                    },
                    async |_, request| {
                        calls += 1;
                        assert!(request.repository.is_none() && request.workspace.is_none());
                        Ok(Response::Provider {
                            owner: owner.into(),
                            result: Ok(ProviderReply::Observation {
                                observation: ProviderObservation {
                                    selection: None,
                                    target,
                                    revision: 2,
                                    phase: SetupPhase::Observed,
                                    evidence: None,
                                    problem: Some(SetupProblem::MissingExecutable),
                                    diagnostic: Some("synthetic missing executable".into()),
                                    login: None,
                                },
                            }),
                        })
                    },
                )
                .await;
            assert!(result.is_err());
            assert_eq!(calls, 1);
        }
        let mut request = envelope();
        request.provider = Some(ProviderRequest::Target);
        request.expected_owner = Some("old-owner".into());
        assert!(matches!(
            reject_request(&request, "owner-a").unwrap(),
            Some(Response::Transport { .. })
        ));
        request.expected_owner = Some("owner-a".into());
        assert!(reject_request(&request, "owner-a").unwrap().is_none());
        request.repository = Some(RepositoryRequest::List {
            after: None,
            refresh: false,
        });
        assert!(reject_request(&request, "owner-a").is_err());
    });
}

#[test]
fn invalid_snapshot_cannot_replace_the_connections_observed_owner() {
    run(async {
        let first = snapshot();
        for invalid in [
            SupervisorSnapshot {
                provider_login_pending: None,
                cursor: SnapshotCursor {
                    owner: "".into(),
                    revision: 1,
                },
                ..first.clone()
            },
            SupervisorSnapshot {
                provider_login_pending: None,
                cursor: SnapshotCursor {
                    owner: "owner-b".into(),
                    revision: u64::MAX,
                },
                ..first.clone()
            },
            SupervisorSnapshot {
                provider_login_pending: None,
                status: SupervisorStatus {
                    supervisor_pid: 0,
                    ..first.status.clone()
                },
                ..first.clone()
            },
        ] {
            let mut scenario =
                DiscoveryScenario::new(vec![Ok(first.clone()), Ok(first.clone()), Ok(invalid)]);
            let connection =
                SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
            connection.snapshot_with(&mut scenario).await.unwrap();
            assert_eq!(
                connection
                    .snapshot_with(&mut scenario)
                    .await
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidData
            );
            assert_eq!(
                connection.observed.lock().unwrap().cursor,
                Some(first.cursor.clone())
            );
        }
    });
}

#[test]
fn application_corpus_matches_pure_rust_serialization() {
    run(async {
        use crate::workspaces::*;
        use serde_json::{Value, json};
        let corpus: Value =
            serde_json::from_str(include_str!("../../../../contracts/observations-v14.json"))
                .unwrap();
        let owner = "owner-a";
        let repository_id = RepositoryId(uuid::Uuid::from_u128(1));
        let action_id = ActionId(uuid::Uuid::from_u128(2));
        let workspace_id = WorkspaceId(uuid::Uuid::from_u128(3));
        let comparison_id = ComparisonId(uuid::Uuid::from_u128(4));
        let file_id = FileId(uuid::Uuid::from_u128(5));
        let core = Core {
            owner: owner.into(),
            pid: 41,
            workspaces: None,
            providers: None,
            journal: Arc::new(crate::observations::Journal::default()),
        };
        assert_eq!(corpus["protocol"], PROTOCOL_VERSION);
        assert_eq!(
            corpus["snapshot"],
            serde_json::to_value(core.snapshot(None)).unwrap()
        );
        assert_eq!(corpus["desktop_pid"], 42);
        let repository_case =
            |request: RepositoryRequest,
             result: std::result::Result<RepositoryReply, RepositoryError>| {
                json!({
                    "request": request, "reply": OwnedReply { owner: owner.into(), result }
                })
            };
        let register = RegisterRepository {
            action_id,
            path: "/projects/one".into(),
            create_initial_workspace: false,
        };
        let repositories = vec![
            repository_case(
                RepositoryRequest::List {
                    after: None,
                    refresh: false,
                },
                Ok(RepositoryReply::Repositories {
                    page: RepositoryPage {
                        repositories: vec![Repository {
                            id: repository_id,
                            name: "one".into(),
                            path: "/projects/one".into(),
                            availability: Availability::Available,
                        }],
                        next: None,
                    },
                }),
            ),
            repository_case(
                RepositoryRequest::Register {
                    request: register.clone(),
                },
                Ok(RepositoryReply::Registration {
                    registration: Registration {
                        action_id,
                        path: register.path,
                        create_initial_workspace: false,
                        initial_workspace_id: None,
                        state: RegistrationState::Completed { repository_id },
                    },
                }),
            ),
            repository_case(
                RepositoryRequest::Action { action_id },
                Ok(RepositoryReply::Action { registration: None }),
            ),
            repository_case(
                RepositoryRequest::Get {
                    id: repository_id,
                    refresh: false,
                },
                Err(RepositoryError::new(
                    ErrorCode::Storage,
                    "Storage unavailable",
                )),
            ),
        ];
        assert_eq!(corpus["repositories"], json!(repositories));
        let workspace = Workspace {
            id: workspace_id,
            repository_id,
            creation_action_id: action_id,
            name: None,
            branch: None,
            path: None,
            legacy: false,
            base: None,
            artifacts: Artifacts {
                branch: BranchArtifact::Unknown,
                folder: FolderArtifact::Unknown,
            },
            status: CreationStatus::Queued,
            cleanup_unconfirmed: false,
        };
        let workspace_case = |request: WorkspaceRequest, reply: WorkspaceReply| {
            json!({
                "request": request, "reply": OwnedReply { owner: owner.into(), result: Ok(reply) }
            })
        };
        let target = PreviewTarget {
            workspace_id,
            comparison_id,
            file_id,
        };
        let workspaces = vec![
            workspace_case(
                WorkspaceRequest::Create {
                    action_id,
                    repository_id,
                },
                WorkspaceReply::Action {
                    action: Some(WorkspaceAction::Create {
                        action_id,
                        repository_id,
                    }),
                    workspace: Some(workspace.clone()),
                },
            ),
            workspace_case(
                WorkspaceRequest::List {
                    repository_id,
                    after: None,
                },
                WorkspaceReply::Workspaces {
                    page: WorkspacePage {
                        workspaces: vec![workspace],
                        next: None,
                    },
                },
            ),
            workspace_case(
                WorkspaceRequest::Changes {
                    workspace_id,
                    refresh: false,
                    comparison_id: None,
                    after: None,
                },
                WorkspaceReply::Changes {
                    observation: ChangesObservation {
                        workspace_id,
                        phase: InspectionPhase::Ready,
                        error: None,
                        page: Some(ChangePage {
                            comparison_id,
                            head: "a".repeat(40),
                            files: vec![],
                            total: 0,
                            next: None,
                        }),
                    },
                },
            ),
            workspace_case(
                WorkspaceRequest::ChangePreview {
                    target: target.clone(),
                    load: true,
                },
                WorkspaceReply::ChangePreview {
                    observation: PreviewObservation {
                        target,
                        phase: InspectionPhase::Ready,
                        error: None,
                        preview: Some(FilePreview {
                            comparison_id,
                            head: "a".repeat(40),
                            rows: vec![DiffRow {
                                kind: RowKind::Added,
                                old_line: None,
                                new_line: Some(1),
                                text: "hello".into(),
                            }],
                            limited: None,
                            unsupported: None,
                            note: None,
                        }),
                    },
                },
            ),
        ];
        assert_eq!(corpus["workspaces"], json!(workspaces));
        let mut failures = vec![RequestFailure::Rejected {
            error: RepositoryError::new(ErrorCode::Capacity, "Local input limit"),
        }];
        for (code, delivery, message) in [
            (TransportCode::Busy, Delivery::NotSent, "Busy"),
            (
                TransportCode::NotConnected,
                Delivery::NotSent,
                "Not connected",
            ),
            (
                TransportCode::OwnerChanged,
                Delivery::Rejected,
                "Owner changed",
            ),
            (
                TransportCode::ReplyUnavailable,
                Delivery::Unknown,
                "Reply unavailable",
            ),
            (
                TransportCode::Incompatible,
                Delivery::Rejected,
                "Incompatible versions",
            ),
        ] {
            failures.push(RequestFailure::transport(code, delivery, message));
        }
        assert_eq!(corpus["failures"], json!(failures));
    });
}

#[test]
fn desktop_admission_leaves_status_and_intent_independent_of_stalled_observations() {
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    initialize(&connection);
    let read = || RepositoryRequest::List {
        after: None,
        refresh: false,
    };
    let first = connection
        .prepare_repository_request("owner-a".into(), read())
        .unwrap();
    let second = connection
        .prepare_repository_request("owner-a".into(), read())
        .unwrap();
    let Err(RequestFailure::Transport { error }) =
        connection.prepare_repository_request("owner-a".into(), read())
    else {
        panic!("third observation must fail before spawn")
    };
    assert_eq!(error.code, TransportCode::Busy);
    assert_eq!(error.delivery, Delivery::NotSent);
    let status = connection.prepare_snapshot().unwrap();
    let intent = connection
        .prepare_repository_request(
            "owner-a".into(),
            RepositoryRequest::Register {
                request: crate::workspaces::RegisterRepository {
                    action_id: crate::workspaces::ActionId(uuid::Uuid::nil()),
                    path: "/repo".into(),
                    create_initial_workspace: false,
                },
            },
        )
        .unwrap();
    assert!(connection.prepare_snapshot().is_err());
    drop((status, intent, first));
    assert!(
        connection
            .prepare_repository_request("owner-a".into(), read())
            .is_ok()
    );
    drop(second);
}

#[test]
fn workspace_variants_share_observation_and_intent_budgets_with_repository_requests() {
    use serde_json::json;
    let connection = SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    initialize(&connection);
    let id = "00000000-0000-4000-8000-000000000001";
    let observations = [
        json!({"operation":"list","repository_id":id,"after":null}),
        json!({"operation":"get","id":id}),
        json!({"operation":"action","action_id":id}),
        json!({"operation":"changes","workspace_id":id,"refresh":true,"comparison_id":null,"after":null}),
        json!({"operation":"change_preview","target":{"workspace_id":id,"comparison_id":id,"file_id":id},"load":true}),
    ];
    for value in observations {
        let request = serde_json::from_value(value).unwrap();
        let first = connection
            .prepare_workspace_request("owner-a".into(), request)
            .unwrap();
        let second = connection
            .prepare_repository_request(
                "owner-a".into(),
                RepositoryRequest::Action {
                    action_id: crate::workspaces::ActionId(uuid::Uuid::nil()),
                },
            )
            .unwrap();
        assert!(
            connection
                .prepare_workspace_request(
                    "owner-a".into(),
                    crate::workspaces::WorkspaceRequest::Get {
                        id: crate::workspaces::WorkspaceId(uuid::Uuid::nil())
                    }
                )
                .is_err()
        );
        drop((first, second));
    }
    for value in [
        json!({"operation":"create","action_id":id,"repository_id":id}),
        json!({"operation":"reconcile","action_id":id,"workspace_id":id}),
    ] {
        let request = serde_json::from_value(value).unwrap();
        let first = connection
            .prepare_workspace_request("owner-a".into(), request)
            .unwrap();
        assert!(
            connection
                .prepare_workspace_request(
                    "owner-a".into(),
                    crate::workspaces::WorkspaceRequest::Create {
                        action_id: crate::workspaces::ActionId(uuid::Uuid::nil()),
                        repository_id: crate::workspaces::RepositoryId(uuid::Uuid::nil())
                    }
                )
                .is_err()
        );
        assert!(
            connection
                .prepare_repository_request(
                    "owner-a".into(),
                    RepositoryRequest::List {
                        after: None,
                        refresh: true
                    }
                )
                .is_ok()
        );
        drop(first);
    }
    let uninitialized =
        SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
    for _ in 0..4 {
        let Err(RequestFailure::Transport { error }) = uninitialized.prepare_repository_request(
            "owner-a".into(),
            RepositoryRequest::List {
                after: None,
                refresh: false,
            },
        ) else {
            panic!("uninitialized requests cannot send")
        };
        assert_eq!(error.code, TransportCode::NotConnected); // Rejected preparation never leaks its acquired permit.
    }
}

fn envelope() -> Request {
    Request {
        provider: None,
        protocol: PROTOCOL_VERSION,
        application_version: env!("CARGO_PKG_VERSION").into(),
        after: None,
        expected_owner: None,
        repository: None,
        workspace: None,
        subscribe: None,
    }
}
struct ClientScenario {
    stream: Option<Stream>,
    checks: VecDeque<io::Result<()>>,
    connect_error: Option<io::ErrorKind>,
    calls: Vec<&'static str>,
    trace: Trace,
    cancel: CancellationToken,
}
impl ClientScenario {
    fn new(input: Vec<u8>) -> Self {
        let stream = Stream::new(input);
        Self {
            trace: stream.0.clone(),
            stream: Some(stream),
            checks: VecDeque::from([Ok(()), Ok(())]),
            connect_error: None,
            calls: vec![],
            cancel: CancellationToken::new(),
        }
    }
    async fn snapshot(&mut self) -> io::Result<SupervisorSnapshot> {
        SupervisorClient::new("/private/state".into())
            .snapshot_with(None, PROTOCOL_VERSION, env!("CARGO_PKG_VERSION"), self)
            .await
    }
}
impl ClientEffects for ClientScenario {
    type Stream = Stream;
    async fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        assert_eq!(
            path,
            if directory {
                Path::new("/private/state")
            } else {
                Path::new("/private/state/supervisor.sock")
            }
        );
        self.calls.push(if directory { "state" } else { "socket" });
        self.checks.pop_front().unwrap()
    }
    async fn connect(&mut self, socket: &Path) -> io::Result<Stream> {
        assert_eq!(socket, Path::new("/private/state/supervisor.sock"));
        self.calls.push("connect");
        if let Some(kind) = self.connect_error {
            return Err(kind.into());
        }
        Ok(self.stream.take().expect("reconnect is forbidden"))
    }
    fn cancellation(&self) -> CancellationToken {
        self.cancel.clone()
    }
}
#[test]
fn status_exchange_and_version_rejection_use_real_protocol_parsers_without_domain_effects() {
    run(async {
        for (protocol, version) in [
            (9, env!("CARGO_PKG_VERSION")),
            (8, env!("CARGO_PKG_VERSION")),
            (7, env!("CARGO_PKG_VERSION")),
            (6, env!("CARGO_PKG_VERSION")),
            (999, env!("CARGO_PKG_VERSION")),
            (9, "99.0.0"),
        ] {
            let request = Request {
                provider: None,
                protocol,
                application_version: version.into(),
                ..envelope()
            };
            let stream = Stream::new(wire(&request));
            let trace = stream.0.clone();
            let calls = Mutex::new(0);
            serve_stream(
                stream,
                "owner-a",
                Arc::default(),
                &CancellationToken::new(),
                async |request| {
                    *calls.lock().unwrap() += 1;
                    assert!(request.expected_owner.is_none());
                    Ok(Response::Snapshot {
                        snapshot: snapshot(),
                    })
                },
            )
            .await
            .unwrap();
            let replies = frames::<Response>(&trace.output());
            assert_eq!(replies.len(), 1);
            if protocol == PROTOCOL_VERSION && version == env!("CARGO_PKG_VERSION") {
                assert_eq!(*calls.lock().unwrap(), 1);
                assert!(matches!(&replies[0], Response::Snapshot { .. }));
            } else {
                assert_eq!(*calls.lock().unwrap(), 0);
                assert!(
                    matches!(&replies[0], Response::Incompatible { message } if message.contains("Install matching"))
                );
            }
        }
    });
}
#[test]
fn wrong_owner_every_target_and_mixed_context_are_rejected_before_effects() {
    run(async {
        for target in [
            serde_json::json!({"repository":{"operation":"register","request":{"action_id":"00000000-0000-0000-0000-000000000001","path":"/repo"}}}),
            serde_json::json!({"repository":{"operation":"list","after":null,"refresh":true}}),
            serde_json::json!({"workspace":{"operation":"create","action_id":"00000000-0000-0000-0000-000000000001","repository_id":"00000000-0000-0000-0000-000000000002"}}),
            serde_json::json!({"workspace":{"operation":"changes","workspace_id":"00000000-0000-0000-0000-000000000003","refresh":true,"comparison_id":null,"after":null}}),
            serde_json::json!({"subscribe":{"subscription_id":"feed-1","after":null}}),
        ] {
            let mut value = serde_json::to_value(envelope()).unwrap();
            value["expected_owner"] = "old-owner".into();
            value
                .as_object_mut()
                .unwrap()
                .extend(target.as_object().unwrap().clone());
            let request: Request = serde_json::from_value(value).unwrap();
            let stream = Stream::new(wire(&request));
            let trace = stream.0.clone();
            serve_stream(
                stream,
                "owner-a",
                Arc::default(),
                &CancellationToken::new(),
                async |_| panic!("wrong owner routed"),
            )
            .await
            .unwrap();
            assert!(matches!(
                &frames::<Response>(&trace.output())[0],
                Response::Transport {
                    error: TransportFailure {
                        code: TransportCode::OwnerChanged,
                        delivery: Delivery::Rejected,
                        ..
                    }
                }
            ));
        }
        let mut request = envelope();
        request.repository = Some(RepositoryRequest::List {
            after: None,
            refresh: false,
        });
        request.subscribe = Some(subscription::Subscribe {
            subscription_id: "feed-1".into(),
            after: None,
        });
        assert!(reject_request(&request, "owner-a").is_err());
        request.subscribe = None;
        request.after = Some(snapshot().cursor);
        assert!(reject_request(&request, "owner-a").is_err());
        request.repository = None;
        request.after = None;
        request.expected_owner = Some("owner-a".into());
        assert!(reject_request(&request, "owner-a").is_err());
    });
}
#[test]
fn malformed_oversized_and_legacy_framing_close_without_dispatch_or_output() {
    run(async {
        for input in [
            b"{\"protocol\":7}\n".to_vec(),
            vec![0, 0, 255, 253],
            vec![0, 0],
            vec![0, 0, 0, 3],
            vec![0, 0, 0, 3, b'x'],
            vec![0, 0, 0, 1, b'x'],
            wire(&serde_json::json!({})),
        ] {
            let stream = Stream::new(input);
            let trace = stream.0.clone();
            assert!(
                serve_stream(
                    stream,
                    "owner-a",
                    Arc::default(),
                    &CancellationToken::new(),
                    async |_| panic!("invalid request dispatched")
                )
                .await
                .is_err()
            );
            assert!(trace.output().is_empty());
            assert_eq!(trace.0.lock().unwrap().drops, 1);
        }
        assert!(encode_payload(&"x".repeat(65_532)).is_err());
    });
}
#[test]
fn client_preserves_versions_cursor_chunked_reply_and_incompatibility_without_retry() {
    run(async {
        let first = snapshot();
        let mut scenario = ClientScenario::new(wire(&Response::Snapshot {
            snapshot: first.clone(),
        }));
        scenario.stream.as_mut().unwrap().0.0.lock().unwrap().chunk = Some(7);
        let client = SupervisorClient::new("/private/state".into());
        assert_eq!(
            client
                .snapshot_with(
                    Some(first.cursor.clone()),
                    PROTOCOL_VERSION,
                    env!("CARGO_PKG_VERSION"),
                    &mut scenario
                )
                .await
                .unwrap()
                .cursor,
            first.cursor
        );
        let sent = frames::<Request>(&scenario.trace.output());
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].protocol, PROTOCOL_VERSION);
        assert_eq!(sent[0].application_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(sent[0].after, Some(first.cursor));
        assert!(sent[0].expected_owner.is_none());
        assert_eq!(scenario.calls, ["state", "socket", "connect"]);
        let mut scenario = ClientScenario::new(wire(&Response::Incompatible {
            message: "Install matching desktop and supervisor builds".into(),
        }));
        assert!(
            scenario
                .snapshot()
                .await
                .unwrap_err()
                .to_string()
                .contains("Install matching")
        );
        assert_eq!(scenario.calls.len(), 3);
    });
}
#[test]
fn client_private_access_and_connect_failure_precede_io_and_never_retry() {
    run(async {
        for failed in 0..3 {
            let mut scenario = ClientScenario::new(vec![]);
            if failed < 2 {
                scenario.checks[failed] = Err(io::ErrorKind::PermissionDenied.into());
            } else {
                scenario.connect_error = Some(io::ErrorKind::ConnectionRefused);
            }
            assert!(scenario.snapshot().await.is_err());
            assert_eq!(scenario.calls.len(), failed + 1);
            assert!(scenario.trace.output().is_empty());
            assert_eq!(scenario.trace.0.lock().unwrap().reads, 0);
        }
    });
}
#[test]
fn targeted_exchange_reports_delivery_at_physical_boundary_without_replay() {
    run(async {
        let request = Request {
            provider: None,
            expected_owner: Some("owner-a".into()),
            repository: Some(RepositoryRequest::List {
                after: None,
                refresh: false,
            }),
            ..envelope()
        };
        for fault in 0..5 {
            let mut scenario = ClientScenario::new(vec![]);
            match fault {
                0 => scenario.connect_error = Some(io::ErrorKind::ConnectionRefused),
                1 => scenario.checks[0] = Err(io::ErrorKind::PermissionDenied.into()),
                2 => {
                    scenario
                        .stream
                        .as_mut()
                        .unwrap()
                        .0
                        .0
                        .lock()
                        .unwrap()
                        .write_after = Some(8)
                }
                3 => {
                    scenario
                        .stream
                        .as_mut()
                        .unwrap()
                        .0
                        .0
                        .lock()
                        .unwrap()
                        .read_error = true
                }
                _ => {}
            }
            let Err(RequestFailure::Transport { error }) =
                exchange_target(Path::new("/private/state"), &request, &mut scenario).await
            else {
                panic!()
            };
            assert_eq!(
                error.delivery,
                if fault < 2 {
                    Delivery::NotSent
                } else {
                    Delivery::Unknown
                }
            );
            assert_eq!(
                error.code,
                if fault < 2 {
                    TransportCode::NotConnected
                } else {
                    TransportCode::ReplyUnavailable
                }
            );
            if fault >= 2 {
                assert_eq!(scenario.trace.0.lock().unwrap().drops, 1);
                assert!(!scenario.trace.output().is_empty());
            }
            assert!(scenario.calls.len() <= 3);
        }
    });
}
#[test]
fn client_bad_lost_or_stalled_replies_never_reconnect_or_repeat() {
    run(async {
        for input in [
            vec![],
            vec![0, 0, 0, 5, b'x'],
            vec![0, 0, 0, 1, b'x'],
            vec![0, 0, 255, 253],
            wire(&Response::Repository {
                owner: "owner-a".into(),
                result: Ok(RepositoryReply::Action { registration: None }),
            }),
        ] {
            let mut scenario = ClientScenario::new(input);
            assert!(scenario.snapshot().await.is_err());
            assert_eq!(scenario.calls.len(), 3);
            assert_eq!(frames::<Request>(&scenario.trace.output()).len(), 1);
            assert_eq!(scenario.trace.0.lock().unwrap().drops, 1);
        }
        let mut scenario = ClientScenario::new(vec![]);
        scenario.stream.as_mut().unwrap().0.0.lock().unwrap().eof = false;
        assert_eq!(
            scenario.snapshot().await.unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        assert_eq!(scenario.calls.len(), 3);
    });
}
#[test]
fn admitted_waiter_drop_retains_capacity_and_allows_status_and_control() {
    run(async {
        let connection =
            SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        initialize(&connection);
        let (start, started) = tokio::sync::oneshot::channel();
        let (finish, release) = tokio::sync::oneshot::channel();
        let admitted = Admitted::new(
            SupervisorConnection::admit(&connection.observations).unwrap(),
            connection.tasks.clone(),
            async move {
                start.send(()).unwrap();
                release.await.unwrap();
                Err::<(), _>(unavailable("lost reply"))
            },
        );
        let waiter = admitted.run();
        started.await.unwrap();
        drop(waiter);
        let second = connection
            .prepare_repository_request(
                "owner-a".into(),
                RepositoryRequest::List {
                    after: None,
                    refresh: false,
                },
            )
            .unwrap();
        assert!(
            connection
                .prepare_repository_request(
                    "owner-a".into(),
                    RepositoryRequest::List {
                        after: None,
                        refresh: false
                    }
                )
                .is_err()
        );
        assert!(connection.prepare_snapshot().is_ok());
        assert!(connection.intents.try_acquire().is_some());
        finish.send(()).unwrap();
        connection.tasks.close_and_wait().await.unwrap();
        assert!(connection.observations.try_acquire().is_some());
        drop(second);
    });
}
#[test]
fn admitted_error_unwind_unstarted_drop_and_server_ceiling_release_capacity() {
    run(async {
        let tasks = Arc::new(Tasks::default());
        let slots = ConnectionSlots::new(1);
        let failed = Admitted::new(slots.try_acquire().unwrap(), tasks.clone(), async {
            Err::<(), _>(unavailable("failure"))
        });
        assert!(failed.run().await.is_err());
        drop(Admitted::new(
            slots.try_acquire().unwrap(),
            tasks.clone(),
            async { Ok(()) },
        ));
        assert!(slots.try_acquire().is_some());
        let panic = Admitted::new(slots.try_acquire().unwrap(), tasks.clone(), async {
            panic!("controlled worker panic");
            #[expect(unreachable_code, reason = "sets controlled panic future output")]
            Ok::<(), RequestFailure>(())
        });
        assert!(panic.run().await.is_err());
        assert!(slots.try_acquire().is_some());
        assert!(tasks.close_and_wait().await.is_err());
        let slots = ConnectionSlots::new(8);
        let permits: Vec<_> = (0..8).map(|_| slots.try_acquire().unwrap()).collect();
        assert!(slots.try_acquire().is_none());
        drop(permits);
        assert!(slots.try_acquire().is_some());
    });
}
#[test]
fn only_one_discovery_is_admitted_and_no_state_lock_spans_effects() {
    run(async {
        struct Inspect<'a> {
            connection: &'a SupervisorConnection,
            effects: DiscoveryScenario,
        }
        impl Discovery for Inspect<'_> {
            async fn prepare(&mut self, path: &Path) -> io::Result<()> {
                assert!(self.connection.observed.try_lock().is_ok());
                let Err(RequestFailure::Transport { error }) = self.connection.prepare_snapshot()
                else {
                    panic!("concurrent discovery admitted")
                };
                assert_eq!(error.code, TransportCode::Busy);
                self.effects.prepare(path).await
            }
            async fn snapshot(
                &mut self,
                state: &Path,
                after: Option<SnapshotCursor>,
            ) -> io::Result<SupervisorSnapshot> {
                assert!(self.connection.observed.try_lock().is_ok());
                self.effects.snapshot(state, after).await
            }
            async fn owner_present(&mut self, state: &Path) -> io::Result<bool> {
                self.effects.owner_present(state).await
            }
            async fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()> {
                self.effects.start(executable, state).await
            }
            fn elapsed(&self) -> Duration {
                self.effects.elapsed()
            }
            async fn wait(&mut self, duration: Duration) {
                self.effects.wait(duration).await;
            }
        }
        let connection =
            SupervisorConnection::new("/private/state".into(), "/app/supervisor".into());
        let _status = SupervisorConnection::admit(&connection.status).unwrap();
        let mut effects = Inspect {
            connection: &connection,
            effects: DiscoveryScenario::new(vec![
                failure(io::ErrorKind::NotFound),
                Ok(snapshot()),
                failure(io::ErrorKind::UnexpectedEof),
                Ok(snapshot()),
            ]),
        };
        assert!(
            connection
                .snapshot_admitted_with(&mut effects)
                .await
                .is_err()
        );
        connection
            .snapshot_admitted_with(&mut effects)
            .await
            .unwrap();
        assert_eq!(
            effects
                .effects
                .calls
                .iter()
                .filter(|&&c| c == "start")
                .count(),
            1
        );
        assert_eq!(
            effects
                .effects
                .calls
                .iter()
                .filter(|&&c| c == "prepare")
                .count(),
            1
        );
    });
}
#[test]
fn blocking_workers_keep_permits_after_waiter_drop_and_cleanup_awaits_completion() {
    run(async {
        let tasks = Arc::new(Tasks::default());
        let slots = ConnectionSlots::new(1);
        let (entered, started) = tokio::sync::oneshot::channel();
        let (finish, release) = std::sync::mpsc::channel();
        let waiter = tasks.blocking(&slots, move || {
            entered.send(()).unwrap();
            release.recv().unwrap();
            Ok(42)
        });
        let mut waiter = Box::pin(waiter);
        assert!(
            std::future::poll_fn(|cx| std::task::Poll::Ready(
                waiter.as_mut().poll(cx).is_pending()
            ))
            .await
        );
        started.await.unwrap();
        drop(waiter);
        assert_eq!(
            tasks.blocking(&slots, || Ok(())).await.unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        {
            let cleanup = tasks.close_and_wait();
            tokio::pin!(cleanup);
            assert!(
                std::future::poll_fn(|cx| std::task::Poll::Ready(
                    cleanup.as_mut().poll(cx).is_pending()
                ))
                .await
            );
        }
        assert!(slots.try_acquire().is_none());
        finish.send(()).unwrap();
        tasks.close_and_wait().await.unwrap();
        assert!(slots.try_acquire().is_some());
    });
}
#[test]
fn server_routes_workspace_and_repository_errors_without_fabricating_empty_records() {
    run(async {
        let core = Core {
            owner: "owner-a".into(),
            pid: 42,
            workspaces: None,
            providers: None,
            journal: Arc::default(),
        };
        for workspace in [false, true] {
            let mut request = envelope();
            request.expected_owner = Some("owner-a".into());
            if workspace {
                request.workspace = Some(crate::workspaces::WorkspaceRequest::Changes {
                    workspace_id: crate::workspaces::WorkspaceId(uuid::Uuid::from_u128(7)),
                    refresh: true,
                    comparison_id: None,
                    after: None,
                });
            } else {
                request.repository = Some(RepositoryRequest::List {
                    after: None,
                    refresh: false,
                });
            }
            let stream = Stream::new(wire(&request));
            let trace = stream.0.clone();
            serve_stream(
                stream,
                &core.owner,
                core.journal.clone(),
                &CancellationToken::new(),
                async |request| Ok(route_request(&core, request)),
            )
            .await
            .unwrap();
            match &frames::<Response>(&trace.output())[0] {
                Response::Repository {
                    owner,
                    result: Err(error),
                }
                | Response::Workspace {
                    owner,
                    result: Err(error),
                } => {
                    assert_eq!(owner, "owner-a");
                    assert_eq!(error.code, ErrorCode::Storage);
                    assert!(error.message.contains("execution owner"));
                }
                _ => panic!("typed storage failure required"),
            }
        }
    });
}
#[test]
fn cancellation_closes_pending_request_and_server_handlers_without_effects() {
    run(async {
        let stream = Stream::pending();
        let trace = stream.0.clone();
        let cancel = CancellationToken::new();
        let future = serve_stream(stream, "owner-a", Arc::default(), &cancel, async |_| {
            panic!("cancelled request dispatched")
        });
        let mut future = Box::pin(future);
        assert!(
            std::future::poll_fn(|cx| std::task::Poll::Ready(
                future.as_mut().poll(cx).is_pending()
            ))
            .await
        );
        cancel.cancel();
        assert_eq!(future.await.unwrap_err().kind(), io::ErrorKind::Interrupted);
        assert_eq!(trace.0.lock().unwrap().drops, 1);
        let request = Request {
            provider: None,
            expected_owner: Some("owner-a".into()),
            repository: Some(RepositoryRequest::List {
                after: None,
                refresh: false,
            }),
            ..envelope()
        };
        let mut effects = ClientScenario::new(vec![]);
        effects.stream.as_mut().unwrap().0.0.lock().unwrap().eof = false;
        let cancel = effects.cancel.clone();
        let mut exchange = Box::pin(exchange_target(
            Path::new("/private/state"),
            &request,
            &mut effects,
        ));
        assert!(
            std::future::poll_fn(|cx| std::task::Poll::Ready(
                exchange.as_mut().poll(cx).is_pending()
            ))
            .await
        );
        cancel.cancel();
        let Err(RequestFailure::Transport { error }) = exchange.await else {
            panic!()
        };
        assert_eq!(error.delivery, Delivery::Unknown);
        assert_eq!(effects.trace.0.lock().unwrap().drops, 1);
    });
}
#[test]
fn oversized_request_rejects_before_private_checks_or_socket_access() {
    run(async {
        let mut effects = ClientScenario::new(vec![]);
        assert!(
            SupervisorClient::new("/private/state".into())
                .snapshot_with(
                    Some(SnapshotCursor {
                        owner: "x".repeat(65_536),
                        revision: 1
                    }),
                    8,
                    env!("CARGO_PKG_VERSION"),
                    &mut effects
                )
                .await
                .is_err()
        );
        assert!(effects.calls.is_empty());
        assert!(effects.trace.output().is_empty());
    });
}
#[test]
fn large_chunked_reply_survives_eof_after_payload_and_preserves_buffered_next_frame() {
    run(async {
        let expected = "x".repeat(45_000);
        let mut bytes = wire(&expected);
        bytes.extend(wire(&42));
        let stream = Stream::new(bytes);
        stream.0.0.lock().unwrap().chunk = Some(1000);
        let mut exchange = FramedExchange::new(stream);
        let cancel = CancellationToken::new();
        assert_eq!(
            read_json::<String, _>(&mut exchange, Instant::now() + IO_TIMEOUT, &cancel)
                .await
                .unwrap(),
            expected
        );
        assert_eq!(
            read_json::<u32, _>(&mut exchange, Instant::now() + IO_TIMEOUT, &cancel)
                .await
                .unwrap(),
            42
        );
    });
}
#[test]
fn two_running_observations_leave_status_and_intent_effects_responsive() {
    run(async {
        let connection = SupervisorConnection::new("/state".into(), "/supervisor".into());
        initialize(&connection);
        let mut finishes = vec![];
        let mut waiters = vec![];
        for _ in 0..2 {
            let (start, started) = tokio::sync::oneshot::channel();
            let (finish, release) = tokio::sync::oneshot::channel();
            let waiter = Admitted::new(
                SupervisorConnection::admit(&connection.observations).unwrap(),
                connection.tasks.clone(),
                async move {
                    start.send(()).unwrap();
                    release.await.unwrap();
                    Ok(())
                },
            )
            .run();
            started.await.unwrap();
            finishes.push(finish);
            waiters.push(waiter);
        }
        assert!(connection.observations.try_acquire().is_none());
        for lane in [&connection.status, &connection.intents] {
            let observed = connection.observed.clone();
            Admitted::new(
                SupervisorConnection::admit(lane).unwrap(),
                connection.tasks.clone(),
                async move {
                    assert!(observed.try_lock().is_ok());
                    Ok(())
                },
            )
            .run()
            .await
            .unwrap();
        }
        assert!(connection.observations.try_acquire().is_none());
        for finish in finishes {
            finish.send(()).unwrap();
        }
        for waiter in waiters {
            waiter.await.unwrap();
        }
        connection.tasks.close_and_wait().await.unwrap();
    });
}
#[test]
fn discovery_cancellation_wakes_startup_delay_without_waiting() {
    run(async {
        let mut discovery = LocalDiscovery::new(SupervisorClient::new("/unused-state".into()));
        let cancel = discovery.client.tasks.cancel.clone();
        let mut wait = Box::pin(discovery.wait(Duration::from_millis(25)));
        assert!(
            std::future::poll_fn(|cx| std::task::Poll::Ready(wait.as_mut().poll(cx).is_pending()))
                .await
        );
        cancel.cancel();
        assert!(
            std::future::poll_fn(|cx| std::task::Poll::Ready(wait.as_mut().poll(cx).is_ready()))
                .await,
            "cancellation must wake discovery without advancing controlled time"
        );
    });
}
