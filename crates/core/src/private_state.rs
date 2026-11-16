//! Shared owner-private file validation for the incumbent host and Git effects.
use std::{
    fs, io,
    os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt},
    path::Path,
};
pub(crate) fn check_private(path: &Path, directory: bool) -> io::Result<()> {
    validate_private(
        path,
        &FileEvidence::from(fs::symlink_metadata(path)?),
        directory,
    )
}

pub(crate) struct FileEvidence {
    pub(crate) owned_by_user: bool,
    pub(crate) mode: u32,
    pub(crate) directory: bool,
    pub(crate) socket: bool,
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
            socket: metadata.file_type().is_socket(),
            regular: metadata.is_file(),
            links: metadata.nlink(),
        }
    }
}

pub(crate) fn validate_private(
    path: &Path,
    metadata: &FileEvidence,
    directory: bool,
) -> io::Result<()> {
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
