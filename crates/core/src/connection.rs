//! Versioned, bounded observations; transport loss never replays actions.
use crate::{
    Core, SnapshotCursor, SupervisorSnapshot, SupervisorStatus,
    supervisor::{check_private, owner_present, prepare_state, socket_path},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{self, BufRead, BufReader, Read, Write},
    os::unix::{net::UnixStream, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};
const MAX_FRAME_BYTES: usize = 4096;
const IO_TIMEOUT: Duration = Duration::from_secs(2);
pub const PROTOCOL_VERSION: u32 = 1;

#[derive(Serialize, Deserialize)]
struct Request {
    protocol: u32,
    application_version: String,
    after: Option<SnapshotCursor>,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Response {
    Snapshot { snapshot: SupervisorSnapshot },
    Incompatible { message: String },
}

/// Desktop connection state lives in Rust core: initialize once, then only retrieve observations.
pub struct SupervisorConnection {
    state: PathBuf,
    executable: PathBuf,
    initialized: bool,
    cursor: Option<SnapshotCursor>,
}
impl SupervisorConnection {
    pub fn new(state: PathBuf, executable: PathBuf) -> Self {
        Self {
            state,
            executable,
            initialized: false,
            cursor: None,
        }
    }

    pub fn snapshot(&mut self) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with(&mut LocalDiscovery::new())
    }

    fn snapshot_with(&mut self, effects: &mut impl Discovery) -> io::Result<SupervisorSnapshot> {
        if !self.initialized {
            discover(&self.state, &self.executable, effects)?;
            self.initialized = true;
        }
        let snapshot = effects.snapshot(&self.state, self.cursor.clone())
            .map_err(|e| io::Error::new(e.kind(), format!(
                "Cannot read supervisor observations: {e}. Refresh to reconnect to this owner. If it has exited, reopen the desktop; no actions have been replayed.")))?;
        self.cursor = Some(snapshot.cursor.clone());
        Ok(snapshot)
    }
}

pub struct SupervisorClient {
    state: PathBuf,
}
impl SupervisorClient {
    pub fn new(state: PathBuf) -> Self {
        Self { state }
    }

    /// Opening the desktop may start an absent owner. Subsequent observations never start one.
    pub fn connect_or_start(state: PathBuf, executable: &Path) -> io::Result<Self> {
        discover(&state, executable, &mut LocalDiscovery::new())?;
        Ok(Self::new(state))
    }

    pub fn status(&self) -> io::Result<SupervisorStatus> {
        Ok(self.snapshot(None)?.status)
    }
    pub fn snapshot(&self, after: Option<SnapshotCursor>) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with_version(after, PROTOCOL_VERSION, env!("CARGO_PKG_VERSION"))
    }
    pub fn snapshot_with_version(
        &self,
        after: Option<SnapshotCursor>,
        protocol: u32,
        application_version: &str,
    ) -> io::Result<SupervisorSnapshot> {
        self.snapshot_with(after, protocol, application_version, &mut LocalClient)
    }

    fn snapshot_with(
        &self,
        after: Option<SnapshotCursor>,
        protocol: u32,
        application_version: &str,
        effects: &mut impl ClientEffects,
    ) -> io::Result<SupervisorSnapshot> {
        effects.check_private(&self.state, true)?;
        let socket = socket_path(&self.state);
        effects.check_private(&socket, false)?;
        let mut stream = effects.connect(&socket)?;
        configure(&stream)?;
        write_frame(
            &mut stream,
            &Request {
                protocol,
                application_version: application_version.into(),
                after,
            },
        )?;
        match read_frame_with_clock(&mut stream, || effects.now())? {
            Response::Snapshot { snapshot } => Ok(snapshot),
            Response::Incompatible { message } => {
                Err(io::Error::new(io::ErrorKind::InvalidData, message))
            }
        }
    }
}

// Client policy and protocol stay above this seam; adapters supply only local I/O outcomes.
trait ClientEffects {
    type Stream: FrameStream;
    fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()>;
    fn connect(&mut self, socket: &Path) -> io::Result<Self::Stream>;
    fn now(&mut self) -> Instant;
}

struct LocalClient;
impl ClientEffects for LocalClient {
    type Stream = UnixStream;
    fn check_private(&mut self, path: &Path, directory: bool) -> io::Result<()> {
        check_private(path, directory)
    }
    fn connect(&mut self, socket: &Path) -> io::Result<Self::Stream> {
        UnixStream::connect(socket)
    }
    fn now(&mut self) -> Instant {
        Instant::now()
    }
}

// Discovery policy remains here; the local implementation supplies OS outcomes.
trait Discovery {
    fn prepare(&mut self, state: &Path) -> io::Result<()>;
    fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot>;
    fn owner_present(&mut self, state: &Path) -> io::Result<bool>;
    fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()>;
    fn elapsed(&self) -> Duration;
    fn wait(&mut self, duration: Duration);
}

struct LocalDiscovery(Instant);
impl LocalDiscovery {
    fn new() -> Self {
        Self(Instant::now())
    }
}
impl Discovery for LocalDiscovery {
    fn prepare(&mut self, state: &Path) -> io::Result<()> {
        prepare_state(state)
    }
    fn snapshot(
        &mut self,
        state: &Path,
        after: Option<SnapshotCursor>,
    ) -> io::Result<SupervisorSnapshot> {
        SupervisorClient::new(state.to_owned()).snapshot(after)
    }
    fn owner_present(&mut self, state: &Path) -> io::Result<bool> {
        owner_present(state)
    }
    fn start(&mut self, executable: &Path, state: &Path) -> io::Result<()> {
        let mut child = supervisor_command(executable, state).spawn().map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Cannot start supervisor {}: {e}. Rebuild or reinstall the application.",
                    executable.display()
                ),
            )
        })?;
        // Reap on every exit path without tying the supervisor lifetime to this client.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
    fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
    fn wait(&mut self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

fn supervisor_command(executable: &Path, state: &Path) -> Command {
    let mut command = Command::new(executable);
    command
        .arg(state)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0);
    command
}

fn discover(state: &Path, executable: &Path, effects: &mut impl Discovery) -> io::Result<()> {
    effects.prepare(state)?;
    match effects.snapshot(state, None) {
        Ok(_) => return Ok(()),
        Err(e)
            if !matches!(
                e.kind(),
                io::ErrorKind::NotFound | io::ErrorKind::ConnectionRefused
            ) =>
        {
            return Err(e);
        }
        Err(_) => {}
    }
    if !effects.owner_present(state)? {
        effects.start(executable, state)?;
    }
    let deadline = effects.elapsed() + Duration::from_secs(5);
    loop {
        match effects.snapshot(state, None) {
            Ok(_) => {
                return Ok(());
            }
            Err(e)
                if !matches!(
                    e.kind(),
                    io::ErrorKind::NotFound
                        | io::ErrorKind::ConnectionRefused
                        | io::ErrorKind::UnexpectedEof
                ) =>
            {
                return Err(e);
            }
            Err(_) => {}
        }
        if effects.elapsed() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Supervisor did not become available. An owner may still hold this state; check the installed supervisor and state permissions, then reopen. Do not remove the owner lock.",
            ));
        }
        effects.wait(Duration::from_millis(25));
    }
}

pub fn serve_connection(core: &Core, mut stream: UnixStream) -> io::Result<()> {
    serve_stream(core, &mut stream)
}

fn serve_stream(core: &Core, stream: &mut impl FrameStream) -> io::Result<()> {
    configure(stream)?;
    let request: Request = read_frame(stream)?;
    if request.protocol != PROTOCOL_VERSION
        || request.application_version != env!("CARGO_PKG_VERSION")
    {
        return write_frame(
            stream,
            &Response::Incompatible {
                message: format!(
                    "Incompatible desktop/supervisor versions (client protocol {} / app {}, supervisor protocol {} / app {}). Install matching desktop and supervisor builds; close the old owner through its matching desktop before reopening.",
                    request.protocol,
                    request.application_version,
                    PROTOCOL_VERSION,
                    env!("CARGO_PKG_VERSION")
                ),
            },
        );
    }
    write_frame(
        stream,
        &Response::Snapshot {
            snapshot: core.snapshot(request.after),
        },
    )
}

trait FrameStream: Read + Write {
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()>;
}
impl FrameStream for UnixStream {
    fn set_read_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        UnixStream::set_read_timeout(self, timeout)
    }
    fn set_write_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        UnixStream::set_write_timeout(self, timeout)
    }
}

fn configure(stream: &impl FrameStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))
}

fn read_frame<T: DeserializeOwned>(stream: &mut impl FrameStream) -> io::Result<T> {
    read_frame_with_clock(stream, Instant::now)
}

fn read_frame_with_clock<T: DeserializeOwned>(
    stream: &mut impl FrameStream,
    mut now: impl FnMut() -> Instant,
) -> io::Result<T> {
    let deadline = now() + IO_TIMEOUT;
    let mut reader = BufReader::new(stream);
    let mut frame = Vec::new();
    // read_until alone would allow an unbounded peer-controlled allocation.
    loop {
        let remaining = deadline.checked_duration_since(now()).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "Supervisor frame deadline exceeded",
            )
        })?;
        reader.get_ref().set_read_timeout(Some(remaining))?;
        let available = reader.fill_buf()?;
        if available.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "incomplete supervisor frame",
            ));
        }
        let length = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |end| end + 1);
        if frame.len() + length > MAX_FRAME_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "supervisor frame exceeds limit",
            ));
        }
        let complete = available[length - 1] == b'\n';
        frame.extend_from_slice(&available[..length]);
        reader.consume(length);
        if complete {
            return serde_json::from_slice(&frame).map_err(io::Error::other);
        }
    }
}

fn write_frame(stream: &mut impl Write, value: &impl Serialize) -> io::Result<()> {
    let mut frame = serde_json::to_vec(value)?;
    frame.push(b'\n');
    if frame.len() > MAX_FRAME_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "supervisor frame exceeds limit",
        ));
    }
    stream.write_all(&frame)
}

#[cfg(test)]
mod tests;
