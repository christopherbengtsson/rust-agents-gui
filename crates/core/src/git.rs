//! Bounded read-only Git queries shared by workspace policy.
use crate::workspaces::{ErrorCode, RepositoryError, Result, ValidatedRepository};
use std::{
    io::{ErrorKind, Read},
    os::{fd::AsRawFd, unix::process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const OUTPUT_LIMIT: usize = 16 * 1024;
const DEADLINE: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Query {
    Inside,
    Root,
    Common,
}
impl Query {
    fn argument(self) -> &'static str {
        match self {
            Self::Inside => "--is-inside-work-tree",
            Self::Root => "--show-toplevel",
            Self::Common => "--git-common-dir",
        }
    }
}
fn command(path: &Path, query: Query) -> Command {
    let mut command = Command::new("git");
    // Git locations must come from the explicit request, never the desktop's environment.
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .args(["--no-pager", "--no-optional-locks", "-C"])
        .arg(path)
        .args(["rev-parse", "--path-format=absolute", query.argument()])
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    command
}
fn failure(message: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::new(
        ErrorCode::Git,
        format!(
            "Cannot inspect the Git repository: {message}. Check the folder, Git installation, access permissions, and Git ownership guidance, then try again."
        ),
    )
}
trait GitEffects {
    fn directory(&mut self, path: &Path) -> Result<PathBuf>;
    fn query(&mut self, path: &Path, query: Query) -> Result<String>;
}
struct LocalGit;
impl GitEffects for LocalGit {
    fn directory(&mut self, path: &Path) -> Result<PathBuf> {
        let canonical = std::fs::canonicalize(path).map_err(|e| {
            RepositoryError::new(
                ErrorCode::InvalidPath,
                format!(
                    "Cannot open {}: {e}. Restore access or choose an existing repository folder.",
                    path.display()
                ),
            )
        })?;
        if !canonical.is_dir() {
            return Err(RepositoryError::new(
                ErrorCode::InvalidPath,
                "Choose a folder containing an existing Git checkout.",
            ));
        }
        Ok(canonical)
    }
    fn query(&mut self, path: &Path, query: Query) -> Result<String> {
        run(command(path, query))
    }
}

pub(crate) fn inspect(path: &str) -> Result<ValidatedRepository> {
    inspect_with(path, &mut LocalGit)
}
fn inspect_with(path: &str, effects: &mut impl GitEffects) -> Result<ValidatedRepository> {
    let directory = effects.directory(Path::new(path))?;
    if effects.query(&directory, Query::Inside)? != "true" {
        return Err(RepositoryError::new(
            ErrorCode::InvalidPath,
            "Choose a Git working checkout. Bare repositories and Git metadata folders cannot be registered.",
        ));
    }
    let root = effects.query(&directory, Query::Root)?;
    let common = effects.query(&directory, Query::Common)?;
    let root = effects.directory(Path::new(&root))?;
    let common = effects.directory(Path::new(&common))?;
    let path = root
        .to_str()
        .ok_or_else(|| failure("The checkout path is not UTF-8"))?
        .to_owned();
    let common_dir = common
        .to_str()
        .ok_or_else(|| failure("The Git directory path is not UTF-8"))?
        .to_owned();
    let name = root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(&path)
        .to_owned();
    Ok(ValidatedRepository {
        name,
        path,
        common_dir,
    })
}

fn nonblocking(pipe: &impl AsRawFd) -> Result<()> {
    // SAFETY: a live pipe supplies the descriptor; fcntl changes only its I/O flags.
    let flags = unsafe { libc::fcntl(pipe.as_raw_fd(), libc::F_GETFL) };
    if flags == -1
        || unsafe { libc::fcntl(pipe.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) } == -1
    {
        return Err(failure(std::io::Error::last_os_error()));
    }
    Ok(())
}
fn drain(pipe: &mut impl Read, bytes: &mut Vec<u8>) -> Result<()> {
    let mut buffer = [0; 2048];
    loop {
        match pipe.read(&mut buffer) {
            Ok(0) => return Ok(()),
            Ok(count) => {
                if bytes.len() + count > OUTPUT_LIMIT {
                    return Err(failure("Git output exceeded its bounded read budget"));
                }
                bytes.extend_from_slice(&buffer[..count]);
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(()),
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(failure(e)),
        }
    }
}
fn run(mut command: Command) -> Result<String> {
    let mut child = command.spawn().map_err(failure)?;
    let setup = (|| {
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| failure("Git stdout unavailable"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| failure("Git stderr unavailable"))?;
        nonblocking(&stdout)?;
        nonblocking(&stderr)?;
        Ok((stdout, stderr))
    })();
    let (stdout, stderr) = match setup {
        Ok(pipes) => pipes,
        Err(error) => {
            let _ = child.kill();
            child.wait().map_err(failure)?;
            return Err(error);
        }
    };
    run_process(&mut LocalProcess {
        child,
        stdout,
        stderr,
        start: Instant::now(),
    })
}

trait GitProcess {
    fn output(&mut self, stdout: &mut Vec<u8>, stderr: &mut Vec<u8>) -> Result<()>;
    fn exited(&mut self) -> Result<Option<bool>>;
    fn elapsed(&self) -> Duration;
    fn wait_tick(&mut self);
    fn stop_and_wait(&mut self) -> Result<()>;
}
struct LocalProcess {
    child: std::process::Child,
    stdout: std::process::ChildStdout,
    stderr: std::process::ChildStderr,
    start: Instant,
}
impl GitProcess for LocalProcess {
    fn output(&mut self, stdout: &mut Vec<u8>, stderr: &mut Vec<u8>) -> Result<()> {
        drain(&mut self.stdout, stdout)?;
        drain(&mut self.stderr, stderr)
    }
    fn exited(&mut self) -> Result<Option<bool>> {
        self.child
            .try_wait()
            .map(|s| s.map(|s| s.success()))
            .map_err(failure)
    }
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    fn wait_tick(&mut self) {
        std::thread::sleep(Duration::from_millis(5));
    }
    fn stop_and_wait(&mut self) -> Result<()> {
        let stopped = self.child.kill();
        self.child.wait().map_err(|e| {
            failure(format!(
                "Git cleanup could not be confirmed: {e}; stop result {stopped:?}"
            ))
        })?;
        Ok(())
    }
}
fn run_process(process: &mut impl GitProcess) -> Result<String> {
    let result = (|| {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        loop {
            process.output(&mut out, &mut err)?;
            if let Some(success) = process.exited()? {
                process.output(&mut out, &mut err)?;
                if !success {
                    return Err(failure(String::from_utf8_lossy(&err)));
                }
                let text = String::from_utf8(out).map_err(failure)?;
                return Ok(text.strip_suffix('\n').unwrap_or(&text).to_owned());
            }
            if process.elapsed() >= DEADLINE {
                return Err(failure("Git inspection timed out"));
            }
            process.wait_tick();
        }
    })();
    if result.is_err() {
        process.stop_and_wait()?;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Probe {
        inside: &'static str,
        calls: Vec<Query>,
    }
    impl GitEffects for Probe {
        fn directory(&mut self, path: &Path) -> Result<PathBuf> {
            Ok(if path == Path::new("/alias") {
                PathBuf::from("/checkout/src")
            } else {
                path.to_owned()
            })
        }
        fn query(&mut self, _: &Path, query: Query) -> Result<String> {
            self.calls.push(query);
            Ok(match query {
                Query::Inside => self.inside,
                Query::Root => "/checkout",
                Query::Common => "/main/.git",
            }
            .into())
        }
    }
    #[test]
    fn nested_alias_resolves_checkout_and_shared_git_identity_without_mutating_queries() {
        let mut effects = Probe {
            inside: "true",
            calls: vec![],
        };
        let result = inspect_with("/alias", &mut effects).unwrap();
        assert_eq!(
            (
                result.path.as_str(),
                result.common_dir.as_str(),
                result.name.as_str()
            ),
            ("/checkout", "/main/.git", "checkout")
        );
        assert_eq!(effects.calls, [Query::Inside, Query::Root, Query::Common]);
        let command = command(Path::new("/explicit path"), Query::Common);
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(
            args,
            [
                "--no-pager",
                "--no-optional-locks",
                "-C",
                "/explicit path",
                "rev-parse",
                "--path-format=absolute",
                "--git-common-dir"
            ]
        );
    }
    #[test]
    fn bare_repository_is_rejected_before_further_git_queries() {
        let mut effects = Probe {
            inside: "false",
            calls: vec![],
        };
        assert_eq!(
            inspect_with("/bare", &mut effects).err().unwrap().code,
            ErrorCode::InvalidPath
        );
        assert_eq!(effects.calls, [Query::Inside]);
    }
    #[test]
    fn output_budget_fails_instead_of_parsing_truncated_output() {
        assert!(
            drain(
                &mut std::io::Cursor::new(vec![b'x'; OUTPUT_LIMIT + 1]),
                &mut vec![]
            )
            .is_err()
        );
    }
    struct Process {
        elapsed: Duration,
        stopped: bool,
        cleanup_failed: bool,
    }
    impl GitProcess for Process {
        fn output(&mut self, _: &mut Vec<u8>, _: &mut Vec<u8>) -> Result<()> {
            Ok(())
        }
        fn exited(&mut self) -> Result<Option<bool>> {
            Ok(None)
        }
        fn elapsed(&self) -> Duration {
            self.elapsed
        }
        fn wait_tick(&mut self) {
            self.elapsed += DEADLINE;
        }
        fn stop_and_wait(&mut self) -> Result<()> {
            self.stopped = true;
            if self.cleanup_failed {
                Err(failure("cleanup could not be confirmed"))
            } else {
                Ok(())
            }
        }
    }
    #[test]
    fn timeout_stops_owned_git_without_real_waits_and_retains_unconfirmed_cleanup() {
        for cleanup_failed in [false, true] {
            let mut process = Process {
                elapsed: Duration::ZERO,
                stopped: false,
                cleanup_failed,
            };
            let result = run_process(&mut process).unwrap_err();
            assert!(process.stopped);
            assert!(result.message.contains(if cleanup_failed {
                "cleanup could not be confirmed"
            } else {
                "timed out"
            }));
        }
    }
}
