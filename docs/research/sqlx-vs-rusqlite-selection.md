# SQLx versus rusqlite: when to choose each

2026-11-15 · Independent primary-source research for the durable-chat decision.
This adds a driver-selection assessment; it does not replace ticket 06a or implement
ticket 06b. No builds, benchmarks, application/provider runs, or private Conductor/RAT
data access were performed.

2026-11-16 follow-up: the separately authorized
[bounded async architecture comparison](storage-async-architecture-comparison.md)
is complete. It selects the maintained rusqlite bridge for the next implementation,
with explicit workload and qualification limits. This source-selection assessment
remains the earlier reasoning checkpoint; its conditional choice is resolved by
the later assessment, without making SQLx's library mechanisms less useful.

## Answer and evidence categories

**Neither library is the automatic choice for many workspaces or agents.** SQLx
is attractive when an async storage interface, maintained connection management,
and optional checked-query/migration tooling remove useful application code.
rusqlite is attractive when SQLite-specific control and whole synchronous transaction
closures are central. Both can serve concurrent async callers through worker threads.
These are selection inferences from the mechanisms below.

For this async supervisor, **SQLx deserves a fully async architecture comparison
before committing to the existing rusqlite recommendation**. Prefer SQLx if its
async APIs/tooling meaningfully simplify storage and the private-open requirement
can be preserved. Prefer rusqlite, potentially with a maintained async bridge, if
direct SQLite APIs and precise open/lifecycle control outweigh that simplification.
The current evidence supports a conditional choice, not a measured driver winner.

- **Documented facts** below come from versioned official docs/source.
- **Application requirements** come from [MVP](../MVP.md#starting-architecture)
  and [verification policy](../agents/verification.md), not library marketing.
- **Measured evidence** is the existing [driver experiment](../evidence/milestone-01-storage/driver-notes.md).
- **Inferences/preferences** are explicitly identified; they are not new runtime qualification.

## Version cohort and dependency constraints

| Candidate            | Verified cohort                      | Meaning                                                                                                                                                                                                                                                                                                                                                                      |
| -------------------- | ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| rusqlite             | 0.42.0; minimum Rust 1.89            | Current application cohort, not a claim that 0.42 is latest. [Manifest](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/Cargo.toml)                                                                                                                                                                                                                              |
| Bundled engine       | libsqlite3-sys 0.40.0; SQLite 3.54.1 | Version is established by the [sys manifest](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/libsqlite3-sys/Cargo.toml) and [bundled header](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/libsqlite3-sys/sqlite3/sqlite3.h).                                                                                                                      |
| SQLx                 | 0.9.0; minimum Rust 1.94             | Fits the recorded Rust 1.99.1 cohort. [Manifest](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/Cargo.toml)                                                                                                                                                                                                                                                       |
| SQLx measured engine | sys 0.37.0; SQLite 3.51.3            | The [allowed range](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/Cargo.toml) is `>=0.30.1,<0.38.0`; [sys manifest](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/libsqlite3-sys/Cargo.toml) and [header](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.39.0/libsqlite3-sys/sqlite3/sqlite3.h) establish the measured engine. |

SQLx documents that only one libsqlite3-sys version can appear in a dependency
graph. Its current range excludes the application's 0.40.0. Consequently, these
exact two stacks cannot simply coexist in one Cargo graph; an isolated comparison
or deliberately reconciled dependency cohort is required. Bundled versus system
linkage is another explicit choice, not an inherent property of async versus sync.
[SQLx linking/version policy](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/lib.rs).

The prior harness records Rust 1.99.1 and Tokio 1.54.1; this research did not rerun
toolchain checks. Tokio behavior discussed here uses the [1.54.1 source](https://raw.githubusercontent.com/tokio-rs/tokio/tokio-1.54.1/tokio/src/task/blocking.rs).

## Async access, pooling, and actual concurrency

**SQLx is more than async syntax.** Its SQLite driver delegates blocking SQLite
calls to one dedicated thread per connection. Configurable command and row buffers
apply backpressure; a slow result consumer can hold up that connection's worker.
This supplies maintained async plumbing, not asynchronous disk I/O inside SQLite.
[Connection](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/mod.rs), [worker](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/worker.rs), [buffer options](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/options/mod.rs).

SQLx's pool reuses connections, caps their number, queues acquisitions fairly,
and supports explicit close that waits for checked-out connections to return.
Options include acquisition timeout, connection lifetime, and setup/reuse hooks.
These are useful library-owned mechanisms absent from a bare rusqlite connection.
[Pool contract](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/pool/mod.rs), [pool options](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/pool/options.rs).

**Inference:** a connection cap bounds active connections, not all waiting tasks,
payload bytes, or work accepted before a caller disconnects. FIFO connection
acquisition is not per-workspace fairness, urgent-intent priority, or transcript
coalescing. Those policies still belong to the application's Rust owners under
the [MVP](../MVP.md#starting-architecture). SQLx buffers are useful underneath them.

rusqlite exposes direct synchronous connection/statement calls. Keep blocking
operations off Tokio executor threads; `spawn_blocking` can bridge finite work,
but started blocking tasks cannot be aborted. Tokio recommends dedicated threads
for persistent blocking workloads. A persistent storage owner is therefore an
intentional design choice, not something a mutex alone supplies.
[rusqlite API source](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/lib.rs), [Tokio blocking guidance](https://raw.githubusercontent.com/tokio-rs/tokio/tokio-1.54.1/tokio/src/task/blocking.rs).

**Inference:** a whole rusqlite transaction closure can make one worker round trip;
SQLx operations await its worker protocol. This changes handoffs and code shape,
without establishing a material performance penalty. SQLx removes worker/pool code
ownership. Assess that benefit and latency, not context-switch counts alone.

SQLite permits multiple simultaneous read transactions from separate connections,
including separate threads/processes, but only one simultaneous writer per
database. WAL permits reader/writer overlap; it does not create parallel writers.
Neither library requires one connection per conversation, and neither gains write
parallelism merely by increasing pool size.
[SQLite transactions](https://www.sqlite.org/lang_transaction.html), [WAL concurrency](https://www.sqlite.org/wal.html).

**Inference for many agents:** many tasks can share a bounded writer and a few
readers with either stack. Demand means transactions/bytes per second, query duration,
durability latency and fairness—not chat count. Parallel readers may help context
requests; long read transactions can impede WAL checkpoint progress.
[WAL checkpoint behavior](https://www.sqlite.org/wal.html#checkpointing).

## Transactions, consistent context, and cancellation

A consistent multi-query context read needs one connection/transaction rather
than unrelated pool queries. SQLite readers see committed state across separate
connections, and WAL read transactions retain their starting snapshot. Both
drivers can express that; snapshot consistency follows the transaction boundary,
not the spelling of `.await`.
[SQLite isolation](https://www.sqlite.org/isolation.html).

rusqlite transactions default to rollback on drop, support explicit commit/rollback
and DEFERRED/IMMEDIATE/EXCLUSIVE starts. SQLx transactions also arrange rollback
when dropped; its drop path starts rollback rather than awaiting a result.
Explicit transaction completion remains valuable when cleanup errors matter.
[rusqlite transaction implementation](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/transaction.rs), [SQLx transaction implementation](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/transaction.rs).

SQLx 0.9's SQLite worker specifically handles a cancelled/unacknowledged BEGIN by
rolling back, shutting down the worker if that rollback fails. It also records
when COMMIT completed without acknowledgement so the dropped transaction's later
rollback does not undo unrelated work. This is meaningful maintained lifecycle
logic. It does **not** mean a cancelled COMMIT was not committed: the source
explicitly accommodates completed-but-unacknowledged COMMIT.
[SQLx worker cancellation paths](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/worker.rs).

SQLite COMMIT can return BUSY with the transaction still active. A lost receiver,
timeout, or driver error must not be interpreted as proof that an admitted action
had no durable effect. **Application inference:** retain action identity/status,
inspect authoritative state, and reconcile before retrying external effects,
with either stack. This follows the [MVP's execution contract](../MVP.md#starting-architecture).
[SQLite COMMIT behavior](https://www.sqlite.org/lang_transaction.html#implicit_versus_explicit_transactions).

SQLx documents explicit connection close to observe closure success/error;
dropping only signals shutdown. rusqlite's explicit close returns the connection
with an error on failure. **Inference:** neither a dropped request future nor a
dropped handle alone is the supervisor's physical-completion record. Admission,
draining, outstanding tasks and shutdown completion remain owned application work.
[SQLx close](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/mod.rs), [rusqlite close](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/lib.rs).

## SQLite-specific access and private opening

rusqlite offers feature-gated incremental BLOB I/O, stepped online backup, custom
SQL functions, and hooks. These are practical reasons to choose it when an actual
feature needs them; ordinary BLOB columns alone do not require incremental I/O.
[BLOB APIs](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/blob/mod.rs), [backup](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/backup.rs), [functions](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/functions.rs), [hooks](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/hooks/mod.rs).

SQLx is not restricted to generic SQL: it exposes locked raw-handle access,
progress/update/commit/rollback hooks and optional preupdate hooks. Its checked
source has no corresponding high-level incremental BLOB or online-backup wrapper;
using raw SQLite APIs adds safety/lifetime work. SQL-based backup remains possible
with either driver. Choose based on required APIs rather than assuming SQLx cannot
use SQLite-specific facilities.
[SQLx handle/hooks](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/mod.rs),
[SQLite backup alternatives](https://www.sqlite.org/backup.html).

A concrete adoption question exists: current [storage opening](../../crates/core/src/storage.rs)
requests `SQLITE_OPEN_NOFOLLOW` alongside private-file checks. SQLx 0.9's inspected
options/flag construction exposes no NOFOLLOW flag; locking its handle happens
after opening. **Inference:** demonstrate an equivalent supported private-open
mechanism, or explicitly assess a maintained extension, before replacing that
guarantee. This is a capability gap to resolve, not proof that SQLx is unsafe or
impossible to adopt.
[SQLx opening flags](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/establish.rs).

## Checked queries, migrations, and maintenance

SQLx `query!` checks literal SQL and result types against a build-time schema or
prepared `.sqlx` metadata. Dynamic SQL is outside that introspection; inferred
nullability can need overrides. Builds bind to the checked database backend.
This catches query/schema mistakes, not recovery or durability policy. Macros are
optional; runtime queries do not acquire these checks merely by using SQLx.
[Query macro contract](https://docs.rs/sqlx/0.9.0/sqlx/macro.query.html).

Offline builds require maintained query metadata: prepare against the intended
schema, commit `.sqlx`, check freshness in CI, and include relevant targets/features.
`SQLX_OFFLINE=true` prevents an incidental DATABASE_URL from forcing live checking.
**Inference:** that is additional preparation infrastructure with a real benefit,
not a free switch and not permission to run database-backed application unit tests.
[SQLx CLI workflow](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-cli/README.md),
[repository testing policy](../agents/verification.md).

SQLx includes migration files/CLI and an embeddable Migrator with applied-version
and checksum validation. **Inference:** SQLx reduces migration tooling ownership
compared with using rusqlite's connection API alone. Current policy requires no
retired-format support; migration convenience should not invent that task.
[SQLx Migrator](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/migrate/migrator.rs),
[rusqlite API](https://raw.githubusercontent.com/rusqlite/rusqlite/v0.42.0/src/lib.rs),
[CLI](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-cli/README.md),
[development policy](../MILESTONE-01.md#development-compatibility-policy).

SQLx supports SQLite/PostgreSQL/MySQL and a runtime-selected Any driver. That can
standardize APIs across a real multi-database product; it does not translate SQLite
SQL, PRAGMAs, schema or transaction semantics into portable equivalents. Checked
macros are backend-specific. **Inference:** hypothetical future remote storage is
not sufficient reason alone to choose SQLx now.
[Supported drivers](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/README.md),
[Any](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/any/mod.rs),
[SQLite-specific PRAGMAs](https://www.sqlite.org/pragma.html).

Both libraries have optional features; SQLx defaults include macros/migrations/Any
and can be trimmed. SQLx 0.9 also requires audited treatment of owned SQL strings;
bound values remain the recommended path. **Inference:** compare the selected
feature graph and code eliminated, not package counts as a proxy for quality.
[SQLx features](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/Cargo.toml),
[SQL string contract](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/sql_str.rs).

## Maintained rusqlite async alternatives

**tokio-rusqlite 0.9.0** is current in the [registry metadata](https://crates.io/api/v1/crates/tokio-rusqlite)
checked directly; its [tagged manifest](https://raw.githubusercontent.com/programatik29/tokio-rusqlite/v0.9.0/Cargo.toml)
requests rusqlite 0.42.0. This is compatible at the declared dependency level with
the current cohort; no build qualification was performed. Cached 0.8.0 docs describe
a different rusqlite dependency and must not be substituted silently.

Its handle sends closures to a per-connection thread; whole transactions can stay
in one closure. The queue is **unbounded**; execution ignores failed reply delivery.
Explicit close reports SQLite close failure; no thread join handle is exposed.
**Inference:** evaluate this bridge before writing another worker, with application
admission, retained completion and shutdown evidence. It is not a bounded scheduler.
[0.9.0 implementation](https://raw.githubusercontent.com/programatik29/tokio-rusqlite/v0.9.0/src/lib.rs).

**deadpool-sqlite 0.14.0** provides an async rusqlite pool with blocking interaction.
Its published manifest requests rusqlite 0.40.0, not this cohort's 0.42.0; do not
recommend adding it without resolving version/linkage compatibility first.
[Pool documentation](https://docs.rs/deadpool-sqlite/0.14.0/deadpool_sqlite/),
[published manifest](https://docs.rs/crate/deadpool-sqlite/0.14.0/source/Cargo.toml.orig).

## Decision matrix and what existing evidence can decide

| Main priority                                                            | Preference               | Qualification for this app                                                         |
| ------------------------------------------------------------------------ | ------------------------ | ---------------------------------------------------------------------------------- |
| Async storage calls and maintained pool/backpressure/lifecycle mechanics | SQLx                     | Preserve private opening and app admission; use a deliberate writer/read topology. |
| Literal query/schema checks and maintained migration tooling             | SQLx                     | Accept metadata preparation/CI and optional feature cost.                          |
| Incremental BLOB/backup/custom SQLite functions and direct flags         | rusqlite                 | Confirm actual feature need; avoid making speculative APIs decisive.               |
| Async callers plus whole rusqlite transaction closures                   | tokio-rusqlite candidate | Bound admission externally and qualify completion/shutdown.                        |
| Many conversations/subagents                                             | Neither by count         | Measure bursts, read fanout, bytes, transaction grouping, latency and fairness.    |
| Future PostgreSQL/MySQL deployment already selected                      | SQLx                     | Rework SQL/data/ownership semantics explicitly; no automatic portability.          |

These preferences are inferences from the cited mechanisms, not benchmark rankings.
The [retained driver experiment](../evidence/milestone-01-storage/driver-notes.md)
found both grouped stacks near the offered 500 synthetic events/s without rejection.
It used SQLx `block_on` per operation behind a synchronous owner worker, different
bundled engines, and short windows. The minimal SQLx build omitted macros/migrations. It does not fairly evaluate a fully
async architecture, checked queries, realistic read fanout, long soaks or app capacity.

Conductor evidence is narrower: retained shipped-code inspection found Tauri SQL
plugin load/select/execute calls and WAL setup. The [plugin documents SQLx backing](https://v2.tauri.app/plugin/sql/);
Conductor's rationale, driver version, transaction design and capacity remain unknown. See the [retained comparison](conductor-frontend-architecture.md#comparison-with-our-process-architecture).
Its choice is useful precedent, not comparative proof for this supervisor.

**Next decision evidence, if separately selected:** compare a genuine async SQLx
storage boundary against rusqlite/bridge with equivalent durability and admission,
then assess code ownership, read fanout, cancelled replies/commits, close/drain,
private opening, and query-preparation workflow. This research authorizes no runs
or migration. Documentation access was supplemented with direct tagged-source
HTTP reads when versioned docs.rs pages failed or caches disagreed; no secondary
article was used as authority.
