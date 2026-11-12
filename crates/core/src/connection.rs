//! Status-only local transport. Discovery and reconnection are added in ticket 02.

use std::{
    io::{self, BufRead, BufReader, Write},
    os::unix::net::UnixStream,
    path::PathBuf,
    time::Duration,
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{Core, SupervisorStatus};

const MAX_FRAME_BYTES: usize = 4096;
const IO_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Request {
    Status,
}

pub struct SupervisorClient {
    socket: PathBuf,
}

impl SupervisorClient {
    pub fn new(socket: PathBuf) -> Self {
        Self { socket }
    }

    pub fn status(&self) -> io::Result<SupervisorStatus> {
        let mut stream = UnixStream::connect(&self.socket)?;
        configure(&stream)?;
        write_frame(&mut stream, &Request::Status)?;
        read_frame(&mut stream)
    }
}

/// Serve one bounded observation, without granting the client execution ownership.
pub fn serve_connection(core: &Core, mut stream: UnixStream) -> io::Result<()> {
    configure(&stream)?;
    match read_frame::<Request>(&mut stream)? {
        Request::Status => write_frame(&mut stream, &core.status()),
    }
}

fn configure(stream: &UnixStream) -> io::Result<()> {
    stream.set_read_timeout(Some(IO_TIMEOUT))?;
    stream.set_write_timeout(Some(IO_TIMEOUT))
}

fn read_frame<T: DeserializeOwned>(stream: &mut UnixStream) -> io::Result<T> {
    let mut reader = BufReader::new(stream);
    let mut frame = Vec::new();
    // read_until alone would allow an unbounded peer-controlled allocation.
    loop {
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
