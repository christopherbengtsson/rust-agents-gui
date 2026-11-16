//! Private process/stdio mechanics. No authentication or readiness policy here.
use super::protocol;
use crate::runtimes::{ProviderTarget, SetupEvidence, SetupFailure, SetupProblem};
use std::{
    ffi::OsString, io, os::unix::fs::DirBuilderExt, path::PathBuf, process::Stdio, time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    process::Command,
    time::{Instant, timeout},
};
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub(in crate::runtimes) struct Launch {
    pub(super) program: PathBuf,
    pub(super) directory: PathBuf,
    pub(super) environment: Vec<(OsString, OsString)>,
    pub(super) fixture: bool,
    pub(super) processes: crate::process_effects::Processes,
}
impl Launch {
    pub(in crate::runtimes) fn credential_scope(&self) -> String {
        self.environment
            .iter()
            .find(|(key, _)| key == "CODEX_HOME" || key == "AW_FIXTURE_CONTROL")
            .map(|(_, value)| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.directory.to_string_lossy().into_owned())
    }
    pub(in crate::runtimes) fn capture(
        processes: crate::process_effects::Processes,
    ) -> (Self, ProviderTarget) {
        let id = uuid::Uuid::new_v4();
        let directory = std::env::temp_dir().join(format!("agent-workspace-setup-{id}"));
        let fixture = std::env::var_os("AGENT_WORKSPACE_CODEX_FIXTURE");
        let program = fixture
            .clone()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("codex"));
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
            executable: None,
            selected_executable: None,
            discovery_source: None,
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
                processes,
                program,
                directory,
                environment,
                fixture: fixture.is_some(),
            },
            target,
        )
    }
    pub(in crate::runtimes) fn executable(&self) -> String {
        self.program.to_string_lossy().into_owned()
    }
    pub(in crate::runtimes) async fn discover(
        &self,
        selected: Option<&std::path::Path>,
        cancel: &CancellationToken,
        uncertain: &(impl Fn() + Sync),
    ) -> Result<(Self, &'static str), SetupFailure> {
        let search = NativeSearch {
            launch: self,
            cancel,
            uncertain,
        };
        super::discovery::discover(self, selected, &search).await
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

struct NativeSearch<'a, F> {
    launch: &'a Launch,
    cancel: &'a CancellationToken,
    uncertain: &'a F,
}
impl<F: Fn() + Sync> super::discovery::Search for NativeSearch<'_, F> {
    async fn resolve(&self, path: &std::path::Path) -> Option<PathBuf> {
        let path = path.to_owned();
        tokio::task::spawn_blocking(move || {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&path).ok()?;
            if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
                return None;
            }
            std::fs::canonicalize(path).ok()
        })
        .await
        .ok()
        .flatten()
    }
    async fn shell_path(&self) -> Option<OsString> {
        use std::os::unix::ffi::OsStringExt;
        let shell = self
            .launch
            .environment
            .iter()
            .find(|(key, _)| key == "SHELL")
            .map(|(_, value)| PathBuf::from(value))
            .filter(|path| path.is_absolute())
            .unwrap_or_else(|| PathBuf::from("/bin/zsh"));
        let mut command = Command::new(shell);
        // Disable job control before startup: a background desktop/helper must
        // never try to acquire its launcher’s controlling terminal.
        command
            .args([
                "-il",
                "+m",
                "-c",
                "printf '\\0AW_PATH_START\\0%s\\0AW_PATH_END\\0' \"$PATH\"",
            ])
            .current_dir("/")
            .env_clear()
            .envs(self.launch.environment.iter().cloned())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0);
        let mut child = self.launch.processes.spawn(command).ok()?;
        let mut stdout = child.stdout.take()?.take(65537);
        let mut bytes = Vec::new();
        let read = self
            .cancel
            .run_until_cancelled(timeout(
                Duration::from_secs(10),
                stdout.read_to_end(&mut bytes),
            ))
            .await;
        drop(stdout);
        let status = child.finish(self.uncertain).await.ok()?;
        if !status.success() || !matches!(read, Some(Ok(Ok(_)))) || bytes.len() > 65536 {
            return None;
        }
        shell_path_bytes(&bytes).map(OsString::from_vec)
    }
}
fn shell_path_bytes(bytes: &[u8]) -> Option<Vec<u8>> {
    let start = b"\0AW_PATH_START\0";
    let end = b"\0AW_PATH_END\0";
    let offset = bytes.windows(start.len()).position(|part| part == start)? + start.len();
    let tail = &bytes[offset..];
    let length = tail.windows(end.len()).position(|part| part == end)?;
    let path = &tail[..length];
    (!path.contains(&0)).then(|| path.to_vec())
}

fn spawn(launch: &Launch, argument: &str) -> Result<crate::process_effects::Process, SetupFailure> {
    launch
        .processes
        .spawn(launch.command(argument))
        .map_err(|error| {
            let missing = error.kind() == io::ErrorKind::NotFound;
            let mut failure = SetupFailure::io(argument, error);
            if missing {
                failure.problem = SetupProblem::MissingExecutable;
            }
            failure
        })
}

pub(in crate::runtimes) async fn probe(
    launch: Launch,
    cancel: &CancellationToken,
    uncertain: impl Fn(),
) -> Result<SetupEvidence, SetupFailure> {
    probe_checked(launch, cancel, uncertain).await.1
}
pub(in crate::runtimes) async fn probe_checked(
    launch: Launch,
    cancel: &CancellationToken,
    uncertain: impl Fn(),
) -> (bool, Result<SetupEvidence, SetupFailure>) {
    let validated = std::sync::atomic::AtomicBool::new(false);
    let result = probe_inner(launch, cancel, uncertain, || {
        validated.store(true, std::sync::atomic::Ordering::SeqCst);
    })
    .await;
    (validated.load(std::sync::atomic::Ordering::SeqCst), result)
}
async fn probe_inner(
    launch: Launch,
    cancel: &CancellationToken,
    uncertain: impl Fn(),
    validated: impl Fn(),
) -> Result<SetupEvidence, SetupFailure> {
    let directory = launch.directory.clone();
    tokio::task::spawn_blocking(move || {
        match std::fs::DirBuilder::new().mode(0o700).create(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
        crate::private_state::check_private(&directory)
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
            "Setup cancelled during native core shutdown",
        ));
    }
    let mut child = spawn(&launch, "app-server")?;
    let stdout = child.stdout.take().expect("piped stdout");
    let stdin = child.stdin.take().expect("piped stdin");
    let result = cancel
        .run_until_cancelled(protocol::inspect_checked(
            stdout,
            stdin,
            Instant::now() + Duration::from_secs(15),
            &observed_version,
            validated,
        ))
        .await
        .unwrap_or_else(|| {
            Err(SetupFailure::at(
                SetupProblem::StatusReadFailed,
                "Setup cancelled during native core shutdown",
            ))
        });
    child
        .finish(&uncertain)
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
        crate::private_state::check_private(&directory)
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
    let stdout = child.stdout.take().expect("piped stdout");
    let stdin = child.stdin.take().expect("piped stdin");
    let fixture = launch
        .environment
        .iter()
        .any(|(key, _)| key == "AW_FIXTURE_CONTROL");
    let processes = &launch.processes;
    let result = protocol::login(stdout, stdin, cancel, reopen, &event, |url| {
        let uncertain = &uncertain;
        async move { open_browser(&url, fixture, processes, uncertain).await }
    })
    .await;
    child
        .finish(&uncertain)
        .await
        .map_err(|error| SetupFailure::io("login physical cleanup", error))?;
    result
}

async fn open_browser(
    url: &str,
    fixture: bool,
    processes: &crate::process_effects::Processes,
    uncertain: &impl Fn(),
) -> bool {
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
    let Ok(child) = processes.spawn_until(command, Instant::now() + Duration::from_secs(5)) else {
        return false;
    };
    child
        .finish(uncertain)
        .await
        .is_ok_and(|status| status.success())
}

async fn version(
    launch: &Launch,
    cancel: &CancellationToken,
    uncertain: &impl Fn(),
) -> Result<String, SetupFailure> {
    let mut child = spawn(launch, "--version")?;
    drop(child.stdin.take());
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
    let status = child
        .finish(uncertain)
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
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shell_path_requires_bounded_framed_path_and_ignores_startup_noise() {
        assert_eq!(
            shell_path_bytes(b"startup text\0AW_PATH_START\0/tool/bin:/usr/bin\0AW_PATH_END\0bye"),
            Some(b"/tool/bin:/usr/bin".to_vec())
        );
        assert_eq!(shell_path_bytes(b"/unframed/path"), None);
        assert_eq!(
            shell_path_bytes(b"\0AW_PATH_START\0/bad\0/path\0AW_PATH_END\0"),
            None
        );
    }
    #[tokio::test]
    async fn launch_uses_the_captured_executable_environment_and_neutral_directory_without_a_shell()
    {
        let launch = Launch {
            processes: crate::process_effects::Processes::new(),
            program: PathBuf::from("/fixture/codex"),
            directory: PathBuf::from("/fixture/neutral"),
            environment: vec![("CODEX_HOME".into(), "/fixture/home".into())],
            fixture: true,
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
}
