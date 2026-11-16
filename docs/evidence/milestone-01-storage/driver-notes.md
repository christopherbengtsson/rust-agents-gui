# Driver and scheduling experiment notes

2026-11-15 · Source baseline `24390a1e7ba108ca88704fbf73deac92915d3ab6` plus this uncommitted research harness. Production storage/dependencies are unchanged. These are disposable real-storage experiments, not application units, desktop results, or native-provider qualification.

## Result

Keep rusqlite unless another concrete need justifies replacing it. The tested stacks both sustain the offered 500 synthetic events/second when one writer coalesces events across conversations with a 16-event/25 ms transaction cap. Neither sustains that rate with a durable transaction per event. Scheduling and commit frequency matter much more than the driver choice in this cohort.

This supports investigating one bounded writer with urgent intent writes and time-bounded transcript grouping. It does not justify delaying user intents, adopting a generic repository/workflow framework, or introducing a reader pool as an immediate requirement. No production change was made.

## Frozen configurations

[Harness and reproduction instructions](driver-bench/README.md), [final manifest](driver-bench/manifest.json), [screen manifest](driver-bench/manifest-screen.json), [environment](driver-results-environment.json), [dependency graphs](driver-results-dependencies.json), [final-source build costs](driver-results-build-costs.json), and [aggregate summary](driver-results-summary.json) retain the configuration and commands. Both independent standalone packages have lockfiles; the application lockfile is unchanged.

- Current rusqlite 0.42.0/libsqlite3-sys 0.40.0 bundles SQLite **3.54.1**.
- Minimal-feature SQLx 0.9.0/libsqlite3-sys 0.37.0 bundles SQLite **3.51.3**.
- Each stack requests a 100 ms busy timeout, 100 cached statements, fullfsync, foreign keys, a 2 MiB page cache per connection, and 1000-page WAL autocheckpoint. Engine/source IDs, compile options and effective journal/sync/fullfsync values are recorded per run.
- Both stacks use the same owner worker, bound data, schema/index, materialized page reads, and transaction shapes. SQLx additionally has its internal SQLite worker and a current-thread Tokio runtime; its adapter uses `block_on` per operation. Results compare these complete stacks, not pure driver overhead or all possible async designs.

## Measured tradeoffs

The following are medians across **five warm three-second admission windows**, excluding one warm-up per case. At 25 active conversations the producer offers 20 events/second/chat. Work remaining after the window is drained and closed. Throughput's denominator includes that drain, explaining the small difference from the offered 500/second.

| Stack / WAL FULL workload        | Events/s median | Rejected transactions median | Queue p95 median | Service p95 median | Formation p95 median |
| -------------------------------- | --------------: | ---------------------------: | ---------------: | -----------------: | -------------------: |
| rusqlite, one event/transaction  |           230.0 |                          747 |       309.184 ms |           5.010 ms |   Excluded/preformed |
| SQLx, one event/transaction      |           218.2 |                          782 |       319.080 ms |           5.071 ms |   Excluded/preformed |
| rusqlite, global 16 events/25 ms |           497.7 |                            0 |         0.048 ms |           5.986 ms |            25.201 ms |
| SQLx, global 16 events/25 ms     |           497.2 |                            0 |         0.038 ms |           8.861 ms |            25.202 ms |

Formation, queue and service distributions are separate. Adding their percentiles would not establish an end-to-end percentile. This harness does not measure desktop navigation or user-visible durability freshness.

Initial single-observation screening at 500 offered events/second showed current DELETE/FULL at roughly 68–72 events/second, DELETE/EXTRA roughly 67–68, WAL/FULL roughly 206–222, and WAL/NORMAL roughly 498. WAL/NORMAL has a different documented durability contract; its speed does not qualify it as an equivalent durable replacement. Opening a connection for every DELETE/FULL transaction reduced observed throughput slightly; commit sync cost dominated this short test. See [screen results](driver-results-screen.jsonl).

The 1/3/10/25 conversation sweep admitted the expected approximately 20/60/200/500 events/second without rejection using global coalescing. These points are not product caps. Larger-history finalists retained 100,000 and 1,000,000 events across 1000 conversations, with mixed latest/older indexed pages during writes:

| Retained history | rusqlite mixed-page p95 median | SQLx mixed-page p95 median | Warm-five writer rejection |
| ---------------- | -----------------------------: | -------------------------: | -------------------------: |
| 100,000 events   |                       0.712 ms |                   0.839 ms |              0 both stacks |
| 1,000,000 events |                       0.974 ms |                   1.072 ms |              0 both stacks |

At one million retained events, throughput medians were 497.55 and 497.71 events/second respectively. Seeding that history took 10.10 and 16.71 seconds respectively; seeding is a separate setup cost, not streaming throughput. See [scale results](driver-results-scale.jsonl).

Optional two-reader probes use a **different read workload**: two 100 Hz recent-page readers replace inline alternating recent/older reads every two transactions. Both grouped stacks still admitted the offered rate without rejection, but this does not isolate the value of a reader pool. No pool recommendation follows from this comparison.

## Costs

Final-source release builds used fresh private targets, warm registry/toolchain/OS caches and stripped release binaries. The fresh binary hashes exactly match the runtime-measured binaries.

| Standalone harness | Active normal target packages, including root | Release binary | Fresh-target build wall | Compiler max RSS | Runtime max RSS | Observed runtime threads |
| ------------------ | --------------------------------------------: | -------------: | ----------------------: | ---------------: | --------------: | -----------------------: |
| rusqlite           |                                            15 |    2,363,808 B |                 13.05 s |    597,147,648 B |     6,701,056 B |                        2 |
| SQLx               |                                            98 |    2,848,992 B |                 15.93 s |    586,203,136 B |     7,290,880 B |                        3 |

Runtime RSS/threads are **single saturation-resource samples**, with `/usr/bin/time -l` and process-specific `ps -M` sampling. The producer deliberately spins against a full bounded queue, so this is neither typical idle cost nor an application memory delta. The application already depends on Tokio and some shared crates. All-target metadata package counts (30/117) are separately labeled and are not active-build counts. SQLx's extra standalone dependencies are a maintenance/cost consideration, not a correctness defect.

The first resource collector counted the mandatory `ps -M` heading as a thread. Raw evidence is retained in `driver-results-resources.jsonl`; the corrected collector excludes the heading and retains raw per-process output in [corrected resource results](driver-results-resources-corrected.jsonl). The summary excludes the superseded resource cohort.

## Completion and limits

- **86 short runtime workload samples** completed, including warm-ups, screening and both resource cohorts; all reported `integrity_check=ok`, zero writer SQL errors and zero reader SQL errors. Queue rejections in overloaded cases are expected explicit admission outcomes.
- Two lifecycle probes (one per stack) passed: an admitted write persisted after its reply receiver was dropped, full-queue work was rejected and absent, and physical worker close/join completed. This is a mechanism probe, not production deadline/cancellation qualification.
- Process kill, actual COMMIT BUSY/rollback, pinned-reader checkpoint, backup, filesystem publication, and synthetic transaction-shape probes are recorded in the separate native experiment evidence. Current application semantics were inspected, not newly runtime-qualified. This driver harness does not establish those guarantees by itself.
- Three-second samples establish bounded observed capacity and repeatability. They do not prove sustained dozens-of-chat operation, end-to-end desktop latency, power-loss durability, filesystem/hardware flush honesty, or every shutdown race.
- Sampled disk metrics cover main/WAL/SHM after transactions; they exclude transient DELETE-journal peaks. Writer close/drain metrics exclude optional readers' final close/checkpoint; total elapsed includes their join. Old screening metric names are overbroad and must be read under these limits.
- Synthetic payloads are 90% 256 B, 9% 2 KiB, 1% 16 KiB, mixed within each retained conversation. No user history, runtime credentials, provider sessions, GitHub/Jira traffic, or RAT state was accessed.

## Retained construction failures

The initial SQLx build failed on 0.9.0's audited SQL-string requirement; the corrected code uses `AssertSqlSafe` only for fixed/allowlisted SQL and binds values. A reopen-per-operation implementation failed Rust ownership checking and was corrected to close/reopen directly in the owning worker. Original failed logs remain alongside corrected and fresh final builds. An ad-hoc console summary command also failed with a Python spacing typo; the corrected summarizer generated the retained summary. These failures did not produce runtime evidence or indicate a product defect.

Supported-version implementation sources: [SQLx 0.9 options](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/src/options/mod.rs), [SQLx SQL-string contract](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-core/src/sql_str.rs), [SQLx SQLite manifest](https://raw.githubusercontent.com/transact-rs/sqlx/v0.9.0/sqlx-sqlite/Cargo.toml), [SQLite synchronization modes](https://www.sqlite.org/pragma.html#pragma_synchronous). The main research report owns the broader primary-source interpretation and selected design.
