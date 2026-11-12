use std::os::unix::fs::PermissionsExt;
use std::{
    path::Path,
    process::{Child, Command, Stdio},
    time::Duration,
};
use workspace_core::connection::SupervisorClient;

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
    let state = tempfile::tempdir_in("/tmp").unwrap();
    std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let clients: Vec<_> = (0..4)
        .map(|_| {
            let path = state.path().to_owned();
            std::thread::spawn(move || {
                SupervisorClient::connect_or_start(path, executable()).unwrap()
            })
        })
        .collect();
    let clients: Vec<_> = clients.into_iter().map(|t| t.join().unwrap()).collect();
    let first = clients[0].snapshot(None).unwrap();
    let _owner = Owner(first.status.supervisor_pid);
    for client in &clients {
        assert_eq!(client.snapshot(None).unwrap().cursor, first.cursor);
    }
    let mut competitor = competing(state.path());
    assert!(!competitor.wait().unwrap().success());
    drop(clients);
    std::thread::sleep(Duration::from_millis(20));
    let reopened =
        SupervisorClient::connect_or_start(state.path().to_owned(), executable()).unwrap();
    let caught_up = reopened.snapshot(Some(first.cursor.clone())).unwrap();
    assert_eq!(caught_up.status.supervisor_pid, first.status.supervisor_pid);
    assert_eq!(caught_up.cursor, first.cursor);
    assert!(!caught_up.reset);
}

#[test]
fn private_endpoint_rejects_incompatible_versions_and_unbounded_frames() {
    use std::{
        io::{Read, Write},
        os::unix::{fs::MetadataExt, net::UnixStream},
    };
    let state = tempfile::tempdir_in("/tmp").unwrap();
    std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let client = SupervisorClient::connect_or_start(state.path().to_owned(), executable()).unwrap();
    let first = client.snapshot(None).unwrap();
    let _owner = Owner(first.status.supervisor_pid);
    let socket = state.path().join("supervisor.sock");
    assert_eq!(
        std::fs::metadata(state.path()).unwrap().mode() & 0o777,
        0o700
    );
    assert_eq!(std::fs::metadata(&socket).unwrap().mode() & 0o777, 0o600);
    for (protocol, version) in [(999, "0.1.0"), (1, "99.0.0")] {
        let error = client
            .snapshot_with_version(None, protocol, version)
            .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Install matching desktop and supervisor builds")
        );
    }
    let mut peer = UnixStream::connect(&socket).unwrap();
    peer.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
    let _ = peer.write_all(&[b'x'; 4097]);
    assert!(matches!(peer.read(&mut [0]), Ok(0) | Err(_)));
    assert_eq!(
        client.status().unwrap().supervisor_pid,
        first.status.supervisor_pid
    );
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o666)).unwrap();
    assert_eq!(
        client.status().unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).unwrap();
}

#[test]
fn public_or_symlinked_state_is_rejected_without_starting_an_owner() {
    let state = tempfile::tempdir_in("/tmp").unwrap();
    std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    let result = SupervisorClient::connect_or_start(state.path().to_owned(), executable());
    assert!(matches!(result, Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied));
    std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let link = state.path().join("link");
    std::os::unix::fs::symlink(state.path(), &link).unwrap();
    assert!(SupervisorClient::connect_or_start(link, executable()).is_err());
    assert!(!state.path().join("supervisor.sock").exists());
}

#[test]
fn killed_desktop_process_leaves_owner_and_reopening_catches_up() {
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
        .unwrap();
    desktop.kill().unwrap();
    desktop.wait().unwrap();
    let reopened =
        SupervisorClient::connect_or_start(state.path().to_owned(), executable()).unwrap();
    let after = reopened.snapshot(Some(before.cursor.clone())).unwrap();
    assert_eq!(after.status.supervisor_pid, owner.0);
    assert_eq!(after.cursor, before.cursor);
    assert!(!after.reset);
}

#[test]
#[ignore = "subprocess helper, launched by killed_desktop_process test"]
fn desktop_process() {
    use std::io::Write;
    let client = SupervisorClient::connect_or_start(
        std::env::var_os("TEST_DESKTOP_STATE").unwrap().into(),
        executable(),
    )
    .unwrap();
    println!("OWNER={}", client.status().unwrap().supervisor_pid);
    std::io::stdout().flush().unwrap();
    loop {
        std::thread::park();
    }
}

#[test]
fn lost_owner_does_not_restart_on_refresh_but_explicit_reopen_recovers_status() {
    use workspace_core::connection::SupervisorConnection;
    let state = tempfile::tempdir_in("/tmp").unwrap();
    std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut desktop = SupervisorConnection::new(state.path().to_owned(), executable().to_owned());
    let before = desktop.snapshot().unwrap();
    let owner = Owner(before.status.supervisor_pid);
    drop(owner);
    let deadline = std::time::Instant::now() + Duration::from_secs(5);
    while SupervisorClient::new(state.path().to_owned())
        .status()
        .is_ok()
    {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(desktop.snapshot().is_err());
    let reopened =
        SupervisorClient::connect_or_start(state.path().to_owned(), executable()).unwrap();
    let after = reopened.snapshot(Some(before.cursor.clone())).unwrap();
    let _owner = Owner(after.status.supervisor_pid);
    assert_ne!(after.cursor.owner, before.cursor.owner);
    assert!(after.reset);
}
