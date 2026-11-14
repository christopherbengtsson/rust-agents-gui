//! Private length-prefixed kernel. Production remains protocol 7 until F04d.
use bytes::{Bytes, BytesMut};
use futures_core::Stream;
use futures_sink::Sink;
use std::{
    future::{Future, poll_fn},
    io,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    time::{Instant, sleep_until},
};
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use tokio_util::{codec::Framed, sync::CancellationToken};

const MAX_PAYLOAD: usize = 65_532;

struct StrictCodec {
    inner: LengthDelimitedCodec,
    incomplete: bool,
}
impl StrictCodec {
    fn new() -> Self {
        Self {
            inner: LengthDelimitedCodec::builder()
                .big_endian()
                .length_field_length(4)
                .length_field_offset(0)
                .length_adjustment(0)
                .num_skip(4)
                .max_frame_length(MAX_PAYLOAD)
                .new_codec(),
            incomplete: false,
        }
    }
}
impl Decoder for StrictCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, source: &mut BytesMut) -> io::Result<Option<BytesMut>> {
        // The inner codec can consume the entire header while still awaiting a body.
        self.incomplete |= !source.is_empty();
        let frame = self.inner.decode(source)?;
        if frame.is_some() {
            self.incomplete = false;
        }
        Ok(frame)
    }

    fn decode_eof(&mut self, source: &mut BytesMut) -> io::Result<Option<BytesMut>> {
        let frame = self.decode(source)?;
        if frame.is_none() && self.incomplete {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Incomplete frame",
            ));
        }
        Ok(frame)
    }
}
impl Encoder<Bytes> for StrictCodec {
    type Error = io::Error;

    fn encode(&mut self, frame: Bytes, target: &mut BytesMut) -> io::Result<()> {
        self.inner.encode(frame, target)
    }
}

const TRANSFER_BUDGET: Duration = Duration::from_secs(2);

/// Observes first progress; codec-owned buffers remain the only frame buffers.
struct Progress<S> {
    stream: S,
    first_byte: Option<Instant>,
}
impl<S: AsyncRead + Unpin> AsyncRead for Progress<S> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let before = buffer.filled().len();
        let result = Pin::new(&mut this.stream).poll_read(cx, buffer);
        if buffer.filled().len() > before && this.first_byte.is_none() {
            this.first_byte = Some(Instant::now());
        }
        result
    }
}
impl<S: AsyncWrite + Unpin> AsyncWrite for Progress<S> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.get_mut().stream).poll_write(cx, bytes)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_flush(cx)
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.get_mut().stream).poll_shutdown(cx)
    }
}
struct ReadBudget {
    operation: Instant,
    transfer: Option<Instant>,
}
impl ReadBudget {
    fn deadline(&self) -> Instant {
        self.transfer
            .map_or(self.operation, |transfer| self.operation.min(transfer))
    }
}

/// One owner across handshake and subsequent frames. No raw stream/buffer escape.
pub(super) struct FramedExchange<S> {
    framed: Option<Framed<Progress<S>, StrictCodec>>,
    read_budget: Option<ReadBudget>,
}
impl<S: AsyncRead + AsyncWrite + Unpin + 'static> FramedExchange<S> {
    /// Transfer exclusive ownership; dropping the transport must close both directions.
    pub(super) fn new(stream: S) -> Self {
        Self {
            framed: Some(Framed::new(
                Progress {
                    stream,
                    first_byte: None,
                },
                StrictCodec::new(),
            )),
            read_budget: None,
        }
    }

    /// A dropped write future closes the connection: an incomplete send is never restarted.
    pub(super) async fn write_frame(
        &mut self,
        frame: Bytes,
        deadline: Instant,
        cancel: &CancellationToken,
    ) -> io::Result<()> {
        let mut guard = WriteGuard {
            exchange: self,
            flushed: false,
        };
        let framed = guard.exchange.framed.as_mut().ok_or_else(closed)?;
        let mut frame = Some(frame);
        let deadline = deadline.min(Instant::now() + TRANSFER_BUDGET);
        let timer = sleep_until(deadline);
        tokio::pin!(timer);
        let cancelled = cancel.cancelled();
        tokio::pin!(cancelled);
        let result = poll_fn(|cx| {
            if cancelled.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(interrupted()));
            }
            // Timer ticks can be coarser than the operation's absolute cap.
            if Instant::now() >= deadline || timer.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(timed_out()));
            }
            if frame.is_some() {
                match Pin::new(&mut *framed).poll_ready(cx) {
                    Poll::Ready(Ok(())) => Pin::new(&mut *framed).start_send(
                        frame
                            .take()
                            .expect("only an unsent frame reaches start_send"),
                    )?,
                    other => return other,
                }
            }
            let result = Pin::new(&mut *framed).poll_flush(cx);
            if Instant::now() >= deadline || timer.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(timed_out()));
            }
            result
        })
        .await;
        guard.flushed = result.is_ok();
        result
    }

    /// Clean frame-boundary EOF is None; malformed/truncated input is an error.
    /// Dropped polling retains both buffered progress and the original deadline.
    pub(super) async fn read_frame(
        &mut self,
        deadline: Instant,
        cancel: &CancellationToken,
    ) -> io::Result<Option<BytesMut>> {
        let framed = self.framed.as_mut().ok_or_else(closed)?;
        let budget = self.read_budget.get_or_insert_with(|| {
            framed.get_mut().first_byte =
                (!framed.read_buffer().is_empty() || framed.codec().incomplete).then(Instant::now);
            ReadBudget {
                operation: deadline,
                transfer: None,
            }
        });
        // Re-polling can tighten an operation cap, never extend an incomplete read.
        budget.operation = budget.operation.min(deadline);
        let timer = sleep_until(budget.deadline());
        tokio::pin!(timer);
        let cancelled = cancel.cancelled();
        tokio::pin!(cancelled);
        let result = poll_fn(|cx| {
            if cancelled.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err(interrupted()));
            }
            if Instant::now() >= budget.deadline() {
                return Poll::Ready(Err(timed_out()));
            }
            let frame = Pin::new(&mut *framed).poll_next(cx);
            if budget.transfer.is_none() {
                budget.transfer = framed
                    .get_ref()
                    .first_byte
                    .map(|first| first + TRANSFER_BUDGET);
            }
            let deadline = budget.deadline();
            if Instant::now() >= deadline {
                return Poll::Ready(Err(timed_out()));
            }
            match frame {
                Poll::Ready(frame) => Poll::Ready(frame.transpose()),
                Poll::Pending => {
                    timer.as_mut().reset(deadline);
                    if timer.as_mut().poll(cx).is_ready() {
                        Poll::Ready(Err(timed_out()))
                    } else {
                        Poll::Pending
                    }
                }
            }
        })
        .await;
        self.read_budget = None;
        if !matches!(result, Ok(Some(_))) {
            self.framed.take();
        }
        result
    }
}
struct WriteGuard<'a, S> {
    exchange: &'a mut FramedExchange<S>,
    flushed: bool,
}
impl<S> Drop for WriteGuard<'_, S> {
    fn drop(&mut self) {
        if !self.flushed {
            self.exchange.framed.take();
            self.exchange.read_budget = None;
        }
    }
}
fn closed() -> io::Error {
    io::Error::new(io::ErrorKind::NotConnected, "Framed exchange is closed")
}
fn timed_out() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "Frame deadline exceeded")
}
fn interrupted() -> io::Error {
    io::Error::new(io::ErrorKind::Interrupted, "Framed exchange cancelled")
}

#[cfg(test)]
mod tests;
