# Tokio transport and F01–F04 crate refactor audit

2026-11-14. Research against the F04 checkout (`7dc4eac`), not an implementation
proposal already accepted into the frontend plan. Builds, tests, benchmarks,
desktop/process qualification and provider runs were not performed. No archived
RAT inspection was needed. Existing research and unrelated edits are preserved.

## Recommendation

Proceed with the user's preferred **ts-rs and Tokio Semaphore** as separately
scoped refactors, preserving the runtime guards and physical-work ownership.
Keep synchronous sockets for this increment. Also consider **rustix's safe poll
wrapper**: it removes a concrete handwritten unsafe OS boundary without changing
the execution model. There are further credible crate replacements, especially
framing and cancellation under an async transport. Saying that nothing else can
be refactored with existing crates would be too broad.

The distinction is between a possible replacement and a useful replacement at
this checkpoint. Async sockets could improve task composition and reduce
connection threads. The current source does not establish a throughput,
navigation-latency or memory problem requiring that migration. That is an
engineering inference from the bounded workload below, not a benchmark result.

## Actual workload and ownership

The [frontend plan](../../.scratch/frontend-navigation/spec.md#native-request-scheduling)
caps desktop work at two observation exchanges, one intent/control exchange, one
status/discovery exchange and one subscription: **five connections**, below the
supervisor's **eight-client ceiling**. The latter is implemented by
[supervisor admission](../../crates/supervisor/src/main.rs), using a permit before
spawning a connection thread. These are transport budgets, not a count of all
Git, storage, runtime or background-worker threads.

[Tauri commands](../../src-tauri/src/main.rs) already admit short exchanges before
`spawn_blocking`, and keep their permit inside the physical work. The persistent
feed has its own named standard thread in
[subscription.rs](../../crates/core/src/connection/subscription.rs), rather than
occupying the blocking pool indefinitely. Connection-state locks do not enclose
socket exchanges. Replacing these sockets does not newly move that work off the
presentation path; it changes how native waiting is represented.

The current feed is low-volume repository/workspace invalidation: 256 journal
entries, at most eight tags per entry, at most 32 tags and 4 KiB per delivery,
one frontend batch in flight, five-second heartbeats and a 15-second
acknowledgement budget. Socket frames are newline-delimited JSON with a **64 KiB
limit including newline** and a two-second transfer budget. The separate
conversation-history workload is future work, with its own retention and
delivery semantics; it must not be assumed to share this journal.
[Plan](../../.scratch/frontend-navigation/spec.md#rust-invalidation-journal),
[journal](../../crates/core/src/observations.rs),
[framing](../../crates/core/src/connection.rs),
[subscription](../../crates/core/src/connection/subscription.rs).

The core is independent of Tauri. It currently has no direct Tokio dependency;
Tokio 1.54.1 and tokio-util 0.7.20 are locked transitively in the desktop graph.
Making them core dependencies changes the standalone supervisor's dependency
graph too. An async core can remain UI-independent, but the supervisor must own
its runtime explicitly rather than obtain lifetime authority from Tauri.
[Core manifest](../../crates/core/Cargo.toml), [lockfile](../../Cargo.lock),
[MVP](../MVP.md#starting-architecture).

## Synchronous Unix sockets versus Tokio async

This comparison concerns the current application design, not the relative speed
of isolated socket APIs. Tokio supplies a nonblocking I/O driver and lightweight
tasks; it does not remove the underlying Unix-domain socket or application wire
protocol. Tokio's UnixStream wraps Mio readiness registration, and converting a
standard socket requires nonblocking mode and an I/O-enabled runtime.
[Tokio 1.54.1 overview](https://docs.rs/tokio/1.54.1/tokio/),
[versioned UnixStream source](https://docs.rs/tokio/1.54.1/src/tokio/net/unix/stream.rs.html).

| Concern                     | Current synchronous design                                                                                            | Tokio async transport                                                                                    | Assessment for this app                                                                             |
| --------------------------- | --------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Responsiveness              | Short exchanges already run on blocking workers; feed has a dedicated thread.                                         | Pending socket work yields rather than occupying a worker.                                               | Both can keep rendering responsive. No measured latency advantage is established.                   |
| Connection scaling          | One supervisor thread per admitted connection, capped at eight; desktop short exchanges also occupy blocking workers. | Many idle connection tasks can share runtime threads.                                                    | A real advantage for substantially more persistent streams; modest potential saving at today's cap. |
| Readiness/deadline plumbing | Manual poll, buffered reads and remaining-time calculations.                                                          | Readiness and timers are provided; `timeout_at` composes an absolute deadline.                           | Can remove OS plumbing, but must retain the two-phase idle/transfer policy.                         |
| Stop coordination           | Socket clone/shutdown, stopped state, Condvar wake and owned join.                                                    | CancellationToken plus an owned task, timer and async acknowledgement notification can simplify waiting. | Useful with async I/O; adding a token to blocking I/O does not make it interruptible.               |
| Partial frames              | One persistent BufReader across handshake and subsequent frames; local frame accumulation terminates on failure.      | Persistent framed decoder or owned buffer across awaits.                                                 | Safe if ownership is designed; a dropped future must not discard progress while reusing the stream. |
| Blocking Git/SQLite         | Work remains on existing blocking execution paths.                                                                    | Still needs bounded blocking workers or a deliberate driver redesign.                                    | Socket conversion does not make domain calls nonblocking.                                           |
| Shutdown evidence           | Explicit socket shutdown and worker join; server connection threads are detached.                                     | Owned tasks can be gathered and joined; abort alone proves no durable flush or process cleanup.          | Async offers better task tools, but requirements still need explicit implementation.                |
| Change size                 | Narrow primitive replacements are possible.                                                                           | Touches client, server listener/handlers, framing, feed, discovery and runtime ownership.                | A coherent transport refactor, larger than replacing a type or adding `async`.                      |

Tokio documents that started `spawn_blocking` work cannot be aborted, its pool
queues after its thread limit, and dedicated threads suit persistent blocking
loops. Therefore keep admission before spawn and keep permits with the actual
blocking operation even if an async waiter is dropped. An async server must also
keep synchronous core dispatch away from executor workers; `select!` branches on
the same task cannot progress while one branch blocks.
[Tokio 1.54.1 blocking-task source](https://docs.rs/tokio/1.54.1/src/tokio/task/blocking.rs.html),
[select runtime characteristics](https://docs.rs/tokio/1.54.1/tokio/macro.select.html#runtime-characteristics).

## Exact replace/retain matrix

| Slice and source                                                                                                                                                                                  | Existing mechanism a crate could replace                                                                                               | Retain application-owned behavior                                                                                                                     | Decision                                                                                                        |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| F01: manifests, preparation, research and ADR                                                                                                                                                     | No substantive Rust runtime mechanism was introduced here.                                                                             | Tooling preparation and documented ownership.                                                                                                         | No artificial crate replacement target.                                                                         |
| F02: wire DTOs in `connection.rs`, `lib.rs`, `workspaces.rs`, `observations.rs`; TS declarations in `src/lib/observation-contracts.ts`, `observation-feed.ts`, `repositories.ts`, `workspaces.ts` | ts-rs can generate repeated TypeScript wire declarations.                                                                              | Runtime decoding, owner/exact-target checks, UUID/sequence/byte bounds, state validity, sticky action uncertainty and corpus.                         | Adopt after representative DTO export establishes Serde fidelity; do not count guards as removable boilerplate. |
| F03: `connection/admission.rs` used by desktop lanes and supervisor                                                                                                                               | `Arc<AtomicUsize>`, compare/update and decrement-on-drop become `Arc<Semaphore>` and `OwnedSemaphorePermit`.                           | `ConnectionSlots`/`Admitted` boundary, exact lane counts, fail-fast Busy, acquire-before-spawn and release after physical completion.                 | Good small refactor now; no async socket migration required.                                                    |
| F04: `connection.rs::FrameStream::wait_readable`                                                                                                                                                  | Manual `libc::pollfd`, duration-to-millisecond conversion and unsafe `libc::poll` become safe rustix poll APIs.                        | Empty-buffer check, timeout/error interpretation, absolute budget, buffered EOF and macOS behavior.                                                   | Credible optional small refactor now.                                                                           |
| F04: `connection.rs::read_buffered_frame*`, `write_timed_frame`, `encode_frame`                                                                                                                   | Tokio readiness/timers plus tokio-util framing can replace buffer/framing machinery.                                                   | Newline wire compatibility, 64 KiB inclusive bound, strict incomplete EOF, outbound bound, first-byte/transfer deadlines and failure visibility.      | Credible async-transport refactor; not a drop-in codec switch.                                                  |
| F04: `connection/subscription.rs::Control`, `NativeSubscription`, `run_subscribed`; supervisor connection workers                                                                                 | CancellationToken, an owned async JoinHandle or JoinSet, and a targeted async notification/channel replace stop/wake/thread mechanics. | Exact acknowledgement identity, one pending batch, generation ownership, accepted cursor, explicit stop/join outcomes, late attachment and no replay. | Useful when transport becomes async; one feed needs no task collection merely for uniformity.                   |
| F04: `observations.rs::Journal`                                                                                                                                                                   | A bounded collection/channel can replace individual storage/wakeup details.                                                            | Cursor replay, sampled resync watermark, identity tags, limits and publication/lock order.                                                            | Keep concrete journal; broadcast is not the required replay abstraction.                                        |
| F02 and older errors: `workspaces.rs::RepositoryError`                                                                                                                                            | thiserror can derive the short Display/Error implementation.                                                                           | Stable wire error codes, delivery classification, exact authority and uncertainty decisions.                                                          | Possible cosmetic cleanup; little current saving.                                                               |

### Semaphore is independent of socket async

For the proposed admission change, use Tokio 1.54.1 with `default-features = false`
and `features = ["sync"]` in the relevant manifest. `try_acquire_owned` returns an
owned permit immediately or an error, with release on permit drop. It needs no
Tokio runtime to execute that nonwaiting path. Do not replace it with awaiting
`acquire_owned`, which changes the native lane to a queue. Do not inspect
`available_permits` and then start work: acquisition itself must decide admission.
[Semaphore](https://docs.rs/tokio/1.54.1/tokio/sync/struct.Semaphore.html#method.try_acquire_owned),
[owned permit](https://docs.rs/tokio/1.54.1/tokio/sync/struct.OwnedSemaphorePermit.html),
[feature manifest](https://docs.rs/crate/tokio/1.54.1/source/Cargo.toml).

Feature activation elsewhere can widen the combined desktop build; a minimal core
declaration does not disable features requested by Tauri. A standalone core or
supervisor using only this primitive need not construct a scheduler, reactor or
timer. This preserves runtime independence, while introducing a direct library
dependency. No binary-size or compile-time saving/cost has been measured.

### Safe polling is a separate synchronous opportunity

The locked rustix 1.1.6 exposes `event::poll`, `PollFd` and `PollFlags` through its
`event` feature. PollFd borrows an `AsFd`; the application need not assemble a raw
pollfd and invoke unsafe FFI. On the macOS libc backend its implementation still
converts the timeout and calls libc poll, preserving the same basic OS operation.
The inspected published source records revision
`4a563ad698b2c20ff848515821023d2265a788d0`.
[Safe poll entry](https://github.com/bytecodealliance/rustix/blob/4a563ad698b2c20ff848515821023d2265a788d0/src/event/poll.rs),
[borrowed descriptor](https://github.com/bytecodealliance/rustix/blob/4a563ad698b2c20ff848515821023d2265a788d0/src/backend/libc/event/poll_fd.rs),
[libc backend](https://github.com/bytecodealliance/rustix/blob/4a563ad698b2c20ff848515821023d2265a788d0/src/backend/libc/event/syscalls.rs).

This removes a real manual safety/conversion seam. It does not remove timeout,
EOF or protocol policy. Existing libc uses in supervisor permission/ownership
handling remain, so this is not evidence the libc dependency can be deleted.
No actual macOS socket qualification of the replacement was performed.

### Code generation has a useful compile-time boundary

ts-rs 12.0.1 derives declarations, understands supported Serde attributes and can
export programmatically. Default large-integer output is `bigint`, requiring an
explicit `number` decision for this protocol's safe JSON sequence values. Probe
flattened FeedAck/FeedBatch, tagged errors, generic OwnedReply, UUID newtypes and
nullable fields against the current corpus. Keep unsupported-Serde warnings
visible. Use an explicit generation/drift command; `#[ts(export)]` writes files
through generated tests and should not turn the effect-free application unit gate
into code preparation.
[ts-rs 12.0.1](https://docs.rs/ts-rs/12.0.1/ts_rs/).

Prefer an optional codegen feature or isolated generation target so application
builds need not execute code generation. Static declarations cannot validate
unknown JSON at runtime. Generated imports can replace manually repeated wire
shapes; semantic refinements and convenient feature projections may remain local.
No representative export or build has been run in this research.

## Framing and cancellation: the real async tradeoffs

Tokio-util 0.7.20 `LinesCodec` is the relevant existing-protocol candidate.
`LengthDelimitedCodec` uses a length prefix, defaulting to a big-endian u32;
adopting it changes the v7 wire format and needs a protocol decision/version bump.
It is not a simpler compatible implementation of newline framing.
[0.7.20 length codec source](https://github.com/tokio-rs/tokio/blob/5ec9e6b8d5a7111fe9cd35a637bd8b4e4eb1e483/tokio-util/src/codec/length_delimited.rs).

The exact LinesCodec source exposes these compatibility details:

- `new()` has no line limit; use a bounded decoder. Its limit excludes newline,
  so a 65,536-byte wire-frame ceiling permits at most 65,535 bytes before newline.
- The encoder does not enforce the decoder's maximum; retain an outbound check.
- `decode_eof` returns remaining unterminated text. Current code rejects an
  incomplete frame, even if its JSON happens to parse. A strict EOF adapter is needed.
- It strips CR before newline; existing valid JSON already permits trailing CR
  whitespace, but byte-limit accounting still needs to happen before stripping.
- An overlong line can enter discard/recovery mode. This application closes the
  malformed connection; do not accidentally continue after the codec error.

These are source-level differences, not measured application failures.
[0.7.20 line codec source](https://github.com/tokio-rs/tokio/blob/5ec9e6b8d5a7111fe9cd35a637bd8b4e4eb1e483/tokio-util/src/codec/lines_codec.rs).
The source revision comes from `.cargo_vcs_info.json` in the
[published 0.7.20 archive](https://crates.io/api/v1/crates/tokio-util/0.7.20/download).

A persistent `Framed` owns buffers across frames and cancelled reads, removing
some handwritten plumbing. It does **not** implement this application's idle versus
transfer deadline: waiting for a server ack can allow 15 seconds before first data,
then at most two seconds to finish, capped by the original 15-second deadline.
The desktop allows heartbeat plus transfer time while idle. A blanket two-second
timeout would kill healthy idle feeds; a fresh timeout per byte permits indefinite
trickling. Preserve a fixed absolute deadline and first-data transition, possibly
through a small policy-aware reader/codec wrapper. Outgoing partial writes need a
single absolute deadline too.
[Current deadline implementation](../../crates/core/src/connection.rs),
[Tokio timeout_at](https://docs.rs/tokio/1.54.1/tokio/time/fn.timeout_at.html),
[0.7.20 Framed source](https://github.com/tokio-rs/tokio/blob/5ec9e6b8d5a7111fe9cd35a637bd8b4e4eb1e483/tokio-util/src/codec/framed.rs).

Cancellation must match the selected future. Tokio explicitly lists `read` and
`read_buf` as cancellation-safe, and `read_exact`, `read_to_end`, `read_to_string`
and `write_all` as unsafe to restart after a losing `select!` branch. Retain read
buffers and write offsets outside discarded futures, retain/poll the same future,
or close the connection on cancellation instead of resuming an incomplete frame.
A partially transmitted action remains potentially admitted: cancelling its waiter
does not justify retry. EOF and closed channels are terminal branches, not an
invitation to spin.
[Tokio 1.54.1 cancellation-safety table](https://docs.rs/tokio/1.54.1/tokio/macro.select.html#cancellation-safety).

CancellationToken requests cooperative stop; plain token drop does not mean cancel.
An intentional drop guard can request it. It cannot itself wake a standard blocking
socket or Condvar. With async sockets, cancellation can race the I/O/ack/timer wait,
then the owner closes its resources and awaits completion. JoinSet owns multiple
tasks and aborts them on drop; `abort_all` still requires draining joins to observe
completion. Neither operation establishes provider-process termination or durable
flush. For a single subscription an owned JoinHandle may be the smaller fit.
[tokio-util 0.7.20 CancellationToken](https://docs.rs/tokio-util/0.7.20/tokio_util/sync/struct.CancellationToken.html),
[Tokio 1.54.1 JoinSet](https://docs.rs/tokio/1.54.1/tokio/task/struct.JoinSet.html).

The journal remains independent of those waiting primitives. Broadcast subscribers
start at new messages and can lag past overwritten entries; application cursor
replay and resync must still be implemented. Similarly, an async oneshot can
signal frontend acceptance, but checking the exact subscription/batch/cursor must
precede sending that signal. Tauri Channel send alone is not accepted dirty state.
[Versioned broadcast source](https://docs.rs/tokio/1.54.1/src/tokio/sync/broadcast.rs.html),
[current exact matching](../../crates/core/src/connection/subscription.rs).

`thiserror` derives Display/Error without prescribing the error abstraction. Here
RepositoryError's existing implementation is only a few lines, while RequestFailure
is primarily a serialized protocol DTO. Adding an Error derive does not simplify
F02's delivery/authority policy. Broader RPC frameworks would require adapting the
same version/owner/ack/recovery semantics; this audit found no established repeated
RPC service boilerplate that justifies that larger replacement.
[thiserror 2.0.21](https://docs.rs/thiserror/2.0.21/thiserror/),
[existing errors](../../crates/core/src/workspaces.rs),
[native protocol](../../crates/core/src/connection.rs).

## Scope now and conditions for a later migration

The narrow next change can remove repeated declarations with ts-rs and replace
atomic permits with Semaphore while preserving existing interfaces. Safe polling
is a separate optional cleanup. These changes do not require reopening the socket
architecture. The agreed F03 plan explicitly retains synchronous exchanges;
research alone does not alter that decision.

If the next transport requirement introduces many persistent streams, actual
blocking-worker pressure, repeated cancellation complexity, or a deliberate async
runtime/process ownership design, revisit a coherent Tokio transport migration.
Its scope should include client and supervisor sockets, strict bounded newline
codec, absolute two-phase deadlines, owned cancellation/completion, journal wake
adaptation and bounded dispatch of blocking core work. Preserve access control,
discovery exclusion, version matching and uncertain-send recovery. This is a real
alternative with maintainability benefits, not a claim that synchronous code is
intrinsically preferable.

Existing [F04 evidence](../evidence/frontend-navigation/f04.md) reports passing
controlled Rust/tooling/frontend checks, while actual socket, fixture-desktop,
latency, packaging and native checks remain unperformed. This research adds no new
qualification. Before selecting on performance, measure representative workloads
and actual desktop responsiveness under [repository policy](../agents/verification.md).
No thread-memory figure, binary-size estimate, line-deletion count or speed ratio
has been inferred from crate documentation.
