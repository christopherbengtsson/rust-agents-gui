# Native core and SQLx architecture

Status: ready-for-agent

Accepted 2026-11-16. This is the final specification reconciled from the
[selected plan](plan.md), [review](review.md) and [primary-source research](review-research.md).
Publication changes documentation and authority only. Production still uses the
incumbent implementation; no implementation, performance or qualification pass is claimed.

## Authority and use

This spec and its [explicit amendments](amendments.md) supersede conflicting
hosting, lifetime, storage, concurrency and refactor sequencing clauses in the
documents named there, including the frontend foundation's prior precedence.
The [MVP](../../docs/MVP.md), [milestone](../../docs/MILESTONE-01.md),
[verification policy](../../docs/agents/verification.md) and
[observation guidance](../../docs/agents/observations.md) retain their remaining authority.
Original plans, reviews, experiments and completed checkpoints remain evidence.

The global [MVP development compatibility policy](../../docs/MVP.md#development-compatibility-policy)
governs this refactor and all subsequent milestones. Build the current design
directly; retired-build compatibility is excluded from engineering and acceptance.
Fresh state is the development baseline, and final MVP qualification starts clean.

Requirements below are binding behavior and ownership. Selected engineering
mechanisms implement them; provisional settings may be tuned with affected
qualification. Qualification gates are evidence still to obtain, not promises
already demonstrated or reasons to reopen the selected direction.

## Problem Statement

The application needs durable, concurrent coding work without a second
application host and its discovery, transport and packaging machinery. Moving
async storage beneath the current workspace lock would retain contention and
leave transaction uncertainty, reservation races and cleanup unaccounted for.
The refactor must remove that machinery while preserving exact actions, saved
work, independent controls and honest recovery.

## Solution

Host the UI-independent Rust core in the native Tauri process, use SQLx directly
inside Rust, and retire the application supervisor. Navigation, hiding/minimizing
and WebView replacement leave native-owned work running. Native-process failure
restores the durable prefix with interrupted/uncertain outcomes. Explicit Quit
finishes owned cleanup and persistence before reporting success.

Repository/workspace/conversation/agent cardinality has no product cap. Resource
admission is bounded and actual conflicts are coordinated. Milestone 01 must
demonstrate two active conversations sharing one workspace while another works
in another workspace. These counts are an acceptance scenario, never maxima.

## User Stories

1. As a developer, I want registered repositories and saved workspaces to keep their identities as history grows, so that resource tuning does not erase or limit my work.
2. As a developer, I want independent workspace operations to progress together, so that one repository's Git work does not stop another.
3. As a developer, I want two conversations to share a workspace deliberately, so that implementation and feedback can use the same files with visible overlap risk.
4. As a developer, I want each send, approval and interrupt to name its exact conversation and turn, so that switching views cannot redirect an operation.
5. As a developer, I want uncertain replies to retain their action identities, so that checking status cannot duplicate work.
6. As a developer, I want readable workspace names and permanent saved locations, so that concurrent creation and retry remain understandable.
7. As a developer, I want local history to survive lost native-provider history, so that I can still read my work when exact resume is unavailable.
8. As a developer, I want bounded history pages and useful limit errors, so that a large history does not freeze navigation or appear empty.
9. As a developer, I want quiet conversations and stop/status controls to progress during heavy output, so that an overloaded conversation remains controllable.
10. As a developer, I want a visible capacity failure with the last durable point, so that incomplete output cannot look complete.
11. As a developer, I want navigation and WebView replacement to preserve native work, so that presentation lifetime does not own execution.
12. As a developer, I want native failure to expose interrupted or unknown effects, so that reopening does not invent successful cleanup or repeat a send.
13. As a developer, I want one state owner and actionable owner conflicts, so that a second app cannot compete for the same records or processes.
14. As a developer, I want repeated Quit requests to observe one continuing drain, so that deadlines cannot abandon work or falsely report completion.
15. As a developer, I want storage errors and stale observations to remain inspectable, so that failed persistence is not hidden by a later successful close.
16. As a developer, I want existing code, worktrees, credentials and old evidence preserved during development reset, so that a format change cannot destroy unrelated work.
17. As a developer, I want provider setup and login to retain their exact scope across navigation, so that moving the host does not broaden authentication authority.
18. As a future user of review automation, I want preparation and publication to have distinct authority, so that opting into local assistance cannot silently publish changes.

## Requirements

| ID | Requirement |
| --- | --- |
| R1 | One native-hosted Rust execution/storage owner per chosen state root; core remains independent of Tauri. Preserve work across presentation changes, with durable recovery after native failure. |
| R2 | Keep repository, workspace, provider, conversation, turn, action and process identity distinct. No fixed product cardinality or one-active-conversation admission rule; reject duplicate ownership of the same native conversation. |
| R3 | Commit exact action identity, payload, targets and intent before external effects. Same ID/same payload observes that action; same ID/changed payload rejects. Uncertainty requires owner-local reconciliation before a potentially duplicate effect. |
| R4 | Preserve atomic registration plus first-creation continuation, readable permanent reservations and exact partial-effect recovery. Allocation and recovery use bounded metadata/materialization work, including across unrelated repositories. |
| R5 | Storage alone owns authoritative SQLx access, transactions, schema, opening/replacement and close. Use one current schema under the global MVP development compatibility policy; remove retired-build support paths and qualify fresh state. Keep SQL and database transactions out of frontend/domain interfaces. |
| R6 | Use validated canonical owner-private state and private regular single-link metadata files. Accept the explicitly selected limitation against same-user path replacement racing validation; do not claim atomic SQLite NOFOLLOW equivalence. |
| R7 | Authoritative reconciliation follows relevant writer settlement and a fresh committed snapshot. Unknown commit acknowledgment is not absence. Unresolved transaction cleanup cannot return a connection to ordinary use. |
| R8 | Admitted effects, physical capacity and outcome settlement outlive the IPC requester. Domain owners continuously consume outcomes/panics; no discarded failure because reply delivery failed. |
| R9 | Bound queued and in-flight bytes, frames, reads, results and physical work. Keep status/stop usable and quiet admitted work progressing. Never silently discard durable output from an admitted execution. |
| R10 | Preserve validated feature observations, exact owner/target/revision checks, last-good stale data, bounded invalidation/catch-up and physical request lifetime across logical cancellation. |
| R11 | Fence new effects before drain, collect all cleanup failures, settle domain work before storage closes and release the state lease last. Deadline means pending/failed cleanup with the same owner retained. |
| R12 | Qualify real storage, native lifecycle, fixture desktop and native providers separately from substituted units. Historical supervisor evidence is not qualification of the new host. |

## Implementation Decisions

### Core lifetime and task ownership

Use concrete workspace, runtime/provider and later conversation handles, with one
host-independent lifetime owner. It owns state lease, phase, admitted-start guards
and concrete domain completion handles. Each domain owns its continuing operations,
continuously reaps its own `JoinSet` and settles its own recoverable outcomes.
The lifetime owner collects compact drain reports; it has no second operation
ledger, result bodies or retry policy. The shutdown coordinator lives outside the
sets it drains. Do not add a parallel TaskTracker/job/shutdown framework.

Startup uses the existing host runtime, then resolves/validates state, acquires the
lease, initializes storage, restores bounded pages of durable actions and reconciles
or fences uncertainty before exposing ready handles. Do not create a runtime per
request or use per-request `block_on`. Tauri supplies its singleton runtime after
rehosting; the incumbent supervisor drives this same lifetime seam until cutover.
Phases distinguish starting, recovering, ready, fenced/failed, stopping and stopped.
Partial initialization visits every acquired resource despite earlier cleanup errors.

Admission has a concrete ordering:

1. Under the short root phase lock, check ready/resource capacity and obtain an
   admitted-start guard. The guard is counted before the request leaves this
   section; Quit closes this section atomically. Root locks span no await.
2. A workspace-owned short async decision gate registers a provisional activity
   with action/workspace/conversation identity and generation before asking another
   owner. It may await typed persistence for the whole decision, never a long Git,
   provider, task join or human operation. Lifecycle fencing uses this same gate.
3. Provider admission checks credential scope and duplicate native-conversation
   ownership through its own short decision, with no workspace gate held. Provider
   code never calls back into workspace admission while holding its gate.
4. Re-enter workspace admission to validate generation/fence and commit intent,
   then register retained execution/completion ownership before releasing the
   admitted-start guard. Any denied/failed/uncertain step settles or fences its
   provisional activity and unwinds acquired permits without guessing rollback.

Root Quit first closes root admission, fences workspace/provider starts, and waits
for admitted-start guards to finish transfer or unwind before final enumeration
and joining. A start accepted before the fence is either owned by the drain or
settled without effects; none can escape between a check and enumeration. Domain
fences must also be checked at the final effect admission. Stop/status bypass
ordinary admission; they never reopen it. Release all decision gates before joining
or long execution. Storage calls back into no domain while owning a connection.

Use one small workspace coordinator/decision gate initially, with retained
per-operation activity records and concurrent dispatched effects. Serialize actual
common-Git-directory mutations per repository; retain a keyed gate through physical
completion and all waiters before retiring it. Replace the global active-workspace
sentinel with exact activity records. External Git locks and edits remain evidence
to revalidate, not effects the app can exclude. Workspace lifecycle fencing accounts
for every affected conversation; future archive behavior remains a separate feature.

### Atomic, bounded reservation

Save the pending creation identity before read-only inspection. Replace load-all
folder/name sets with indexed candidate lookups and a typed atomic claim. A claim
transaction reads or claims the repository folder in the global namespace, claims
the per-repository readable name, and records final branch/path/reservation with
the action together. Unique constraints back both namespaces. A candidate conflict
before reservation rolls back the attempt and advances a checked candidate cursor;
a lost claim/commit acknowledgment first reconciles the same action after settlement.

Bound candidate attempts per scheduling turn and persist a continuation cursor
under the same pending action when more work is needed. Yield to controls and
other actions between turns. Exhausting a turn is pending allocation, not a total
name/history cap. Use the existing catalogue and numeric variants without loading
all historical reservations or walking all numeric tiers each time. Filesystem/Git
probes are bounded outside the transaction. After a reservation commits, retries
retain its exact location; an external collision requires explicit recovery.
Audit startup, pending-action and filesystem enumeration for the same boundedness.

### Storage topology, opening and transaction settlement

Select direct SQLx 0.9.0, defaults disabled, SQLite/Tokio features only as needed,
with ordinary bound runtime queries. Resolve and lock one coherent SQLite-linked
graph; remove production rusqlite in the same cutover. The existing Rust 1.99
requirement covers the assessed SQLx minimum. Record the actual bundled SQLite
version/source and verify the WAL-reset fix before adoption. Query macros/offline
metadata, custom VFS/raw-handle adoption and driver forks are deferred.

Choose the split topology: one max-one writer pool and a small read-only reader
pool. It earns the second lifetime through reserved writer access and mechanical
reader separation, using maintained pools rather than a custom acquisition
scheduler. Reader count is tunable. Writer initializes current schema and persistent
WAL first; readers use `read_only(true)`, no creation and no live `immutable` mode.
Shared establishment logic validates effective settings on initial and replacement
connections; readers verify persistent WAL without trying to switch journal mode.
Do not open a connection per workspace, conversation or operation.

Require WAL, FULL synchronization, macOS fullfsync and foreign keys for authoritative
state. Validate directory ownership/mode and database/journal/WAL/SHM file type,
ownership, link count and private modes before opening and through supported
establishment/reuse hooks. Hooks run after native opening and do not restore atomic
NOFOLLOW. Use safe private creation and retain native NOFOLLOW for application-created
lease/files where available. Do not change process-wide umask after threads start.

Each typed write retains an acquired pool connection outside the scope of a
transaction borrowed from it. On a statement/commit/explicit-rollback error, finish
the borrowed scope and explicitly retire that acquired connection with supported
fallible close; never return it to idle first. Retain original failure, retirement
failure and action uncertainty separately. Reconcile a possible committed result
on a fresh connection only after physical settlement/retirement is established.
An unresolved close keeps the writer fenced; do not overlap an unconfirmed old
writer with a replacement authorized to repeat effects.

For abnormal drop/panic fallback, `after_release` first awaits worker ping to order
queued cleanup, then rejects on ping failure or `is_in_transaction()`. This is a
conservative guard, not an exact engine-state diagnosis. Hook rejection closes
through SQLx; domain panic handling separately records uncertainty and fences the
affected operation. Default pool ping alone is insufficient. This choice follows
the [published cleanup analysis](review-research.md#the-concrete-reuse-hole) and
the supported [pool return hook](https://github.com/transact-rs/sqlx/blob/003b698e99e024f3621b8043a2426fde5b741171/sqlx-core/src/pool/options.rs#L446).

Retained tasks publish an action's writer-settlement receipt independently of
reply delivery. Status of an unsettled writer is pending/unknown; a reader cannot
turn its stale absence into non-admission. Consistent multi-query reads use one
short read transaction/connection and return the committed position/revision.
Keyset-page rows and bytes, including materialization/driver buffers; oversized
content produces a typed limit result. Release snapshots before other effects or
human input. No arbitrary SQL/transaction/closure scheduler escapes storage.

If an effect outcome cannot be persisted, retain bounded volatile fault/uncertainty
with the exact action and last durable point, fence affected new effects and expose
the storage failure. A lost receiver, later successful close or reclaimed result
body cannot erase it or convert it into durable success. Recovery decisions remain
with the owning domain; diagnostic logs are not the only retained indication.

Retain ordinary automatic checkpoints initially. Observe duration, WAL growth and
busy/incomplete/frame results of explicit probes; bound and own that work. Short
readers and release of stalled reads precede a new checkpoint service. Do not unlink
WAL/SHM or require truncation for every close. Busy timeout is not an operation
deadline; checkpoint pages are not a WAL-size limit. Later backup must use a
consistent SQLite operation. Kill testing establishes process-crash recovery,
not power-loss durability. See the [SQLite WAL contract](https://www.sqlite.org/wal.html).

### Process effects and provider failure domains

Use Tokio child pipes/waits and maintained channels/semaphores to remove workspace
wake threads and Git read/sleep polling. Keep one shared process mechanism for
bounded output, deadlines, exact owned identity, signaling/escalation and observed
cleanup. Finite Git collection and long-lived provider protocol adapters remain
different interfaces. Preserve unreaped-leader/group-quiescence evidence and its
uncertainty. Leader wait, kill-on-drop, ECHILD and process-group signals alone do
not prove all descendants stopped or native-crash containment. The reassessed
process-wrap 10.0.1 does not remove that gap; add no process wrapper now.

For the first concurrent Codex execution slice, select one app-server process per
independently active native conversation, released only after owned cleanup; no
process is allocated for merely viewing historical conversations. Keep exact native
conversation exclusion across all processes. Readiness/login remain provider-owned
and credential-scope exclusion still applies. This engineering choice isolates
stdio pressure; it is not a new application supervisor or a product concurrency cap.
Qualify this topology with the actual supported provider before claiming independent
native control. Future sharing requires evidence of equivalent independent control,
not a blind pooling optimization. The pinned Codex stream has a
[serialized stdout writer](https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/app-server-transport/src/transport/stdio.rs#L127).

Protocol reading/routing must not await bulk persistence. Use bounded per-process
frame/decode and control capacity; try admission to bounded durable-output buffers.
At saturation, latch an exact capacity/gap outcome, request owned stop and fence
that connection. Drain only within the cleanup budget; an unresponsive peer escalates
through the process owner. Never drain unbounded data into memory while waiting for
storage, and never claim an interrupt was received merely because it was written.
Loss of one execution connection identifies its conversation; shared storage/native
failure can fence all affected work explicitly. Provider/tool RSS is measured
separately from app buffers. Frame/item limits are fixed from the selected chat
schema/payload scope in ticket 07 before enabling sends, including oversized inputs.

### Durable chat handoff and fair persistence

Foundation tickets implement existing workspace/provider persistence only. Milestone
07 adds transcript events to the same database as actions/turns, indexed by exact
conversation and durable position; commit that position with the batch. Presented
output, provider receipt, durable position and invalidation cursor remain distinct.
First attachment implementation should use bounded database content with separate
content reads; attachments, external blob policy and their schemas are outside this
refactor and Milestone 01's current text-chat scope.

At chat's first use, implement only a private count/byte/time-bounded writer policy.
Maintain per-conversation FIFO, rotate ready conversations within each ready workspace,
and rotate workspaces after at most one bounded batch. Fairness means progress for
both a quiet conversation sharing the hot workspace and one in another workspace;
it does not promise equal byte bandwidth. When urgent durable intent and output
are both pending, alternate one urgent write and one output batch. Stop signals
do not wait for that queue. Running SQLite work is not preemptible. No preallocated
queue/share for every historical workspace, deficit/credit algorithm or borrowing
scheduler is selected. Add complexity only against a failed declared trace.

### Bounds, observations and diagnostics

Admit before core task/queue/effect submission and bound waiting interests. Tauri
has already received/deserialized command arguments at this point: bound app-generated
inputs before invoke and measure IPC/decode copies separately. Core command payload
budget is not a bound on all IPC memory. Keep physical permits with actual work and
result permits with retained bodies. Long turns retain small identity/state and
stream budgets, not one ordinary request slot for their duration. Retention
exhaustion fences/rejects new work and preserves compact uncertainty/fault evidence.

Retain Router navigation, Query's sole validated observation cache, XState feed and
inspection coordination, Zustand's allowlisted presentation memory and p-queue's
unsettled physical registry. Keep existing feature interfaces, pending identities
and freshness generations. Replace socket delivery with app-scoped Tauri channels:
incarnation/subscription identity, bounded hints, accept-before-ack, gaps/resync and
stop/join-before-replacement all remain. The journal is volatile, not a transcript.
Generated Rust declarations/structural validators remain authoritative; replace
supervisor identity with core incarnation, preserve contextual checks, and delete
peer protocol negotiation without a dual wire format.

For inactive Query retention choose a small cache-owned count eviction policy,
initially 128 inactive entries, plus existing time GC. Evict only inactive bodies;
retain pending identities/freshness obligations and independently account active
and unsettled physical results. Defer serialized-byte LRU and the proposed 16 MiB
Query guarantee. Bounded page bytes plus entry counts aid accounting but do not
equal JavaScript heap size; measure heap/retention in qualification.

Errors identify action/target, observed state, last durable point, reason and safe
next action. Distinguish rejected admission, failed read/effect, unknown effect and
pending cleanup. Add scoped tracing/thiserror during changed paths, with identities,
timing/pressure/shutdown phases and original error sources; exclude secrets, auth
URLs, transcript bodies and environment dumps. Logs supplement authoritative faults.

### Quit, failure and development cutover

Explicit Quit synchronously prevents ordinary Tauri exit and starts/observes one
retained async shutdown. Fence admission and producers; collect provider/conversation
and workspace task/process results while storage remains open; settle final writes
and reads; observe checkpoint results; await both pool closures; stop/join delivery
and host resources; release the lease last and then allow exit. Attempt all cleanup
stages even when another fails. Success requires clean required outcomes, not merely
empty queues. `Pool::close().await` returns no aggregate close-error inventory or
literal worker-thread join; preserve known operation/retirement errors independently.

A deadline retains the same owner, handles and pending/failed report. Repeated Quit
observes that attempt. Force-kill is recovery, never successful drain. Route any
later restart/update through completed shutdown before Tauri restart; ordinary exit
interception does not cover every restart path in the
[Tauri lifecycle contract](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.5/crates/tauri/src/app.rs#L79).

Native restart gets a new incarnation, reads the durable prefix and reconciles
recorded Git/provider effects. Uncertain survivors fence conflicting work; saved
PIDs do not authorize signaling. WebView replacement attaches to the existing
incarnation with status/catch-up and no replay. Viewing history never starts work.

Cutover inventory must identify old/new state and worktree roots. Stop the matching
old build and observe cleanup. Reuse the existing lease filename/inode for any same-root
handover; never unlink/recreate it to bypass a live owner. Choose a fresh private
metadata namespace with an explicit current schema identifier, independent of the
retired supervisor protocol/binary hash. Explicit state override remains exact.
Reject incompatible metadata actionably and use a deliberate documented reset
rather than silently initializing over it. Old app metadata/history/preferences
need no migration, import or continued readability; retaining old builds or a closed
database/WAL/SHM set is optional, not a compatibility or acceptance requirement.
Preserve checkouts, all managed worktrees/branches, provider homes/credentials/native
history and evidence. New registrations use fresh managed locations and cannot take
over old worktrees. Audit the launcher's three-second escalation now that killing
Tauri kills the core. This spec executes no reset.

## Provisional Settings

These are starting configurations, not capacities, latency guarantees, schema
restrictions or reasons to defer a concrete implementation decision. Record the
effective values and tune only with affected qualification. Keep existing tighter
feature limits unless deliberately changed and checked; budgets at different layers
are not additive permission to bypass one another.

| Setting | Initial value | Full/closed behavior or qualification |
| --- | --- | --- |
| Core ordinary requests | 64 outstanding; 64 KiB/item; 4 MiB retained payload | Typed busy/not-admitted before core submission; bounded waiters; IPC/decode copies separate. |
| Core control/status | 8 bounded interests; direct coalescing stop token | Progress during ordinary saturation; cleanup can still be pending at an unresponsive effect. |
| Finite physical effects | 8 ordinary, 2 cleanup reserve | Acquire before dispatch; hold until physical completion. |
| SQLx pools/acquisition | 1 writer, 2 read-only readers; 1 s acquire | Bounded admitted waiters; timeout never proves an earlier write absent. No timed churn initially. |
| Busy/checkpoint | 100 ms busy; 1,000-page autocheckpoint | Record effective settings, contention and WAL progress; neither is a total deadline/size bound. |
| Metadata pages/interests | 32 rows, 48 KiB/page, 8 retained native read interests | Keyset continuation or typed oversized/unavailable result; include decode and driver-row bytes. |
| Reservation attempts | 32 candidates/turn | Persist cursor and yield under same action; no cap on saved names. |
| Invalidation | 256 journal entries; 32 tags/4 KiB batch; one unacked batch/subscription | Coalesce or resync on gap; no unbounded channel-to-WebView queue. |
| Inactive Query entries | 128 plus existing time GC | Remove inactive bodies only; measure heap, preserve identity/freshness. |
| Chat batch triggers (07) | 64 events OR 256 KiB OR oldest eligible event age 25 ms | Flush sooner on final/stop; timers use controlled time in units. Tune against production trace. |
| Chat output (07) | 16 MiB globally, initial 4 MiB hard per active conversation including queued/in-flight app output | No workspace-share/borrowing scheduler. Frame/decode/driver copies budgeted explicitly before enablement; stop/fence on exhaustion with durable prefix/gap. |

WAL/FULL/fullfsync/foreign-key requirements are deliberately outside this tunable
table. Weaker durability or independent-control promises require an explicit
authority/product amendment with evidence, not quiet tuning.

## Testing Decisions and Qualification Gates

Follow repository substitution policy, using the existing isolated unit entry point.
Test real owner-local decisions with scenario-specific storage/filesystem/Git/process
outcomes, controlled events/time and the smallest useful owning-module interface.
No real SQLite, sockets, Git/worktrees or provider/supervisor processes in application
units; no new integration-test framework. Private seams are valid. Unit success
cannot establish disk, OS, desktop or native-provider behavior.

| Gate | Required evidence and owner |
| --- | --- |
| Q1 · Lifetime (01) | Admission versus fence; lost requester; panic attribution; collect-all-errors; partial startup; blocking completion; repeated Quit/deadline; lease-last. Separately exercise actual lease/process fixture ownership. |
| Q2 · SQLx cutover (02–04) | One resolved graph/engine with WAL fix; validated initial/replacement opens; effective read-only readers/settings; atomic writes; failed commit/rollback retirement and settled-read ordering; bounded acquisitions/pages; close/checkpoint behavior. Substituted decisions and separate real storage faults, pinned readers, disk/full or controlled I/O faults and kill points. Unperformed fault cases remain open. |
| Q3 · Workspace/provider/process (03–05) | Cross-repository folder conflicts, same-repository names, duplicate/payload mismatch, bounded continuation, stale generations and cross-kind capacity; independent effects; exact credential scope; waiter loss/panic; partial Git/process cleanup. Preserve exact matching and native cleanup uncertainty. |
| Q4 · Host/package (06–07) | Actual native-owned Tauri commands/feed, owner mismatch, WebView replacement, login fixtures across navigation, native failure/reopen, Quit during startup/activity, competing same-root owner and failed cleanup. Delete supervisor resources/peer paths and verify dev/build/package plus remaining executable provenance. |
| Q5 · Foundation handoff (08) | Cumulative current workspace/provider foundation under concurrent operations and bounded read/write pressure, usable status/stop, historical coverage mapping, settings/metrics and unresolved limits. No streamed-chat fairness or native execution pass may be claimed before its producer exists. |
| Q6 · Chat and milestone (07–13 in milestone tracker) | New chat schemas/fixtures; selected per-active-conversation process topology; three-conversation sustained and overload traces; same-workspace and cross-workspace quiet progress; approvals/interrupt, output accounting/durable positions, history/exact resume, native crash/recovery and final owned Quit. Complete fixture desktop first; actual provider scenarios require a separately agreed allowance. |

For Q6 declare the sustained offered trace and timestamps/acceptance envelope before
running: two conversations share workspace A (one hot, one quiet), a third runs in B,
slow writer/acquisition, approvals and interrupt in quiet conversations, plus history
reads/navigation. Under sustained supported load require all admitted output to settle,
quiet work to progress, navigation feedback within 100 ms and recent history within
one second on the test Mac. Under forced overload require bounded memory, useful
control/status and an explicit affected-connection failure with prefix/gap; do not
score rejected output as successful sustained handling. Declare control/settlement
deadlines from the supported protocol before the cohort and record actual receipt,
completion and pending/unknown separately. Default tuning cannot waive these outcomes.

Record source/dirty identity, versions/engine/schema, hardware, exact commands,
profile/cache/threads, counts, exit status and wall time. For timing claims follow
one warm-up/five warm runs with median/range; separate compilation, full verification
and packaging. Record stream/commit/control latency, offered/admitted/committed/failed/
outstanding bytes/events, per-conversation/workspace progress, app buffers/heap versus
child RSS, acquisition/checkpoint/WAL metrics, build/bundle deltas and drain results.
One build/check at a time per worktree; retain target claims and failed attempts.

Coverage replacement is an implementation gate: map useful old assertions to new
owning-module units/actual qualification before deleting them. Retain registration
continuation, reservation immutability, no duplicate effects, exact approvals,
credential scope, owner exclusion, cleanup and observation freshness. Retire old
format-only fixtures under development policy and socket/peer mechanics when removed;
their results remain historical. Inspect current scripts before selecting checks;
replace the retired `qualify:process` cohort with identifiable storage/native lifecycle
qualification, not deletion of all process evidence. Keep generated preparation,
static checks, compilation, units, complete verification and packaging distinct.

## Dependency-Ordered Implementation Tickets

| Ticket | Delivers | Blocked by |
| --- | --- | --- |
| [01](issues/01-own-core-startup-and-complete-shutdown.md) | Whole-core completion and lease ordering on incumbent host | None |
| [02](issues/02-establish-sqlx-storage-and-settlement.md) | Current-schema SQLx and transaction retirement | 01 |
| [03](issues/03-admit-concurrent-workspaces-and-atomic-reservations.md) | Async workspace decisions and bounded atomic names | 02 |
| [04](issues/04-persist-provider-actions-through-shared-storage.md) | Shared provider persistence and completed driver cutover | 03 |
| [05](issues/05-own-bounded-async-process-effects.md) | Async process mechanics with preserved cleanup evidence | 04 |
| [06](issues/06-host-direct-core-actions-and-observations-in-tauri.md) | Native-hosted action/feed and lifecycle slice | 05 |
| [07](issues/07-delete-supervisor-and-qualify-packaging.md) | Delete old host/transport/staging, qualify package/start | 06 |
| [08](issues/08-qualify-foundation-and-publish-chat-handoff.md) | Cumulative foundation qualification and actual chat handoff | 07 |

02–04 are one dependency-ordered integration checkpoint: the assessed drivers cannot
coexist unchanged. Work on one integration branch with one driver; intermediate
checkpoints may be unbuildable and are not independently merged or claimed green.
04 requires the combined gate. Do not create a compatibility implementation or
legacy-host framework to manufacture intermediate green builds. Other tickets
produce verifiable increments; 06 makes the native path exclusive, and 07 removes
unreachable incumbent source/tooling after focused native checks.

Then use the amended existing milestone tickets: **07 → {08,09}; 09 → 10;
{08,10} → 11 → 12 → 13**. Milestone 07 also requires this feature's 08 and the
qualified provider-connection/workspace handoffs. Milestone 12 owns cumulative
three-conversation control/pressure qualification; 13 owns the separately authorized
native cohort. These are existing capability tickets, not duplicate chat implementations.

| Existing milestone ticket | Delivers | Blocked by |
| --- | --- | --- |
| [M07](../milestone-01/issues/07-durable-streamed-chat-and-uncertain-send-recovery.md) | Concurrent durable chat, writer policy and initial stream-pressure evidence | SQLx 08; completed workspace 04/provider-independent 01; connection 04/parent 06 |
| [M08](../milestone-01/issues/08-answer-exact-pending-approvals.md) | Exact concurrent approval controls | M07 |
| [M09](../milestone-01/issues/09-interrupt-work-and-quit-with-owned-cleanup.md) | Exact concurrent interrupt and complete Quit | M07 |
| [M10](../milestone-01/issues/10-read-history-and-resume-exact-conversation.md) | Paged local history and explicit exact resume | M09 |
| [M11](../milestone-01/issues/11-recover-from-supervisor-failure-and-surviving-runtimes.md) | Native failure and surviving-runtime reconciliation | M08, M10 |
| [M12](../milestone-01/issues/12-verify-complete-desktop-workflow-with-fixtures.md) | Cumulative fixture workflow and three-conversation control/pressure | Completed inspection 05; M11 |
| [M13](../milestone-01/issues/13-bounded-native-qualification-and-milestone-evidence.md) | Separately authorized native provider cohort and milestone evidence | M12; exact live allowance before submissions |

Select one ticket per future session and confirm checked predecessor evidence;
publication authorizes none of their execution in this session.

## Out of Scope and Future Direction

Keep Conductor's workspace/concurrent-agent workflow as the product baseline;
shipped dependency/bridge evidence does not establish its private ownership,
durability or scheduling. Keep official Codex and future Claude adapters with
runtime-owned credentials. No frontend SQL plugin, alternate driver contest,
generic actor/job/workflow framework, watcher, terminal/PTY, keep-awake feature,
single-instance UX, cloud client or after-Quit execution is added.

Incoming reviews, own-PR assistance, unified inbox and automation remain committed
future directions; Jira is optional. Future `pull_requests` owns remote host/repository/
PR identity and reconciliation independently of local identities. `reviews` owns
exact base/head/comparison/draft revisions, independent supplied review context,
findings and preserved human edits. `automation` owns opt-in trigger/grant/job intent;
attention is a projection, never execution truth. Manual and agent calls use the
same targeted Rust actions and record initiator separately from authority.

Prefer installed `gh`/`gh api` with CLI-owned credentials and explicit remote targets
when those features arrive. Preparation grants do not authorize publication, push,
CI rerun or merge: human approval remains the default. Bind approvals to exact scope,
revision and draft; record remote intent and reconcile uncertain writes before retry.
Runtime tools/permissions must enforce any promised unattended local-only grant;
otherwise block unattended execution or require tool approval. Measure usage without
claiming a hard billing cap. Persist chaining/trigger identities when first used;
add no future tables/scheduler now. Native-crash uninterrupted work, independent
hosts or running after Quit would reopen lifetime authority only if later requested.

## Further Notes

All nine review findings are resolved by the corresponding mechanisms above:
transaction retirement; global atomic reservations; isolated execution streams and
overload transition; ordered admission/fencing; bounded candidate continuation;
read-only split pools; provisional settings/simple rotation/cache count; domain-owned
reaping and retained shutdown; smaller cutover checkpoints with chat pressure at Q6.
No material product/authority question remains open. A failed gate calls for an
engineering fix or a concrete evidence-backed request to amend a promise, not a
silent weakening of it.

Source cohort: plan/review `24390a1e7ba108ca88704fbf73deac92915d3ab6` plus pre-existing
local documents. Assessed SQLx 0.9.0 published source `003b698e…`, Tokio 1.54.1,
Tokio-util 0.7.20, Tauri 2.12.5. Codex readiness is 0.169.0/source `d32063f…`, not
the older standalone proof; newer stable compatibility remains capability-based.
Implementation must record its resolved cohort. Research/benchmarks prove no
throughput winner, unrestricted capacity, power-loss durability or native chat pass.
