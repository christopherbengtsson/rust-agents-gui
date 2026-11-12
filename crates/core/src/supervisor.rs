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
    prepare_state_with(
        state,
        |path| {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(path)
        },
        |path| check_private(path, true),
    )
}

fn prepare_state_with(
    state: &Path,
    create: impl FnOnce(&Path) -> io::Result<()>,
    check: impl FnOnce(&Path) -> io::Result<()>,
) -> io::Result<()> {
    match create(state) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e),
    }
    check(state)
}

pub(crate) fn check_private(path: &Path, directory: bool) -> io::Result<()> {
    validate_private(
        path,
        &FileEvidence::from(fs::symlink_metadata(path)?),
        directory,
    )
}

struct FileEvidence {
    owned_by_user: bool,
    mode: u32,
    directory: bool,
    socket: bool,
    regular: bool,
    links: u64,
}
impl From<fs::Metadata> for FileEvidence {
    fn from(metadata: fs::Metadata) -> Self {
        Self {
            // SAFETY: geteuid has no preconditions and does not mutate process state.
            owned_by_user: metadata.uid() == unsafe { libc::geteuid() },
            mode: metadata.permissions().mode(),
            directory: metadata.is_dir(),
            socket: metadata.file_type().is_socket(),
            regular: metadata.is_file(),
            links: metadata.nlink(),
        }
    }
}

fn validate_private(path: &Path, metadata: &FileEvidence, directory: bool) -> io::Result<()> {
    if !metadata.owned_by_user
        || metadata.mode & 0o077 != 0
        || (directory && !metadata.directory)
        || (!directory && !metadata.socket)
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
    validate_lock(&FileEvidence::from(file.metadata()?))?;
    Ok(file)
}

fn validate_lock(metadata: &FileEvidence) -> io::Result<()> {
    if !metadata.regular
        || !metadata.owned_by_user
        || metadata.mode & 0o077 != 0
        || metadata.links != 1
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "Owner lock must be a private regular file owned by this user",
        ));
    }
    Ok(())
}

pub(crate) fn owner_present(state: &Path) -> io::Result<bool> {
    probe_owner(state, &mut LocalOwner)
}

fn probe_owner(state: &Path, effects: &mut impl OwnerEffects) -> io::Result<bool> {
    let lock = effects.open_lock(state)?;
    match effects.acquire(&lock) {
        Ok(()) => Ok(false),
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(true),
        Err(e) => Err(e),
    }
}

pub struct SupervisorOwner {
    // Never delete the lock file: contenders must always lock the same inode.
    _lock: File,
    pub listener: UnixListener,
}
impl SupervisorOwner {
    pub fn bind(state: &Path) -> io::Result<Self> {
        let (lock, listener) = admit_owner(state, &mut LocalOwner)?;
        Ok(Self {
            _lock: lock,
            listener,
        })
    }
}

trait OwnerEffects {
    type Lock;
    type Listener;
    fn prepare(&mut self, state: &Path) -> io::Result<()>;
    fn open_lock(&mut self, state: &Path) -> io::Result<Self::Lock>;
    fn acquire(&mut self, lock: &Self::Lock) -> io::Result<()>;
    fn socket_info(&mut self, socket: &Path) -> io::Result<FileEvidence>;
    fn remove_socket(&mut self, socket: &Path) -> io::Result<()>;
    fn bind_socket(&mut self, socket: &Path) -> io::Result<Self::Listener>;
    fn make_private(&mut self, socket: &Path) -> io::Result<()>;
}

struct LocalOwner;
impl OwnerEffects for LocalOwner {
    type Lock = File;
    type Listener = UnixListener;
    fn prepare(&mut self, state: &Path) -> io::Result<()> {
        prepare_state(state)
    }
    fn open_lock(&mut self, state: &Path) -> io::Result<File> {
        owner_lock(state)
    }
    fn acquire(&mut self, lock: &File) -> io::Result<()> {
        lock.try_lock().map_err(|error| match error {
            std::fs::TryLockError::WouldBlock => io::ErrorKind::WouldBlock.into(),
            std::fs::TryLockError::Error(error) => error,
        })
    }
    fn socket_info(&mut self, socket: &Path) -> io::Result<FileEvidence> {
        Ok(fs::symlink_metadata(socket)?.into())
    }
    fn remove_socket(&mut self, socket: &Path) -> io::Result<()> {
        fs::remove_file(socket)
    }
    fn bind_socket(&mut self, socket: &Path) -> io::Result<UnixListener> {
        UnixListener::bind(socket)
    }
    fn make_private(&mut self, socket: &Path) -> io::Result<()> {
        fs::set_permissions(socket, fs::Permissions::from_mode(0o600))
    }
}

fn admit_owner<E: OwnerEffects>(
    state: &Path,
    effects: &mut E,
) -> io::Result<(E::Lock, E::Listener)> {
    effects.prepare(state)?;
    let lock = effects.open_lock(state)?;
    effects.acquire(&lock).map_err(|e| io::Error::new(io::ErrorKind::AddrInUse,
            format!("Another supervisor owns this application state. Reconnect to it; do not remove owner.lock: {e}")))?;
    let socket = socket_path(state);
    match effects.socket_info(&socket) {
        Ok(metadata) => {
            validate_private(&socket, &metadata, false)?;
            effects.remove_socket(&socket)?;
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    let listener = effects.bind_socket(&socket).map_err(|e| io::Error::new(e.kind(),
            format!("Cannot bind {}: {e}. Select a shorter private state path if the Unix socket path is too long.", socket.display())))?;
    effects.make_private(&socket)?;
    Ok((lock, listener))
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

#[cfg(test)]
mod tests;
