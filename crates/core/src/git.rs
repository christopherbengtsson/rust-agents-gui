//! Bounded Git execution shared by workspace policy.
use crate::workspaces::{ErrorCode, RepositoryError, Result, ValidatedRepository};
use std::{
    io::{ErrorKind, Read},
    os::{fd::AsRawFd, unix::process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::process_effects::Processes;
#[path = "git/inspection.rs"]
mod inspection;
pub(crate) use inspection::LocalInspection;

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
struct LocalGit<'a>(&'a Processes);
impl GitEffects for LocalGit<'_> {
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
        run(self.0, command(path, query))
    }
}

pub(crate) fn inspect(processes: &Processes, path: &str) -> Result<ValidatedRepository> {
    inspect_with(path, &mut LocalGit(processes))
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

fn run(processes: &Processes, command: Command) -> Result<String> {
    let (success, bytes) = run_output(processes, command, None)?;
    if !success {
        return Err(failure(String::from_utf8_lossy(&bytes)));
    }
    let text = String::from_utf8(bytes).map_err(failure)?;
    Ok(text.strip_suffix('\n').unwrap_or(&text).to_owned())
}
fn run_output(
    processes: &Processes,
    command: Command,
    input: Option<&[u8]>,
) -> Result<(bool, Vec<u8>)> {
    processes.git(command, input).map_err(|error| match error {
        crate::process_effects::GitFailure::Cleanup { original, cleanup } => RepositoryError::new(ErrorCode::CleanupUnconfirmed, format!("Git cleanup unconfirmed; effect: {original:?}; cleanup: {cleanup:?}. Preserve the workspace and reconcile.")),
        crate::process_effects::GitFailure::Effect(error) if error.kind() == ErrorKind::WouldBlock => RepositoryError::new(ErrorCode::Capacity, error.to_string()),
        crate::process_effects::GitFailure::Effect(error) => failure(error),
    })
}

/// Production Git/filesystem mechanics. Creation/recovery decisions remain in workspaces.
pub(crate) struct LocalCreation(pub(crate) Processes);
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
fn query_optional(
    processes: &Processes,
    repository: &Path,
    reference: &str,
) -> Result<Option<String>> {
    let output = run_output(
        processes,
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
    Path::new(workspace.path()?)
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
            crate::private_state::check_private(path).map_err(failure)?;
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
fn validate_repository(
    processes: &Processes,
    repository: &crate::workspaces::RepositoryRecord,
) -> Result<()> {
    let actual = inspect(processes, &repository.path)?;
    if actual.path != repository.path || actual.common_dir != repository.common_dir {
        return Err(failure(
            "The registered location now identifies a different repository. Restore its original location",
        ));
    }
    Ok(())
}
fn occupied_inventory(
    refs: &str,
    worktrees: &str,
    names: std::collections::HashSet<String>,
) -> Result<crate::workspaces::naming::Occupied> {
    let mut result = crate::workspaces::naming::Occupied {
        names,
        worktrees: vec![],
    };
    for reference in refs.lines() {
        if reference == "refs/heads/workspace" {
            return Err(failure(
                "An existing workspace branch blocks the workspace/ namespace. Rename that branch before reconciling.",
            ));
        }
        if let Some(name) = reference.strip_prefix("refs/heads/workspace/") {
            result
                .names
                .insert(name.split('/').next().unwrap_or(name).to_ascii_lowercase());
        }
    }
    for field in worktrees.split('\0') {
        if let Some(path) = field.strip_prefix("worktree ") {
            result.worktrees.push(PathBuf::from(path));
        }
    }
    Ok(result)
}
/// Candidate-specific probes avoid enumerating managed folders or retained names.
/// Git inventory remains subject to the shared runner's byte/deadline bounds.
pub(crate) fn candidate_occupied(
    processes: &Processes,
    repository: &crate::workspaces::RepositoryRecord,
    root: &Path,
    directory: &Path,
    folder: &str,
    name: &str,
    folder_saved: bool,
) -> Result<crate::workspaces::naming::Occupancy> {
    validate_repository(processes, repository)?;
    if !folder_saved && location(&root.join(folder))?.is_some() {
        return Ok(crate::workspaces::naming::Occupancy::RepositoryFolder);
    }
    if location(&directory.join(name))?.is_some() {
        return Ok(crate::workspaces::naming::Occupancy::WorkspaceName);
    }
    let path = Path::new(&repository.path);
    let reference = format!("refs/heads/workspace/{name}");
    if query_optional(processes, path, "refs/heads/workspace")?.is_some() {
        return Err(failure(
            "An existing workspace branch blocks the workspace/ namespace; rename it before reconciling",
        ));
    }
    let refs = run(
        processes,
        creation_command(path, &["for-each-ref", "--format=%(refname)", &reference]),
    )?;
    let worktrees = run(
        processes,
        creation_command(path, &["worktree", "list", "--porcelain", "-z"]),
    )?;
    let occupied = occupied_inventory(&refs, &worktrees, Default::default())?;
    if occupied.worktrees.iter().any(|p| directory.starts_with(p)) {
        return Err(failure(
            "A registered Git worktree occupies the managed repository folder or its ancestor; restore that location before reconciliation",
        ));
    }
    let wrapper = directory.join(name);
    Ok(
        if occupied.names.contains(name)
            || occupied
                .worktrees
                .iter()
                .any(|p| p.starts_with(&wrapper) || wrapper.starts_with(p))
        {
            crate::workspaces::naming::Occupancy::WorkspaceName
        } else {
            crate::workspaces::naming::Occupancy::Available
        },
    )
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
        validate_repository(&self.0, repository)?;
        let path = Path::new(&repository.path);
        let origin = query_optional(&self.0, path, "refs/remotes/origin/HEAD")?;
        let main = query_optional(&self.0, path, "refs/heads/main")?;
        let master = query_optional(&self.0, path, "refs/heads/master")?;
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
        validate_repository(&self.0, repository)?;
        let path = Path::new(&repository.path);
        let base = workspace
            .base
            .as_ref()
            .ok_or_else(|| failure("Workspace has no pinned commit"))?;
        let branch = query_optional(
            &self.0,
            path,
            &format!("refs/heads/{}", workspace.branch()?),
        )?;
        let owner = query_optional(&self.0, path, &ownership_ref(workspace))?;
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
                } else if location(Path::new(workspace.path()?))?.is_none() {
                    FolderArtifact::Reserved
                } else {
                    let inspected = inspect(&self.0, workspace.path()?);
                    if let Ok(actual) = inspected {
                        if actual.path == workspace.path()?
                            && actual.common_dir == repository.common_dir
                        {
                            let target = Path::new(workspace.path()?);
                            let head = query_optional(&self.0, target, "HEAD")?;
                            let symbolic = run_output(
                                &self.0,
                                creation_command(target, &["symbolic-ref", "--quiet", "HEAD"]),
                                None,
                            )?;
                            let clean = run_output(
                                &self.0,
                                creation_command(
                                    target,
                                    &["status", "--porcelain=v1", "-z", "--untracked-files=normal"],
                                ),
                                None,
                            )?;
                            if head.as_deref() == Some(&base.commit)
                                && symbolic.0
                                && symbolic.1
                                    == format!("refs/heads/{}\n", workspace.branch()?).as_bytes()
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
                    crate::private_state::check_private(directory).map_err(failure)?
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
        validate_repository(&self.0, repository)?;
        let base = workspace
            .base
            .as_ref()
            .ok_or_else(|| failure("Missing pinned base"))?;
        let input = format!(
            "start\ncreate refs/heads/{} {}\ncreate {} {}\nprepare\ncommit\n",
            workspace.branch()?,
            base.commit,
            ownership_ref(workspace),
            base.commit
        );
        let (success, output) = run_leased(
            &self.0,
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
        validate_repository(&self.0, repository)?;
        let (success, output) = run_leased(
            &self.0,
            workspace,
            creation_command(
                Path::new(&repository.path),
                &[
                    "worktree",
                    "add",
                    "--quiet",
                    "--",
                    workspace.path()?,
                    workspace.branch()?,
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
    processes: &Processes,
    workspace: &Workspace,
    mut command: Command,
    input: Option<&[u8]>,
) -> Result<(bool, Vec<u8>)> {
    let lease = open_execution_lock(workspace)?;
    lease.try_lock().map_err(|_| RepositoryError::new(ErrorCode::CleanupUnconfirmed,"Another owned Git effect holds this workspace's execution lease. Reconcile after it exits."))?;
    let descriptor = lease.as_raw_fd();
    // SAFETY: lease outlives spawn and execution. The child callback uses only
    // async-signal-safe fcntl, with no allocation, environment access or mutexes.
    // Inheritance retains the kernel lease even if the native owner crashes.
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
    let outcome = run_output(processes, command, input);
    drop(lease);
    if !LocalCreation(processes.clone()).quiescent(workspace)? {
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
    fn batch_inventory_keeps_nested_refs_missing_worktree_locations_and_occupied_folders() {
        let occupied = occupied_inventory("refs/heads/workspace/marta/subtask\nrefs/heads/workspace/pele\n", "worktree /repo\0HEAD abc\0\0worktree /state/repo/marta/checkout\0branch refs/heads/workspace/marta\0\0", std::collections::HashSet::from(["mia-hamm".into()])).unwrap();
        assert_eq!(
            occupied.names,
            std::collections::HashSet::from(["marta".into(), "pele".into(), "mia-hamm".into()])
        );
        assert!(
            occupied
                .worktrees
                .contains(&PathBuf::from("/state/repo/marta/checkout"))
        );
        assert!(occupied_inventory("refs/heads/workspace", "", Default::default()).is_err());
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
}
