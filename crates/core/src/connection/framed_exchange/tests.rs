use super::*;

#[test]
fn complete_nonzero_header_without_body_is_truncated_even_after_decode_consumes_it() {
    let mut codec = StrictCodec::new();
    let mut input = BytesMut::from(&b"\0\0\0\x03"[..]);
    assert!(codec.decode(&mut input).unwrap().is_none());
    assert!(input.is_empty());
    assert_eq!(
        codec.decode_eof(&mut input).unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
}

#[test]
fn codec_bounds_encoding_and_rejects_advertised_oversize_before_body_reservation() {
    let mut codec = StrictCodec::new();
    let mut output = BytesMut::new();
    codec
        .encode(Bytes::from(vec![b'x'; MAX_PAYLOAD]), &mut output)
        .unwrap();
    assert_eq!(output.len(), 65_536);
    assert_eq!(&output[..4], &[0, 0, 255, 252]);
    assert_eq!(
        codec.decode(&mut output).unwrap().unwrap().len(),
        MAX_PAYLOAD
    );
    assert!(
        codec
            .encode(Bytes::from(vec![0; MAX_PAYLOAD + 1]), &mut BytesMut::new())
            .is_err()
    );
    // The literal header advertises 65,533 payload bytes, with no body supplied.
    let mut input = BytesMut::from(&b"\0\0\xff\xfd"[..]);
    let capacity = input.capacity();
    assert_eq!(
        codec.decode(&mut input).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(input.capacity(), capacity);
}

#[test]
fn codec_preserves_split_and_coalesced_frames_and_empty_payloads() {
    let mut codec = StrictCodec::new();
    let mut input = BytesMut::from(&b"\0\0"[..]);
    assert!(codec.decode(&mut input).unwrap().is_none());
    input.extend_from_slice(b"\0\x03a");
    assert!(codec.decode(&mut input).unwrap().is_none());
    input.extend_from_slice(b"bc\0\0\0\0\0\0\0\x02de");
    assert_eq!(&codec.decode(&mut input).unwrap().unwrap()[..], b"abc");
    assert!(codec.decode(&mut input).unwrap().unwrap().is_empty());
    assert_eq!(&codec.decode(&mut input).unwrap().unwrap()[..], b"de");
    assert!(codec.decode_eof(&mut input).unwrap().is_none());
}

#[test]
fn codec_distinguishes_partial_header_and_body_from_clean_boundary_eof() {
    for bytes in [&b"\0"[..], &b"\0\0\0\x03a"[..]] {
        let mut codec = StrictCodec::new();
        let mut input = BytesMut::from(bytes);
        assert_eq!(
            codec.decode_eof(&mut input).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
    let mut codec = StrictCodec::new();
    let mut input = BytesMut::from(&b"\0\0\0\x01x\0\0\0\x03"[..]);
    assert_eq!(&codec.decode_eof(&mut input).unwrap().unwrap()[..], b"x");
    assert_eq!(
        codec.decode_eof(&mut input).unwrap_err().kind(),
        io::ErrorKind::UnexpectedEof
    );
    assert!(
        StrictCodec::new()
            .decode_eof(&mut BytesMut::new())
            .unwrap()
            .is_none()
    );
}

use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite, ReadBuf},
    time::Instant,
};
use tokio_util::sync::CancellationToken;

#[derive(Default)]
struct ScenarioState {
    input: VecDeque<Bytes>,
    eof: bool,
    output: Vec<u8>,
    write_allowance: usize,
    flush_ready: bool,
    fail_read: bool,
    zero_write: bool,
    fail_write: bool,
    fail_flush: bool,
    read_calls: usize,
    write_calls: usize,
    flush_calls: usize,
    drops: usize,
    reader: Option<Waker>,
    writer: Option<Waker>,
}
#[derive(Clone, Default)]
struct Control(Arc<Mutex<ScenarioState>>);
impl Control {
    fn feed(&self, bytes: &'static [u8]) {
        let mut state = self.0.lock().unwrap();
        state.input.push_back(Bytes::from_static(bytes));
        if let Some(waker) = state.reader.take() {
            waker.wake();
        }
    }
    fn end(&self) {
        let mut state = self.0.lock().unwrap();
        state.eof = true;
        if let Some(waker) = state.reader.take() {
            waker.wake();
        }
    }
    fn allow_write(&self, bytes: usize, flush: bool) {
        let mut state = self.0.lock().unwrap();
        state.write_allowance = bytes;
        state.flush_ready = flush;
        if let Some(waker) = state.writer.take() {
            waker.wake();
        }
    }
}
struct Scenario(Control);
impl Drop for Scenario {
    fn drop(&mut self) {
        self.0
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .drops += 1;
    }
}
impl AsyncRead for Scenario {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buffer: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let mut state = self.0.0.lock().unwrap();
        state.read_calls += 1;
        if state.fail_read {
            return Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()));
        }
        if let Some(mut chunk) = state.input.pop_front() {
            let count = chunk.len().min(buffer.remaining());
            buffer.put_slice(&chunk.split_to(count));
            if !chunk.is_empty() {
                state.input.push_front(chunk);
            }
            Poll::Ready(Ok(()))
        } else if state.eof {
            Poll::Ready(Ok(()))
        } else {
            state.reader = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}
impl AsyncWrite for Scenario {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bytes: &[u8],
    ) -> Poll<io::Result<usize>> {
        let mut state = self.0.0.lock().unwrap();
        state.write_calls += 1;
        if state.zero_write {
            return Poll::Ready(Ok(0));
        }
        if state.fail_write {
            return Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()));
        }
        if state.write_allowance == 0 {
            state.writer = Some(cx.waker().clone());
            return Poll::Pending;
        }
        let count = bytes.len().min(state.write_allowance);
        state.output.extend_from_slice(&bytes[..count]);
        state.write_allowance -= count;
        Poll::Ready(Ok(count))
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        let mut state = self.0.0.lock().unwrap();
        state.flush_calls += 1;
        if state.fail_flush {
            Poll::Ready(Err(io::ErrorKind::BrokenPipe.into()))
        } else if state.flush_ready {
            Poll::Ready(Ok(()))
        } else {
            state.writer = Some(cx.waker().clone());
            Poll::Pending
        }
    }
    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.poll_flush(cx)
    }
}
fn run(future: impl Future<Output = ()>) {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .start_paused(true)
        .build()
        .unwrap()
        .block_on(future);
}
fn poll<F: Future>(future: Pin<&mut F>) -> Poll<F::Output> {
    future.poll(&mut Context::from_waker(Waker::noop()))
}

#[test]
fn exchange_returns_complete_frames_and_keeps_prefetched_next_frame() {
    run(async {
        let control = Control::default();
        control.feed(b"\0\0\0\x03abc\0\0\0\x02de");
        control.end();
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        let cancel = CancellationToken::new();
        assert_eq!(
            &exchange
                .read_frame(Instant::now() + Duration::from_secs(7), &cancel)
                .await
                .unwrap()
                .unwrap()[..],
            b"abc"
        );
        let reads = control.0.lock().unwrap().read_calls;
        assert_eq!(
            &exchange
                .read_frame(Instant::now() + Duration::from_secs(7), &cancel)
                .await
                .unwrap()
                .unwrap()[..],
            b"de"
        );
        assert_eq!(control.0.lock().unwrap().read_calls, reads);
        assert!(
            exchange
                .read_frame(Instant::now() + Duration::from_secs(7), &cancel)
                .await
                .unwrap()
                .is_none()
        );
    });
}

#[test]
fn first_byte_starts_one_transfer_budget_and_slow_fragments_cannot_extend_it() {
    run(async {
        let control = Control::default();
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        let cancel = CancellationToken::new();
        let mut read =
            Box::pin(exchange.read_frame(Instant::now() + Duration::from_secs(7), &cancel));
        assert!(poll(read.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(4)).await;
        control.feed(b"\0\0");
        assert!(poll(read.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(1)).await;
        control.feed(b"\0\x03a");
        assert!(poll(read.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(1)).await;
        let Poll::Ready(Err(error)) = poll(read.as_mut()) else {
            panic!("fragments extended the original transfer budget")
        };
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert_eq!(control.0.lock().unwrap().drops, 1);
    });
}

#[test]
fn read_caps_idle_ack_and_buffered_transfer_without_extending_cancelled_polling() {
    run(async {
        let cancel = CancellationToken::new();
        // No first data: expire at the original idle deadline.
        let control = Control::default();
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        let mut read =
            Box::pin(exchange.read_frame(Instant::now() + Duration::from_secs(7), &cancel));
        assert!(poll(read.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(7)).await;
        assert_eq!(
            poll(read.as_mut()).map(|result| result.unwrap_err().kind()),
            Poll::Ready(io::ErrorKind::TimedOut)
        );
        drop(read);
        assert_eq!(control.0.lock().unwrap().drops, 1);

        // First ack bytes arrive at t=14s; its original t=15s cap still applies.
        let control = Control::default();
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        let mut read =
            Box::pin(exchange.read_frame(Instant::now() + Duration::from_secs(15), &cancel));
        assert!(poll(read.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(14)).await;
        control.feed(b"\0\0\0\x03");
        assert!(poll(read.as_mut()).is_pending());
        drop(read);
        tokio::time::advance(Duration::from_secs(1)).await;
        // A new polling future cannot reset either the operation or transfer cap.
        assert_eq!(
            exchange
                .read_frame(Instant::now() + Duration::from_secs(15), &cancel)
                .await
                .unwrap_err()
                .kind(),
            io::ErrorKind::TimedOut
        );

        // A partial next frame buffered beside a complete frame starts immediately.
        let control = Control::default();
        control.feed(b"\0\0\0\x01x\0\0\0\x03a");
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        assert!(
            exchange
                .read_frame(Instant::now() + Duration::from_secs(7), &cancel)
                .await
                .unwrap()
                .is_some()
        );
        let mut read =
            Box::pin(exchange.read_frame(Instant::now() + Duration::from_secs(7), &cancel));
        assert!(poll(read.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(2)).await;
        assert_eq!(
            poll(read.as_mut()).map(|result| result.unwrap_err().kind()),
            Poll::Ready(io::ErrorKind::TimedOut)
        );
    });
}

#[derive(Default)]
struct Wakes(std::sync::atomic::AtomicUsize);
impl std::task::Wake for Wakes {
    fn wake(self: Arc<Self>) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }
}
#[test]
fn cancellation_wakes_idle_or_partial_read_and_closes_terminally() {
    run(async {
        for partial in [false, true] {
            let control = Control::default();
            if partial {
                control.feed(b"\0\0\0\x03a");
            }
            let mut exchange = FramedExchange::new(Scenario(control.clone()));
            let cancel = CancellationToken::new();
            let wakes = Arc::new(Wakes::default());
            let waker = Waker::from(wakes.clone());
            let mut read =
                Box::pin(exchange.read_frame(Instant::now() + Duration::from_secs(7), &cancel));
            assert!(
                read.as_mut()
                    .poll(&mut Context::from_waker(&waker))
                    .is_pending()
            );
            cancel.cancel();
            assert!(
                wakes.0.load(std::sync::atomic::Ordering::SeqCst) > 0,
                "cancellation must wake pending I/O"
            );
            assert_eq!(read.await.unwrap_err().kind(), io::ErrorKind::Interrupted);
            assert_eq!(control.0.lock().unwrap().drops, 1);
            let calls = control.0.lock().unwrap().read_calls;
            assert_eq!(
                exchange
                    .read_frame(
                        Instant::now() + Duration::from_secs(7),
                        &CancellationToken::new()
                    )
                    .await
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::NotConnected
            );
            assert_eq!(control.0.lock().unwrap().read_calls, calls);
        }
    });
}

#[test]
fn writes_encode_once_across_partial_progress_and_wait_for_flush() {
    run(async {
        let control = Control::default();
        control.allow_write(2, false);
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        let cancel = CancellationToken::new();
        let mut write = Box::pin(exchange.write_frame(
            Bytes::from_static(b"abc"),
            Instant::now() + Duration::from_secs(15),
            &cancel,
        ));
        assert!(poll(write.as_mut()).is_pending());
        assert_eq!(control.0.lock().unwrap().output, b"\0\0");
        control.allow_write(5, false);
        assert!(poll(write.as_mut()).is_pending());
        assert_eq!(control.0.lock().unwrap().output, b"\0\0\0\x03abc");
        control.allow_write(0, true);
        assert_eq!(
            poll(write.as_mut()).map(|result| result.unwrap()),
            Poll::Ready(())
        );
        drop(write);
        assert_eq!(control.0.lock().unwrap().drops, 0);
        control.allow_write(usize::MAX, true);
        exchange
            .write_frame(
                Bytes::from_static(b"de"),
                Instant::now() + Duration::from_secs(2),
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(
            control.0.lock().unwrap().output,
            b"\0\0\0\x03abc\0\0\0\x02de"
        );
    });
}

#[test]
fn write_deadline_covers_partial_header_body_and_flush_without_fragment_renewal() {
    run(async {
        for initial in [2, 5, 7] {
            let control = Control::default();
            control.allow_write(initial, false);
            let mut exchange = FramedExchange::new(Scenario(control.clone()));
            let cancel = CancellationToken::new();
            let mut write = Box::pin(exchange.write_frame(
                Bytes::from_static(b"abc"),
                Instant::now() + Duration::from_secs(15),
                &cancel,
            ));
            assert!(poll(write.as_mut()).is_pending());
            tokio::time::advance(Duration::from_secs(1)).await;
            control.allow_write(1, false);
            assert!(poll(write.as_mut()).is_pending());
            tokio::time::advance(Duration::from_secs(1)).await;
            assert_eq!(
                poll(write.as_mut()).map(|result| result.unwrap_err().kind()),
                Poll::Ready(io::ErrorKind::TimedOut)
            );
            drop(write);
            assert_eq!(control.0.lock().unwrap().drops, 1);
            let calls = control.0.lock().unwrap().write_calls;
            assert_eq!(
                exchange
                    .write_frame(
                        Bytes::from_static(b"new"),
                        Instant::now() + Duration::from_secs(2),
                        &CancellationToken::new()
                    )
                    .await
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::NotConnected
            );
            assert_eq!(control.0.lock().unwrap().write_calls, calls);
        }
    });
}

#[test]
fn cancelled_or_abandoned_write_closes_without_restarting_partial_io() {
    run(async {
        for token_cancel in [false, true] {
            let control = Control::default();
            control.allow_write(5, false);
            let mut exchange = FramedExchange::new(Scenario(control.clone()));
            let cancel = CancellationToken::new();
            let wakes = Arc::new(Wakes::default());
            let waker = Waker::from(wakes.clone());
            let mut write = Box::pin(exchange.write_frame(
                Bytes::from_static(b"abc"),
                Instant::now() + Duration::from_secs(2),
                &cancel,
            ));
            assert!(
                write
                    .as_mut()
                    .poll(&mut Context::from_waker(&waker))
                    .is_pending()
            );
            if token_cancel {
                cancel.cancel();
                assert!(wakes.0.load(std::sync::atomic::Ordering::SeqCst) > 0);
                assert_eq!(
                    poll(write.as_mut()).map(|result| result.unwrap_err().kind()),
                    Poll::Ready(io::ErrorKind::Interrupted)
                );
            }
            drop(write);
            assert_eq!(control.0.lock().unwrap().drops, 1);
            assert_eq!(control.0.lock().unwrap().output, b"\0\0\0\x03a");
            assert_eq!(
                exchange
                    .write_frame(
                        Bytes::from_static(b"again"),
                        Instant::now() + Duration::from_secs(2),
                        &CancellationToken::new()
                    )
                    .await
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::NotConnected
            );
            assert_eq!(
                exchange
                    .read_frame(
                        Instant::now() + Duration::from_secs(2),
                        &CancellationToken::new()
                    )
                    .await
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::NotConnected
            );
        }
    });
}

#[test]
fn eof_decode_and_read_errors_close_the_exchange_once_without_more_io() {
    run(async {
        for (bytes, expected) in [
            (&b""[..], None),
            (&b"\0\0"[..], Some(io::ErrorKind::UnexpectedEof)),
            (&b"\0\0\0\x03"[..], Some(io::ErrorKind::UnexpectedEof)),
            (&b"\0\0\0\x03a"[..], Some(io::ErrorKind::UnexpectedEof)),
            (&b"\0\0\xff\xfd"[..], Some(io::ErrorKind::InvalidData)),
        ] {
            let control = Control::default();
            if !bytes.is_empty() {
                control.feed(bytes);
            }
            control.end();
            let mut exchange = FramedExchange::new(Scenario(control.clone()));
            let cancel = CancellationToken::new();
            let result = exchange
                .read_frame(Instant::now() + Duration::from_secs(2), &cancel)
                .await;
            match expected {
                Some(kind) => assert_eq!(result.unwrap_err().kind(), kind),
                None => assert!(result.unwrap().is_none()),
            }
            assert_eq!(control.0.lock().unwrap().drops, 1);
            let calls = control.0.lock().unwrap().read_calls;
            assert_eq!(
                exchange
                    .read_frame(Instant::now() + Duration::from_secs(2), &cancel)
                    .await
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::NotConnected
            );
            assert_eq!(control.0.lock().unwrap().read_calls, calls);
        }
        let control = Control::default();
        control.0.lock().unwrap().fail_read = true;
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        assert_eq!(
            exchange
                .read_frame(
                    Instant::now() + Duration::from_secs(2),
                    &CancellationToken::new()
                )
                .await
                .unwrap_err()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
        assert_eq!(control.0.lock().unwrap().drops, 1);
    });
}

#[test]
fn write_errors_and_oversize_close_without_retry_or_queued_output() {
    run(async {
        for (write_error, flush_error, zero, oversize, kind) in [
            (true, false, false, false, io::ErrorKind::BrokenPipe),
            (false, true, false, false, io::ErrorKind::BrokenPipe),
            (false, false, true, false, io::ErrorKind::WriteZero),
            (false, false, false, true, io::ErrorKind::InvalidInput),
        ] {
            let control = Control::default();
            control.allow_write(usize::MAX, true);
            {
                let mut state = control.0.lock().unwrap();
                state.fail_write = write_error;
                state.fail_flush = flush_error;
                state.zero_write = zero;
            }
            let mut exchange = FramedExchange::new(Scenario(control.clone()));
            let bytes = if oversize {
                Bytes::from(vec![0; MAX_PAYLOAD + 1])
            } else {
                Bytes::from_static(b"abc")
            };
            assert_eq!(
                exchange
                    .write_frame(
                        bytes,
                        Instant::now() + Duration::from_secs(2),
                        &CancellationToken::new()
                    )
                    .await
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(control.0.lock().unwrap().drops, 1);
            if !flush_error {
                assert!(control.0.lock().unwrap().output.is_empty());
            }
            assert_eq!(
                control.0.lock().unwrap().write_calls,
                usize::from(!oversize)
            );
        }
    });
}

#[test]
fn already_cancelled_or_expired_operations_do_no_physical_io() {
    run(async {
        for write in [false, true] {
            for cancel_first in [false, true] {
                let control = Control::default();
                control.feed(b"\0\0\0\x01x");
                control.allow_write(usize::MAX, true);
                let mut exchange = FramedExchange::new(Scenario(control.clone()));
                let cancel = CancellationToken::new();
                if cancel_first {
                    cancel.cancel();
                }
                let deadline = if cancel_first {
                    Instant::now() + Duration::from_secs(2)
                } else {
                    Instant::now()
                };
                let error = if write {
                    exchange
                        .write_frame(Bytes::from_static(b"x"), deadline, &cancel)
                        .await
                        .unwrap_err()
                } else {
                    exchange.read_frame(deadline, &cancel).await.unwrap_err()
                };
                assert_eq!(
                    error.kind(),
                    if cancel_first {
                        io::ErrorKind::Interrupted
                    } else {
                        io::ErrorKind::TimedOut
                    }
                );
                let state = control.0.lock().unwrap();
                assert_eq!(
                    (state.read_calls, state.write_calls, state.drops),
                    (0, 0, 1)
                );
            }
        }
    });
}

#[test]
fn dropped_read_polling_retains_transfer_progress_and_can_finish_before_original_deadline() {
    run(async {
        let cancel = CancellationToken::new();
        for complete in [false, true] {
            let control = Control::default();
            control.feed(b"\0\0\0\x03a");
            let mut exchange = FramedExchange::new(Scenario(control.clone()));
            let mut read =
                Box::pin(exchange.read_frame(Instant::now() + Duration::from_secs(15), &cancel));
            assert!(poll(read.as_mut()).is_pending());
            drop(read);
            assert_eq!(control.0.lock().unwrap().drops, 0);
            tokio::time::advance(Duration::from_secs(1)).await;
            if complete {
                control.feed(b"bc");
            } else {
                tokio::time::advance(Duration::from_secs(1)).await;
            }
            let result = exchange
                .read_frame(Instant::now() + Duration::from_secs(15), &cancel)
                .await;
            if complete {
                assert_eq!(&result.unwrap().unwrap()[..], b"abc");
            } else {
                assert_eq!(result.unwrap_err().kind(), io::ErrorKind::TimedOut);
            }
        }
    });
}

#[test]
fn handshake_write_preserves_buffered_next_read_and_write_uses_operation_cap() {
    run(async {
        let control = Control::default();
        control.feed(b"\0\0\0\x01x\0\0\0\x01y");
        control.allow_write(usize::MAX, true);
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        let cancel = CancellationToken::new();
        assert_eq!(
            &exchange
                .read_frame(Instant::now() + Duration::from_secs(2), &cancel)
                .await
                .unwrap()
                .unwrap()[..],
            b"x"
        );
        exchange
            .write_frame(
                Bytes::from_static(b"ack"),
                Instant::now() + Duration::from_secs(2),
                &cancel,
            )
            .await
            .unwrap();
        let reads = control.0.lock().unwrap().read_calls;
        assert_eq!(
            &exchange
                .read_frame(Instant::now() + Duration::from_secs(7), &cancel)
                .await
                .unwrap()
                .unwrap()[..],
            b"y"
        );
        assert_eq!(control.0.lock().unwrap().read_calls, reads);
        control.allow_write(1, false);
        let mut write = Box::pin(exchange.write_frame(
            Bytes::from_static(b"x"),
            Instant::now() + Duration::from_secs(1),
            &cancel,
        ));
        assert!(poll(write.as_mut()).is_pending());
        tokio::time::advance(Duration::from_secs(1)).await;
        assert_eq!(
            poll(write.as_mut()).map(|result| result.unwrap_err().kind()),
            Poll::Ready(io::ErrorKind::TimedOut)
        );
    });
}

#[test]
fn write_rejects_progress_after_absolute_cap_even_between_timer_ticks() {
    run(async {
        let control = Control::default();
        let mut exchange = FramedExchange::new(Scenario(control.clone()));
        let cancel = CancellationToken::new();
        let mut write = Box::pin(exchange.write_frame(
            Bytes::from_static(b"x"),
            Instant::now() + Duration::from_micros(1),
            &cancel,
        ));
        assert!(poll(write.as_mut()).is_pending());
        tokio::time::advance(Duration::from_micros(1)).await;
        control.allow_write(usize::MAX, true);
        assert_eq!(
            poll(write.as_mut()).map(|result| result.unwrap_err().kind()),
            Poll::Ready(io::ErrorKind::TimedOut)
        );
        assert!(control.0.lock().unwrap().output.is_empty());
    });
}

#[test]
fn deadline_timer_wakes_pending_reads_and_writes() {
    run(async {
        for write in [false, true] {
            let control = Control::default();
            let mut exchange = FramedExchange::new(Scenario(control));
            let cancel = CancellationToken::new();
            let wakes = Arc::new(Wakes::default());
            let waker = Waker::from(wakes.clone());
            let deadline = Instant::now() + Duration::from_secs(1);
            let mut operation: Pin<Box<dyn Future<Output = io::Result<()>>>> = if write {
                Box::pin(exchange.write_frame(Bytes::from_static(b"x"), deadline, &cancel))
            } else {
                Box::pin(async { exchange.read_frame(deadline, &cancel).await.map(|_| ()) })
            };
            assert!(
                operation
                    .as_mut()
                    .poll(&mut Context::from_waker(&waker))
                    .is_pending()
            );
            tokio::time::advance(Duration::from_secs(1)).await;
            tokio::task::yield_now().await;
            assert!(wakes.0.load(std::sync::atomic::Ordering::SeqCst) > 0);
            assert_eq!(operation.await.unwrap_err().kind(), io::ErrorKind::TimedOut);
        }
    });
}
