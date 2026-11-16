# Storage evaluation evidence

2026-11-15 · Research ticket 06a · Baseline `24390a1e7ba108ca88704fbf73deac92915d3ab6`.

The [recommendation](../../research/milestone-01-storage-evaluation.md) retains
rusqlite, recommends WAL/FULL/fullfsync and intentional writer ownership, keeps
authoritative transcripts/content together, and defers a separate cache database.
No production storage, IPC, dependency or schema changes were made.

## Cohort and interpretation

- Apple M4 Pro, 12 logical CPUs, 48 GiB RAM, macOS 26.2 build 25C56; Rust 1.99.1.
  [Environment and preserved-file hashes](environment.json),
  [driver environment](driver-results-environment.json).
- Two separate standalone release packages: rusqlite 0.42.0 / SQLite 3.54.1 and
  SQLx 0.9.0 / SQLite 3.51.3. These are bundled-stack comparisons with different
  engines/compile options, not isolated proof of driver overhead.
- Common offered workload: 20 events/second/chat, 1/3/10/25 active conversations,
  1,000 retained conversations, 10k/100k/1m events. Deterministic payload classes
  are 256 B, 2 KiB and 16 KiB. Both drivers materialize bounded 100-row latest/older
  pages; every tenth transaction also updates metadata. See the exact
  [final manifest](driver-bench/manifest.json) and
  [screen manifest](driver-bench/manifest-screen.json).
- Application-owned admission is a 64-job queue. The driver data budget is 8 GiB;
  the native probe budget is 1 GiB. All data is synthetic, in fresh private roots.
  These are measurement points, not product limits. No Conductor/user databases,
  provider history, credentials or RAT files were inspected.
- A run lasts 3 seconds plus physical drain. Finalists and large-history cases
  use one warm-up plus five measured runs; initial screens and the active-count
  sweep are individual samples. OS/registry caches are warm. These short cohorts
  do not establish sustained capacity, tail behavior under a long soak, or UI
  responsiveness. Native attachment/cache/transcript comparisons also use one
  warm-up plus five measurements.
- Timed driver builds/runs and native probes ran in separate windows. Source edits
  and small JSON/document operations occurred outside their process ownership.
  No two worktree Cargo builds were run simultaneously. Fresh builds use private
  targets and warm registry/toolchain/OS caches, not cold-machine measurements.

## Driver and scheduling results

At 25 chats and 500 offered events/second, warm-five medians:

| WAL/FULL workload                           | rusqlite events/s | SQLx events/s | Rejected transactions/run, rusqlite / SQLx |
| ------------------------------------------- | ----------------: | ------------: | -----------------------------------------: |
| One event per commit                        |             230.0 |         218.2 |                                  747 / 782 |
| Global grouping, at most 16 events or 25 ms |             497.7 |         497.2 |                                      0 / 0 |

The grouping experiment measures formation delay separately: about 25.2 ms at p95
in both stacks. Queue and transaction times are separate distributions; adding
their percentiles would not produce an end-to-end percentile. Preformed 16-event
bursts in the original screen exclude formation and can hide up to 750 ms of
per-chat formation delay. They are not evidence for waiting on per-chat batches.

At one million retained events with global grouping, warm-five throughput medians
were 497.5 / 497.7 events/s, without rejected transactions or recorded write errors.
The median of each run's mixed latest/older history-page p99 was 1.127 / 1.148 ms.
The raw five-run ranges remain in [summary.json](summary.json) and the
[driver summary](driver-results-summary.json); these are run-quantile summaries,
not pooled percentiles.

The extra-reader cohort changes offered read traffic to two readers polling at
10 ms intervals instead of inline reads every two writer transactions. It therefore
establishes successful overlap at that workload, not an isolated reader-pool win.
Sampled database bytes exclude in-transaction rollback-journal peaks; writer-close
time excludes the last reader's possible close/checkpoint work. See the
[driver method and commands](driver-bench/README.md) and
[driver analysis](driver-notes.md).

## Dependency and resource costs

| Standalone final-source release harness                       |    rusqlite |        SQLx |
| ------------------------------------------------------------- | ----------: | ----------: |
| Active normal target dependency graph, including harness root | 15 packages | 98 packages |
| Stripped binary                                               | 2,363,808 B | 2,848,992 B |
| Fresh private-target build wall time                          |     13.05 s |     15.93 s |
| Maximum RSS reported by `time -l`, observed saturation cohort | 6,701,056 B | 7,290,880 B |
| Observed process threads, one writer connection               |           2 |           3 |

Builds are single measurements, not five-run estimates. Binary sizes cover the
standalone harness and selected features, not predicted application-bundle deltas.
Runtime resources come from an explicitly observed, unpaced saturation workload;
observer overhead makes those runs separate from the paced finalists. Original
resource data counted a `ps` header as a thread; corrected raw output and counts
are retained in [corrected resources](driver-results-resources-corrected.jsonl).
See [build commands/hashes](driver-results-build-costs.json),
[dependency graphs](driver-results-dependencies.json), and raw logs.

## Recovery, files and backup results

The [native probe](storage-probes.py) uses a private clang-built SQLite 3.54.1
library through Python ctypes, with its actual source ID, settings and compile
options recorded. It shares the app's amalgamation source, not its built artifact
or complete compile configuration. The system Python SQLite 3.51.2 was not used
for the experiments.

- All **24 final process-kill points passed**: 18 transaction/file-publication
  points across DELETE/FULL, DELETE/EXTRA and WAL/FULL, plus six WAL/FULL
  before/after checkpoint, backup and explicit-close points. Committed records
  remained consistent and readable. Attachment reference hashes matched files;
  pre-commit temporary/orphan files remained distinguishable.
- Busy rollback-mode COMMIT returned code 5 after 123.9 ms while its transaction
  remained active. Explicit rollback restored committed-state reads. A configured
  100 ms busy timeout is not an exact wall-time deadline or proof of rollback.
- A pinned reader retained a 7,399,552-byte WAL. PASSIVE returned `0,1796,15`:
  SQL success with only 15 of 1,796 frames checkpointed. After reader release,
  TRUNCATE returned `0,0,0` in 24.7 ms.
- VACUUM INTO took 71.3 ms and produced an integrity-valid, internally consistent
  161-row snapshot while the source reached 201 rows. Seven foreground commits
  overlapped the backup interval; their median was 6.06 ms, maximum 17.77 ms.
  This is one overlap sample, not a controlled backup-overhead estimate.
  The raw `final_rows` field is the expected count from the successfully completed
  insert loop, rather than an independent post-run source count query.
- Shared-context reads kept an earlier consistent snapshot while another connection
  committed completion, next-session intent, a new context version and a pending
  external-operation identity together. No external effect was performed. This
  qualifies SQLite primitives on synthetic records, not coordinator policy.
- The final full probe retained about 201 MB beneath its private root, checked
  184 generated files were mode-0600 regular single-link files, then removed the
  root. These checks do not replace the application's alias/access-control units.

Warm-five native layout results, median [minimum–maximum]:

| Workload                                        |         SQLite content |                                  External files |
| ----------------------------------------------- | ---------------------: | ----------------------------------------------: |
| 1,024 transcript events, commits every 16       | 322.7 [316.7–325.8] ms | JSONL + offset metadata: 646.5 [624.3–655.2] ms |
| Three bounded transcript pages                  |    1.04 [0.96–1.35] ms |                             2.41 [1.86–3.98] ms |
| 12 MiB attachments: sixteen 256 KiB + two 4 MiB | 169.9 [161.8–176.8] ms |                          190.8 [185.9–195.0] ms |
| Read and SHA-256 verify attachment content      |    8.80 [8.59–9.42] ms |                          11.02 [10.08–12.15] ms |

The transcript-file path has both a synced append and a metadata commit. JSON
encoding/decoding and ctypes overhead are included; these are end-to-end synthetic
layout costs, not pure filesystem/SQLite microbenchmarks. Initial JSONL directory
publication sync is not included, making that write result incomplete as a full
publication protocol. No cold-read/media-size cutoff is established.

Rebuilding 1,000 reconstructible 4 KiB diff entries beside ten real authoritative
version increments took 130.0 [104.0–132.1] ms in the same database and 133.0
[108.1–139.0] ms in a second database. Both retained the authoritative record after
discard. This sequential synthetic cache probe establishes no concurrency benefit
and supplies no reason to add a cache database today.

## Reproduction, failures and qualification limits

From the workspace root, build and run the native probes:

```sh
python3 docs/evidence/milestone-01-storage/build-native.py > /tmp/storage-native-build.json
```

The resulting JSON supplies the private library path. Invoke `storage-probes.py`
with `--library <private_root>/libsqlite3.dylib`, once normally and once with
`--maintenance-only`; each creates and cleans a fresh private data root. Exact
executed commands, source SHA-256 and exit statuses are in
[native-command.json](native-command.json), with
[build provenance](native-build.json), [results](native-results.json),
[maintenance results](native-results-maintenance.json) and corresponding logs.
Use the separate [driver instructions](driver-bench/README.md) for locked builds,
screen/finalist/scale/resource phases and regeneration of summaries.

Retained failures: two initial driver compilation errors (SQLx 0.9 SQL-safety
argument type and connection ownership during explicit close) preceded corrected
builds. The first native open failed because macOS TMPDIR used the `/var` alias
while SQLite NOFOLLOW was requested; canonicalizing the freshly created private
root corrected the harness. The earlier passing native cohort is retained as
`native-results-initial.json`; final results include the reviewed source and
maintenance/context additions. No failure is claimed as a product defect.

**Verification classification:** real standalone SQLite/filesystem/process-kill
experiments and Rust harness compilation passed. Deterministic application units,
full application static/generated checks, app packaging, fixture desktop,
real authentication and native-agent execution were not run for this research.
No application behavior changed. In-process kill markers cover before/after
maintenance, not an instrumented interruption inside its VFS calls. Power loss,
hardware flush honesty, long-soak WAL growth, cold reads, incremental Online Backup,
full backup cancellation/partial-output handling and production responsiveness
remain unqualified. Scope application follow-up accordingly.

Independent architectural/evidence review found no blocking issue. Its two minor
clarifications were addressed: the combined summary counts the six maintenance
kill points separately and in the total, and the backup's expected source-count
field is identified above. No application qualification follows from this review.
