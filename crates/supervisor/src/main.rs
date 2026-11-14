use std::{
    env,
    io::{self, Write},
    path::PathBuf,
    sync::Arc,
};
use workspace_core::{
    Core,
    connection::{ConnectionServer, ConnectionSlots},
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
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()?;
    runtime.block_on(run(owner, core))
}
async fn run(owner: SupervisorOwner, core: Arc<Core>) -> io::Result<()> {
    owner.listener.set_nonblocking(true)?;
    let listener = tokio::net::UnixListener::from_std(owner.listener.try_clone()?)?;
    let server = Arc::new(ConnectionServer::new(core));
    let cancel = tokio_util::sync::CancellationToken::new();
    let slots = ConnectionSlots::new(8);
    let mut handlers = tokio::task::JoinSet::new();
    println!("ready");
    io::stdout().flush()?;
    let outcome = 'accept: loop {
        // Ready accepts must not starve completion collection and grow the task set.
        while let Some(result) = handlers.try_join_next() {
            if let Err(error) = result {
                break 'accept Err(io::Error::other(format!(
                    "Supervisor handler failed: {error}"
                )));
            }
        }
        let incoming = if handlers.is_empty() {
            listener.accept().await
        } else {
            let accept = std::pin::pin!(listener.accept());
            let completion = std::pin::pin!(handlers.join_next());
            match futures_util::future::select(accept, completion).await {
                futures_util::future::Either::Left((incoming, _)) => incoming,
                futures_util::future::Either::Right((Some(Err(error)), _)) => {
                    break Err(io::Error::other(format!(
                        "Supervisor handler failed: {error}"
                    )));
                }
                futures_util::future::Either::Right((_, _)) => continue,
            }
        };
        let (stream, _) = match incoming {
            Ok(incoming) => incoming,
            Err(error) => break Err(error),
        };
        let Some(permit) = slots.try_acquire() else {
            continue;
        };
        let server = server.clone();
        let cancel = cancel.clone();
        handlers.spawn(async move {
            let _permit = permit;
            // Invalid/abandoned clients are isolated; no peer-controlled output is logged.
            server.serve(stream, &cancel).await
        });
    };
    cancel.cancel();
    while handlers.join_next().await.is_some() {}
    server.finish().await?;
    drop(owner); // Exclusive owner lock remains held through handler/blocking completion.
    outcome
}
