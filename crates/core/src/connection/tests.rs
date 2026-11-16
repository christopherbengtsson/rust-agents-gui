use super::*;
use crate::Core;
pub(super) fn run<T>(future: impl Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .start_paused(true)
        .build()
        .unwrap()
        .block_on(future)
}
#[test]
fn application_corpus_matches_pure_rust_serialization() {
    run(async {
        use crate::workspaces::*;
        use serde_json::{Value, json};
        let corpus: Value = serde_json::from_str(include_str!(
            "../../../../contracts/observations-native.json"
        ))
        .unwrap();
        let incarnation = "incarnation-a";
        let repository_id = RepositoryId(uuid::Uuid::from_u128(1));
        let action_id = ActionId(uuid::Uuid::from_u128(2));
        let workspace_id = WorkspaceId(uuid::Uuid::from_u128(3));
        let comparison_id = ComparisonId(uuid::Uuid::from_u128(4));
        let file_id = FileId(uuid::Uuid::from_u128(5));
        let core = Arc::new(Core {
            keepalive: Mutex::new(None),
            lifetime: Arc::default(),

            incarnation: incarnation.into(),
            pid: 41,
            workspaces: None,
            storage: None,
            processes: None,
            providers: None,
            journal: Arc::new(crate::observations::Journal::default()),
        });
        assert_eq!(
            corpus["snapshot"],
            serde_json::to_value(core.snapshot(None)).unwrap()
        );
        assert_eq!(corpus["desktop_pid"], 42);
        let repository_case =
            |request: RepositoryRequest,
             result: std::result::Result<RepositoryReply, RepositoryError>| {
                json!({
                    "request": request, "reply": OwnedReply { incarnation: incarnation.into(), result }
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
                "request": request, "reply": OwnedReply { incarnation: incarnation.into(), result: Ok(reply) }
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
                TransportCode::IncarnationChanged,
                Delivery::Rejected,
                "Owner changed",
            ),
            (
                TransportCode::ReplyUnavailable,
                Delivery::Unknown,
                "Reply unavailable",
            ),
        ] {
            failures.push(RequestFailure::transport(code, delivery, message));
        }
        assert_eq!(corpus["failures"], json!(failures));
    });
}
