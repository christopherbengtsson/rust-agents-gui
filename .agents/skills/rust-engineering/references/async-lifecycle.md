# Async lifecycle

Apply these patterns when async execution is part of the selected implementation;
they do not require migrating synchronous code or adding Tokio.

For tokio-util codecs, buffered frame ownership or read/write deadlines, read
[Framed I/O](framed-io.md). This reference owns cooperative task cancellation and
completion; the framing reference applies that lifecycle to byte streams.

## Runtime placement and blocking dispatch

Follow [observation guidance](../../../../docs/agents/observations.md#physical-completion-and-cancellation)
for application runtime ownership and transport admission. A library operation
uses the supplied execution context rather than constructing a runtime per call.
Current-thread execution can drive concurrent I/O when its owner continuously
polls it; the connection count alone does not select the runtime flavor. See
[Tokio runtime configurations](https://docs.rs/tokio/1.54.1/tokio/runtime/index.html).

Keep synchronous Git, storage and filesystem work behind the existing effect seams
and dispatch it away from executor workers. Admit bounded work before submitting
it; a blocking pool's internal queue is not application admission. Transfer the
owned permit into the actual worker so dropping its async waiter cannot free
capacity. Preserve a completion owner when that waiter goes away. Merely wrapping
an operation in `async fn` or replacing one call with `tokio::fs` does not define
the surrounding operation's resource budget or durability boundary.

`spawn_blocking` suits work that eventually finishes. Persistent blocking loops
need an intentionally owned execution strategy; avoid occupying blocking-pool
threads indefinitely. Started blocking work continues after handle abort or a
runtime shutdown timeout. See [Tokio blocking dispatch](https://docs.rs/tokio/1.54.1/tokio/task/fn.spawn_blocking.html).

## Bounded delivery

Bound queue capacity and individual payload/frame sizes. A bounded count of
arbitrarily large messages still permits arbitrarily large memory use. Include
in-flight work in the resource budget.

Choose full-queue and closed-receiver behavior for the message's meaning. Commands
and durable events need explicit admission or delivery outcomes; latest-value
observations can permit coalescing. Keep process output draining and control paths
responsive when a presentation consumer is slow.

A Tokio broadcast receiver can miss overwritten messages. Treat `Lagged` as a gap
requiring the applicable snapshot/cursor recovery; a broadcast queue alone cannot
provide durable history or complete delivery. See [broadcast lagging](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html#lagging).

## Locks and effects

Use short synchronous lock scopes for ordinary shared data and keep blocking work
off executor threads. A Tokio mutex can intentionally span an await when serializing
an async resource is required; document that scope and its contention cost.

For extract/await/reacquire flows, revalidate identity and state before applying
the result. Unlocking around an effect changes the race surface; cloning data out
alone does not preserve the operation's authority. See [Tokio mutex guidance](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html#which-kind-of-mutex-should-you-use).

## Cancellation and owned cleanup

Inspect every operation raced in `select!` or a timeout for partial progress and
external effects. Keep framing buffers and progress outside futures that may be
dropped, or retain and poll the same future across iterations. Distinguish stream
EOF and channel closure from a pending operation to avoid spinning. See [select
cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html#cancellation-safety).

Choose `join!`, `try_join!` and `select!` by their failure and cancellation contract,
not only by latency. An early error can drop sibling futures; dropping a future
or a task handle does not establish completion of effects it already started.
Keep handles/recovery evidence with an owner that can finish cleanup after the
requesting future has returned. Independent observations can run together within
their bounds; effect ordering and intent-before-effect remain domain decisions.

Assign an owner to spawned tasks and observe their completion. A cancellation
token requests cooperative shutdown; use explicit cancellation or an intentional
drop guard rather than assuming token drop cancels its children. After requesting
stop, join owned tasks and account for errors and deadlines. `JoinSet` aborts on
drop, which is a fallback mechanism rather than evidence of completed cleanup.

Task cancellation and provider-process termination are separate outcomes. Await
required cleanup and retain any failure or uncertainty needed by the lifecycle
owner. Started blocking tasks also need their own stop strategy; aborting an async
handle does not stop them. See [CancellationToken](https://docs.rs/tokio-util/latest/tokio_util/sync/struct.CancellationToken.html),
[JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html), and
[Child lifecycle](https://docs.rs/tokio/latest/tokio/process/struct.Child.html#caveats).

Sources: `async-tokio-runtime`, `async-spawn-blocking`, `async-tokio-fs`,
`async-join-parallel`, `async-try-join`, `async-select-racing`,
`async-bounded-channel`, `async-broadcast-pubsub`, `async-no-lock-await`,
`async-cancel-safety`, `async-cancellation-token`, and `async-joinset-structured`
in the pinned upstream collection. Consult the dependency version selected by the
workspace when using the linked API documentation.
