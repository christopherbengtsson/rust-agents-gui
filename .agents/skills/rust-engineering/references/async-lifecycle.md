# Async lifecycle

Apply these patterns when async execution is part of the selected implementation;
they do not require migrating synchronous code or adding Tokio.

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

Sources: `async-bounded-channel`, `async-broadcast-pubsub`, `async-no-lock-await`,
`async-cancel-safety`, `async-cancellation-token`, and `async-joinset-structured`
in the pinned upstream collection. Consult the dependency version selected by the
workspace when using the linked API documentation.
