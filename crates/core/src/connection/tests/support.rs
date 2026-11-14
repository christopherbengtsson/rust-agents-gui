//! Controlled byte streams; no sockets, filesystem, storage or processes.
use super::*;
use std::{
    pin::Pin,
    task::{Context, Poll, Waker},
};
use tokio::io::ReadBuf;
#[derive(Default)]
pub(in crate::connection) struct State {
    pub input: VecDeque<u8>,
    pub output: Vec<u8>,
    pub eof: bool,
    pub read_error: bool,
    pub write_after: Option<usize>,
    pub chunk: Option<usize>,
    pub drops: usize,
    pub reads: usize,
    pub writes: usize,
    pub reader: Option<Waker>,
}
#[derive(Clone, Default)]
pub(in crate::connection) struct Trace(pub Arc<Mutex<State>>);
impl Trace {
    pub fn feed(&self, bytes: &[u8]) {
        let mut state = self.0.lock().unwrap();
        state.input.extend(bytes);
        if let Some(waker) = state.reader.take() {
            waker.wake();
        }
    }
    pub fn output(&self) -> Vec<u8> {
        self.0.lock().unwrap().output.clone()
    }
}
pub(in crate::connection) struct Stream(pub Trace);
impl Stream {
    pub fn new(input: Vec<u8>) -> Self {
        let trace = Trace::default();
        {
            let mut state = trace.0.lock().unwrap();
            state.input = input.into();
            state.eof = true;
        }
        Self(trace)
    }
    pub fn pending() -> Self {
        Self(Trace::default())
    }
}
impl AsyncRead for Stream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut state = self.0.0.lock().unwrap();
        state.reads += 1;
        if state.read_error {
            return Poll::Ready(Err(io::ErrorKind::ConnectionReset.into()));
        }
        if state.input.is_empty() && !state.eof {
            state.reader = Some(cx.waker().clone());
            return Poll::Pending;
        }
        let length = buf
            .remaining()
            .min(state.input.len())
            .min(state.chunk.unwrap_or(usize::MAX));
        for _ in 0..length {
            buf.put_slice(&[state.input.pop_front().unwrap()]);
        }
        Poll::Ready(Ok(()))
    }
}
impl AsyncWrite for Stream {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut state = self.0.0.lock().unwrap();
        state.writes += 1;
        let remaining = state
            .write_after
            .map(|limit| limit.saturating_sub(state.output.len()));
        if remaining == Some(0) {
            return Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()));
        }
        let length = bytes.len().min(remaining.unwrap_or(usize::MAX));
        state.output.extend_from_slice(&bytes[..length]);
        Poll::Ready(Ok(length))
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        self.0.0.lock().unwrap().drops += 1;
    }
}
pub(in crate::connection) fn wire(value: &impl Serialize) -> Vec<u8> {
    let payload = serde_json::to_vec(value).unwrap();
    let mut frame = u32::try_from(payload.len()).unwrap().to_be_bytes().to_vec();
    frame.extend(payload);
    frame
}
pub(in crate::connection) fn frames<T: DeserializeOwned>(mut bytes: &[u8]) -> Vec<T> {
    let mut values = vec![];
    while !bytes.is_empty() {
        let length = usize::try_from(u32::from_be_bytes(bytes[..4].try_into().unwrap())).unwrap();
        values.push(serde_json::from_slice(&bytes[4..4 + length]).unwrap());
        bytes = &bytes[4 + length..];
    }
    values
}
pub(in crate::connection) fn initialize(connection: &SupervisorConnection) {
    let mut state = connection.observed.lock().unwrap();
    state.initialized = true;
    state.cursor = Some(SnapshotCursor {
        owner: "owner-a".into(),
        revision: 999,
    });
}
