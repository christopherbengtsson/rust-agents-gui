# 06b: Prepare storage ownership for durable chat

Status: wontfix

Blocked by: [06a — Storage evaluation](06a-evaluate-storage-before-durable-chat.md)

## Superseded — 2026-11-16

The [accepted native-core/SQLx spec](../../sqlx-architecture/spec.md) and
[amendments A6/A8](../../sqlx-architecture/amendments.md) replace this ticket's
bridge, opening and supervisor implementation instructions with
[SQLx tickets 01–08](../../sqlx-architecture/spec.md#dependency-ordered-implementation-tickets).
Do not implement the retained body below. It is historical planning, not completed
work; acceptance stays unchecked. `wontfix` retires this superseded route, not the
storage capability. Milestone 07 now waits for SQLx 08's actual qualified handoff.
Completed 06a research/evidence remains intact. No production change or provider
allowance follows from this amendment.

## Start here

Implement and qualify existing workspace/provider persistence with one shared
storage lifecycle before [07](07-durable-streamed-chat-and-uncertain-send-recovery.md).
This ticket is prepared for a separately selected implementation session; this
revision implements no production change and checks no acceptance item.

Read [MVP ownership](../../../docs/MVP.md#starting-architecture),
[milestone scope and qualification](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md), and
[physical-completion guidance](../../../docs/agents/observations.md#physical-completion-and-cancellation).
Consume the [final evaluation](../../../docs/research/milestone-01-storage-evaluation.md),
[driver-selection checkpoint](../../../docs/research/sqlx-vs-rusqlite-selection.md),
[bounded async comparison](../../../docs/research/storage-async-architecture-comparison.md)
and its [evidence limits](../../../docs/evidence/storage-async-comparison/README.md).
The bounded comparison supersedes the earlier provisional driver choice;
earlier measurements remain historical evidence.

## Implementation choices and limits

| Choice | Basis and implementation obligation |
| --- | --- |
| Bundled rusqlite 0.42.0 through maintained tokio-rusqlite 0.9.0 | Preserves supported opening flags and performed comparably to genuine async SQLx. This is a maintenance/behavior choice, not a performance victory. Inventory mechanisms removed, retained and added; prefer maintained mechanisms internally. A need for a general pool or substantial custom driver machinery must trigger a documented reassessment rather than an automatic framework build. SQLx remains an alternative requiring a supported opening solution or an explicit behavior decision; this ticket preserves current opening behavior. |
| One application-scoped writer; initially two read handles | One writer matches SQLite serialization. Two readers are a provisional internal default matching the comparison, not a demonstrated optimum or a public contract. Qualify bounded contended acquisition; fixed serial reader tasks did not do so. Any topology adjustment needs a recorded contention/maintenance rationale and affected qualification, with no connection-per-workspace/chat/child policy. |
| WAL/FULL/fullfsync; foreign keys; 100 ms busy timeout; 1,000-page automatic checkpoints | Starting settings for authoritative local state. Busy timeout is not an operation deadline, the checkpoint threshold is not a WAL-size bound, and process-crash checks do not establish power-loss behavior. Preserve durability while qualifying pressure and checkpoint handling. |

Keep SQL, closure scheduling, handle selection and transaction cleanup inside
`storage`; retain domain-shaped interfaces and controlled-outcome seams. A single
production adapter does not invalidate the repository's required test seams.
Keep the core independent of Tauri, workspace identity provider-independent and
provider mappings conversation-owned. Temporary execution admission limits must
remain separate from schema identity/cardinality restrictions.

## Module ownership and change scope

| Owner / files | Responsibility in this ticket |
| --- | --- |
| Supervisor main and `Core` composition | Construct the runtime before async initialization; inject one shared storage owner into the domain modules; coordinate startup cleanup and complete shutdown while exclusive ownership is retained. |
| `storage.rs` and private storage modules as needed | Validated opening/reopening, connection health, SQL/transactions, physical admission/completion, bounded result retention, checkpoint evidence and explicit close. |
| `workspaces.rs` and `workspaces/creation.rs` | Whole registration/creation admission decisions, reservations, retries/reconciliation and the workspace coordinator's ability to await storage. Git/filesystem/inspection work stays with its existing owners and blocking effect seams. |
| Provider connection login/selection modules | Await shared persistence while retaining intent-before-authentication, exact target/revision checks and cleanup fencing. Remove replaced database-only `spawn_blocking` and reconnect code. |
| Connection dispatch and shutdown observations | Await async domain operations, preserve unrelated blocking dispatch, and report complete/pending/failed whole-core shutdown. Reuse useful task/semaphore mechanisms without making storage depend on transport policy. |
| Tauri | Present explicitly targeted actions and authoritative outcomes; reconnect retrieves status. It owns neither a storage scheduler nor reconciliation/retry policy. |

## Acceptance

### Composition and sequencing

- [ ] Open one shared storage owner within the continuously driven supervisor Tokio runtime after exclusive owner admission. Centralize schema initialization before domain consumers start. On partial startup failure, observe cleanup of every successfully opened handle and started owner before releasing ownership. Use fresh private current-build state; document owned stop/restart for incompatible formats, without earlier-format migrations/readers.
- [ ] Adapt workspace coordination to await persistence on that runtime, preserving its Git/filesystem/process dispatch and intent-before-effect ordering. Remove only replaced storage-only dispatch; introduce neither a runtime per operation nor a custom per-connection thread loop.
- [ ] Preserve the **whole** registration/creation admission sequence across awaits, including duplicate action identity/payload, cross-kind conflicts, shared pending capacity and active-workspace authority. An owning-module async operation gate or equivalent reservation must serialize conflicting decisions; individual writer closures are insufficient. Keep synchronous state guards outside waits and revalidate exact identity/state before applying results or publishing observations. Preserve atomic registration/first-workspace continuation and immutable reserved locations.

### Admission, completion and recovery

- [ ] Document numeric limits and full/closed behavior for queued and physical operations, acquisition waiters/read interests, input bytes, closure copies, materialized read results and undelivered completions. Account for each retained allocation through its actual lifetime; reject before submission or wait only within a bounded admitted interest. Admit before task creation/bridge submission, including the bridge's unbounded queue. Bound page rows/bytes and oversized-item behavior. Demonstrate cleanup/control progress under saturation; numeric defaults are implementation choices to record and qualify here, not copied product caps from the harness.
- [ ] Retain each submitted operation's physical completion and outcome independently of caller interest. Transfer capacity guards into the actual bridge closure, or establish an equally strong physical lifetime. A permit alone is not outcome retention: existing connection tasks can discard an operation error on failed reply delivery. Keep a storage completion owner through settlement, bounded result delivery/reclamation, and failure reporting. Release physical and result capacity at their respective completion points. Preserve recoverable domain status and an unacknowledged failure indication when reclaiming bulky results; retention exhaustion fences/rejects new work rather than silently erasing uncertainty.
- [ ] Exercise each fault case below independently. Retain earlier operation failures even when later `close()` succeeds; distinguish drained database effects and closed handles from unobserved OS-thread exit. Reply-loss research did not qualify owner abort/panic.
- [ ] Before an authoritative reconciliation read, await settlement of the corresponding admitted writer operation and establish a fresh committed snapshot. Inspect transaction state after failed COMMIT; complete cleanup/recovery or fence the affected handle before reuse. A reader observing absence while the original write remains pending is not retry authority. Preserve owning-module retry/reconciliation decisions; neither admission failure nor receiver loss authorizes repeating uncertain external effects. Reopening uses the shared validated path and cannot overlap an unresolved old writer as a replacement owner.

| Fault case | Required result |
| --- | --- |
| Request/reply receiver disappears | Admitted effects, capacity and outcome remain owned until settlement; later status/reconciliation identifies the original operation. |
| Operation async waiter/task is aborted or panics, before or after bridge submission | Application-lifetime completion ownership survives it. Submitted work keeps physical capacity; its result or explicit uncertainty remains retained. No automatic replay. |
| Bridge closure/worker fails, or storage completion ownership fails | Fence affected admission, retain failure/uncertainty and report failed/pending cleanup. A closed channel or successful later close is insufficient evidence of clean completion. Do not replace a writer while its effects/cleanup remain unconfirmed. |
| Supervisor/runtime is lost | Apply current-build crash recovery from the durable prefix and reconcile surviving work. Do not claim in-process draining or successful Quit after root loss. This is separate from recoverable waiter cancellation. |

### Opening and shutdown

- [ ] Centralize create/open/reopen validation for the database and journal/WAL/SHM paths: native NOFOLLOW, private regular single-link files and established private-directory/owner checks. Validate effective PRAGMAs on initialized/recovered handles. Preserve existing behavior without inventing a broader attacker guarantee; retain recovery material on failure.
- [ ] Observe checkpoint frame counts, duration and WAL size, and define the response to long readers, incomplete checkpoints and storage pressure. Keep read transactions short. Retain automatic checkpoints initially; any explicit checkpoint is bounded/owned and its incomplete result remains visible. Truncation is an explicit idle/drained maintenance decision, not a prerequisite for every successful request or clean close. Preserve live WAL/recovery files.
- [ ] Make whole-core shutdown explicit: fence new domain actions; settle workspace/provider producers and their final persistence; fence storage admission; drain every admitted operation and retained failure; observe checkpoint/close results for all handles; then report successful Quit and release exclusive ownership. Keep reconciliation/shutdown status available while pending or failed. Do not rely on workspace `Drop` or provider-only shutdown status. Include normal Quit, accept/handler failure and partial initialization; collect cleanup errors without skipping remaining cleanup because an earlier stage failed. A deadline reports pending/failed cleanup and retains ownership of continuing work; it does not prove cancellation or permit a competing owner.

### Verification and handoff

- [ ] Complete the separate verification cohorts below and publish exact commands, source/cohort, outcomes and qualification limits. Preserve useful controlled-outcome coverage or record its replacement mapping before removal. Production storage remains unqualified until these checks pass.
- [ ] Publish 07's storage handoff: actual current interfaces, admission/error/lifecycle semantics, committed-read ordering and the conceptual capabilities for atomic intent/metadata/event writes, bounded indexed history and accepted durable positions. Keep conversation decisions and concrete schema/grouping implementations in 07 at first use. Identify how urgent intent and cleanup can progress alongside future bulk writes without prebuilding the conversation model.

## Verification cohorts

| Cohort | Required coverage |
| --- | --- |
| Dependency-substituted application units | Concurrent duplicate/cross-kind/capacity admissions, stale state after awaits, atomic continuation and immutable retry locations, every fault-table decision, active transaction after failed COMMIT, rollback/reopen failure, read-after-uncertain-write ordering, saturation/result retention and shutdown racing admission. Use controlled events/time; real SQLite/filesystem/process effects remain separate qualification. |
| Current-build actual storage/process qualification | Fresh private state; effective settings and centralized open/reopen protections for database and sidecars (aliases, links, permissions); acknowledged/uncertain writes across restart; actual bridge waiter cancellation and worker failure; bounded contended acquisition/results; saturation with cleanup progress; pinned-reader/checkpoint behavior; all-handle close, owner exclusion and partial/abnormal shutdown. Record failures and unsupported/unperformed probes explicitly; substitutes cannot establish these mechanisms. |
| Affected fixture desktop paths | Registration plus first workspace, immutable retry locations, provider selection and login intent/cleanup persistence using fixtures, desktop-only reconnect, Quit/restart and visible storage/cleanup failure. Keep production core/supervisor paths and private disposable state; no live authentication or agent submissions. |
| Preparation/static/build | Follow existing repository gates; report these separately from units, real-storage qualification and fixture desktop. Generated-contract preparation is conditional on changed contracts. No live-provider submission is authorized here. |

## Handoff boundary

Complete this implementation and scoped qualification before 07. Ticket 07 owns
concrete event count/byte/time grouping, fairness, provider backpressure and
accepted durable-position behavior; its explicit storage handoff gate must pass
before chat is considered qualified. Immediately committed action intent remains
distinct from grouped streamed output.

The original 16-event workload overloaded both stacks; the single 64-event
diagnostic improved results but still rejected hot-workspace output. Neither is a
qualified production default or sustained-capacity claim. Already-admitted
provider output requires backpressure or visible capacity failure with retained
durable history, never silent event dropping.

Future inbox, full GitHub review drafting/publication, shared manual/agent controls
and automation are later directions; Jira remains optional. Here they probe atomic
local updates, exact action identity, revision-aware reads and owner-local
reconciliation. Implement existing persistence and lifecycle only; conversation
execution, coordinator workflows, GitHub/Jira integration, remote deployment and
speculative cache/schema work remain outside 06b. Transcript/attachment decisions
in the research govern their first concrete use.

## Comments

2026-11-15: Proposed by the separately authorized 06a research. Acceptance is
unchecked and needs triage; no production storage change is implemented by its
publication.

2026-11-16: The user selected the bounded async architecture comparison.
[Results, source-backed maintenance inventory and qualification limits](../../../docs/research/storage-async-architecture-comparison.md)
select the maintained rusqlite bridge while preserving current opening behavior.
Both genuine async stacks accepted the baseline; neither original heavy grouping
policy accepted its full offered trace, and the single larger-group diagnostic
remains separately labeled. This ticket is now implementation-ready; acceptance
is unchecked, no production migration has occurred, and future feature ideas are
probes rather than commitments. The SQLx opening option remains an explicit future
adoption question, not a blanket safety verdict or measured performance rejection.

### 2026-11-16 — architecture review findings addressed in ticket preparation

The review found that the earlier readiness claim omitted whole-decision
serialization across awaits, an authoritative writer-settlement barrier, retained
operation outcomes, complete byte/result admission, and whole-core shutdown.
The acceptance above now makes those obligations explicit and names composition,
workspace coordination, provider persistence and connection shutdown as change
scope. The two-reader topology is a provisional internal default with contended
acquisition qualification, not an established capacity requirement.

The ticket is ready for a separately selected implementation session under these
revised requirements. Concrete types, budget values and internal scheduling choices
remain implementation work to document and qualify; acceptance stays unchecked.
This update implements no storage change, reruns no research, grants no provider
allowance, and preserves the earlier dated claims and evidence as history.

### Original review request (historical context; not acceptance)

> Review .scratch/milestone-01/issues/06b-prepare-storage-for-durable-chat.md using $ask-matt and $improve-codebase-architecture. Review only; do not start implementation.
>
> Read AGENTS.md, the ticket and its linked policies. Main context:
> - docs/research/storage-async-architecture-comparison.md
> - docs/research/sqlx-vs-rusqlite-selection.md
> - docs/evidence/storage-async-comparison/README.md
>
> Research is complete; production storage remains unchanged. The recommendation is rusqlite 0.42 + maintained tokio-rusqlite 0.9, one writer/two bounded readers, WAL/FULL/fullfsync, preserving current private-opening behavior. Challenge these choices rather than treating them as settled:
> - A unified inbox for assigned tickets, requested reviews, own PR feedback and failing CI. (will be implemented)
> - Complete GitHub review drafting and publication. (will be implemented)
> - Native Jira context and reliable status synchronization. (Maybe, maybe not)
> - Consistent manual and agent controls for the same operations. (will be implemented)
> - Automation that connects these stages and exposes failures or decisions requiring attention.(will be implemented)
> (docs/IDEAS.md)
>
> Both genuine async candidates handled the baseline similarly. Heavy workspace/chat/subagent traffic overloaded both with small transaction groups; one larger-group diagnostic improved results but did not qualify sustained capacity. Future PR/review/Jira/coordinator ideas are architectural probes, not committed features.
>
> Assess whether 06b is implementation-ready: scope, module ownership, maintained mechanisms versus custom code, locks across awaits, admission/backpressure, physical completion, recovery, shutdown, and verification. Reply-loss probes did not qualify arbitrary owner abort/panic.
>
> Return concrete findings, evidence, unresolved decisions, and recommended ticket revisions. Preserve existing work and historical evidence.
