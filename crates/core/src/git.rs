//! Bounded Git execution shared by workspace policy.
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
fn base_command(path: &Path) -> Command {
    let mut command = Command::new("git");
    // Git locations come from the explicit target, never the desktop environment.
    for (key, _) in std::env::vars_os() {
        if key.to_string_lossy().starts_with("GIT_") {
            command.env_remove(key);
        }
    }
    command
        .args(["--no-pager", "--no-optional-locks", "-C"])
        .arg(path)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    command
}
fn command(path: &Path, query: Query) -> Command {
    let mut command = base_command(path);
    command.args(["rev-parse", "--path-format=absolute", query.argument()]);
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
fn run(command: Command) -> Result<String> {
    let (success, bytes) = run_output(command, None)?;
    if !success {
        return Err(failure(String::from_utf8_lossy(&bytes)));
    }
    let text = String::from_utf8(bytes).map_err(failure)?;
    Ok(text.strip_suffix('\n').unwrap_or(&text).to_owned())
}
fn run_output(mut command: Command, input: Option<&[u8]>) -> Result<(bool, Vec<u8>)> {
    if input.is_some_and(|bytes| bytes.len() > 1024) {
        return Err(failure("Git command input exceeds its bounded budget"));
    }
    if input.is_some() {
        command.stdin(Stdio::piped());
    }
    let mut child = command.spawn().map_err(failure)?;
    let setup = (|| {
        if let Some(input) = input {
            if input.len() > 1024 {
                return Err(failure("Git command input exceeds its bounded budget"));
            }
            child
                .stdin
                .take()
                .ok_or_else(|| failure("Git stdin unavailable"))?
                .write_all(input)
                .map_err(failure)?;
        }
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
            stop_child(&mut child)?;
            return Err(error);
        }
    };
    run_process_output(&mut LocalProcess {
        child,
        stdout,
        stderr,
        start: Instant::now(),
        reaped: false,
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
    reaped: bool,
}
impl GitProcess for LocalProcess {
    fn output(&mut self, stdout: &mut Vec<u8>, stderr: &mut Vec<u8>) -> Result<()> {
        drain(&mut self.stdout, stdout)?;
        drain(&mut self.stderr, stderr)
    }
    fn exited(&mut self) -> Result<Option<bool>> {
        let status = self.child.try_wait().map_err(failure)?;
        if status.is_some() {
            self.reaped = true;
        }
        Ok(status.map(|s| s.success()))
    }
    fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
    fn wait_tick(&mut self) {
        std::thread::sleep(Duration::from_millis(5));
    }
    fn stop_and_wait(&mut self) -> Result<()> {
        if self.reaped {
            return Err(RepositoryError::new(
                ErrorCode::CleanupUnconfirmed,
                "Git exited before its output/descendant cleanup could be confirmed. Preserve its workspace and reconcile; a reaped PID was not signaled.",
            ));
        }
        stop_child(&mut self.child)
    }
}
fn stop_child(child: &mut std::process::Child) -> Result<()> {
    let group = i32::try_from(child.id()).map_err(failure)?;
    // SAFETY: the live child was launched into its own process group; no saved PID is used.
    let signal = unsafe { libc::kill(-group, libc::SIGKILL) };
    if signal != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
        return Err(RepositoryError::new(
            ErrorCode::CleanupUnconfirmed,
            "Git process-group stop could not be confirmed. Preserve its workspace and investigate before continuing.",
        ));
    }
    let start = Instant::now();
    loop {
        if child.try_wait().map_err(failure)?.is_some() {
            // SAFETY: signal 0 observes this owned process group without signaling it.
            if unsafe { libc::kill(-group, 0) } == -1
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)
            {
                return Ok(());
            }
        }
        if start.elapsed() >= Duration::from_millis(500) {
            return Err(RepositoryError::new(
                ErrorCode::CleanupUnconfirmed,
                "Git cleanup could not be confirmed within its bounded stop window. Preserve the workspace; do not repeat creation.",
            ));
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
#[cfg(test)]
fn run_process(process: &mut impl GitProcess) -> Result<String> {
    let (success, bytes) = run_process_output(process)?;
    if !success {
        return Err(failure(String::from_utf8_lossy(&bytes)));
    }
    String::from_utf8(bytes).map_err(failure)
}
fn run_process_output(process: &mut impl GitProcess) -> Result<(bool, Vec<u8>)> {
    let result = (|| {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        loop {
            process.output(&mut out, &mut err)?;
            if let Some(success) = process.exited()? {
                process.output(&mut out, &mut err)?;
                return Ok((success, if success { out } else { err }));
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

/// Production Git/filesystem mechanics. Creation/recovery decisions remain in workspaces.
pub(crate) struct LocalCreation;
use crate::workspaces::{
    Artifacts, BranchArtifact, CreationEffects, FolderArtifact, PinnedBase, Workspace,
};
use std::io::Write;
fn creation_command(repository: &Path, args: &[&str]) -> Command {
    let mut command = base_command(repository);
    // Workspace creation must not execute deferred project setup hooks.
    command
        .args([
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "submodule.recurse=false",
        ])
        .args(args);
    command
}
fn query_optional(repository: &Path, reference: &str) -> Result<Option<String>> {
    let output = run_output(
        creation_command(
            repository,
            &[
                "rev-parse",
                "--verify",
                "--quiet",
                "--end-of-options",
                &format!("{reference}^{{commit}}"),
            ],
        ),
        None,
    )?;
    match output {
        (true, bytes) => {
            let text = String::from_utf8(bytes).map_err(failure)?;
            let commit = text.trim_end_matches('\n');
            if !matches!(commit.len(), 40 | 64) || !commit.bytes().all(|b| b.is_ascii_hexdigit()) {
                return Err(failure("Git returned an invalid commit identity"));
            }
            Ok(Some(commit.to_owned()))
        }
        (false, _) => Ok(None),
    }
}
fn ownership_ref(workspace: &Workspace) -> String {
    format!("refs/agent-workspace/{}", workspace.id.0)
}
fn wrapper(workspace: &Workspace) -> Result<&Path> {
    Path::new(&workspace.path)
        .parent()
        .ok_or_else(|| failure("Workspace folder has no parent"))
}
fn check_workspace_ancestors(workspace: &Workspace) -> Result<()> {
    let parent = wrapper(workspace)?;
    let repository_dir = parent
        .parent()
        .ok_or_else(|| failure("Invalid workspace repository directory"))?;
    let root = repository_dir
        .parent()
        .ok_or_else(|| failure("Invalid managed workspace root"))?;
    for path in [root, repository_dir, parent] {
        if location(path)?.is_some() {
            crate::supervisor::check_private(path, true).map_err(failure)?;
        }
    }
    Ok(())
}
fn marker(workspace: &Workspace) -> Result<Vec<u8>> {
    serde_json::to_vec(&(
        workspace.id,
        workspace.creation_action_id,
        workspace.repository_id,
    ))
    .map_err(RepositoryError::storage)
}
fn location(path: &Path) -> Result<Option<std::fs::Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(m) => Ok(Some(m)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(failure(e)),
    }
}
fn validate_repository(repository: &crate::workspaces::RepositoryRecord) -> Result<()> {
    let actual = inspect(&repository.path)?;
    if actual.path != repository.path || actual.common_dir != repository.common_dir {
        return Err(failure(
            "The registered location now identifies a different repository. Restore its original location",
        ));
    }
    Ok(())
}
impl CreationEffects for LocalCreation {
    fn quiescent(&mut self, workspace: &Workspace) -> Result<bool> {
        check_workspace_ancestors(workspace)?;
        let path = wrapper(workspace)?.join("execution.lock");
        if location(&path)?.is_none() {
            return Ok(true);
        }
        let lock = open_execution_lock(workspace)?;
        match lock.try_lock() {
            Ok(()) => Ok(true),
            Err(std::fs::TryLockError::WouldBlock) => Ok(false),
            Err(std::fs::TryLockError::Error(e)) => Err(failure(e)),
        }
    }
    fn base(&mut self, repository: &crate::workspaces::RepositoryRecord) -> Result<PinnedBase> {
        validate_repository(repository)?;
        let path = Path::new(&repository.path);
        let origin = query_optional(path, "refs/remotes/origin/HEAD")?;
        let main = query_optional(path, "refs/heads/main")?;
        let master = query_optional(path, "refs/heads/master")?;
        let (reference, commit) =
            crate::workspaces::choose_base(origin.as_deref(), main.as_deref(), master.as_deref())?;
        Ok(PinnedBase {
            reference: reference.into(),
            commit: commit.into(),
        })
    }
    fn inspect(
        &mut self,
        repository: &crate::workspaces::RepositoryRecord,
        workspace: &Workspace,
    ) -> Result<Artifacts> {
        validate_repository(repository)?;
        let path = Path::new(&repository.path);
        let base = workspace
            .base
            .as_ref()
            .ok_or_else(|| failure("Workspace has no pinned commit"))?;
        let branch = query_optional(path, &format!("refs/heads/{}", workspace.branch))?;
        let owner = query_optional(path, &ownership_ref(workspace))?;
        let branch = match (branch, owner) {
            (None, None) => BranchArtifact::Absent,
            (Some(actual), Some(owner)) if actual == base.commit && owner == base.commit => {
                BranchArtifact::Owned
            }
            _ => BranchArtifact::Conflict,
        };
        let parent = wrapper(workspace)?;
        let folder = match location(parent)? {
            None => FolderArtifact::Absent,
            Some(m) if !m.is_dir() || m.file_type().is_symlink() => FolderArtifact::Conflict,
            Some(_) => {
                let marker_path = parent.join("owner.json");
                let owned = match location(&marker_path)? {
                    Some(m) if m.is_file() && m.len() <= 4096 && !m.file_type().is_symlink() => {
                        std::fs::read(&marker_path).map_err(failure)? == marker(workspace)?
                    }
                    _ => false,
                };
                if !owned {
                    FolderArtifact::Conflict
                } else if location(Path::new(&workspace.path))?.is_none() {
                    FolderArtifact::Reserved
                } else {
                    let inspected = inspect(&workspace.path);
                    if let Ok(actual) = inspected {
                        if actual.path == workspace.path
                            && actual.common_dir == repository.common_dir
                        {
                            let target = Path::new(&workspace.path);
                            let head = query_optional(target, "HEAD")?;
                            let symbolic = run_output(
                                creation_command(target, &["symbolic-ref", "--quiet", "HEAD"]),
                                None,
                            )?;
                            let clean = run_output(
                                creation_command(
                                    target,
                                    &["status", "--porcelain=v1", "-z", "--untracked-files=normal"],
                                ),
                                None,
                            )?;
                            if head.as_deref() == Some(&base.commit)
                                && symbolic.0
                                && symbolic.1
                                    == format!("refs/heads/{}\n", workspace.branch).as_bytes()
                                && clean.0
                                && clean.1.is_empty()
                            {
                                FolderArtifact::Worktree
                            } else {
                                FolderArtifact::Partial
                            }
                        } else {
                            FolderArtifact::Conflict
                        }
                    } else {
                        FolderArtifact::Partial
                    }
                }
            }
        };
        Ok(Artifacts { branch, folder })
    }
    fn reserve(&mut self, workspace: &Workspace) -> Result<()> {
        use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
        let parent = wrapper(workspace)?;
        let repository_dir = parent
            .parent()
            .ok_or_else(|| failure("Invalid workspace root"))?;
        let root = repository_dir
            .parent()
            .ok_or_else(|| failure("Invalid managed root"))?;
        for directory in [root, repository_dir, parent] {
            match std::fs::DirBuilder::new().mode(0o700).create(directory) {
                Ok(()) => {}
                Err(e) if e.kind() == ErrorKind::AlreadyExists && directory != parent => {
                    crate::supervisor::check_private(directory, true).map_err(failure)?
                }
                Err(e) => return Err(failure(e)),
            }
        }
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(parent.join("execution.lock"))
            .and_then(|f| f.sync_all())
            .map_err(failure)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(parent.join("owner.json"))
            .map_err(failure)?;
        file.write_all(&marker(workspace)?).map_err(failure)?;
        file.sync_all().map_err(failure)?;
        for directory in [parent, repository_dir, root] {
            std::fs::File::open(directory)
                .and_then(|f| f.sync_all())
                .map_err(failure)?;
        }
        Ok(())
    }
    fn branch(
        &mut self,
        repository: &crate::workspaces::RepositoryRecord,
        workspace: &Workspace,
    ) -> Result<()> {
        validate_repository(repository)?;
        let base = workspace
            .base
            .as_ref()
            .ok_or_else(|| failure("Missing pinned base"))?;
        let input = format!(
            "start\ncreate refs/heads/{} {}\ncreate {} {}\nprepare\ncommit\n",
            workspace.branch,
            base.commit,
            ownership_ref(workspace),
            base.commit
        );
        let (success, output) = run_leased(
            workspace,
            creation_command(Path::new(&repository.path), &["update-ref", "--stdin"]),
            Some(input.as_bytes()),
        )?;
        if success {
            Ok(())
        } else {
            Err(failure(String::from_utf8_lossy(&output)))
        }
    }
    fn worktree(
        &mut self,
        repository: &crate::workspaces::RepositoryRecord,
        workspace: &Workspace,
    ) -> Result<()> {
        validate_repository(repository)?;
        let (success, output) = run_leased(
            workspace,
            creation_command(
                Path::new(&repository.path),
                &[
                    "worktree",
                    "add",
                    "--quiet",
                    "--",
                    &workspace.path,
                    &workspace.branch,
                ],
            ),
            None,
        )?;
        if success {
            Ok(())
        } else {
            Err(failure(String::from_utf8_lossy(&output)))
        }
    }
}

fn open_execution_lock(workspace: &Workspace) -> Result<std::fs::File> {
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(wrapper(workspace)?.join("execution.lock"))
        .map_err(failure)?;
    let metadata = file.metadata().map_err(failure)?;
    // SAFETY: geteuid has no preconditions and does not change state.
    if !metadata.is_file()
        || metadata.nlink() != 1
        || metadata.mode() & 0o077 != 0
        || metadata.uid() != unsafe { libc::geteuid() }
    {
        return Err(failure(
            "Workspace execution lease must be a private owned regular file",
        ));
    }
    Ok(file)
}
fn run_leased(
    workspace: &Workspace,
    mut command: Command,
    input: Option<&[u8]>,
) -> Result<(bool, Vec<u8>)> {
    let lease = open_execution_lock(workspace)?;
    lease.try_lock().map_err(|_| RepositoryError::new(ErrorCode::CleanupUnconfirmed,"Another owned Git effect holds this workspace's execution lease. Reconcile after it exits."))?;
    let descriptor = lease.as_raw_fd();
    // SAFETY: lease outlives spawn and execution. The child callback uses only
    // async-signal-safe fcntl, with no allocation, environment access or mutexes.
    // Inheritance retains the kernel lease even if the supervisor crashes.
    unsafe {
        command.pre_exec(move || {
            let flags = libc::fcntl(descriptor, libc::F_GETFD);
            if flags == -1
                || libc::fcntl(descriptor, libc::F_SETFD, flags & !libc::FD_CLOEXEC) == -1
            {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let outcome = run_output(command, input);
    drop(lease);
    if !LocalCreation.quiescent(workspace)? {
        return Err(RepositoryError::new(
            ErrorCode::CleanupUnconfirmed,
            "Git exited while owned descendant work still holds the execution lease. Reconcile after that work exits; no further creation was performed.",
        ));
    }
    outcome
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
