# Supersession amendments — native core and SQLx

Effective 2026-11-16 for subsequent work. Authority: the user-selected direction
in the [plan](plan.md), reconciled into the [final spec](spec.md). These amendments
publish requirements; they do not assert that production has changed. Each affected
authority has a local pointer to this record. Earlier text/results remain readable
for provenance and apply only where compatible with the replacement below.

## A1 · Precedence and publication

The repository's frontend-spec precedence is narrowed: the SQLx architecture spec
owns native hosting, core lifetime/incarnation, storage, concurrency and the refactor
sequence, including their necessary contract/tooling changes. It supersedes the
frontend foundation on those subjects. The frontend spec retains presentation,
cache, freshness, navigation and all compatible policy. MVP/milestone/verification
owners retain their respective authority outside these explicit amendments.

This is an accepted specification, not another proposal awaiting architecture
approval. Engineering choices are settled in the spec; actual adoption/qualification
is unchecked implementation work. No live-provider/authentication allowance,
state reset, production change or completion of a parent ticket follows from publication.

## A2 · Product host and lifetime

Applies to [MVP Starting architecture](../../docs/MVP.md#starting-architecture),
[Milestone 01 scope/acceptance](../../docs/MILESTONE-01.md#scope),
the [milestone implementation spec](../milestone-01/spec.md),
and [frontend ownership](../frontend-navigation/spec.md#ipc-contracts-and-supervisor-ownership).

- Replace the separate supervisor hosting/execution/storage requirement with one
  UI-independent Rust core inside the native Tauri process. Tauri remains a thin
  adapter; Rust domains retain admission, reconciliation and persistence authority.
- Replace native desktop crash/disconnection survival with **presentation continuity**
  while the native owner lives, and **durable interrupted/uncertain recovery** after
  native failure. Distinguish WebView reload/replacement from native termination.
  Surviving child runtimes require reconciliation/fencing; no uninterrupted execution
  guarantee or automatic parent-death containment is implied.
- Replace supervisor start/discovery/reconnect/peer-version acceptance with one
  native startup, state lease exclusion, core incarnation and typed direct action/
  observation attachment. Retain lost-reply action lookup and exact owner matching.
- Explicit Quit fences/drains every domain, then storage/feed/host resources,
  retaining failures and lease until actual completion. Repeated Quit observes the
  same attempt. Hide/minimize and close-to-hide keep native-owned work running.
  Later update/restart must complete this drain before framework restart.

The old bootstrap/supervisor tickets and their checked results remain historical
cohorts, not instructions to recreate the host. Earlier MVP claims of an independent
client/supervisor seam are superseded; a future independent host needs an explicit
lifetime decision. Runtime credentials and all other MVP scope remain unchanged.

## A3 · Milestone acceptance replacements

| Existing clause | Replacement acceptance |
| --- | --- |
| Working scope and spec identity policy: one active conversation application-wide; concurrent work deferred | No single-active rule or fixed identity cap. Demonstrate two active Codex conversations sharing workspace A while a third runs in B. Maintain exact turn/action/control routing, native-conversation exclusion and visible overlapping-edit risk. Resource limits are internal admission safeguards. Claude execution remains deferred. |
| A4 chat under single-conversation load | Retain multiline streaming, saved/received/unknown delivery and usable navigation/control; qualify admitted output, durable positions and quiet-conversation progress in the concurrent scenario. |
| A8 desktop reconnects to same supervisor; supervisor failure recovers history | Quit completes whole-core cleanup/persistence before success. Reopen restores identity/history/native mapping with no automatic turn. WebView replacement observes same native incarnation without replay; native failure/reopen restores durable prefix and interrupted/unknown outcomes and reconciles/fences surviving work. Storage/cleanup failure remains visible. |
| A10 discover one supervisor, reject incompatible peer versions | Acquire one state lease before storage/effects; reject competing same-root ownership. Target current core incarnation and reject stale attachment/results. Direct contract changes regenerate declarations/validators; retired peer protocol support is removed. Preserve duplicate-ID lookup/payload mismatch, no replay, exact recovery and no PID-only signaling authority. |
| Fixture verification uses core/supervisor, desktop-only kill and supervisor kill | Use actual Tauri-hosted production core with fixtures: WebView replacement, native kill/reopen, lease exclusion, pending/failed Quit, exact status/catch-up and durable recovery. Process/storage qualification stays separate from units. |
| Navigation/history measurements under single-conversation workload; concurrent qualification excluded | Measure 100 ms navigation feedback and one-second recent-history opening on the test Mac under the agreed three-conversation trace. Record separate sustained/overload and actual control/settlement metrics. |
| Native milestone closure | Still requires separately agreed submissions/model/effort/time/usage/stop conditions, fixture desktop first, and actual concurrent routing/approvals/interrupt/cleanup/history/resume on selected topology. No old allowance is reused. |

A1/A2/A3/A5/A6/A7/A9 retain their compatible behavior, strengthened by exact concurrent
targeting. Existing attachment, fetch/setup, archive, terminal and multi-provider
feature deferrals remain. Milestone completion stays open until required evidence
passes; acceptance publication checks no implementation box.

## A4 · Frontend, observations and ADRs 0006/0007

Supersede only process/socket/protocol-specific instructions in
[frontend spec sections 1–3 and delivery/qualification](../frontend-navigation/spec.md),
[ADR 0006](../../docs/adr/0006-routed-observation-presentation.md),
[ADR 0007](../../docs/adr/0007-library-backed-observation-modules.md),
[observation guidance](../../docs/agents/observations.md),
[TanStack guidance](../../docs/agents/tanstack.md) and
[component guide](../../src/components/README.md).

Direct native handles and an app-scoped bounded Tauri-channel delivery adapter
replace private sockets, protocol-8 framing, negotiation, client counts and
supervisor discovery as *application transport*. Remove those paths on cutover.
Keep provider-specific framing, strict EOF/deadline/cancellation reasoning where
actually used; application length-prefix settings do not define Codex framing.

Replace supervisor-owner fields/context with native core incarnation throughout
commands, observations, pending interests, validators and contextual checks in one
current-contract change. WebView loss grants neither core replacement nor replay.
Keep bounded invalidation journal, subscription identity, accept-before-ack, gap/
resync, exact target/membership/revision assertions and unsettled physical registry.
Stop/join old subscription setup/delivery before replacement, above routed views.

Retain Router, Query as the sole observation cache, XState coordination, allowlisted
Zustand presentation memory, generated Rust contracts and standalone validators,
disabled automatic action retry and last-good stale observations. Add only a small
Query-owned inactive-entry count bound; the proposal's serialized-byte LRU/second
body index and hard 16 MiB JavaScript-heap claim are not selected. Current GC time
alone is not represented as a hard bound. Preserve freshness and pending identities
through eviction and qualify actual heap.

Supervisor diagnostics become core startup/storage/runtime diagnostics, distinct
from provider readiness. `SupervisorConnection` is an incumbent source name, not a
required future component. Retain accessible journey/layout and working controls;
rehost tickets qualify their changed lifecycle without redesigning the product.

## A5 · Provider connection and login

Applies explicitly to [Codex connection ownership/extension constraints](../codex-connection/spec.md#ownership-and-protocol),
its completed/current tickets, provider progress contracts and the component guide's
browser-sign-in lifecycle. Replace supervisor-owned setup and native-desktop-crash
survival with native-core ownership and A2's two failure classes. Core lifetime
owns whole-app Quit; remove whole-app BeginShutdown/FinishShutdown provider actions.
Providers keep their own stop/join mechanism and aggregate cleanup results.

Retain credential-scope exclusion, provider/configuration/attempt identity,
readiness/authentication distinction, compatibility probes, explicit browser action,
navigation continuity, late-result rejection, cancellation/cleanup fences and no
credential extraction/sign-out/replacement. Readiness compatibility stays stable
>=0.169.0 with required capability checks; source cohort is not a new exact pin.
Setup fixtures reject thread/turn/approval methods and are not chat evidence.
Requalify the host-sensitive journey with fixtures; do not erase earlier results
or infer real authentication/native execution from them.

## A6 · Storage selection and private opening

Supersede the bridge recommendation/adoption opening gate in
[storage evaluation](../../docs/research/milestone-01-storage-evaluation.md),
[driver selection](../../docs/research/sqlx-vs-rusqlite-selection.md),
[async comparison](../../docs/research/storage-async-architecture-comparison.md),
[private-opening assessment](../../docs/research/conductor-private-storage-opening.md)
and [old 06b](../milestone-01/issues/06b-prepare-storage-for-durable-chat.md).

Use direct Rust SQLx and the spec's read-only split pools, explicit failed-write
retirement/return guard and settled-read ordering. The accepted owner-private
canonical file/alias validation contract excludes same-user path replacement
races; exact SQLite NOFOLLOW preservation is no longer an adoption gate. Do not
claim equivalent protection. Fresh current-build state, WAL/FULL/fullfsync and
foreign keys remain required; busy/checkpoint/buffer counts are provisional.

The original research remains unedited evidence, including failed/unperformed
probes and comparable baseline results. No performance winner or additional
database experiment is implied. The new required qualification is the selected
production graph/engine, storage failure/recovery and host behavior.

06b is retired with `Status: wontfix` for its superseded implementation instructions;
its acceptance is not marked complete. New SQLx 01–08 replace it. Completed 06a
research is retained; it does not block repeating already completed investigation.
Milestone 07 consumes SQLx 08's *actual qualified handoff*, not this publication.

## A7 · Workspace invariants and development state

[ADRs 0003](../../docs/adr/0003-shared-local-repository-identity.md) and
[0004](../../docs/adr/0004-immediate-workspaces-with-durable-onboarding.md)
remain substantively valid. Retain shared Git identity, immutable pinned bases,
atomic registration/first-creation continuation and original checkout protection.

Supersede only retired schema migration/legacy-reader and supervisor-protocol/hash
coexistence instructions in [ADR 0005](../../docs/adr/0005-readable-workspace-reservations.md)
and the [readable-workspace plan](../milestone-01/readable-workspaces-plan.md).
Retain permanent readable reservations and every historical code/worktree location.
Allocation now uses bounded candidate continuation and atomic global-folder/per-repository
claims; physical mutations use actual repository conflicts, not a global active
workspace sentinel. A committed reservation is never silently changed on retry.

Fresh development metadata uses an explicit current-schema namespace independent of
the supervisor binary/protocol. Explicit root overrides remain exact; same-root
ownership uses the same lease inode. Inventory and stop the old owner first and
create new managed locations. Old app metadata/history/preferences are disposable
under the global policy below; preserving an old build or database/WAL/SHM set is
optional. No migration/import or old-worktree takeover is required or authorized
here; this documentation session performs no deletion or reset.

## A8 · Verification, tooling and dependency order

[Verification](../../docs/agents/verification.md) keeps substituted application
units, controlled time/events, private owning-module seams, measurement rules and
separate actual qualification. Replace retired supervisor staging prerequisites
and socket/peer qualification only when their paths are removed. Preserve target
ownership safeguards, read-only generated drift checks and provenance for any
remaining provider/bridge executable packaging. Removing the supervisor does not
remove storage/native process/descendant qualification.

The [new task graph](spec.md#dependency-ordered-implementation-tickets) replaces
plan stages A–E and old 06b execution. Publication completes stage A's documentation
work only. SQLx 02–04 have one integration-branch completion gate; no dual drivers.
06 rehosts, 07 deletes/packages, 08 qualifies the current foundation. Foundation
qualification excludes unimplemented streaming guarantees.

Milestone **07 → {08,09}; 09 → 10; {08,10} → 11 → 12 → 13** retains capability
ownership. 07 also waits for SQLx 08, workspace 04/provider-independent handoff and
qualified connection 04/parent 06. 07 owns actual event grouping/stream pressure;
08 exact approvals; 09 exact interrupt/whole-core Quit; 10 local history/exact resume;
11 native-failure/survivor recovery; 12 cumulative concurrent fixture/control pressure;
13 separately authorized native closure. Their amended current instructions replace
supervisor reconnect/single-active clauses, not their other acceptance.

## A9 · Future product direction

Retain the plan's committed incoming review/own-PR assistance/inbox/automation
direction and optional Jira. Prefer gh/gh api with CLI-owned authentication later.
Preparation opt-in never grants publication/push/CI-rerun/merge authority. Distinct
remote identity, exact revisions/draft approval, provider permission enforcement,
uncertain-write reconciliation and native-only scheduling lifetime remain design
constraints. Add no future tables, workflow engine, grants UI or integration now.

## A10 · Global MVP development compatibility policy — 2026-11-16

The [MVP development compatibility policy](../../docs/MVP.md#development-compatibility-policy)
now governs all current and future milestones, feature specs and tickets through
MVP completion. Backward compatibility with retired application builds is excluded
from design, implementation, tests and acceptance. This replaces earlier wording
limited to Milestone 01 or describing old-format support as merely optional work.

Implement only the current design and remove compatibility-only paths. Use fresh
development state instead of migrations/imports; final MVP qualification starts
from clean state without relying on retired builds/data. Earlier requirements in
the SQLx spec, A7 and tickets to keep old app metadata/builds readable or permanently
available are superseded. Optional investigation copies create no support obligation.

Current-build persistence/recovery, ownership/cleanup, selected external-provider
compatibility, repository/worktree protection, runtime credentials/native history
and retained evidence keep their requirements. State reset is documented and
deliberate, never silent or performed by this amendment. The original plan and
review/research remain unchanged historical inputs.

## Publication record

Published final spec, linked scoped amendments, architecture ADR and eight unchecked
implementation tickets; amended existing milestone consumers. Original plan/reviews,
research/evidence and checked historical ticket results are preserved. No product or
authority decision was reopened. No dependency installation, app reset, provider run,
production code/configuration change or application qualification was performed.
