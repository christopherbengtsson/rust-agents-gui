//! Exclusive native state lease. Never unlink or replace owner.lock.
use std::{
    fs::{self, File, OpenOptions},
    io,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::Path,
};
fn prepare_state(state: &Path) -> io::Result<()> {
    prepare_state_with(
        state,
        |path| {
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(path)
        },
        check_private,
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

pub(crate) use crate::private_state::{FileEvidence, check_private};

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

/// Socket-free exclusive state ownership; never unlink the lease inode.
pub(crate) struct StateLease {
    _file: Option<File>,
    #[cfg(test)]
    released: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
}
impl StateLease {
    /// Acquire the same lease inode without replacing its existing inode.
    pub(crate) fn acquire(state: &Path) -> io::Result<Self> {
        let lock = admit(state, prepare_state, owner_lock, acquire)?;
        Ok(Self {
            _file: Some(lock),
            #[cfg(test)]
            released: None,
        })
    }
}
#[cfg(test)]
impl StateLease {
    pub(crate) fn controlled(released: std::sync::Arc<std::sync::atomic::AtomicBool>) -> Self {
        Self {
            _file: None,
            released: Some(released),
        }
    }
}
#[cfg(test)]
impl Drop for StateLease {
    fn drop(&mut self) {
        if let Some(released) = &self.released {
            released.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

fn admit<L>(
    state: &Path,
    prepare: impl FnOnce(&Path) -> io::Result<()>,
    open: impl FnOnce(&Path) -> io::Result<L>,
    lock: impl FnOnce(&L) -> io::Result<()>,
) -> io::Result<L> {
    prepare(state)?;
    let lease = open(state)?;
    lock(&lease).map_err(|error| {
        if error.kind() == io::ErrorKind::WouldBlock {
            io::Error::new(error.kind(), format!("Another native core may own this state. Close its app and check status; never remove owner.lock: {error}"))
        } else { error }
    })?;
    Ok(lease)
}

fn acquire(lock: &File) -> io::Result<()> {
    lock.try_lock().map_err(|error| match error {
        std::fs::TryLockError::WouldBlock => io::ErrorKind::WouldBlock.into(),
        std::fs::TryLockError::Error(error) => error,
    })
}
#[cfg(test)]
mod tests;
