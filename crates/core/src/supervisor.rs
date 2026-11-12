//! Exclusive local owner admission. Only the lock holder may replace a stale socket.
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::unix::{
        fs::{DirBuilderExt, FileTypeExt, MetadataExt, OpenOptionsExt, PermissionsExt},
        net::UnixListener,
    },
    path::{Path, PathBuf},
};

pub fn prepare_state(state: &Path) -> io::Result<()> {
    match fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(state)
    {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    check_private(state, true)
}

pub(crate) fn check_private(path: &Path, directory: bool) -> io::Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    // SAFETY: geteuid has no preconditions and does not mutate process state.
    let uid = unsafe { libc::geteuid() };
    if metadata.uid() != uid
        || metadata.permissions().mode() & 0o077 != 0
        || (directory && !metadata.is_dir())
        || (!directory && !metadata.file_type().is_socket())
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "{} must be owned by this user, private (directory 700 / socket 600), and not a symlink. Select a private application state directory.",
                path.display()
            ),
        ));
    }
    Ok(())
}

fn owner_lock(state: &Path) -> io::Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(state.join("owner.lock"))?;
    let metadata = file.metadata()?;
    // SAFETY: geteuid has no preconditions.
    if !metadata.is_file()
        || metadata.uid() != unsafe { libc::geteuid() }
        || metadata.permissions().mode() & 0o077 != 0
        || metadata.nlink() != 1
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Owner lock must be a private regular file owned by this user",
        ));
    }
    Ok(file)
}

pub(crate) fn owner_present(state: &Path) -> io::Result<bool> {
    let file = owner_lock(state)?;
    match file.try_lock() {
        Ok(()) => Ok(false),
        Err(std::fs::TryLockError::WouldBlock) => Ok(true),
        Err(std::fs::TryLockError::Error(e)) => Err(e),
    }
}

pub struct SupervisorOwner {
    // Never delete the lock file: contenders must always lock the same inode.
    _lock: File,
    pub listener: UnixListener,
}
impl SupervisorOwner {
    pub fn bind(state: &Path) -> io::Result<Self> {
        prepare_state(state)?;
        let lock = owner_lock(state)?;
        lock.try_lock().map_err(|e| io::Error::new(io::ErrorKind::AddrInUse,
            format!("Another supervisor owns this application state. Reconnect to it; do not remove owner.lock: {e}")))?;
        let socket = socket_path(state);
        match fs::symlink_metadata(&socket) {
            Ok(_) => {
                check_private(&socket, false)?;
                fs::remove_file(&socket)?;
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let listener = UnixListener::bind(&socket).map_err(|e| io::Error::new(e.kind(),
            format!("Cannot bind {}: {e}. Select a shorter private state path if the Unix socket path is too long.", socket.display())))?;
        fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
        Ok(Self {
            _lock: lock,
            listener,
        })
    }
}

pub fn socket_path(state: &Path) -> PathBuf {
    state.join("supervisor.sock")
}

/// Call only at supervisor startup, before creating threads. Unix umask is process-wide.
pub fn restrict_creation_permissions() {
    // SAFETY: umask has no memory preconditions; startup owns the process policy.
    unsafe {
        libc::umask(0o077);
    }
}
