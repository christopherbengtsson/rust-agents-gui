# Synchronous resources

## Guards and destruction order

Keep lock, permit, and ownership guards alive through the work they protect.
A named binding such as `_guard` retains a returned guard; `let _ = acquire()?;`
discards that temporary guard at the statement. Retain the guard itself when
moving protected work into another owner.

Struct fields drop in declaration order; locals drop in reverse declaration
order. Arrange dependent resource lifetimes accordingly, or use an explicit
shutdown method when completion order and errors matter. Include early returns
and unwinding in the lifetime argument. RAII releases scope-bound resources;
fallible or blocking completion needs an observable outcome when the caller
depends on it. For persistence completion, read [Durable I/O](durable-io.md).

## Threads and processes

Assign an owner to thread completion and child-process wait/stop outcomes. Dropping
a standard `JoinHandle` detaches its thread; dropping `std::process::Child` neither
stops nor waits for the child. Retain handles when later completion or cleanup must
be observed. An intentional detached waiter can reap a process whose lifetime
belongs elsewhere; make that ownership and outcome path explicit.

Process wait establishes that child's exit, not termination of all its descendants.
Use the runtime owner's stop/cleanup mechanism and its evidence for owned child work.
Keep blocking joins and waits off presentation paths; arrange a stop strategy for
work that may otherwise prevent shutdown indefinitely. Select async task mechanics
from [Async lifecycle](async-lifecycle.md) when applicable.

## Sources

- [Rust destruction rules](https://doc.rust-lang.org/reference/destructors.html).
- [JoinHandle](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html) and [Child](https://doc.rust-lang.org/std/process/struct.Child.html).

Curated from upstream `mem-drop-order` at the revision pinned in the skill entry
point, with standard thread/process contracts checked separately. Preserve the
MVP's distinction between conversation, turn, and runtime lifetimes.
