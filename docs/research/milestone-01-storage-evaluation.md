# Storage evaluation before durable chat

2026-11-15 · Milestone 01 ticket 06a · Research and disposable experiments.
Baseline source: `24390a1e7ba108ca88704fbf73deac92915d3ab6`.

## Decision and scope

**Recommendation:** retain bundled rusqlite, use one intentional Rust-owned writer,
and adopt WAL/FULL/fullfsync for authoritative state in a separately selected
implementation. Keep transcripts and initially supported attachment content in
the authoritative database. Do not add a disposable cache database yet.
The short measured cohorts support these choices; they do not qualify a production
storage implementation or establish a universal capacity ceiling.

**Follow-up selection clarification, 2026-11-15:** treat retaining rusqlite as
the baseline recommendation for this bounded cohort, not a settled driver choice
for the full async/subagent workload. The
[SQLx versus rusqlite selection research](sqlx-vs-rusqlite-selection.md)
weighs library-owned pooling, backpressure, cancellation and tooling alongside
SQLite-specific control. The earlier benchmark put SQLx behind a synchronous
worker and did not measure hierarchical subagent fan-out or workspace fairness.
Its measurements remain valid for their recorded configurations.

**Bounded comparison completed, 2026-11-16:** the
[genuine async architecture comparison](storage-async-architecture-comparison.md)
selects rusqlite with maintained tokio-rusqlite workers and preserves current
opening behavior. Both stacks accepted its baseline; the original heavy grouping
policy overloaded both, and a larger-group diagnostic remains separately labeled.
Use that later assessment and prepared 06b for implementation choices; this
document retains its original measured cohort and durability/layout evidence.

The design horizon is the full local workspace: concurrent conversations, durable
history, background reviews, chained agent sessions, shared context, and eventual
GitHub/Jira synchronization. The accepted deployment remains one local Rust
supervisor owning authoritative storage. Future remote coordination is a seam to
identify, not another deployment to implement in this study. These directions
extend workload coverage without changing the [MVP's module ownership](../MVP.md#starting-architecture)
or implementing a workflow engine.

This research authorizes no production storage migration, frontend SQL access,
provider submission, or ticket-07 implementation. No Conductor databases,
runtime credentials, provider history, or RAT files are research inputs.

## Actual starting point and correctness gates

The current [storage module](../../crates/core/src/storage.rs) contains SQLite
mechanics; [workspaces](../../crates/core/src/workspaces.rs) retains registration,
creation, and recovery decisions. `OwnerState` keeps a persistent `SqliteStore`
behind its mutex, shared by workspace requests and the workspace worker.
`ProviderLoginStore` and `ProviderSelectionStore` open another connection for each
read/write; their domain owners dispatch persistence through `spawn_blocking`.
These are different connection lifetimes before any driver is changed.

Every `SqliteStore::connect` currently sets a 100 ms busy timeout, `DELETE`,
`synchronous=FULL`, `fullfsync=ON`, and foreign keys. Opening validates private,
regular, single-link database/sidecar files and creates the database with mode
0600 and no-follow flags. These are code observations, not fresh OS qualification.

Candidate probes must preserve these existing application promises:

- `RepositoryStore::finish` saves registration completion, the repository, and
  initial workspace/action in one transaction ([storage implementation](../../crates/core/src/storage.rs)).
- Allocated workspace name, branch, and path remain immutable on recovery;
  uniqueness reservations survive failed creation and later retries.
- Reads expose committed state after failed commits. `ensure_committed` replaces
  a connection still inside a transaction; registration recovery reads the saved
  action rather than creating another identity ([recovery caller](../../crates/core/src/workspaces.rs)).
- Provider bindings belong to conversations; workspace identity remains provider
  independent ([completed prerequisite](../../.scratch/provider-independent-workspaces/issues/01-retire-workspace-provider.md)).
- Private files, bounded admission, retained physical work, visible failure, and
  shutdown completion remain requirements. A faster candidate failing them is
  not an equivalent replacement.

The existing private transaction seam supplies dependency outcomes to deterministic
units. Keep it or equivalent useful coverage. Repository policy permits such seams
with one production adapter; the goal is locality, not interface proliferation.
Storage should hide mechanics while callers retain admission/retry/reconciliation.

## Supported source cohort and fair comparison

| Stack          | Source-established version                                              | Consequence                                                                                             |
| -------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| Current core   | rusqlite 0.42.0, locked libsqlite3-sys 0.40.0; bundled SQLite 3.54.1    | Preserve this as the shipping baseline.                                                                 |
| SQLx candidate | SQLx 0.9.0; minimum Rust 1.94.0                                         | Compatible with repository Rust 1.99.1. Disable defaults and select SQLite/runtime features explicitly. |
| SQLx engine    | Accepted sys range `>=0.30.1,<0.38.0`; sys 0.37.0 bundles SQLite 3.51.3 | Separate pinned packages; bundled-stack results include an engine-version difference.                   |

Sources: [current bundled header](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/libsqlite3-sys/sqlite3/sqlite3.h),
[SQLx package/features/MSRV](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/Cargo.toml),
[SQLx driver dependency range](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/Cargo.toml),
[sys 0.37.0 manifest](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/libsqlite3-sys/Cargo.toml),
and [its bundled header](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/libsqlite3-sys/sqlite3/sqlite3.h).

SQLx documents that only one sqlite3-linked sys version may occur in a dependency
graph; its range excludes the current core's sys 0.40.0. Adding both to this
workspace would not provide a neutral comparison. The `sqlite` feature statically
bundles SQLite; unbundled linkage is a separate configuration with its own runtime
library/compile-option provenance. [SQLx linking guidance](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/lib.rs).

SQLx's SQLite interface uses a worker thread and channels over blocking SQLite.
Dropping a connection requests worker shutdown and cannot report close errors;
explicit `close()` observes completion or error. Its defaults include a 5 s busy
timeout and 100 cached statements. Compare equivalent connection reuse, timeout,
cache, transaction, and sync settings before crediting the driver for a topology
change. [Connection implementation](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/mod.rs),
[connection defaults](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/options/mod.rs).

## Journal, synchronization, and transaction meaning

| Configuration              | Documented meaning to preserve in result labels                                                        |
| -------------------------- | ------------------------------------------------------------------------------------------------------ |
| DELETE / FULL / fullfsync  | Current baseline; rollback-mode FULL is not universally power-loss durable.                            |
| DELETE / EXTRA / fullfsync | Adds directory synchronization after deleting the rollback journal.                                    |
| WAL / FULL / fullfsync     | Adds commit-time WAL synchronization; documented durable tier subject to platform/storage assumptions. |
| WAL / NORMAL               | Can lose acknowledged transactions after OS failure/power loss; a separate durability tradeoff.        |

`fullfsync` chooses the macOS `F_FULLFSYNC` mechanism; it does not mean `EXTRA`.
With it enabled, separate `checkpoint_fullfsync` changes nothing. Read back
effective settings: unsupported PRAGMAs may be silently ignored.
[SQLite PRAGMA reference](https://www.sqlite.org/pragma.html).
Apple explains the drive-cache ordering distinction between fsync and
F_FULLFSYNC, but its web manual is archived; retain local supported-OS evidence
when qualifying the target filesystem. [Apple fsync manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html).

SQLite allows several readers but one writer. `COMMIT` may return `SQLITE_BUSY`
while the transaction remains active; the caller must inspect transaction state
and finish cleanup before exposing a committed-state read. A timeout is not proof
of rollback. [Transaction documentation](https://www.sqlite.org/lang_transaction.html).
Started Tokio blocking tasks cannot be aborted; runtime shutdown may wait for
their completion. Persistent blocking loops require intentional ownership.
[Tokio 1.54.1 blocking-task documentation](https://raw.githubusercontent.com/tokio-rs/tokio/tokio-1.54.1/tokio/src/task/blocking.rs).

WAL permits reader/writer overlap, not concurrent writers. The default automatic
checkpoint threshold is 1,000 pages; long read transactions can retain WAL growth.
The WAL file is part of persistent database state, so copying only the live main
file is unsafe. Record threshold-crossing latency, WAL size, read-transaction
duration, recovery BUSY responses, and reopen/close cost. Both identified engines
include the WAL-reset corruption fix introduced in 3.51.3; avoid resolving an
older unfixed engine. [WAL documentation and fix](https://www.sqlite.org/wal.html).

PASSIVE checkpoints need not finish and do not invoke the busy handler. FULL waits
for eligible readers/writer; RESTART additionally waits to restart WAL; TRUNCATE
also reduces its size to zero on success. Capture log and checkpointed frame
counts, not just a successful SQL return. [Checkpoint API](https://www.sqlite.org/c3ref/wal_checkpoint_v2.html).

## History, attachments, backup, and disposable caches

Authoritative transcript rows in the same database can commit with turn/action
metadata. External transcript files add publication ordering and recovery work;
neither SQLite transactions nor attached-database guarantees include ordinary
files. WAL also does not atomically commit across attached databases as a set.
This supports probing a single authoritative database first, not assuming every
payload must be a database BLOB. [ATTACH guarantees](https://www.sqlite.org/lang_attach.html),
[rollback atomic-commit scope](https://www.sqlite.org/atomiccommit.html).

Use indexed conversation/sequence paging, bounded page bytes, and short read
transactions. Compare append-only rows against file offsets using equivalent
payloads and sync boundaries. This is an experiment hypothesis, not an implemented
history schema. Do not make UI feed cursors authoritative transcript positions.

SQLite's own small-BLOB experiments show that database storage can be competitive
with individual files; older large-BLOB results vary by size/page size. Those
studies used earlier SQLite/Linux/hardware and provide no current-macOS cutoff.
Measure actual attachment sizes and access patterns before choosing a threshold.
[Small-BLOB study and caveats](https://www.sqlite.org/fasterthanfs.html),
[older internal/external comparison](https://www.sqlite.org/intern-v-extern-blob.html).

For external attachments, probe private temporary creation, complete write and
explicit sync, publication to an immutable final identity, then database reference
commit. Recovery must distinguish incomplete temporary files, published orphans,
and committed references; retain recovery material on uncertainty. This protocol
is an architectural hypothesis requiring experiments and later owning-module
implementation. Rust file close ignores errors, so destruction is insufficient
evidence of completed persistence. [Rust 1.99.1 File source](https://raw.githubusercontent.com/rust-lang/rust/1.99.1/library/std/src/fs.rs).

The Online Backup API permits incremental copying with short source read locks;
measure foreground interference as well as completion and restore integrity.
`VACUUM INTO` creates a compact consistent copy, trades additional CPU for less
output, and is not incremental; interrupted output can be incomplete. With NORMAL
or FULL synchronization SQLite syncs its output before reporting completion.
Neither method includes independent attachment files: a complete application
backup needs a consistent referenced-file inventory and publication protocol.
[Backup overview](https://www.sqlite.org/backup.html),
[VACUUM INTO behavior](https://www.sqlite.org/lang_vacuum.html#vacuum_with_an_into_clause).

A disposable diff/file-index cache should be reconstructible after deletion and
carry no cross-database correctness invariant. Benchmark cache churn beside
authoritative traffic before separating it. The observed Conductor two-database
configuration establishes configuration code, not necessity or measured benefit
for this app ([retained comparison](conductor-frontend-architecture.md#comparison-with-our-process-architecture)).

## Future-workflow probes and evidence limits

Probe completion plus next-session intent in one transaction; a consistent version
of shared context; and an external-operation identity/status retained before a
simulated remote effect. Those patterns explore transaction leverage for future
owners without moving workflow policy into storage or promising exactly-once
GitHub/Jira effects. Future remote coordination would require new ownership,
conflict, and synchronization decisions; SQLx alone supplies none of them.

Real SQLite, kill-point, backup, and filesystem experiments are qualification,
separate from deterministic application units. Sampled process-crash recovery while
the OS remains alive establishes neither power-loss durability nor universal crash
safety, and storage latency alone does not qualify desktop responsiveness.

No previous-development-format reader or migration is required. Any subsequent
incompatible implementation uses fresh private application state, rejects mixed
running peers, and observes owned stop/join before changing state paths. Preserve
repository files/worktrees, runtime credentials, and archived evidence. Qualification
of the future selected implementation remains separate from this research.

Source access limit: the versioned rusqlite 0.42.0 `Connection` page on docs.rs
returned an access error in the browser tool. Tagged upstream source, the local
locked crate/header, and repository call sites supplied the retained evidence.

## Resulting design and implementation handoff

The measurements and reproducible commands are retained in the
[evidence](../evidence/milestone-01-storage/README.md). Compare complete shipping
stacks rather than claiming an engine-controlled driver speed comparison.

- **Driver and ownership:** retain rusqlite 0.42.0 with its bundled engine. SQLx
  supplies a credible async interface but no demonstrated capability advantage
  for this local workload that offsets its larger dependency/thread machinery.
  Put connection lifetime, admission and physical completion in the storage
  module; do not give each chat a connection or reuse the workspace store as a
  universal domain interface. One concrete writer is sufficient for the sampled
  grouped workload. Reader-pool benefit was not isolated; add readers only after
  a matched-load experiment or actual interactive contention warrants them.
- **Scheduling and grouping:** a bounded dedicated writer thread with a reusable
  connection is the starting design. Commit action intent before effects and
  acknowledge it only after commit. Group streamed events across conversations
  with both byte/count limits and a short deadline; 16 events/25 ms is the measured
  starting candidate, not a per-chat wait for 16 events. Expose an accepted durable
  position independently of live presentation. Preserve fairness for intent,
  history and shutdown; overload is visible admission failure, never silent loss.
- **Journal and checkpoints:** choose WAL/FULL/fullfsync with foreign keys and a
  short busy timeout; do not weaken authoritative state to NORMAL. Initially
  retain the measured 1,000-page auto-checkpoint threshold. Add checkpoint frame,
  duration and WAL-size observations, short read transactions, and bounded PASSIVE
  maintenance opportunities. Do not disable automatic checkpoints until an owned
  maintenance schedule is implemented and qualified. TRUNCATE belongs at an
  explicit idle/drained maintenance point, with its actual result checked; it must
  not block navigation or run after every event.
- **History and files:** store transcript events with their action/turn metadata
  transactionally and page through an indexed conversation/sequence position with
  row and byte bounds. Prefer database attachment content initially: the tested
  256 KiB/4 MiB payloads showed no file-publication benefit warranting an orphan
  recovery protocol. This establishes no maximum attachment size or universal
  BLOB cutoff. Re-evaluate very large media with a concrete workload before
  externalizing it. Never make the transient observation-feed cursor a durable
  transcript cursor.
- **Backup and caches:** use a consistent SQLite backup operation; the measured
  VACUUM INTO path works as a simple explicit local snapshot, not a latency-free
  background operation. Qualify incremental Online Backup before adopting it for
  recurring backups. Keep no second cache database now; if a reconstructible cache
  later demonstrably interferes with authoritative work or needs independent
  discard/retention, separation becomes justified without cross-database atomicity.

WAL alone will not remove the current workspace-owner mutex contention. A follow-up
must account for lock scopes when persistence waits move to another worker and
revalidate identity/state at the owning seam. Storage hides mechanics; registration,
conversation, login and future coordinator modules retain their decisions.
Maintainability and correct ownership are selection criteria alongside latency.

The proposed [06b implementation ticket](../../.scratch/milestone-01/issues/06b-prepare-storage-for-durable-chat.md)
is intentionally unimplemented and needs triage before being selected. Settle its
sequencing with ticket 07; this report does not silently start either ticket.
No production format changed during research, so this session requires no application
reset or restart. A later incompatible implementation uses fresh private state
after owned stop/join; never remove a live WAL/SHM pair or reset repository,
credential, provider-history or archived-evidence data.
