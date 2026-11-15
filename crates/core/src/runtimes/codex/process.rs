//! Private process/stdio mechanics. No authentication or readiness policy here.
use super::protocol;
use crate::runtimes::{ProviderTarget, SetupEvidence, SetupProblem};
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
            supported_version: protocol::VERSION.into(),
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

fn spawn(launch: &Launch, argument: &str) -> Result<Child, SetupProblem> {
    launch.command(argument).spawn().map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            SetupProblem::MissingExecutable
        } else {
            SetupProblem::StatusReadFailed
        }
    })
}

trait Cleanup {
    fn signal(&mut self, signal: i32) -> io::Result<()>;
    async fn wait(&mut self, budget: Duration) -> io::Result<bool>;
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
    async fn wait(&mut self, budget: Duration) -> io::Result<bool> {
        timeout(budget, Child::wait(self))
            .await
            .map(|result| result.map(|_| true))
            .unwrap_or(Ok(false))
    }
}
async fn cleanup(child: &mut impl Cleanup) -> bool {
    if child.wait(Duration::from_secs(2)).await.unwrap_or(false) {
        return true;
    }
    if child.signal(libc::SIGTERM).is_ok()
        && child.wait(Duration::from_secs(2)).await.unwrap_or(false)
    {
        return true;
    }
    child.signal(libc::SIGKILL).is_ok() && child.wait(Duration::from_secs(2)).await.unwrap_or(false)
}

pub(in crate::runtimes) async fn probe(
    launch: Launch,
    cancel: &CancellationToken,
    uncertain: impl Fn(),
) -> Result<SetupEvidence, SetupProblem> {
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
    .map_err(|_| SetupProblem::StatusReadFailed)?
    .map_err(|_| SetupProblem::StatusReadFailed)?;
    version(&launch, cancel, &uncertain).await?;
    if cancel.is_cancelled() {
        return Err(SetupProblem::StatusReadFailed);
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
        ))
        .await
        .unwrap_or(Err(SetupProblem::StatusReadFailed));
    finish(&mut child, drain, &uncertain).await;
    result
}
async fn version(
    launch: &Launch,
    cancel: &CancellationToken,
    uncertain: &impl Fn(),
) -> Result<(), SetupProblem> {
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
                .map_err(|_| SetupProblem::StatusReadFailed)?;
            if bytes.len() > 1024 {
                return Err(SetupProblem::IncompatibleProtocol);
            }
            let status = child
                .wait()
                .await
                .map_err(|_| SetupProblem::StatusReadFailed)?;
            protocol::version(&bytes, status.success())
        }))
        .await
    {
        Some(Ok(result)) => result,
        _ => Err(SetupProblem::StatusReadFailed),
    };
    drop(stdout);
    finish(&mut child, drain, uncertain).await;
    result
}
async fn finish(
    child: &mut Child,
    drain: tokio::task::JoinHandle<io::Result<u64>>,
    uncertain: &impl Fn(),
) {
    if !cleanup(child).await {
        uncertain();
        // Keep the owned handle and worker alive. Logical timeout never grants
        // replacement authority; a later real exit is still observed/reaped.
        if child.wait().await.is_err() {
            std::future::pending::<()>().await;
        }
    }
    let mut drain = drain;
    if timeout(Duration::from_secs(2), &mut drain).await.is_err() {
        drain.abort();
        let _ = drain.await; // Join the stopped pipe reader before releasing work.
    }
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
        async fn wait(&mut self, _: Duration) -> io::Result<bool> {
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
