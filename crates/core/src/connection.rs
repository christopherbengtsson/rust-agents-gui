//! Versioned, bounded observations; transport loss never replays actions.
use crate::{
    Core, SnapshotCursor, SupervisorSnapshot, SupervisorStatus,
    supervisor::{check_private, owner_present, prepare_state, socket_path},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    io::{self, BufRead, BufReader, Write},
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
    client: Option<SupervisorClient>,
    cursor: Option<SnapshotCursor>,
}
impl SupervisorConnection {
    pub fn new(state: PathBuf, executable: PathBuf) -> Self {
        Self {
            state,
            executable,
            client: None,
            cursor: None,
        }
    }

    pub fn snapshot(&mut self) -> io::Result<SupervisorSnapshot> {
        if self.client.is_none() {
            self.client = Some(SupervisorClient::connect_or_start(
                self.state.clone(),
                &self.executable,
            )?);
        }
        let snapshot = self.client.as_ref().unwrap().snapshot(self.cursor.clone())
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
        prepare_state(&state)?;
        let client = Self::new(state);
        match client.status() {
            Ok(_) => return Ok(client),
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
        if !owner_present(&client.state)? {
            let mut child = Command::new(executable).arg(&client.state)
                .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .process_group(0).spawn().map_err(|e| io::Error::new(e.kind(),
                    format!("Cannot start supervisor {}: {e}. Rebuild or reinstall the application.", executable.display())))?;
            // Reap on every exit path without tying the supervisor lifetime to this client.
            std::thread::spawn(move || {
                let _ = child.wait();
            });
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            match client.status() {
                Ok(_) => {
                    return Ok(client);
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
            if Instant::now() >= deadline {
                return Err(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "Supervisor did not become available. An owner may still hold this state; check the installed supervisor and state permissions, then reopen. Do not remove the owner lock.",
                ));
            }
            std::thread::sleep(Duration::from_millis(25));
        }
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
        check_private(&self.state, true)?;
        let socket = socket_path(&self.state);
        check_private(&socket, false)?;
        let mut stream = UnixStream::connect(socket)?;
        configure(&stream)?;
        write_frame(
            &mut stream,
            &Request {
                protocol,
                application_version: application_version.into(),
                after,
            },
        )?;
        match read_frame(&mut stream)? {
            Response::Snapshot { snapshot } => Ok(snapshot),
            Response::Incompatible { message } => {
                Err(io::Error::new(io::ErrorKind::InvalidData, message))
            }
        }
    }
}

pub fn serve_connection(core: &Core, mut stream: UnixStream) -> io::Result<()> {
    configure(&stream)?;
    let request: Request = read_frame(&mut stream)?;
    if request.protocol != PROTOCOL_VERSION
        || request.application_version != env!("CARGO_PKG_VERSION")
    {
        return write_frame(
            &mut stream,
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
        &mut stream,
        &Response::Snapshot {
            snapshot: core.snapshot(request.after),
        },
    )
}

fn configure(stream: &UnixStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))
}

fn read_frame<T: DeserializeOwned>(stream: &mut UnixStream) -> io::Result<T> {
    let deadline = std::time::Instant::now() + IO_TIMEOUT;
    let mut reader = BufReader::new(stream);
    let mut frame = Vec::new();
    // read_until alone would allow an unbounded peer-controlled allocation.
    loop {
        let remaining = deadline
            .checked_duration_since(std::time::Instant::now())
            .ok_or_else(|| {
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

fn write_frame(stream: &mut UnixStream, value: &impl Serialize) -> io::Result<()> {
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
