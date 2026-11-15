//! Private process/stdio mechanics. No authentication or readiness policy here.
use super::protocol;
use crate::runtimes::{ProviderTarget, SetupEvidence, SetupFailure, SetupProblem};
use std::{
    ffi::OsString, io, os::unix::fs::DirBuilderExt, path::PathBuf, process::Stdio, time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    process::{Child, Command},
    time::{Instant, timeout},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub(in crate::runtimes) struct Launch {
    program: PathBuf,
    directory: PathBuf,
    environment: Vec<(OsString, OsString)>,
}
impl Launch {
    pub(in crate::runtimes) fn credential_scope(&self) -> String {
        self.environment
            .iter()
            .find(|(key, _)| key == "CODEX_HOME" || key == "AW_FIXTURE_CONTROL")
            .map(|(_, value)| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.directory.to_string_lossy().into_owned())
    }
    pub(in crate::runtimes) fn capture() -> (Self, ProviderTarget) {
        let id = uuid::Uuid::new_v4();
        let directory = std::env::temp_dir().join(format!("agent-workspace-setup-{id}"));
        let fixture = std::env::var_os("AGENT_WORKSPACE_CODEX_FIXTURE");
        let program = fixture.clone().map(PathBuf::from).unwrap_or_else(|| {
            std::env::var_os("PATH")
                .and_then(|paths| {
                    std::env::split_paths(&paths)
                        .map(|path| path.join("codex"))
                        .find(|path| path.is_file())
                })
                .unwrap_or_else(|| PathBuf::from("codex"))
        });
        // Keep the resolved path stable across a standalone-runtime symlink update.
        let program = std::fs::canonicalize(&program).unwrap_or(program);
        let home = std::env::var_os("CODEX_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".codex")
            });
        let home = if home.is_absolute() {
            home
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("/"))
                .join(home)
        };
        let home = std::fs::canonicalize(&home).unwrap_or(home);
        let mut environment: Vec<(OsString, OsString)> = if fixture.is_some() {
            // Fixtures never receive the user's runtime credentials/environment.
            vec![
                (OsString::from("PATH"), OsString::from("/usr/bin:/bin")),
                (
                    OsString::from("AW_FIXTURE_SCENARIO"),
                    std::env::var_os("AW_FIXTURE_SCENARIO").unwrap_or_default(),
                ),
                (
                    OsString::from("AW_FIXTURE_CONTROL"),
                    std::env::var_os("AW_FIXTURE_CONTROL").unwrap_or_default(),
                ),
            ]
        } else {
            std::env::vars_os().collect()
        };
        if fixture.is_none() {
            environment.retain(|(name, _)| name != "CODEX_HOME");
            environment.push((OsString::from("CODEX_HOME"), home.clone().into_os_string()));
        }
        let target = ProviderTarget {
            id,
            provider: "codex".into(),
            executable: program.to_string_lossy().into_owned(),
            codex_home: if fixture.is_some() {
                "Fixture-owned configuration".into()
            } else {
                home.to_string_lossy().into_owned()
            },
            working_directory: directory.to_string_lossy().into_owned(),
            minimum_supported_version: protocol::MINIMUM_VERSION.to_string(),
        };
        (
            Self {
                program,
                directory,
                environment,
            },
            target,
        )
    }
    fn command(&self, argument: &str) -> Command {
        let mut command = Command::new(&self.program);
        command
            .arg(argument)
            .current_dir(&self.directory)
            .env_clear()
            .envs(self.environment.iter().cloned())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        command
    }
    pub(in crate::runtimes) fn remove_neutral_directory(&self) -> io::Result<()> {
        match std::fs::remove_dir(&self.directory) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}

fn spawn(launch: &Launch, argument: &str) -> Result<Child, SetupFailure> {
    launch.command(argument).spawn().map_err(|error| {
        let missing = error.kind() == io::ErrorKind::NotFound;
        let mut failure = SetupFailure::io(argument, error);
        if missing {
            failure.problem = SetupProblem::MissingExecutable;
        }
        failure
    })
}

trait Cleanup {
    fn signal(&mut self, signal: i32) -> io::Result<()>;
    async fn quiescent(&mut self, budget: Duration) -> io::Result<bool>;
}
impl Cleanup for Child {
    fn signal(&mut self, signal: i32) -> io::Result<()> {
        let pid = self
            .id()
            .and_then(|pid| i32::try_from(pid).ok())
            .ok_or_else(|| io::Error::other("owned child identity unavailable"))?;
        // SAFETY: process_group(0) created this group, and the unreaped child
        // handle still owns its identity. Never signal from a stored/reaped PID.
        if unsafe { libc::kill(-pid, signal) } == 0 {
            Ok(())
        } else {
            let error = io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ESRCH) {
                Ok(())
            } else {
                Err(error)
            }
        }
    }
    async fn quiescent(&mut self, budget: Duration) -> io::Result<bool> {
        let pid = self
            .id()
            .and_then(|pid| i32::try_from(pid).ok())
            .ok_or_else(|| io::Error::other("owned group leader unavailable"))?;
        let deadline = Instant::now() + budget;
        loop {
            if group_quiescent(pid)? {
                return Ok(true);
            }
            if Instant::now() >= deadline {
                return Ok(false);
            }
            tokio::time::sleep_until((Instant::now() + Duration::from_millis(20)).min(deadline))
                .await;
        }
    }
}
async fn cleanup(child: &mut impl Cleanup) -> bool {
    if child
        .quiescent(Duration::from_secs(2))
        .await
        .unwrap_or(false)
    {
        return true;
    }
    if child.signal(libc::SIGTERM).is_ok()
        && child
            .quiescent(Duration::from_secs(2))
            .await
            .unwrap_or(false)
    {
        return true;
    }
    child.signal(libc::SIGKILL).is_ok()
        && child
            .quiescent(Duration::from_secs(2))
            .await
            .unwrap_or(false)
}

pub(in crate::runtimes) async fn probe(
    launch: Launch,
    cancel: &CancellationToken,
    uncertain: impl Fn(),
) -> Result<SetupEvidence, SetupFailure> {
    let directory = launch.directory.clone();
    tokio::task::spawn_blocking(move || {
        match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        crate::supervisor::check_private(&directory, true)
    })
    .await
    .map_err(|_| {
        SetupFailure::at(
            SetupProblem::StatusReadFailed,
            "Setup-directory worker failed",
        )
    })?
    .map_err(|error| SetupFailure::io("private setup directory", error))?;
    let observed_version = version(&launch, cancel, &uncertain).await?;
    if cancel.is_cancelled() {
        return Err(SetupFailure::at(
            SetupProblem::StatusReadFailed,
            "Setup cancelled during supervisor shutdown",
        ));
    }
    let mut child = spawn(&launch, "app-server")?;
    let stderr = child.stderr.take().expect("piped stderr");
    let drain = tokio::spawn(async move {
        tokio::io::copy(
            &mut tokio::io::BufReader::new(stderr),
            &mut tokio::io::sink(),
        )
        .await
    });
    let stdout = child.stdout.take().expect("piped stdout");
    let stdin = child.stdin.take().expect("piped stdin");
    let result = cancel
        .run_until_cancelled(protocol::inspect(
            stdout,
            stdin,
            Instant::now() + Duration::from_secs(15),
            &observed_version,
        ))
        .await
        .unwrap_or_else(|| {
            Err(SetupFailure::at(
                SetupProblem::StatusReadFailed,
                "Setup cancelled during supervisor shutdown",
            ))
        });
    finish(&mut child, drain, &uncertain)
        .await
        .map_err(|error| SetupFailure::io("owned process cleanup", error))?;
    result
}

pub(in crate::runtimes) async fn browser_login(
    launch: Launch,
    cancel: &CancellationToken,
    reopen: tokio::sync::mpsc::Receiver<()>,
    event: impl Fn(protocol::LoginEvent),
) -> Result<crate::runtimes::LoginOutcome, SetupFailure> {
    let directory = launch.directory.clone();
    tokio::task::spawn_blocking(move || {
        match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        crate::supervisor::check_private(&directory, true)
    })
    .await
    .map_err(|_| {
        SetupFailure::at(
            SetupProblem::StatusReadFailed,
            "Login directory worker failed",
        )
    })?
    .map_err(|error| SetupFailure::io("private login directory", error))?;
    let uncertain = || event(protocol::LoginEvent::CleanupUncertain);
    if cancel.is_cancelled() {
        return Ok(crate::runtimes::LoginOutcome::Cancelled);
    }
    version(&launch, cancel, &uncertain).await?;
    if cancel.is_cancelled() {
        return Ok(crate::runtimes::LoginOutcome::Cancelled);
    }
    let mut child = spawn(&launch, "app-server")?;
    let stderr = child.stderr.take().expect("piped stderr");
    let drain = tokio::spawn(async move {
        tokio::io::copy(
            &mut tokio::io::BufReader::new(stderr),
            &mut tokio::io::sink(),
        )
        .await
    });
    let stdout = child.stdout.take().expect("piped stdout");
    let stdin = child.stdin.take().expect("piped stdin");
    let fixture = launch
        .environment
        .iter()
        .any(|(key, _)| key == "AW_FIXTURE_CONTROL");
    let result = protocol::login(stdout, stdin, cancel, reopen, &event, |url| {
        let uncertain = &uncertain;
        async move { open_browser(&url, fixture, uncertain).await }
    })
    .await;
    finish(&mut child, drain, &uncertain)
        .await
        .map_err(|error| SetupFailure::io("login physical cleanup", error))?;
    result
}

async fn open_browser(url: &str, fixture: bool, uncertain: &impl Fn()) -> bool {
    if !protocol::browser_url(url, fixture) {
        return false;
    }
    let mut command = Command::new("/usr/bin/open");
    command
        .arg(url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    let Ok(mut child) = command.spawn() else {
        return false;
    };
    // Keep the unreaped identity until the same physical group cleanup used by
    // provider children completes. The user's browser is not an owned resource.
    let success = child
        .quiescent(Duration::from_secs(5))
        .await
        .unwrap_or(false);
    if !cleanup(&mut child).await {
        uncertain();
        while !child
            .quiescent(Duration::from_millis(100))
            .await
            .unwrap_or(false)
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    child
        .wait()
        .await
        .is_ok_and(|status| success && status.success())
}
async fn version(
    launch: &Launch,
    cancel: &CancellationToken,
    uncertain: &impl Fn(),
) -> Result<String, SetupFailure> {
    let mut child = spawn(launch, "--version")?;
    drop(child.stdin.take());
    let stderr = child.stderr.take().expect("piped stderr");
    let drain = tokio::spawn(async move {
        tokio::io::copy(
            &mut tokio::io::BufReader::new(stderr),
            &mut tokio::io::sink(),
        )
        .await
    });
    let mut stdout = child.stdout.take().expect("piped stdout").take(1025);
    let mut bytes = Vec::new();
    let result = match cancel
        .run_until_cancelled(timeout(Duration::from_secs(5), async {
            stdout
                .read_to_end(&mut bytes)
                .await
                .map_err(|error| SetupFailure::io("version stdout", error))?;
            if bytes.len() > 1024 {
                return Err(SetupFailure::at(
                    SetupProblem::IncompatibleProtocol,
                    "Version output exceeded 1024 bytes",
                ));
            }
            Ok(())
        }))
        .await
    {
        Some(Ok(result)) => result,
        _ => Err(SetupFailure::at(
            SetupProblem::StatusReadFailed,
            "Version read exceeded its five-second deadline or was cancelled",
        )),
    };
    drop(stdout);
    let status = finish(&mut child, drain, uncertain)
        .await
        .map_err(|error| SetupFailure::io("version cleanup", error))?;
    result?;
    protocol::version(&bytes, status.success()).map_err(|problem| {
        SetupFailure::at(
            problem,
            format!(
                "Version command exit {:?}; requires stable codex-cli >= {} with valid SemVer",
                status.code(),
                protocol::MINIMUM_VERSION
            ),
        )
    })
}
async fn finish(
    child: &mut Child,
    drain: tokio::task::JoinHandle<io::Result<u64>>,
    uncertain: &impl Fn(),
) -> io::Result<std::process::ExitStatus> {
    if !cleanup(child).await {
        uncertain();
        // Never reap the group leader while a live member or an unknown group
        // remains. Its unreaped PID prevents process-group identity reuse.
        while !child
            .quiescent(Duration::from_millis(100))
            .await
            .unwrap_or(false)
        {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
    let mut drain = drain;
    let drained = match timeout(Duration::from_secs(2), &mut drain).await {
        Ok(result) => result,
        Err(_) => {
            uncertain();
            drain.await
        }
    };
    // A pipe that outlives the quiescent group may belong to escaped work.
    // Keep ownership/fencing until its physical reader actually completes.
    if !matches!(drained, Ok(Ok(_))) {
        uncertain();
        std::future::pending::<()>().await;
    }
    child.wait().await
}

#[cfg(target_os = "macos")]
fn group_quiescent(group: i32) -> io::Result<bool> {
    let group_id = u32::try_from(group).map_err(io::Error::other)?;
    let mut status = std::mem::MaybeUninit::<libc::siginfo_t>::zeroed();
    // SAFETY: correctly aligned initialized storage; P_PID targets the retained
    // direct child only. WNOWAIT observes exit without relinquishing its identity.
    if unsafe {
        libc::waitid(
            libc::P_PID,
            group_id,
            status.as_mut_ptr(),
            libc::WEXITED | libc::WNOHANG | libc::WNOWAIT,
        )
    } != 0
    {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: successful waitid initialized this record; si_pid is its PID field.
    let exited = unsafe { status.assume_init().si_pid() } == group;
    if !exited {
        return Ok(false);
    }
    let mut pids = [0i32; 512];
    let capacity = i32::try_from(std::mem::size_of_val(&pids)).expect("bounded PID array");
    // SAFETY: initialized writable array with the exact supplied byte capacity.
    // PROC_PGRP_ONLY=2 comes from the selected macOS SDK sys/proc_info.h.
    let bytes = unsafe {
        libc::proc_listpids(
            2,
            u32::try_from(group).map_err(io::Error::other)?,
            pids.as_mut_ptr().cast(),
            capacity,
        )
    };
    if bytes <= 0 || bytes >= capacity || bytes % 4 != 0 {
        return Err(io::Error::other("owned group enumeration incomplete"));
    }
    let count = usize::try_from(bytes / 4).map_err(io::Error::other)?;
    let mut leader_seen = false;
    for pid in &pids[..count] {
        if *pid == group {
            leader_seen = true;
            continue;
        }
        let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::uninit();
        let size =
            i32::try_from(std::mem::size_of::<libc::proc_bsdinfo>()).expect("fixed OS record");
        // SAFETY: aligned storage of exactly the structure size required by this
        // SDK flavor. Initialized fields are read only after exact byte success.
        let read = unsafe {
            libc::proc_pidinfo(
                *pid,
                libc::PROC_PIDTBSDINFO,
                0,
                info.as_mut_ptr().cast(),
                size,
            )
        };
        if read != size {
            let error = io::Error::last_os_error();
            // macOS hides zombies from proc_pidinfo (ESRCH), even though they
            // remain in proc_listpids. They own no running work or open pipes.
            if error.raw_os_error() == Some(libc::ESRCH) {
                continue;
            }
            return Err(error);
        }
        // SAFETY: proc_pidinfo returned the entire initialized proc_bsdinfo.
        let info = unsafe { info.assume_init() };
        if info.pbi_pgid != u32::try_from(group).map_err(io::Error::other)? {
            return Err(io::Error::other(
                "owned group membership changed during inspection",
            ));
        }
        leader_seen |= *pid == group;
        if info.pbi_status != libc::SZOMB {
            return Ok(false);
        }
    }
    if !leader_seen {
        return Err(io::Error::other("unreaped owned group leader missing"));
    }
    Ok(true)
}
#[cfg(not(target_os = "macos"))]
fn group_quiescent(_group: i32) -> io::Result<bool> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "process-group qualification currently supports macOS only",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake {
        waits: std::collections::VecDeque<io::Result<bool>>,
        signals: Vec<i32>,
        fail_signal: bool,
    }
    impl Cleanup for Fake {
        fn signal(&mut self, signal: i32) -> io::Result<()> {
            self.signals.push(signal);
            if self.fail_signal {
                Err(io::Error::other("synthetic"))
            } else {
                Ok(())
            }
        }
        async fn quiescent(&mut self, _: Duration) -> io::Result<bool> {
            self.waits.pop_front().unwrap()
        }
    }
    #[test]
    fn launch_uses_the_captured_executable_environment_and_neutral_directory_without_a_shell() {
        let launch = Launch {
            program: PathBuf::from("/fixture/codex"),
            directory: PathBuf::from("/fixture/neutral"),
            environment: vec![("CODEX_HOME".into(), "/fixture/home".into())],
        };
        let command = launch.command("app-server");
        let command = command.as_std();
        assert_eq!(command.get_program(), "/fixture/codex");
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["app-server"]);
        assert_eq!(
            command.get_current_dir(),
            Some(std::path::Path::new("/fixture/neutral"))
        );
        assert_eq!(
            command.get_envs().collect::<Vec<_>>(),
            [(
                std::ffi::OsStr::new("CODEX_HOME"),
                Some(std::ffi::OsStr::new("/fixture/home"))
            )]
        );
    }
    #[test]
    fn cleanup_escalates_only_while_owned_and_retains_uncertainty() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .start_paused(true)
            .build()
            .unwrap()
            .block_on(async {
                let mut fake = Fake {
                    waits: [Ok(false), Ok(false), Ok(true)].into(),
                    signals: vec![],
                    fail_signal: false,
                };
                assert!(cleanup(&mut fake).await);
                assert_eq!(fake.signals, [libc::SIGTERM, libc::SIGKILL]);
                let mut fake = Fake {
                    waits: [Ok(false)].into(),
                    signals: vec![],
                    fail_signal: true,
                };
                assert!(!cleanup(&mut fake).await);
            });
    }
}
