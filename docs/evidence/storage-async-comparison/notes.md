# Standalone async storage evidence notes

The source cohort is frozen in `harness/manifest.json`, `harness/common.rs`,
`harness/driver.rs`, the two exact-version Cargo workspaces and their lockfiles.
This is disposable real SQLite research, not an application unit/integration suite,
fixture-desktop qualification, native-provider qualification, or production migration.
The application source, dependencies and contracts were not changed.

The runtime has one current-thread Tokio 1.54.1 executor. A shared async owner awaits
SQLx native pool/transaction calls or maintained tokio-rusqlite whole-transaction
closures. There is no synchronous outer owner worker and no per-operation block_on.
Both topologies use one writer and two readers; SQLx supplies a reader pool while the
bridge assigns the two bounded reader tasks their respective connection. Source IDs,
compile options and effective settings are retained per result. The engine cohorts
are deliberately recorded separately; this is a stack comparison, not isolated driver
or engine overhead. Only one candidate runs/builds at a time.

The offered event trace, identities, payload bytes and absolute scheduled arrivals
are deterministic and equal across stacks. Baseline offers15,000 events in30s;
heavy offers66,000 (60,000 base plus6,000 workspace0 burst events). Each reader offers
exactly600 requests at absolute50ms ticks. Slow readers catch up serially; none are
silently skipped. Both materialize identical page/inbox field tuples and page byte
bounds. End-to-end event latency is measured from each scheduled arrival directly
to its successful COMMIT, including formation, admission queue and service time.
Per-workspace accounting asserts offered=admitted+rejected and admitted=committed.
These identities model workspace/root-chat/child streams and ordered events. They
make no claim about provider turn mapping or availability of rich child content.

Shared application-like code supplies count/byte admission, workspace quotas and
round robin, urgent priority, batching, physical in-flight accounting and drain.
Those are still owning-core decisions in a future production implementation. The
8MiB bound is admitted logical event payload bytes including physical work; it is
not a total allocation/RSS guarantee. The bridge clones a physical batch into its
closure while the owner retains original timestamps/budget records; SQLx may also
retain encoded values. At most16 events of at most64KiB form a batch; two read pages
are each capped at256KiB. Audit allocation after the timed window is additional.
The32 urgent bound is queued intents plus at most one physically executing intent.

Each run seeds100,000 retained256B events and then uses the declared mixed payloads
for live output. This is a bounded12minute aggregate admission cohort, not a long
soak. PASSIVE checkpoint tuples/latencies are sampled on the writer between groups
every2s; final TRUNCATE is measured after readers settle. These samples do not claim
pinned-reader checkpoint progress, VFS/hardware flush honesty or power-loss durability.
FULL/fullfsync settings establish requested SQLite behavior, not a crash proof.

Focused checks before/after the cohort exercise an admitted operation whose reply
receiver is dropped, rejection of a second identity while its physical capacity is
retained, completion owner drain, acknowledged driver close, actual rollback-journal
COMMIT BUSY with retained-transaction retry, an atomic completion/context/next-intent
transaction, one consistent read-context transaction, and editable draft revision
with a stable synthetic external-operation identity. No remote operation is issued.
Acknowledged SQLite/driver close is measured; neither API exposes an OS worker join
handle here, so the evidence does not assert explicit OS-thread join.

Durable audits compare every admitted live record against the retained expected
id/stream/seq/workspace/root/parent/full-payload FNV1a64 hash, plus count/signature,
parent-routing SQL checks, foreign-key checks and integrity_check. This FNV comparison
is noncryptographic. A separate post-close Python read records a SHA256 digest of all
live rows and another integrity check. All admitted urgent IDs/workspaces are compared
exactly with durable intent rows. Explicit overload rejections are expected outcomes;
they are not SQL failures or silent drops.

Native private-open is a separate adoption gate. tokio-rusqlite0.9.0 exposes genuine
rusqlite SQLITE_OPEN_NOFOLLOW; a real final-component alias must fail. SQLx0.9.0's
supported SqliteConnectOptions do not expose the native flag: its ordinary options
open the real alias in the probe. That is a failed adoption gate, while the throughput
comparison continues. No SQLx fork or custom VFS was introduced.

## Retained construction evidence

`results-build-*-attempt1.log` records the initial fresh-target compile. Later final
build logs identify corrected/frozen source builds. Cargo builds passed; the SQLx
source has one harmless unused shared CONFIG constant warning.

`results-run-attempt1.log` and the first failed entry in `results-commands.jsonl`
retain a real NOFOLLOW open failure before the cohort. Python's macOS temp directory
spelling started at `/var/...`; `/var` is an alias of `/private/var`. The bridge's
native NOFOLLOW rejected that parent alias (SQLite extended code1550). The runner
now canonicalizes its freshly created private root; flags were preserved. No measured
workload was produced by the failed attempt. The first private directory contained
no live application data. The corrected environment is separately retained.

The runner records exact commands, exit status, wall times, release binary hashes,
source hashes and dirty baseline, toolchain/hardware, /usr/bin/time-l raw statistics,
and process-specific ps-M/RSS samples excluding the header. Resource maximums include
setup/audits/drain; samples are supporting process observations, not an application
memory delta. Application verification and packaging were intentionally unperformed.

Additional API-shape differences are intentional stack costs rather than hidden
SQLite equivalence claims: SQLx requests a100-statement cache; the bridge leaves
rusqlite0.42.0's default16-statement cache capacity and explicitly caches the hot event/page statements.
Bridge whole transactions execute in one maintained closure; SQLx awaits each native
statement inside its transaction. Shared context uses the same multi-query snapshot
shape in both. SQLx pools the two readers; the bridge has two statically assigned
reader connections. Neither adapter has an application-owned worker thread loop.

The focused COMMIT-BUSY check deliberately uses explicit BEGIN/COMMIT on a retained
connection (both drivers), so SQLite's still-active transaction can be retried after
the reader releases. It does not claim that SQLx's consuming Transaction::commit API
returns its transaction handle on failure; ordinary workload transactions use the
native transaction APIs and did not encounter SQL errors. Recovery policy remains
with the owning core; this is a focused supported-API mechanism proof.

Focused cancellation scope is reply-receiver drop after admission while an explicitly
retained async owner continues awaiting the operation. The semaphore permit remains
in that owner, not inside the bridge's physical closure. Arbitrary owner abort/panic,
cancelled COMMIT, or dropped library futures were not exercised. A future production
owner must preserve physical completion independently or move physical permits into
the physical call; this harness does not qualify those failure paths. The25ms group
setting is an eligibility trigger under contention, not a maximum commit deadline.
`reader_drain_s` is elapsed beyond the30s admission boundary after readers are joined;
it includes any preceding writer drain and is not an isolated incremental read drain.

An ad-hoc source-edit helper also stopped with Python `ValueError: substring not
found` when matching a rustfmt-spaced cfg marker. It stopped before its intended
file writes; the corrected helper was rerun before the frozen builds/cohort. This
was harness construction, not a storage/runtime failure. `results-build-history.json`
maps retained construction compile logs to their exact manifest/target commands;
final-source fresh-target costs and runtime-binary matching are reported separately.

## Completed frozen cohort

All24 workload runs (one warm-up plus five measured per driver/workload) and all
four focused before/after checks completed successfully. `results-summary.json`
aggregates only the20 measured workload runs and retains median/range. Both stacks
committed all15,000 baseline events per run without rejection. In heavy, measured
median commits were54,554 bridge and54,624 SQLx out of66,000 offered; corresponding
rejection medians were11,446 and11,376. Wide overlapping run ranges do not establish
a stable driver winner. End-to-end heavy p95 medians were726.7ms bridge and701.3ms
SQLx, with explicit backlog admission rejections. This is capacity evidence for
this particular16-event grouping policy and device, not a stream/chat cardinality
limit or authority to discard provider output. Production needs upstream backpressure
or visible admission/capacity fencing and recovery semantics.

Every run completed exactly600 reads per reader and150 urgent intents, with exact
durable intent IDs/workspaces matched. All full-record audits, route/foreign-key
checks and both in-process/post-close integrity checks passed. Both stacks were
observed with four OS threads (one main plus three maintained connection workers).
Loop heartbeat p95 medians stayed below3ms in these synthetic runs. This is executor
liveness evidence, not measured desktop navigation responsiveness.

Whole-process memory maximums are particularly easy to misread: the post-window
SQLx audit fetch_all retains all positive payload rows, while the bridge audit
hashes rows while iterating. That instrumentation raises SQLx's process maximum
outside the admission window (roughly45MiB baseline/122MiB heavy versus10/15MiB
bridge). It is not evidence of a steady-state driver memory gap. Timed page/inbox
materialization is equivalent; the process-specific ps samples support the bounded
live-window memory observation. Keep raw maxima, samples and this distinction together.

## Separately authorized grouping diagnostic

After the frozen cohort, one additional30s heavy run per stack changed only the
batch-count eligibility/maximum from16 to64. The25ms age trigger, offered event/read
trace, queues/quotas, urgent lane, SQL and settings stayed identical. Exact as-run
source/manifest and build/runtime commands are retained under `harness/group64-*`
and `results-group64-*`. Original source and binary hashes remain unchanged.

| Single diagnostic | Committed /66,000 | Explicit rejections | Arrival→commit p95 | Urgent p95 |
| --- | ---: | ---: | ---: | ---: |
| bridge, group64 |65,439|561|42.59ms|17.43ms|
| SQLx, group64 |65,382|618|43.72ms|23.78ms|

Every remaining rejection belonged to the intentionally hot workspace0; all19 other
workspaces admitted all3,000 offered events. Both completed the identical600 reads
per reader and150 urgent intents and passed all audits/integrity checks. Loop heartbeat
p95 was1.92ms bridge and2.08ms SQLx. This single pair shows that the observed group16
backlog was strongly policy-dependent, rather than a demonstrated fundamental driver
ceiling. It is not five-run capacity qualification, an unlimited-capacity claim or a
production batching selection. Maximum physical batch payload becomes64×64KiB=4MiB;
the original admitted logical8MiB budget remains, with closure/driver copies additional.

Final-source fresh-target release builds (warm registry/toolchain/OS caches) took
14.73s bridge and16.13s SQLx, producing2,856,096B and3,437,632B stripped binaries.
Both hashes exactly matched the binaries used in all frozen runtime samples. Normal
dependency graph package counts including root were28 and99; actual compile-log
package lines including build dependencies were33 and106. These are standalone
harness costs and library-ownership facts, not the application dependency delta.

`results-final-provenance.json` confirms every originally frozen source hash still
matches, both original binary hashes match fresh builds/runtime, all26 workload runs
plus four focused checks passed final accounting/integrity, and private databases,
failed-construction state and diagnostic/fresh targets were removed. Original private
build targets remain outside the repository (~95MiB bridge/~223MiB SQLx) for binary
review. The largest observed closed disposable database was156,180,480B, comfortably
within the declared8GiB source/build/data space budget. No application source/dependency
status changes were present. `results-summary-progress.log` is explicitly intermediate;
`results-summary.json`/`.log` are the completed five-run aggregates.
