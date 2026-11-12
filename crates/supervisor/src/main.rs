use std::{
    env, fs,
    io::{self, Write},
    os::unix::{fs::PermissionsExt, net::UnixListener},
    path::PathBuf,
};
use workspace_core::{Core, connection::serve_connection};

fn main() -> io::Result<()> {
    let mut arguments = env::args_os().skip(1);
    let socket = PathBuf::from(arguments.next().ok_or_else(|| {
        io::Error::other("usage: workspace-supervisor <socket-in-private-directory>")
    })?);
    if arguments.next().is_some() {
        return Err(io::Error::other("expected exactly one socket path"));
    }
    let parent = socket
        .parent()
        .ok_or_else(|| io::Error::other("socket needs a private parent directory"))?;
    let metadata = fs::symlink_metadata(parent)?;
    if !metadata.is_dir() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(io::Error::other(
            "socket parent must be a private directory (chmod 700)",
        ));
    }
    // Never unlink an occupied/stale endpoint to take ownership.
    let listener = UnixListener::bind(&socket)?;
    fs::set_permissions(&socket, fs::Permissions::from_mode(0o600))?;
    let core = Core::new();
    println!("ready");
    io::stdout().flush()?;
    for connection in listener.incoming() {
        match connection.and_then(|stream| serve_connection(&core, stream)) {
            Ok(()) => {}
            Err(error) => eprintln!("supervisor connection failed: {error}"),
        }
    }
    Ok(())
}
