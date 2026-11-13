use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
use workspace_core::{Core, connection::serve_connection, supervisor::SupervisorOwner};

fn main() -> io::Result<()> {
    let mut arguments = env::args_os().skip(1);
    let state = PathBuf::from(arguments.next().ok_or_else(|| {
        io::Error::other("usage: workspace-supervisor <private-application-state-directory>")
    })?);
    if arguments.next().is_some() {
        return Err(io::Error::other("expected exactly one state directory"));
    }
    // Set the process-wide mask before any threads or socket creation.
    workspace_core::supervisor::restrict_creation_permissions();
    let owner = SupervisorOwner::bind(&state)?;
    let core = Arc::new(Core::with_state(&state));
    let active = Arc::new(AtomicUsize::new(0));
    println!("ready");
    io::stdout().flush()?;
    for stream in owner.listener.incoming() {
        let stream = stream?;
        if active.load(Ordering::SeqCst) >= 8 {
            continue;
        }
        active.fetch_add(1, Ordering::SeqCst);
        let (core, active) = (core.clone(), active.clone());
        std::thread::spawn(move || {
            // Invalid/abandoned clients are isolated; no peer-controlled output is logged.
            let _ = serve_connection(&core, stream);
            active.fetch_sub(1, Ordering::SeqCst);
        });
    }
    Ok(())
}
