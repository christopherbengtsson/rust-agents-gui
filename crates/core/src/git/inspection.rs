//! Read-only inspection mechanics using the shared bounded Git runner.
use super::*;
use crate::workspaces::{
    CONTENT_BYTES, FileEvidence, InspectionEffects, RepositoryRecord, UnsupportedContent,
};
use std::{
    ffi::{CString, OsStr},
    fs::File,
    os::{
        fd::FromRawFd,
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
};

pub(crate) struct LocalInspection;

fn inspection_command(root: &Path, args: &[&str], path: Option<&[u8]>) -> Command {
    let mut command = base_command(root);
    command
        .env("LC_ALL", "C")
        .args([
            "--literal-pathspecs",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.untrackedCache=false",
            "-c",
            "diff.algorithm=myers",
            "-c",
            "diff.suppressBlankEmpty=false",
        ])
        .args(args);
    if let Some(path) = path {
        command.arg("--").arg(OsStr::from_bytes(path));
    }
    command
}
fn diff_command(root: &Path, head: &str, path: &[u8]) -> Command {
    inspection_command(
        root,
        &[
            "diff",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--no-color",
            "--no-relative",
            "--unified=3",
            "--no-indent-heuristic",
            "--src-prefix=a/",
            "--dst-prefix=b/",
            "--output-indicator-new=+",
            "--output-indicator-old=-",
            "--output-indicator-context= ",
            head,
        ],
        Some(path),
    )
}
fn bytes(command: Command) -> Result<Vec<u8>> {
    let (success, output) = run_output(command, None)?;
    if success {
        Ok(output)
    } else {
        Err(failure(String::from_utf8_lossy(&output)))
    }
}
fn fs_error(error: impl std::fmt::Display) -> RepositoryError {
    failure(format!("Cannot read workspace file: {error}"))
}

// Traverse relative to held directory descriptors. No component follows symlinks.
fn parent(root: &Path, path: &[u8]) -> std::io::Result<(File, CString)> {
    let mut directory = File::options()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(root)?;
    let mut components = path.split(|b| *b == b'/').peekable();
    while let Some(component) = components.next() {
        if component.is_empty() || component == b"." || component == b".." {
            return Err(std::io::Error::new(
                ErrorKind::InvalidInput,
                "Unsafe relative path",
            ));
        }
        let component =
            CString::new(component).map_err(|e| std::io::Error::new(ErrorKind::InvalidInput, e))?;
        if components.peek().is_none() {
            return Ok((directory, component));
        }
        // SAFETY: directory is live; component is a NUL-terminated relative name.
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                component.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }
        // SAFETY: openat returned a fresh owned descriptor.
        directory = unsafe { File::from_raw_fd(fd) };
    }
    Err(std::io::Error::new(
        ErrorKind::InvalidInput,
        "Empty relative path",
    ))
}
fn stat(directory: &File, name: &CString) -> Result<Option<libc::stat>> {
    let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: metadata points to writable stat storage; directory and name are live.
    if unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            metadata.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        let error = std::io::Error::last_os_error();
        if error.kind() == ErrorKind::NotFound {
            return Ok(None);
        }
        return Err(fs_error(error));
    }
    // SAFETY: successful fstatat initialized metadata.
    Ok(Some(unsafe { metadata.assume_init() }))
}
fn evidence(metadata: &std::fs::Metadata) -> FileEvidence {
    FileEvidence::Regular {
        device: metadata.dev(),
        inode: metadata.ino(),
        size: metadata.len(),
        modified: (metadata.mtime(), metadata.mtime_nsec()),
        changed: (metadata.ctime(), metadata.ctime_nsec()),
    }
}
fn open_regular(root: &Path, path: &[u8]) -> Result<std::result::Result<File, FileEvidence>> {
    let (directory, name) = match parent(root, path) {
        Ok(value) => value,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Err(FileEvidence::Missing)),
        Err(error) => return Err(fs_error(error)),
    };
    let Some(metadata) = stat(&directory, &name)? else {
        return Ok(Err(FileEvidence::Missing));
    };
    let kind = metadata.st_mode & libc::S_IFMT;
    if kind != libc::S_IFREG {
        return Ok(Err(FileEvidence::Unsupported(if kind == libc::S_IFLNK {
            UnsupportedContent::Symlink
        } else {
            UnsupportedContent::SpecialFile
        })));
    }
    if metadata.st_size < 0
        || u64::try_from(metadata.st_size).map_err(fs_error)? > CONTENT_BYTES as u64
    {
        return Ok(Err(FileEvidence::Unsupported(
            UnsupportedContent::Oversized,
        )));
    }
    // O_NONBLOCK avoids a FIFO substitution race; fstat below rejects changed types.
    // SAFETY: held directory/name are live; a successful descriptor is taken below.
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err(fs_error(std::io::Error::last_os_error()));
    }
    // SAFETY: fd is fresh and owned.
    let file = unsafe { File::from_raw_fd(fd) };
    let actual = file.metadata().map_err(fs_error)?;
    if !actual.is_file()
        || actual.dev() != u64::try_from(metadata.st_dev).map_err(fs_error)?
        || actual.ino() != metadata.st_ino
    {
        return Err(fs_error(
            "File changed while opening; refresh the comparison",
        ));
    }
    Ok(Ok(file))
}
impl InspectionEffects for LocalInspection {
    fn validate(&mut self, workspace: &Workspace, repository: &RepositoryRecord) -> Result<()> {
        let actual = inspect(workspace.path()?)?;
        if actual.path != workspace.path()? || actual.common_dir != repository.common_dir {
            return Err(fs_error(
                "Saved workspace target no longer matches its registered Git identity",
            ));
        }
        Ok(())
    }
    fn head(&mut self, path: &Path) -> Result<String> {
        let output = bytes(inspection_command(
            path,
            &["rev-parse", "--verify", "--end-of-options", "HEAD^{commit}"],
            None,
        ))?;
        let text = std::str::from_utf8(&output)
            .map_err(fs_error)?
            .trim_end_matches('\n');
        if !matches!(text.len(), 40 | 64) || !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(failure("Invalid HEAD commit"));
        }
        Ok(text.into())
    }
    fn status(&mut self, path: &Path) -> Result<Vec<u8>> {
        bytes(inspection_command(
            path,
            &[
                "status",
                "--porcelain=v2",
                "-z",
                "--untracked-files=all",
                "--ignore-submodules=none",
                "--no-renames",
            ],
            None,
        ))
    }
    fn evidence(&mut self, root: &Path, path: &[u8]) -> Result<FileEvidence> {
        Ok(match open_regular(root, path)? {
            Ok(file) => evidence(&file.metadata().map_err(fs_error)?),
            Err(value) => value,
        })
    }
    fn read(&mut self, root: &Path, path: &[u8]) -> Result<Vec<u8>> {
        let mut file = open_regular(root, path)?
            .map_err(|_| fs_error("File is unavailable as bounded regular text; refresh"))?;
        let before = evidence(&file.metadata().map_err(fs_error)?);
        let mut data = Vec::new();
        (&mut file)
            .take(u64::try_from(CONTENT_BYTES + 1).map_err(fs_error)?)
            .read_to_end(&mut data)
            .map_err(fs_error)?;
        if evidence(&file.metadata().map_err(fs_error)?) != before {
            return Err(fs_error("File changed during read; refresh"));
        }
        Ok(data)
    }
    fn diff(&mut self, root: &Path, head: &str, path: &[u8]) -> Result<Vec<u8>> {
        bytes(diff_command(root, head, path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inspection_commands_preserve_literal_raw_paths_and_isolated_environment() {
        let command = diff_command(
            Path::new("/saved/workspace"),
            &"a".repeat(40),
            b":(glob)*\xff",
        );
        let args: Vec<_> = command.get_args().collect();
        for flag in [
            "--literal-pathspecs",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "--no-color",
            "--unified=3",
        ] {
            assert!(args.contains(&OsStr::new(flag)));
        }
        assert!(args.contains(&OsStr::new("--no-optional-locks")));
        assert_eq!(args.last().unwrap().as_bytes(), b":(glob)*\xff");
        assert!(
            command
                .get_envs()
                .any(|(key, value)| key == "GIT_OPTIONAL_LOCKS" && value == Some(OsStr::new("0")))
        );
        assert!(
            command
                .get_envs()
                .any(|(key, value)| key == "LC_ALL" && value == Some(OsStr::new("C")))
        );
    }
}
