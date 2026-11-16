# Bounded async storage architecture comparison

2026-11-15–16 · Follow-up to 06a and the SQLx/rusqlite source research.

This is a disposable comparison of two storage implementations, not a production
migration or application/native-provider qualification. The
[architecture assessment](../../research/storage-async-architecture-comparison.md)
owns the final recommendation; the harness manifest and raw results own the
measured configuration and values. Earlier 06a results remain historical evidence.

## Question and fixed ownership

Compare native async SQLx with rusqlite behind the maintained tokio-rusqlite
bridge. Both use one continuously driven current-thread Tokio runtime, one writer,
two readers, explicit bounded application admission, and WAL/FULL/fullfsync.
SQLx operations are awaited directly, rather than driven with per-operation
`block_on` behind a synchronous owner.

Library mechanisms are preferred when they preserve the required behavior and
reduce code ownership. This does not delegate durable action identity, uncertainty,
workspace fairness, streamed-event grouping or shutdown fencing to a generic pool.
Both maintained candidates remove custom per-connection worker construction.

## Workload and horizon

The [frozen manifest](harness/manifest.json) defines exact counts, byte limits,
rates, bursts, read traces, admission and scheduling. The intended comparison is
one warm-up and five measured 30-second windows per driver/workload:

| Workload                   | Workspaces | Root chats per workspace | Children per root | Total parent/child streams |                Offered base rate |
| -------------------------- | ---------: | -----------------------: | ----------------: | -------------------------: | -------------------------------: |
| Baseline                   |         10 |                        5 |                 4 |                        250 |                     500 events/s |
| Heavy with a hot workspace |         20 |                        5 |                 4 |                        500 | 2,000 events/s plus timed bursts |

These are synthetic independent output sources, not a claim that every provider
exposes every child's content or that a runtime child is an application
conversation. The hierarchy tests routing/accounting and load; it does not build
child-agent controls or impose product cardinality limits.

Future ideas are transaction/read probes: versioned context, parent completion
with next-session intent, local editable draft revisions, retained external action
identity, and context/inbox reads during output. They do not implement a coordinator,
PR workflow, GitHub/Jira synchronization, or a remote database deployment. Manual
and agent effects remain subject to the same future owning-module policy.

Measure direct scheduled-event-arrival to commit latency, per-workspace outcomes,
matched offered read load, urgent intent latency, executor heartbeat lateness,
checkpoint progress and drain. Count successful, rejected and remaining work;
explicit admission rejection is distinct from silently lost admitted work.

## Qualification rules

- Build separate pinned standalone Cargo graphs, since the selected drivers use
  incompatible libsqlite3-sys ranges. Record actual engines, options, source and
  binary hashes; engine differences prevent attributing results solely to drivers.
- Keep builds, binaries and synthetic databases in private disposable locations;
  no application/user/provider history is an input. Serialize builds and measured
  runs. The parent and research/review agents perform no competing benchmarks.
- Review offered-trace equivalence, hierarchy/order/accounting and completion
  assertions before the measured cohort. Retain failed construction/preflight
  attempts without treating them as product defects.
- Probe cancelled/dropped callers, actual busy commits, context/continuation
  atomicity and connection close separately. Drained SQLite effects and closed
  handles are different claims from observing every OS thread exit.
- Treat private opening as a separate adoption gate. A fast regular-file run
  cannot establish preservation of the current NOFOLLOW behavior. Do not add a
  custom VFS or unmaintained fork just to manufacture a passing comparison.
- Real SQLite/filesystem/threads are experiments under repository verification
  policy, not application unit or integration tests. Application fixtures,
  provider submissions, power loss and full desktop responsiveness remain separate.

## Results

The original cohort completed 24 workload windows: four warm-ups and 20 measured
runs, plus four focused before/after mechanism checks. All accepted-record,
hierarchy, payload-hash, urgent-ID, foreign-key and post-close integrity checks
passed. Failed SQLx opening-behavior preservation is reported independently below.
The [summary](results-summary.json) contains five-run medians/ranges and per-workspace
outcomes; [raw results](results.jsonl) retain every window.

| Warm-five measure                              | Maintained rusqlite bridge |            SQLx |
| ---------------------------------------------- | -------------------------: | --------------: |
| Baseline committed / offered per run           |            15,000 / 15,000 | 15,000 / 15,000 |
| Baseline direct scheduled arrival → commit p95 |                   32.07 ms |        32.52 ms |
| Heavy committed / 66,000 offered, median       |                     54,554 |          54,624 |
| Heavy explicit rejections, median              |                     11,446 |          11,376 |
| Heavy direct arrival → commit p95, median      |                  726.71 ms |       701.34 ms |
| Heavy urgent intent p95, median                |                   32.93 ms |        28.72 ms |
| Observed threads                               |                          4 |               4 |

Original heavy run ranges overlap and both reject offered output: neither supports
the complete offered heavy trace under the original 16-event policy. Pressure
reached non-hot workspaces too. All 600 requests per reader and 150 urgent intents
per window completed with exact identity/accounting. Read completions see each
stack's actual committed prefix, not identical concurrent snapshot contents.
One measured statistic per run is aggregated across five runs; this is not a
pooled percentile or a desktop measurement.

### Bounded grouping diagnostic

Observed overload justified one additional matched pair, changing only the group
count from 16 to 64 in separate copied source/targets. This is **one run per stack**,
not warm-five capacity qualification; time threshold, durability, quotas and trace
were retained. See [diagnostic summary](results-group64-summary.json),
[raw results](results-group64.jsonl), [provenance](results-group64-provenance.json),
and its separate [source](harness/group64-common.rs)/[manifest](harness/group64-manifest.json).

| Single heavy diagnostic            | Maintained rusqlite bridge |            SQLx |
| ---------------------------------- | -------------------------: | --------------: |
| Committed / offered                |            65,439 / 66,000 | 65,382 / 66,000 |
| Rejected, entirely hot workspace 0 |                        561 |             618 |
| Quiet-workspace rejections         |                          0 |               0 |
| Direct arrival → commit p95        |                   42.59 ms |        43.72 ms |

This supports investigating shared grouping/byte budgets rather than attributing
the original overload to a driver ceiling. It does not prove isolation or sustained
capacity. Production already-admitted provider output requires backpressure or
visible capacity failure with retained history, never silent dropping of chunks.

### Mechanisms, maintenance and costs

- The retained-owner reply-disappearance probe preserves capacity while that owner
  remains alive, commits accepted identity 1, rejects identity 2, then drains and
  closes. It does not cancel the owner or an in-flight COMMIT. Its permit lives in
  the async owner, not inside the bridge closure; arbitrary abort/panic and physical
  capacity retention after owner loss remain implementation requirements.
- Both real busy-COMMIT probes retained the transaction and successfully retried
  after a reader released. This explicitly uses supported manual BEGIN/COMMIT on
  a retained connection, not a claim that a consuming transaction API returns its
  handle on error.
- Synthetic context/continuation/draft-counter probes passed. They do not implement
  child execution, review edit/staleness rules, remote reconciliation or publication.
- The bridge rejects an actual alias with native NOFOLLOW; SQLx's ordinary supported
  options follow it. Exact current opening-behavior preservation is a failed SQLx
  adoption gate, not a blanket safety verdict. No custom VFS/fork was added.
- Both libraries supply their connection workers. The [ownership inventory](results-code-ownership.json)
  separates shared policy from driver mechanisms and source sizes. Source size is
  not a depth/quality score; no production code was removed in this experiment.
- Fresh final-source private-target builds matched runtime binary hashes. Bridge:
  14.73 s, 2,856,096 B stripped binary, 28 active normal packages including root.
  SQLx: 16.13 s, 3,437,632 B, 99 packages. These single build observations use warm
  registry/toolchain/OS caches and are not predicted application deltas.
- Whole-process RSS covers setup and different post-window audits: SQLx materializes
  every live payload while the bridge hashes rows incrementally. Do not interpret
  the resulting peak difference as steady-state driver overhead. Supporting `ps`
  samples and raw `time -l` output are retained with that scope.

## Reproduction and provenance

Use a separate copy of this evidence directory for a new run so captured outputs
are not overwritten or mixed. The runner appends JSONL results and writes environment
metadata; the following commands intentionally rerun all roughly 12 minutes of
original admission windows. Run one command at a time with other measurements idle:

```sh
cargo build --release --locked --manifest-path docs/evidence/storage-async-comparison/harness/bridge/Cargo.toml --target-dir /tmp/storage-async-bridge-target
cargo build --release --locked --manifest-path docs/evidence/storage-async-comparison/harness/sqlx/Cargo.toml --target-dir /tmp/storage-async-sqlx-target
python3 docs/evidence/storage-async-comparison/harness/run.py
```

The runner expects those two target paths. Before reusing them, confirm they are
the task-owned private targets, not aliases or another workspace's targets; adjust
the copied runner if using different paths. It creates a fresh canonical mode-0700
data root for each invocation, cleans each closed database, and removes the root
on successful completion. `run-group64.py` performs the separately labeled copied
source/build diagnostic and writes its own evidence. It is optional reproduction,
not part of the original five-run cohort. Exact executed commands and statuses are
in [results-commands.jsonl](results-commands.jsonl), with
[environment](results-environment.json), [build costs](results-build-costs.json),
[build history](results-build-history.json) and
[final source/binary checks](results-final-provenance.json).

The separate Python post-close audit uses the installed stdlib SQLite 3.51.2 for
SELECT/hash/integrity queries on closed fixture databases; the timed workload,
native opening and busy probes use their recorded bundled engines. This auditor
is not a live WAL concurrency or durability experiment.

The pre-cohort independent review corrected unequal/skipped reads, unequal payload
materialization, additive-only auditing and incomplete admission/urgent accounting.
The first runtime attempt rejected the macOS `/var` alias under genuine NOFOLLOW;
canonicalizing the fresh private root fixed the harness. Original logs are retained.
An ad-hoc source-edit helper also failed before writing when a rustfmt-spaced marker
did not match; it produced no runtime evidence. These are construction failures,
not application defects. [Notes](notes.md) preserve further interpretation details.

## Final scope

26 workload executions and four focused probes completed; admission rejections in
overloaded fixtures are expected and counted, not silently lost accepted events.
The original 24-run sources/binaries are unchanged by the two diagnostic runs.
Synthetic databases, failed roots, fresh-cost targets and diagnostic targets were
removed; original private build targets remain for reproducibility/review.

Unperformed: application units/static/generated preparation/build/package,
fixture desktop, real authentication/provider turns, power loss, cold-cache reads,
long soaks, arbitrary owner abort/panic, provider turn mapping, full parent/child
cleanup and actual external workflow behavior. Fixed two-reader tasks do not force
high-contention pool acquisition or qualify arbitrary concurrent context clients.
The 25 ms grouping threshold is eligibility, not a commit-latency guarantee.
Close acknowledgment establishes the measured effect/connection boundary, not
observed OS-thread joining. No production storage, dependencies or contracts changed.
