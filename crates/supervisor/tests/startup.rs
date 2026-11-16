use std::{
    io::{BufRead, BufReader},
    os::unix::fs::PermissionsExt,
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::Duration,
};
use workspace_core::{CoreState, connection::SupervisorClient};

struct Supervisor(Child);

impl Drop for Supervisor {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
fn desktop_client_receives_status_from_a_separate_supervisor_process() {
    run(async {
        // /tmp keeps the socket below macOS's Unix socket path length limit.
        let state = tempfile::tempdir_in("/tmp").unwrap();
        std::fs::set_permissions(state.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        let mut process = Supervisor(
            Command::new(env!("CARGO_BIN_EXE_workspace-supervisor"))
                .arg(state.path())
                .stdout(Stdio::piped())
                .spawn()
                .unwrap(),
        );
        let stdout = process.0.stdout.take().unwrap();
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            let mut ready = String::new();
            let result = BufReader::new(stdout).read_line(&mut ready);
            let _ = sender.send((result, ready));
        });
        let (read, ready) = receiver.recv_timeout(Duration::from_secs(5)).unwrap();
        read.unwrap();
        assert_eq!(ready.trim(), "ready");

        let client = SupervisorClient::new(state.path().to_owned());
        let status = client.status().await.unwrap();
        assert_eq!(status.state, CoreState::Ready);
        assert_eq!(status.native_pid, process.0.id());
        assert_ne!(status.native_pid, std::process::id());
        assert_eq!(status.application_version, "0.1.0");
        // A second observation uses the same owner, without creating work.
        assert_eq!(client.status().await.unwrap().native_pid, status.native_pid);
    });
}

fn run<T>(future: impl std::future::Future<Output = T>) -> T {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(future)
}
