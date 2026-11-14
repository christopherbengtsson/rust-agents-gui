use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};
use workspace_core::{
    Core,
    connection::{ConnectionSlots, serve_connection},
    supervisor::SupervisorOwner,
};

fn main() -> io::Result<()> {
    let mut arguments = env::args_os().skip(1);
    let first = arguments.next().ok_or_else(|| {
        io::Error::other("usage: workspace-supervisor <private-application-state-directory>")
    })?;
    if first == "--protocol-version" {
        if arguments.next().is_some() {
            return Err(io::Error::other("protocol metadata takes no arguments"));
        }
        println!("{}", workspace_core::connection::PROTOCOL_VERSION);
        return Ok(());
    }
    let state = PathBuf::from(first);
    if arguments.next().is_some() {
        return Err(io::Error::other("expected exactly one state directory"));
    }
    // Set the process-wide mask before any threads or socket creation.
    workspace_core::supervisor::restrict_creation_permissions();
    let owner = SupervisorOwner::bind(&state)?;
    let core = Arc::new(Core::with_state(&state).map_err(io::Error::other)?);
    let slots = ConnectionSlots::new(8);
    println!("ready");
    io::stdout().flush()?;
    for stream in owner.listener.incoming() {
        let stream = stream?;
        let Some(permit) = slots.try_acquire() else {
            continue;
        };
        let core = core.clone();
        std::thread::spawn(move || {
            let _permit = permit;
            // Invalid/abandoned clients are isolated; no peer-controlled output is logged.
            let _ = serve_connection(&core, stream);
        });
    }
    Ok(())
}
