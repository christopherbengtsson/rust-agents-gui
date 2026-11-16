//! Shared owner-private file validation for the native host and Git effects.
use std::{
    fs, io,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};
pub(crate) fn check_private(path: &Path) -> io::Result<()> {
    validate_private(path, &FileEvidence::from(fs::symlink_metadata(path)?))
}

pub(crate) struct FileEvidence {
    pub(crate) owned_by_user: bool,
    pub(crate) mode: u32,
    pub(crate) directory: bool,
    pub(crate) regular: bool,
    pub(crate) links: u64,
}
impl From<fs::Metadata> for FileEvidence {
    fn from(metadata: fs::Metadata) -> Self {
        Self {
            // SAFETY: geteuid has no preconditions and does not mutate process state.
            owned_by_user: metadata.uid() == unsafe { libc::geteuid() },
            mode: metadata.permissions().mode(),
            directory: metadata.is_dir(),
            regular: metadata.is_file(),
            links: metadata.nlink(),
        }
    }
}

pub(crate) fn validate_private(path: &Path, metadata: &FileEvidence) -> io::Result<()> {
    if !metadata.owned_by_user || metadata.mode & 0o077 != 0 || !metadata.directory {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!(
                "{} must be owned by this user, private (directory 700), and not a symlink. Select a private application state directory.",
                path.display()
            ),
        ));
    }
    Ok(())
}
