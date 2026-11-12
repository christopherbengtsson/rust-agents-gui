# Durable I/O

## Completion and errors

Identify what a successful operation establishes: bytes accepted by a writer,
buffers flushed to an underlying writer, a database transaction committed, or the
storage guarantee required by the caller. Report completion at that actual boundary.

Observe fallible completion explicitly when the outcome matters. `BufWriter`
attempts to flush on drop but ignores errors; `File` also ignores closing errors.
Flush buffered data before requesting the underlying file's required sync operation.
Choose sync and database settings from the storage contract and target platform;
a buffer flush alone establishes neither database commit nor crash durability.

Handle commit errors using the database API's resulting transaction state. For
example, SQLite can return `SQLITE_BUSY` from COMMIT while leaving the transaction
active. Preserve that distinction instead of assuming every error rolled back.

## Partial effects across stores

A database transaction covers its database changes, not separate transcript or
attachment files, Git operations, or provider execution. At each such boundary,
identify completed effects, retained recovery data, and uncertain outcomes. Follow
the owning module's MVP policy for ordering, reconciliation, and retry decisions.

For archive/restore and content publication, account for partial writes, failed
flush/sync/commit, and incomplete cleanup before reporting success or deleting
recovery material. An explicit finish method can expose these outcomes; destructor
cleanup remains a fallback. Keep process/disk work off presentation execution paths
using the existing execution seam.

## Sources

- [Rust File source, 1.99.1](https://raw.githubusercontent.com/rust-lang/rust/1.99.1/library/std/src/fs.rs): closing errors and sync operations.
- [BufWriter](https://doc.rust-lang.org/std/io/struct.BufWriter.html): buffering and fallible flush.
- [SQLite transactions](https://www.sqlite.org/lang_transaction.html): COMMIT outcomes and transaction state.
- [SQLite atomic commit](https://www.sqlite.org/atomiccommit.html): database transaction scope; its described mechanism applies to rollback mode.

Curated from upstream `perf-io-buffering` and `anti-empty-catch`, with storage
completion semantics checked against the primary sources above. Check the selected
API version and platform when implementing a durability guarantee.
