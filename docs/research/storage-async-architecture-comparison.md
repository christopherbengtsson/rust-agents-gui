# Async SQLite storage: bounded architecture comparison

2026-11-15–16 · Primary-source assessment and bounded disposable experiments.
This compares genuine async SQLx 0.9.0 with rusqlite 0.42.0 behind
tokio-rusqlite 0.9.0. It changes no production interface or accepted product scope.

## Decision being tested

**SQLx has a real maintenance advantage to test:** maintained async query/transaction composition, a connection pool, bounded driver buffers, and optional schema checks/migration tooling can replace application mechanisms. **The rusqlite bridge is also genuinely async at its caller interface:** it supplies maintained per-connection workers and preserves whole synchronous transaction closures and existing open flags. Neither alternative eliminates application admission, durable action identity, reconciliation, file publication or owned shutdown. These are architecture inferences from the source-backed inventory below, not benchmark results.

Select on equivalent required behavior and measured implementation burden, not agent count, SQLx's async label, inertia toward the current driver, or dependency counts alone. The earlier [selection research](sqlx-vs-rusqlite-selection.md) and [synchronous-owner experiment](../evidence/milestone-01-storage/driver-notes.md) remain historical evidence; the latter's SQLx `block_on` per operation does not establish the fully async result.

## Fixed ownership and existing callers

[MVP](../MVP.md#starting-architecture) assigns sole execution/storage ownership to the local supervisor: `storage` owns authoritative metadata and durable transcripts/attachments; `workspaces`, `conversations`, and `runtimes` retain their respective domain policy. Tauri submits explicitly targeted actions and reads observations. Retry retrieves action status; uncertain external effects require owning-module reconciliation. The driver decision cannot relocate these decisions to the desktop or pool.

Current friction is concrete:

- [storage.rs](../../crates/core/src/storage.rs) implements synchronous `RepositoryStore`/`CreationStore` plus two provider persistence interfaces. Repository registration atomically stores its first-workspace continuation; workspace writes preserve reserved locations. The existing transaction-outcome seam substitutes begin/commit/rollback evidence. These are useful semantics, not disposable wrappers.
- [workspaces.rs](../../crates/core/src/workspaces.rs) retains the store inside `OwnerState`, serializes storage and state decisions under its mutex, and owns a worker rotating creation, repository probes and inspection. A pool added under this same lock would not create read concurrency; replacing the driver does not remove the Git/inspection worker's domain work.
- [ConnectionServer](../../crates/core/src/connection.rs) dispatches non-provider synchronous routes through bounded, retained blocking tasks. [tasks.rs](../../crates/core/src/connection/tasks.rs) keeps a permit inside physical work and awaits its completion. An async store can remove **storage-only** blocking dispatch as routes become async; private-filesystem/Git/other synchronous effects still need dispatch.
- [provider login](../../crates/core/src/runtimes/provider_connection/login.rs) and [executable selection](../../crates/core/src/runtimes/provider_connection/selection.rs) each invoke persistence with `spawn_blocking`. Their [stores](../../crates/core/src/storage.rs) reconnect on each operation and apply connection PRAGMAs. Either maintained async candidate can centralize worker/connection reuse and remove these call-site wrappers while retaining login intent, stale-selection checks and cleanup outcomes locally.
- [supervisor main](../../crates/supervisor/src/main.rs) constructs the current synchronous core before entering one continuously driven current-thread Tokio runtime. Actual async construction/shutdown must use that runtime owner, rather than constructing a runtime per database call.

**Deletion test:** delete the storage module and private opening, query decoding, atomic registration/continuation, location immutability, cleanup errors and durable commits spread into callers. It is already a useful deep module. Deepening means hiding connection/dispatch/lifecycle mechanics behind its domain-shaped interface; exposing pools, SQL transactions or closure scheduling to every owner would lose locality. Existing controlled-outcome seams remain justified under [repository testing policy](../agents/verification.md), even with one production driver. [ADR 0007](../adr/0007-library-backed-observation-modules.md) supplies the same maintained-mechanism/retained-policy precedent; Rust's [design principles](https://rust-unofficial.github.io/patterns/additional_resources/design-principles.html) are tradeoffs, not a requirement for a generic storage framework.

## Source-verified library mechanisms

SQLx SQLite uses one dedicated blocking worker thread per connection. Its command queue and row delivery are bounded; buffer sizes are configurable. Streaming a result does not make SQLite disk operations asynchronous. A blocked result consumer can occupy the worker. [SQLx 0.9 connection](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/mod.rs), [worker](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/worker.rs), [options](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/options/mod.rs).

Its pool supplies connection reuse, a connection cap, FIFO acquisition, acquisition timeouts and explicit close waiting for checked-out connections. Cancellation during acquisition can discard a connection. These mechanisms do not bound all waiting application tasks or retained payload bytes and do not implement fairness between workspaces, urgent intent and bulk output. Pool scheduling is useful underneath application admission. [Pool](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/pool/mod.rs), [pool options](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/pool/options.rs). The distinction between FIFO acquisition and application fairness is an inference from those contracts and the [MVP](../MVP.md#starting-architecture).

SQLx transactions roll back on drop; the SQLite worker handles cancellation while beginning a transaction and distinguishes a commit processed without acknowledgment from an uncommitted transaction. This removes custom driver bookkeeping but cannot prove that a cancelled application action had no durable effect. Commit outcome and action status still need authoritative reconciliation. [Transaction](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/transaction.rs), [worker](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/worker.rs).

tokio-rusqlite 0.9.0 requests rusqlite 0.42.0. It sends `Send + 'static` closures to a per-connection thread through an **unbounded** channel. A whole transaction can execute in one closure; dropping the reply receiver does not cancel that closure. It supports `open_with_flags`; no thread join handle is exposed. `close` processes SQLite close and returns its error/connection on failure, but also treats channel/receiver closure as success; therefore close alone is insufficient failure evidence if worker execution panicked. [Manifest](https://raw.githubusercontent.com/programatik29/tokio-rusqlite/v0.9.0/Cargo.toml), [implementation](https://raw.githubusercontent.com/programatik29/tokio-rusqlite/v0.9.0/src/lib.rs).

**Completion inference:** acquire application capacity before task creation/closure submission, retain an owner task independently of the caller, and keep its permit inside the actual closure until it returns. Fence admission, await every retained operation result, then explicitly close. This can establish completion of admitted database effects without joining the OS thread. The bridge's successful close acknowledgment occurs after SQLite close and before the worker returns; it is not OS-thread-exit evidence. A failed operation receiver remains a failure even if subsequent close returns success. These are application obligations derived from the [bridge implementation](https://raw.githubusercontent.com/programatik29/tokio-rusqlite/v0.9.0/src/lib.rs) and [physical-completion policy](../agents/observations.md#physical-completion-and-cancellation).

SQLx explicitly closes via worker shutdown, dropping database references before acknowledgment. The source describes this as worker termination; its implementation also exposes no `std::thread::JoinHandle` to callers. For either stack, record database effect completion/closure separately from literal OS-thread exit. Do not reject the bridge merely for lacking a join if the required observable guarantee is drained effects and closed resources. [SQLx close](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/mod.rs), [shutdown](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/worker.rs).

## Private opening: contract versus implementation

The accepted contract is private local state, exclusive supervisor ownership and preserved history; it does not separately specify an arbitrary same-user attacker model or mandate a particular flag. The implementation nevertheless deliberately rejects aliases for the database/journal/WAL/SHM, creates metadata with mode 0600 and `O_NOFOLLOW`, and requests `SQLITE_OPEN_NOFOLLOW` on **every** SQLite connection in [storage.rs:197](../../crates/core/src/storage.rs#L197). [Supervisor code](../../crates/core/src/supervisor.rs) verifies user ownership/private directory permissions; `Workspaces::open` canonicalizes the state directory. Preserve these established behaviors unless explicitly choosing and justifying an alternative; do not silently weaken them or promote them into an invented broader security promise.

SQLite's NOFOLLOW open flag rejects a database filename containing a symbolic link. SQLx 0.9's public options and actual flag construction have no NOFOLLOW switch or raw open-flags override. Its locked raw handle is obtained **after** SQLite opens the path; it cannot retroactively enforce the opening flag. [SQLite opening contract](https://www.sqlite.org/c3ref/open.html), [SQLx options](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/options/mod.rs), [flag construction](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/establish.rs), [handle access](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/mod.rs).

**Finding:** no supported exact equivalent was identified in this cohort. SQLx's VFS selector chooses a registered VFS; it does not itself add NOFOLLOW. SQLite's documented URI parameters and SQLx's URL parser supply no documented NOFOLLOW substitute. Preflight metadata checks and a private canonical parent provide useful protection but are not an atomic replacement for the SQLite open flag. [SQLx establish](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/connection/establish.rs), [parser](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/options/parse.rs), [SQLite URI parameters](https://www.sqlite.org/uri.html).

**Gate, not safety verdict:** the bridge can retain the current flags through its supported opening interface. SQLx adoption needs either (a) a maintained supported upstream opening option/extension that preserves the behavior on pool reopens, with its release/pinning and qualification cost, or (b) an explicit contract assessment accepting an evidenced alternative. A small upstream NOFOLLOW option is plausible engineering work, not a currently available API. This comparison performs no custom VFS, unsafe raw-handle adoption or fork. An unresolved gate makes unchanged SQLx adoption unqualified; it does not establish that SQLx is unsafe or inherently impossible to use.

## Maintenance inventory and behavior equivalence

| Responsibility                                                   | SQLx async candidate                                                 | Maintained rusqlite bridge candidate                                                                 |
| ---------------------------------------------------------------- | -------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------- |
| Per-connection worker, reply delivery                            | Library-owned; remove custom worker/reply mechanism                  | Library-owned; remove custom worker/reply mechanism                                                  |
| Connection pool/reuse and acquisition lifecycle                  | Built in; select limits/hooks                                        | Reuse bridge handles; a multi-reader pool/selection mechanism remains application-owned if needed    |
| Transaction mechanics                                            | Async transaction ownership/drop cleanup supplied                    | Whole synchronous closure, rusqlite transaction cleanup; no per-statement await or captures          |
| Caller blocking dispatch for database-only work                  | Disappears behind awaited storage                                    | Disappears behind awaited storage; closure captures/error conversion remain internal                 |
| Command/row buffering                                            | Driver-owned configurable bounds                                     | Internal queue unbounded; external admission and result byte limits must actually cap submitted work |
| Admitted action count/bytes, writer ordering, workspace fairness | Application-owned                                                    | Application-owned                                                                                    |
| Cancelled caller, retained completion, shutdown fencing/drain    | Application-owned tasks plus pool/connection close                   | Application-owned tasks plus bridge close; failure receiver must be retained                         |
| SQL/schema mistakes                                              | Optional literal-query checks; runtime queries alone do not add them | Runtime statement preparation/decoding and separate qualification                                    |
| Schema evolution tooling                                         | Migrator/files/checksums supplied if selected                        | Keep a small owned initializer or choose maintained tooling when needed                              |
| Private opening                                                  | Unresolved supported NOFOLLOW equivalent                             | Existing supported open flags retained                                                               |
| Domain identity, stale revisions, recovery and file publication  | Remain in owning core modules                                        | Remain in owning core modules                                                                        |

SQLx `query!` checks literal SQL/bind/results against the intended database schema; dynamic SQL and application semantics are outside that check. Offline `.sqlx` metadata adds a real schema-preparation/freshness gate and CLI version/feature ownership. Do not charge those costs to a runtime-query-only candidate while claiming checked-query benefits. [Macro contract](https://docs.rs/sqlx/0.9.0/sqlx/macro.query.html), [offline workflow](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-cli/README.md). Its Migrator validates applied migrations/checksums; the milestone's retired-format compatibility exemption still applies. [Migrator](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/migrate/migrator.rs), [development policy](../MILESTONE-01.md#development-compatibility-policy).

Driver costs must include actual selected dependencies/build work: SQLx 0.9's `libsqlite3-sys` range is `>=0.30.1,<0.38.0`, excluding the application's rusqlite cohort's 0.40.0. Keep isolated harness graphs and record actual engines; different SQLite versions can confound a bundled-driver comparison. The previous [selection research](sqlx-vs-rusqlite-selection.md#version-cohort-and-dependency-constraints) identifies that cohort; [SQLx manifest](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/Cargo.toml) owns its declared range.

## Comparable experiment and gates

The isolated harness should exercise the same behavior-shaped workload against both implementations without committing a new production abstraction. Keep identical schema, payload bounds, offered load, application admission, transaction grouping, writer ordering, busy timeout, PRAGMAs and shutdown outcomes. Await actual async operations; SQLx must not be benchmarked through per-operation `block_on` behind a synchronous owner. Source/build metadata, engine/version/profile and commands belong beside results under [storage-async-comparison](../evidence/storage-async-comparison/).

Start with one writer and the same explicitly bounded reader topology. A SQLx pool may be the maintained implementation of reader acquisition, but a different number of readers is a topology experiment, not a pure driver result. DELETE versus WAL must likewise be separate labeled cohorts: SQLite still serializes writers; WAL permits readers alongside a writer whereas rollback journaling changes reader/writer interaction. More connections do not imply more simultaneous writers. [SQLite isolation](https://www.sqlite.org/isolation.html).

Required gates before a driver recommendation:

1. **Durability/order:** intent and action outcome boundaries match; read-after-commit is correct; rejected work has not been admitted; explicit commit errors remain visible. SQLite COMMIT can return BUSY while leaving the transaction active; an error is not proof of rollback. [SQLite transactions](https://www.sqlite.org/lang_transaction.html).
2. **Cancellation/completion:** cancel a caller while an admitted operation is retained; verify capacity remains held until effects complete, outcome is recoverable, and no request-future disappearance becomes automatic retry authority.
3. **Admission/drain:** bound outstanding count/bytes before dispatch; report accepted/rejected/completed work, outstanding work at cancellation, and close/drain success/failure. Pool buffers are not substitutes for this gate.
4. **Private opening:** report retained behavior or the unresolved supported-path gap above independently of throughput. A benchmark on fresh regular files does not qualify alias handling.
5. **Usefulness/cost:** compare read fanout/tail latency, throughput, fairness observations, memory and build/dependency costs; inventory code and mechanisms actually removed, retained or added. A smaller interface and localized failure policy count even without a throughput win.

Application unit tests continue to substitute storage/filesystem outcomes and exercise owning-module decisions. Real SQLite measurements, private-path effects and actual shutdown qualification are separately labeled evidence, not newly added application unit/integration tests. Preserve useful existing coverage before replacement. No actual app, provider, user database or RAT access is needed. [Verification policy](../agents/verification.md), [physical completion](../agents/observations.md#physical-completion-and-cancellation).

## Future workflow probes, not feature commitments

[MVP future direction](../MVP.md#later-pr-inbox-and-prepared-reviews) names independent `pull_requests` and `reviews` owners and editable local handoffs bound to exact revisions. The following probes assess locality; they authorize no tables, interfaces, workflow engine or remote actions:

| Probe                                       | Driver-independent behavior that must remain local                                                                                                               |
| ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Local PR self-review and draft revisions    | Store exact comparison/head/base and draft version; retain human edits and stale-state decisions with the review owner                                           |
| Automatic agent review and editable handoff | Durable preparation identity, evidence/context revision and explicit human publication authority; reconcile uncertain remote writes                              |
| Completion schedules a child session        | Save trigger/action identity and immutable context version before execution; distinguish parent completion, admission and child execution outcomes               |
| Inbox/Jira/GitHub synchronization           | Separate remote object identities from workspace/conversation IDs; retain cursors, deduplication and uncertain-write reconciliation in the relevant future owner |
| Manual and agent commands                   | Both target the same domain action semantics and authority checks, without a second persistence policy in presentation or provider adapters                      |

Neither driver implements these policies. SQLx's query/schema tooling can reduce later schema-edit mistakes; rusqlite closures can keep multi-record decisions in one transaction without await-related lifetime friction. Only an implemented probe or comparative code change can establish the larger maintenance gain. Agent count alone is not a storage capacity model; bursts, payload sizes, read fanout and transaction duration are the useful measurements.

## Measured comparison

The [evidence and reproduction](../evidence/storage-async-comparison/README.md)
retain the frozen source, manifest, two standalone lockfiles, commands, failed
construction attempts, exact binary hashes and raw results. One continuously
driven current-thread Tokio runtime awaits native SQLx operations or maintained
bridge calls. Neither is driven through per-query `block_on` on an outer worker.
Both use one writer and two readers, WAL/FULL/fullfsync, foreign keys, a 100 ms busy
timeout and 1,000-page automatic checkpoints. Their bundled engines differ:
SQLx uses SQLite 3.51.3; the bridge uses 3.54.1. Statement caches also differ
(SQLx explicitly 100, bridge default 16); this is not an isolated driver-overhead
benchmark.

The original cohort completed **24 thirty-second windows**, one warm-up plus five
measured runs per stack/workload. Ten workspaces with five roots and four children
per root produce 250 synthetic parent/child streams at 500 events/s. The heavy
case uses 20 workspaces/500 streams at 2,000 events/s plus hot-workspace bursts,
offering 66,000 events per window. Each run starts with 100,000 retained events,
offers exactly 600 scheduled reads per reader and 150 urgent intents, and samples
PASSIVE checkpoints every two seconds. Shared count/byte admission, workspace
quotas, round robin, urgent preference and 16-event/25 ms grouping are identical.

Warm-five medians of each run's measurements (ranges retain all five runs):

| Measure                                                    | Maintained rusqlite bridge |                      SQLx |
| ---------------------------------------------------------- | -------------------------: | ------------------------: |
| Baseline accepted/committed events per run                 |            15,000 / 15,000 |           15,000 / 15,000 |
| Baseline event arrival → commit p95                        |     32.07 ms [31.87–32.17] |    32.52 ms [32.24–33.08] |
| Heavy committed / 66,000 offered                           |     54,554 [47,401–57,658] |    54,624 [47,739–55,604] |
| Heavy explicit rejections                                  |      11,446 [8,342–18,599] |    11,376 [10,396–18,261] |
| Heavy accepted event arrival → commit p95                  |  726.71 ms [654.07–848.58] | 701.34 ms [687.82–873.60] |
| Heavy urgent intent p95                                    |     32.93 ms [30.17–37.26] |    28.72 ms [26.13–37.67] |
| Heavy recent-page scheduled → completion p95, worst reader |        3.65 ms [3.20–4.42] |       4.15 ms [3.35–4.43] |
| Observed threads                                           |                          4 |                         4 |

Both baseline stacks persisted the full offered workload. **Neither original
heavy stack sustained the full offered workload:** rejecting output is not success
at 2,000 events/s plus bursts. Per-workspace accounting and latency show pressure
also reached non-hot workspaces; round robin does not guarantee isolation or a
durability deadline. All accepted event identities, hierarchy routes, sequences and
full payload hashes, all urgent identities, foreign keys and post-close integrity
checks passed. Run ranges overlap materially, so these results do not select a
performance winner. Histories and urgent intents remained usable in this fixture,
not newly qualified in the production desktop.

Fresh private-target release builds took 14.73 s for the bridge and 16.13 s for
SQLx with warm registry/toolchain/OS caches. Stripped binaries were 2,856,096 B and
3,437,632 B; active normal dependency graphs contained 28 and 99 packages including
the harness root. These are one-off standalone costs, not application bundle deltas.
The [ownership inventory](../evidence/storage-async-comparison/results-code-ownership.json)
records source size but does not turn line counts into a depth or quality score.
Whole-process RSS includes different post-window audit implementations (SQLx
materializes every live payload, bridge hashes rows incrementally), so its large
difference must not be presented as steady-state driver memory overhead.

Four before/after mechanism probes passed retained-owner/reply-disappearance,
rejected identity absence, busy COMMIT then same-transaction retry, synthetic
context/completion/next-intent transactions and acknowledged close. The bridge
also rejected an actual database symlink with NOFOLLOW. SQLx's public opening
options followed it, so exact preservation of that existing guard remains a
**failed adoption gate**, independent of throughput.

Focused reply-drop probes keep the async owner alive and joined; their permit is
not inside the bridge closure. They do not qualify arbitrary owner-task abort,
panic, cancelled COMMIT or physical capacity retention after such cancellation.
The production implementation must preserve those lifetimes independently and
test its owning-module decisions with controlled outcomes.

## Recommendation and maintenance judgment

**Choose rusqlite 0.42.0 with the maintained tokio-rusqlite 0.9.0 bridge for the
next storage implementation**, behind the existing core storage seam. This uses a
maintained worker rather than constructing another one, preserves the supported
opening behavior, and performed comparably in the matched async workload. It is
not a claim that synchronous database access is intrinsically better or that SQLx
is unsafe. SQLx remains attractive when its native pool/tooling benefits are
needed and the opening behavior is resolved through a supported path or explicit
design decision; do not introduce a custom VFS or unmaintained fork merely to
retain a feature on paper.

Built-in mechanisms are a **strong preference when they fit**. Both candidates
already delegate worker implementation. SQLx additionally delegates pool
acquisition/reuse and transaction-cancellation bookkeeping; the bridge keeps whole
transactions local and retains supported SQLite flags. This fixed two-reader
experiment did not demonstrate an additional production pool requirement or
remove any production code. SQLx query macros and migrations were disabled;
their benefits and preparation costs remain optional, documented opportunities.

Keep domain-shaped storage interfaces and controlled-outcome seams. Callers should
not learn pool configuration, raw SQL, closure scheduling or commit-cleanup rules.
Core owners still decide admission, uncertainty, exact identities and retries.
Do not expose an unbounded bridge queue directly: admit count/bytes before work,
retain completion independently of caller interest, fence new work during shutdown,
drain results (including failures), and explicitly close. A close acknowledgment
does not erase an earlier failed receiver or prove literal OS-thread joining.

The heavier workload rules out treating 16 events as a universal transaction cap.
Further bounded grouping/byte-budget qualification belongs with the selected
implementation; any single tuning diagnostic is labeled separately in the evidence.
Production provider output already admitted to execution must receive backpressure
or a visible capacity/failure response and retained durable prefix, never silent
discard matching this fixture's offered-event rejections. Identity/storage must
remain capable of many workspaces, conversations and child sources; temporary
execution admission limits do not become permanent row/cardinality restrictions.

Future feature ideas remain probes, not commitments. The experiments establish
synthetic transaction shapes and hierarchical event routing, not provider turn
mapping, real child execution, editable review/staleness rules, remote deduplication,
GitHub/Jira publication, parent-child cleanup or crash recovery for a new production
implementation. Full app units/static/build/desktop/provider qualification and
power-loss/long-soak/cold-read checks were not performed. Preserve earlier results
as checkpoint evidence rather than relabeling them.

### Separate grouping diagnostic

After preserving the original source and binaries, one matched 30-second heavy
run per stack changed only the maximum group count from 16 to 64, retaining the
25 ms eligibility threshold, quota, read trace and durability. This was authorized
as a bounded diagnostic after observing overload, not another five-run cohort.

| Single heavy run, maximum 64 events           | Maintained rusqlite bridge |     SQLx |
| --------------------------------------------- | -------------------------: | -------: |
| Committed / 66,000 offered                    |                     65,439 |   65,382 |
| Explicit rejections, all from hot workspace 0 |                        561 |      618 |
| Quiet-workspace rejected events               |                          0 |        0 |
| Accepted event arrival → commit p95           |                   42.59 ms | 43.72 ms |
| Urgent intent p95                             |                   17.43 ms | 23.78 ms |

This is evidence that the shared transaction-grouping policy materially affected
this workload. It is not proof of a universal driver ceiling, guaranteed workspace
isolation, or sustained capacity at the complete offered rate: the hot workspace
still had rejections. Preserve bounded count/bytes/time controls and qualify the
selected implementation under overload rather than freezing 16 as a product cap.
The diagnostic's worst-case physical batch copy is 64 × 64 KiB (4 MiB), separate
from the logical admitted-payload budget. Exact patch, manifest, source/binary
provenance and results are retained beside the original cohort.
