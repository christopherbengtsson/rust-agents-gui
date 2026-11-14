# Framed I/O

Apply when using tokio-util codecs or adapting asynchronous byte streams into
messages. Read [Async lifecycle](async-lifecycle.md#cancellation-and-owned-cleanup)
for task ownership and cooperative cancellation. Obtain wire format, limits,
deadlines and recovery policy from the owning protocol; library defaults do not
define those requirements. Consult the selected dependency version's docs/source.

## Own the stream and its buffers

Use `Framed`, `FramedRead` or `FramedWrite` to own the codec and buffered I/O across
operations. Keep that owner alive across handshake/message boundaries and cancelled
waits. Recreating it for each message can discard unread bytes or unwritten output.
If changing wrappers, preserve the codec and read/write buffers through the parts
APIs; extracting only the underlying stream loses buffered state.

Keep raw-stream access coordinated with the codec. Reads or writes that bypass its
buffers can corrupt framing. Splitting a framed transport changes how tasks access
it, not who must coordinate shutdown and account for outstanding output.

## Wire shape and bounds

Choose a codec that implements the actual protocol. For length-delimited frames,
configure field width, byte order, offset, adjustment, skipped bytes and maximum
length deliberately. Distinguish payload size, advertised length and total wire
size, including framing overhead. Verify decoding and encoding limits separately;
an inbound maximum does not automatically establish every outbound bound.

Reject an oversized advertised frame before reserving its body. Bound queued and
buffered output as well as individual frames; sink backpressure is not a total
application memory budget. Delegate ordinary buffering/encoding to the codec and
keep adapters focused on concrete protocol gaps.

## Incomplete input and EOF

For a decoder, `Ok(None)` means more input is needed, not that a message completed.
At EOF, distinguish an idle frame boundary from a partial header or body. The
default `Decoder::decode_eof` checks remaining bytes, but a stateful codec can have
consumed a header while still awaiting its body. An empty byte buffer alone then
does not establish a clean frame boundary. If the protocol requires strict EOF,
retain the needed pending-frame evidence in a focused adapter.

Classify malformed frames using the protocol's error contract. Continue only when
the codec and protocol define a valid recovery boundary; otherwise terminate the
connection. Treat terminal EOF as completion/error rather than repeatedly polling
it in a ready loop.

## Deadlines, cancellation and delivery

Keep absolute deadlines outside per-fragment polls so slow fragments cannot renew
the operation indefinitely. When idle and transfer budgets differ, retain the
first-data transition across polls and account for bytes already buffered. Bound
writes through flush as well as reads through complete decoding.

Inspect the exact read/send/flush future's cancellation contract. Keeping `Framed`
alive can preserve buffered progress; it does not guarantee that a cancelled send
retains its input item or that no bytes reached the peer. Retain and drive the same
future/state when safe, or close the affected transport and preserve delivery
uncertainty. Encoding, local flushing and peer acknowledgement establish different
outcomes; retry authority belongs to the operation's owner.

A `CancellationToken` requests cooperative stop. The owner still needs to wake
relevant waits, finish the applicable transport cleanup and observe task completion.
Use the lifecycle reference for token/handle ownership rather than treating token
cancellation or task disposal as completed cleanup.

## Completion

Through the owning interface, cover split/coalesced frames, exact/oversized limits,
partial-header/body EOF, a consumed header advertising a nonzero body with no body
bytes, buffered next frames, slow fragments, partial writes and cancellation during progress. Select
controlled codec/stream/time checks and actual transport qualification through the
[verification policy](../../../../docs/agents/verification.md); pure checks do not
establish OS socket or process behavior.

## Sources

- [tokio-util Framed](https://docs.rs/tokio-util/latest/tokio_util/codec/struct.Framed.html): stream/sink ownership, parts APIs and cancellation contracts.
- [Decoder](https://docs.rs/tokio-util/latest/tokio_util/codec/trait.Decoder.html) and [Encoder](https://docs.rs/tokio-util/latest/tokio_util/codec/trait.Encoder.html): buffered progress and EOF behavior.
- [Length-delimited codec](https://docs.rs/tokio-util/latest/tokio_util/codec/length_delimited/index.html): wire configuration and size semantics.
- [CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html): cooperative stop signals and token lifetimes.
