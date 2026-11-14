use std::os::unix::fs::PermissionsExt;
use std::{
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
use workspace_core::connection::{PROTOCOL_VERSION, SupervisorClient};

struct Owner(u32);
impl Drop for Owner {
    fn drop(&mut self) {
        // The test obtained this PID from a process it just launched in private state.
        let _ = Command::new("kill")
            .args(["-TERM", &self.0.to_string()])
            .status();
    }
}
fn executable() -> &'static Path {
    Path::new(env!("CARGO_BIN_EXE_workspace-supervisor"))
}
fn competing(state: &Path) -> Child {
    Command::new(executable())
        .arg(state)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap()
}
#[test]
fn concurrent_desktops_discover_one_owner_and_reconnect_to_its_snapshot() {
    run(async {
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let clients: Vec<_> = (0..4)
            .map(|_| {
                let path = state.path().to_owned();
                std::thread::spawn(move || {
                    run(async move {
                        SupervisorClient::connect_or_start(path, executable())
                            .await
                            .unwrap()
                    })
                })
            })
            .collect();
        let clients: Vec<_> = clients.into_iter().map(|t| t.join().unwrap()).collect();
        let first = clients[0].snapshot(None).await.unwrap();
        let _owner = Owner(first.status.supervisor_pid);
        for client in &clients {
            assert_eq!(client.snapshot(None).await.unwrap().cursor, first.cursor);
        }
        let mut competitor = competing(state.path());
        assert!(!competitor.wait().unwrap().success());
        drop(clients);
        std::thread::sleep(Duration::from_millis(20));
        let reopened = SupervisorClient::connect_or_start(state.path().to_owned(), executable())
            .await
            .unwrap();
        let caught_up = reopened.snapshot(Some(first.cursor.clone())).await.unwrap();
        assert_eq!(caught_up.status.supervisor_pid, first.status.supervisor_pid);
        assert_eq!(caught_up.cursor, first.cursor);
        assert!(!caught_up.reset);
    });
}

#[test]
fn private_endpoint_rejects_incompatible_versions_and_unbounded_frames() {
    run(async {
        use std::{
            io::{Read, Write},
            os::unix::{fs::MetadataExt, net::UnixStream},
        };
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let client = SupervisorClient::connect_or_start(state.path().to_owned(), executable())
            .await
            .unwrap();
        let first = client.snapshot(None).await.unwrap();
        let _owner = Owner(first.status.supervisor_pid);
        let socket = state.path().join("supervisor.sock");
        assert_eq!(
            std::fs::metadata(state.path()).unwrap().mode() & 0o777,
            0o700
        );
        assert_eq!(std::fs::metadata(&socket).unwrap().mode() & 0o777, 0o600);
        for (protocol, version) in [(999, "0.1.0"), (PROTOCOL_VERSION, "99.0.0")] {
            let error = client
                .snapshot_with_version(None, protocol, version)
                .await
                .unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("Install matching desktop and supervisor builds")
            );
        }
        let mut peer = UnixStream::connect(&socket).unwrap();
        peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        let _ = peer.write_all(&[0, 0, 255, 253]);
        assert!(matches!(peer.read(&mut [0]), Ok(0) | Err(_)));
        assert_eq!(
            client.status().await.unwrap().supervisor_pid,
            first.status.supervisor_pid
        );
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o666)).unwrap();
        assert_eq!(
            client.status().await.unwrap_err().kind(),
            std::io::ErrorKind::PermissionDenied
        );
        std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
    });
}

#[test]
fn public_or_symlinked_state_is_rejected_without_starting_an_owner() {
    run(async {
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
        let result =
            SupervisorClient::connect_or_start(state.path().to_owned(), executable()).await;
        assert!(matches!(result, Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied));
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let link = state.path().join("link");
        std::os::unix::fs::symlink(state.path(), &link).unwrap();
        assert!(
            SupervisorClient::connect_or_start(link, executable())
                .await
                .is_err()
        );
        assert!(!state.path().join("supervisor.sock").exists());
    });
}

#[test]
fn killed_desktop_process_leaves_owner_and_reopening_catches_up() {
    run(async {
        use std::io::{BufRead, BufReader};
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut desktop = Command::new(std::env::current_exe().unwrap())
            .args(["--ignored", "--exact", "desktop_process", "--nocapture"])
            .env("TEST_DESKTOP_STATE", state.path())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = desktop.stdout.take().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                if let Some(pid) = line.strip_prefix("OWNER=") {
                    let _ = tx.send(pid.parse::<u32>().unwrap());
                    break;
                }
            }
        });
        let owner = Owner(rx.recv_timeout(Duration::from_secs(8)).unwrap());
        let before = SupervisorClient::new(state.path().to_owned())
            .snapshot(None)
            .await
            .unwrap();
        desktop.kill().unwrap();
        desktop.wait().unwrap();
        let reopened = SupervisorClient::connect_or_start(state.path().to_owned(), executable())
            .await
            .unwrap();
        let after = reopened
            .snapshot(Some(before.cursor.clone()))
            .await
            .unwrap();
        assert_eq!(after.status.supervisor_pid, owner.0);
        assert_eq!(after.cursor, before.cursor);
        assert!(!after.reset);
    });
}

#[test]
#[ignore = "subprocess helper, launched by killed_desktop_process test"]
fn desktop_process() {
    run(async {
        use std::io::Write;
        let client = SupervisorClient::connect_or_start(
            std::env::var_os("TEST_DESKTOP_STATE").unwrap().into(),
            executable(),
        )
        .await
        .unwrap();
        println!("OWNER={}", client.status().await.unwrap().supervisor_pid);
        std::io::stdout().flush().unwrap();
        loop {
            std::thread::park();
        }
    });
}

#[test]
fn lost_owner_does_not_restart_on_refresh_but_explicit_reopen_recovers_status() {
    run(async {
        use workspace_core::connection::SupervisorConnection;
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let desktop = SupervisorConnection::new(state.path().to_owned(), executable().to_owned());
        let before = desktop.snapshot().await.unwrap();
        let owner = Owner(before.status.supervisor_pid);
        drop(owner);
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while SupervisorClient::new(state.path().to_owned())
            .status()
            .await
            .is_ok()
        {
            assert!(std::time::Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        assert!(desktop.snapshot().await.is_err());
        let reopened = SupervisorClient::connect_or_start(state.path().to_owned(), executable())
            .await
            .unwrap();
        let after = reopened
            .snapshot(Some(before.cursor.clone()))
            .await
            .unwrap();
        let _owner = Owner(after.status.supervisor_pid);
        assert_ne!(after.cursor.owner, before.cursor.owner);
        assert!(after.reset);
    });
}

fn run<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}

#[test]
fn cancelled_native_waiters_retain_lanes_and_cleanup_joins_actual_socket_work() {
    use workspace_core::connection::{
        Delivery, RequestFailure, SupervisorConnection, TransportCode,
    };
    use workspace_core::workspaces::RepositoryRequest;
    run(async {
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let connection =
            SupervisorConnection::new(state.path().to_owned(), executable().to_owned());
        let snapshot = connection.snapshot().await.unwrap();
        let owner = Owner(snapshot.status.supervisor_pid);
        // This is the exact disposable owner launched by this check. Pausing it
        // supplies a real stopped peer without altering provider/user processes.
        assert!(
            Command::new("kill")
                .args(["-STOP", &owner.0.to_string()])
                .status()
                .unwrap()
                .success()
        );
        struct Resume(u32);
        impl Drop for Resume {
            fn drop(&mut self) {
                let _ = Command::new("kill")
                    .args(["-CONT", &self.0.to_string()])
                    .status();
            }
        }
        let _resume = Resume(owner.0);
        let read = || {
            connection.prepare_repository_request(
                snapshot.cursor.owner.clone(),
                RepositoryRequest::List {
                    after: None,
                    refresh: false,
                },
            )
        };
        drop(read().unwrap().run());
        drop(read().unwrap().run());
        assert!(
            matches!(read(), Err(RequestFailure::Transport { error }) if error.code == TransportCode::Busy && error.delivery == Delivery::NotSent)
        );
        let intent = || {
            connection.prepare_workspace_request(
                snapshot.cursor.owner.clone(),
                workspace_core::workspaces::WorkspaceRequest::Create {
                    action_id: workspace_core::workspaces::ActionId(Default::default()),
                    repository_id: workspace_core::workspaces::RepositoryId(Default::default()),
                },
            )
        };
        drop(intent().unwrap().run());
        assert!(
            matches!(intent(), Err(RequestFailure::Transport { error }) if error.code == TransportCode::Busy)
        );
        drop(connection.prepare_snapshot().unwrap().run());
        assert!(
            matches!(connection.prepare_snapshot(), Err(RequestFailure::Transport { error }) if error.code == TransportCode::Busy)
        );
        connection
            .start_subscription(
                snapshot.cursor.owner.clone(),
                "stopped-peer".into(),
                None,
                |_| Ok(()),
            )
            .unwrap();
        assert!(
            matches!(connection.start_subscription(snapshot.cursor.owner.clone(), "blocked-replacement".into(), None, |_| Ok(())), Err(RequestFailure::Transport { error }) if error.code == TransportCode::Busy)
        );
        // Drive physical opens/writes while their result receivers are abandoned.
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(
            matches!(read(), Err(RequestFailure::Transport { error }) if error.code == TransportCode::Busy)
        );
        let start = std::time::Instant::now();
        connection.stop_observations().await.unwrap();
        assert!(start.elapsed() < Duration::from_secs(2));
        assert!(read().unwrap().run().await.is_err());
        // Cleanup completion precedes admission of work on a fresh connection.
        drop(_resume);
        assert_eq!(
            SupervisorClient::new(state.path().to_owned())
                .status()
                .await
                .unwrap()
                .supervisor_pid,
            owner.0
        );
    });
}

#[test]
fn actual_subscription_stop_joins_late_setup_and_replacement_then_heartbeats() {
    use workspace_core::connection::{
        FeedEvent, RequestFailure, SupervisorConnection, TransportCode,
    };
    run(async {
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let connection =
            SupervisorConnection::new(state.path().to_owned(), executable().to_owned());
        let snapshot = connection.snapshot().await.unwrap();
        let _owner = Owner(snapshot.status.supervisor_pid);
        let (send, mut events) = tokio::sync::mpsc::unbounded_channel();
        let deliver = move |event| send.send(event).map_err(std::io::Error::other);
        connection
            .start_subscription(
                snapshot.cursor.owner.clone(),
                "late-setup".into(),
                None,
                deliver.clone(),
            )
            .unwrap();
        assert!(
            matches!(connection.start_subscription(snapshot.cursor.owner.clone(), "competitor".into(), None, deliver.clone()), Err(RequestFailure::Transport { error }) if error.code == TransportCode::Busy)
        );
        // Stop before the runtime polls the registered setup worker.
        connection.stop_subscription("late-setup").await.unwrap();
        connection.stop_subscription("late-setup").await.unwrap();
        while events.try_recv().is_ok() {}
        connection
            .start_subscription(
                snapshot.cursor.owner.clone(),
                "replacement".into(),
                None,
                deliver,
            )
            .unwrap();
        let FeedEvent::Batch { batch } =
            tokio::time::timeout(Duration::from_secs(3), events.recv())
                .await
                .unwrap()
                .unwrap()
        else {
            panic!("expected baseline")
        };
        assert_eq!(batch.acknowledgement.subscription_id, "replacement");
        let accepted = batch.acknowledgement.cursor.clone();
        connection
            .acknowledge_subscription(batch.acknowledgement)
            .unwrap();
        let FeedEvent::Batch { batch } =
            tokio::time::timeout(Duration::from_secs(8), events.recv())
                .await
                .unwrap()
                .unwrap()
        else {
            panic!("expected heartbeat")
        };
        assert_eq!(batch.acknowledgement.cursor, accepted);
        assert!(matches!(
            batch.sample,
            workspace_core::observations::Sample::Heartbeat { .. }
        ));
        connection.stop_subscription("replacement").await.unwrap();
        while events.try_recv().is_ok() {}
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert!(events.try_recv().is_err(), "delivery after joined stop");
        connection.stop_observations().await.unwrap();
    });
}

#[test]
fn replacement_owner_rejects_a_retained_desktop_target_before_status_refresh() {
    use workspace_core::connection::{
        Delivery, RequestFailure, SupervisorConnection, TransportCode,
    };
    use workspace_core::workspaces::{ActionId, RepositoryId, WorkspaceRequest};
    run(async {
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let desktop = SupervisorConnection::new(state.path().to_owned(), executable().to_owned());
        let before = desktop.snapshot().await.unwrap();
        drop(Owner(before.status.supervisor_pid));
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while SupervisorClient::new(state.path().to_owned())
            .status()
            .await
            .is_ok()
        {
            assert!(std::time::Instant::now() < deadline);
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        // A separate explicit opener starts the replacement; the established
        // desktop has neither observed nor authorized adopting its identity.
        let replacement = SupervisorClient::connect_or_start(state.path().to_owned(), executable())
            .await
            .unwrap();
        let after = replacement.snapshot(None).await.unwrap();
        let _owner = Owner(after.status.supervisor_pid);
        assert_ne!(after.cursor.owner, before.cursor.owner);
        let error = desktop
            .prepare_workspace_request(
                before.cursor.owner,
                WorkspaceRequest::Create {
                    action_id: ActionId(Default::default()),
                    repository_id: RepositoryId(Default::default()),
                },
            )
            .unwrap()
            .run()
            .await
            .unwrap_err();
        assert!(
            matches!(error, RequestFailure::Transport { error } if error.code == TransportCode::OwnerChanged && error.delivery == Delivery::Rejected)
        );
        assert_eq!(
            desktop.snapshot().await.unwrap().cursor.owner,
            after.cursor.owner
        );
        desktop.stop_observations().await.unwrap();
    });
}
