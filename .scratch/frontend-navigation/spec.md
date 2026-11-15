# Frontend and observation architecture foundation

Status: ready-for-agent

Agreed 2026-11-14; amended by the F04a documentation publication below. Branch:
`feature/m01-frontend-navigation`; baseline `99adb6366ef7eed1cc72d310f75b6fbec361d436`.

Authority follows the [repository precedence rule](../../AGENTS.md#instruction-precedence).
Read the [MVP](../../docs/MVP.md), [Milestone 01](../../docs/MILESTONE-01.md),
[verification policy](../../docs/agents/verification.md), and
[component guide](../../src/components/README.md) for their respective policy and context.

### Development compatibility amendment — 2026-11-15

For subsequent Milestone 01 work, this spec adopts the milestone's
[development compatibility policy](../../docs/MILESTONE-01.md#development-compatibility-policy).
Earlier application storage, IPC and preference formats need no backward support,
migration or compatibility-only tests. This supersedes such requirements in the
foundation plan and its completed tickets for future changes; their recorded
F05/F06/F08 migration results remain historical evidence. Current-format validation,
persistence/recovery, owner exclusion, physical cleanup and external provider
protocol qualification remain required.

## 1. Purpose, scope, and publication

Build the foundation for our own Conductor-style application before starting
Milestone 01 ticket 6:

- TanStack Router for typed navigation and future deep-link destinations.
- TanStack Query for shared asynchronous observations and cache freshness.
- Zustand for retained presentation context.
- XState for app-lifetime feed and inspection coordination.
- Generated Rust-owned contracts and structural validators.
- Reliable, owner-bound IPC contracts.
- Bounded native request scheduling.
- Push-driven repository/workspace progress with snapshot recovery.
- Full-page Settings opened through a sidebar-footer gear.

Preserve the MVP process architecture: React presentation → Tauri desktop bridge →
separate supervisor process hosting the Rust core → official runtimes. The Rust
core is a library inside the supervisor, not a process before it. Tauri forwards
targeted requests and presents observations. Rust remains authoritative for persistence, execution, admission, reconciliation, and recovery.

The first implementation step is saving this complete plan here, then publishing
numbered implementation tickets under `issues/` before changing code.

Record Conductor research separately, including installed version 0.93.0, asset
hashes, relevant excerpts, and evidence limitations. Shipped code establishes
runtime behavior and dependencies; unavailable source maps prevent conclusions
about its original source organization.

Use one feature branch. Each ticket produces a working increment, relevant passing
checks, and a dated evidence comment. Preserve completed ticket-5 evidence.

## Starting a fresh session

1. Implement only the ticket selected by the user. The ticket order is a dependency
   chain, not authorization to continue into successors. Use each ticket's checked
   acceptance and dated evidence as its current completion record; confirm the
   predecessor before product edits.
2. Continue `feature/m01-frontend-navigation` from its current tip. Inspect the
   working tree and index; preserve existing edits/staging. The recorded baseline
   is historical evidence, not an instruction to reset the branch.
3. Read the selected ticket and its linked plan sections. Read the authority
   documents above and applicable repository-local engineering skills. For the
   rationale or a Conductor comparison, consult
   [research](../../docs/research/conductor-frontend-architecture.md) and
   [ADR 0006](../../docs/adr/0006-routed-observation-presentation.md). Current refactor
   rationale is in
   [library research](../../docs/research/frontend-navigation-library-refactor.md)
   and [ADR 0007](../../docs/adr/0007-library-backed-observation-modules.md).
   [F04a evidence](../../docs/evidence/frontend-navigation/f04a.md) records this
   documentation-only handoff. Read the selected predecessor's evidence for actual
   implementation state; a dependency decision does not mean it is installed.
4. Use each ticket's source pointers as baseline footholds; follow definitions moved
   by predecessors. Read current scripts/configuration to select checks. Prove new
   tests are selected by the gate; choosing a new filename alone does not do that.
5. Finish a working increment, run relevant configured checks, and append a dated
   comment linking source identity, evidence, and remaining qualification. Check
   only verified acceptance. Keep successor tickets and foundation acceptance open
   until their own gates pass; report the next ticket without starting it.

Current product checkpoint: F01–F04 at `7dc4eac` remain the historical protocol-7
baseline. F04b–F04f implemented generated contracts, protocol-8 async transport,
physical scheduling, Query observations and XState coordination. F04g records the
focused actual socket/process/fixture-desktop, packaging and matched-baseline
qualification in [its evidence](../../docs/evidence/frontend-navigation/f04g.md).
Historical F04 evidence retains its original controlled-only limits; the new
baseline measurements belong to F04g. [F05 evidence](../../docs/evidence/frontend-navigation/f05.md)
now records retained presentation persistence/integration, passing deterministic
checks, production packaging and focused private-state desktop restoration.
[F06 evidence](../../docs/evidence/frontend-navigation/f06.md) records route authority,
restoration, completion eligibility and generated-route preparation, with passing
controlled gates, packaging and focused private-state desktop smoke.
[F07 evidence](../../docs/evidence/frontend-navigation/f07.md) records routed General
Settings, history/focus return and retained-context checks. The F07 user decision
below simplifies absent return context to home. [F08 evidence](../../docs/evidence/frontend-navigation/f08.md) records cumulative qualification and scoped fixes.
Material costs are returned for decision; F09 and milestone successors remain separate and unimplemented.

F04a changes documentation/tickets only: manifests, lockfiles, generated assets,
product implementation and implementation acceptance stay untouched. Add each
approved dependency in the implementation ticket that first uses it. Implement
only the ticket selected by the user; the dependency chain grants no authorization
to continue. Each ticket must leave a working increment, identify superseded code,
and replace useful coverage before removing it. The spec owns requirements, ADRs
own rationale, research owns sources/limits, and tickets own completion evidence.
A fresh session must not depend on this chat.

## 2. Ownership and module structure

| Module                   | Responsibility                                                                             |
| ------------------------ | ------------------------------------------------------------------------------------------ |
| Rust domain modules      | Durable state, execution, admission, reconciliation, recovery                              |
| Rust connection          | Discovery, owner binding, physical admission, delivery evidence, subscription ownership    |
| Internal framed exchange | Shared bounded framing, persistent buffers, deadlines, cancellation mechanics              |
| Physical reads           | One app-scoped queue, pending admission, unsettled physical identity, coalescing, draining |
| Feature observations     | Validated Query definitions, exact targets, interests, dirty generations, freshness        |
| Feed acceptance          | Subscription lifecycle, accepted cursor, exact acknowledgements, stalls and replacement    |
| Inspection coordination  | Summary/preview sequencing, visibility, refresh priority, comparison context and draining  |
| Router                   | Current destination, repository/workspace identity, active navigable view, history         |
| Zustand                  | Lightweight reading context and allowed presentation preferences                           |
| Shared UI foundation     | Controls and primitives independent of feature transport                                   |

Share mechanisms once; keep feature decisions local. Repository, workspace and
inspection observation modules hide keys, decoding, semantic assertions, freshness
and interests behind feature-specific interfaces. Screens supply domain identity
and visibility and receive observations/targeted callbacks; they do not configure
queues, drive actor states or acknowledge hints. Avoid a public observation
framework, broad re-export barrels and per-command forwarding modules. A deep
module earns its interface when removing it would redistribute decisions among
callers. An existing useful module can keep its interface while libraries replace
its implementation; depth is not a line-count ratio.

Create one app-scoped Query client, physical-read module, presentation store,
connection observation owner and feed/inspection actors above routed layouts.
Composition owns construction and dependency adapters, not feature policy. Views
subscribe narrowly and release interests; they never terminate app-lifetime actors.
Query owns observation bodies/last-good data; XState owns coordination and IDs;
Zustand owns presentation memory. Actor snapshots are not mirrored into Zustand,
and actor context holds no duplicate summary/preview cache. Owner-bound intent
callbacks capture the observation's owner/target rather than silently adopting a
new global owner at submission.

Keep controlled I/O/time and application dependency seams inside owning modules.
The repository verification policy permits private interfaces as test surfaces and
requires substituted storage/Git/process outcomes even with one production adapter.
Pure decisions need no invented adapter. Preserve useful tests until replacement
assertions pass and record the old-to-new coverage mapping.

### Approved libraries and generated contracts

Use stable maintained libraries; record release, adoption, maintenance and fit
sources in [research](../../docs/research/frontend-navigation-library-refactor.md).
The agreed versions are ts-rs 12.0.1, Schemars 1.2.3, Ajv 8.20.0, ajv-formats
3.0.1, Tokio 1.54.1, tokio-util 0.7.20, p-queue 10.0.0, XState 5.33.4 and
@xstate/react 6.1.0. Retain existing Query 5.107.1 and Zustand 5.1.0. Land manifest
and lockfile changes with first use; recheck compatibility on any necessary version
change and record the reason. Maintenance/robustness and deletion of custom
coordination can justify adoption without a measured performance bottleneck.

Rust Serde wire types remain authoritative. F04b adds an optional application-owned
`codegen` feature and explicit exporter, not generated file-writing tests. Produce
committed declarations, schemas and standalone ESM validators under
`src/generated/contracts/`. Export all frontend wire roots and their dependencies,
not native resource internals. Keep private types private through owning-module
export helpers. Use deterministic explicit configuration, including safe JSON
large-integer representation and supported Serde/UUID handling.

Reply/observation schemas use Schemars draft 2020-12 serialization settings;
Ajv2020 compiles them ahead of time. Register UUID formats with ajv-formats.
Request-deserialization schemas, if exported, are separately named; defaults do
not make serialized response fields optional. Retain current CSP, generated runtime
helpers as needed, and visible unsupported-Serde diagnostics. Check both generators
against actual Rust serialization and the shared synthetic application corpus.

Generated structural checks precede named semantic assertions: owner, exact target,
request/reply kind, cursor/watermark/batch relationships, state combinations, safe
integers, UTF-8 byte budgets and normalized-path/custom-deserializer rules. Preserve
variant-specific exclusions without imposing blanket closed-object rules that
break flattened Serde shapes. Preference format changes follow the development
compatibility amendment above; earlier formats need no migration contract.

One preparation owner provides explicit write and read-only drift modes, shared by
dev/check/build/Tauri/CI; check drift before any generation can repair it. F06 adds
route generation to that owner. Generation is tooling, separate from application
units, typechecking, bundling and packaging. Preserve Cargo target ownership and
sidecar provenance. Frontend libraries remain implementation details of the owning
module; follow [observation engineering guidance](../../docs/agents/observations.md).

### IPC contracts and supervisor ownership

- Carry caller-observed expected owner through Tauri and targeted native requests.
- Reject owner mismatches before domain dispatch or effects.
- Return serving owner with feature results.
- Validate owner, reply kind, exact target, state combinations, and payload bounds before caching.
- Add runtime validation for supervisor snapshots and desktop identity.
- Distinguish Busy, NotConnected, OwnerChanged, and ReplyUnavailable transport outcomes from domain rejection.
- A later attempt rejected before sending cannot erase uncertainty from an earlier transmission of the same action.
- Keep expected-owner context separate from durable action payloads, retaining identity for lookup after reconnect.
- Bump the local protocol and update compatibility evidence.
- Check a small shared contract corpus through pure Rust serializer and TypeScript decoder tests.

Keep semantic assertions after generated structural validation. Do not stamp feature
results with a separately sampled global revision suggesting an atomic snapshot.

### Native request scheduling

F04c–F04d replace synchronous exchanges with Tokio. Semaphore acquisition is
nonwaiting and precedes task creation or blocking submission; owned permits stay
with physical work. The supervisor runs a continuously driven current-thread
runtime with I/O/time enabled; Tauri uses its existing runtime. The core library
constructs no runtime. Bound synchronous filesystem/Git/SQLite/domain dispatch
through blocking execution; async sockets do not make those operations nonblocking.

| Lane                     | Maximum active |
| ------------------------ | -------------: |
| Observation exchanges    |              2 |
| Intent/control exchanges |              1 |
| Status/discovery         |              1 |
| Dedicated subscription   |              1 |

Five desktop connections fit below the current eight-client supervisor ceiling.
Use short state locks; perform socket I/O outside them. Serialize resource cleanup
inside its owning module and observe task completion. Logical waiter cancellation
cannot release physical admission. Server handlers have owned task completion,
including on cancellation; started blocking work retains its own permit until it
finishes. Discovery remains Rust-owned, exclusively admitted and effect-substitutable.

The shared frontend physical-read module uses p-queue with two active reads and
at most 64 distinct queued targets. It coalesces complete physical request identity,
including owner, operation, cursor and effectful refresh/load flags. An unsettled
promise registry keeps active work discoverable after logical cancellation; it is
not an observation cache. Cancel obsolete queued interests only, detaching their
abort forwarding once execution starts. Running abort/queue timeouts cannot release
slots while IPC continues. Preserve rejection of the newest distinct read when all
64 queued targets remain relevant; equivalent interests still attach.
Intent submissions remain explicit. Discovery stays Rust-owned; established
connection loss grants no authority to start another supervisor or replay actions.

### Shared framed exchange

F04c introduces one internal framed-exchange module; F04d switches request and
subscription callers together to protocol 8. Tokio-util's length-delimited codec
uses a four-byte big-endian u32 payload length, default offset/adjustment/skip, and
a 65,532-byte payload maximum: prefix plus payload stays within 65,536 bytes.
The codec owns buffering and ordinary length validation. A small adapter preserves
strict EOF (including complete header with no body), first-byte observation and
absolute deadlines; it does not implement a second frame buffer.

Retain the two-second transfer budget, capped by the operation/ack deadline after
first data. Buffered data starts the transfer phase immediately. Idle feed waiting
permits the heartbeat budget; a timeout per fragment must not extend the absolute
budget. Outbound partial writes have one absolute budget too. Timeout/cancellation
closes the affected connection and awaits owned cleanup; it never restarts a
partially transmitted action. Connection policy classifies delivery evidence;
framing knows neither domain reconciliation nor Query targets.

Keep persistent framing state across handshake, frames and cancelled polling.
Reject incompatible framing/version visibly before domain effects, preserve
application-version matching and retain v6/v7 corpus evidence. Protocol-8 development
state/provenance follows the existing version isolation; communication failure
cannot justify replacing an existing supervisor or falling back/replaying actions.

## 3. Push observations, Query, Zustand and XState

### Rust invalidation journal

Implement a concrete in-memory journal:

- 256 retained entries, each containing a cursor and at most eight typed identity tags.
- Tags cover repository collections/details, registration actions, workspace collections/details, and workspace inspection.
- At most 32 tags and 4 KiB per delivery.
- Overflow produces ResyncRequired at a sampled watermark.

Carry identities and invalidation hints, not authoritative records, paths,
transcripts, or provider tokens. Audit publication for registration admission and
completion, ambiguous persistence outcomes, availability checking/results,
workspace create/reconcile admission/stages/results, visible fault overlays, and
inspection queued/running/completed/failed/preview changes. Existing revision sites
do not cover every visible transition.

Publish under owning-state protection with a consistent lock order. Release locks
before notification/delivery. Subscribers never call domain modules while holding
the journal lock.

### Subscription and recovery

Use one acknowledged native subscription above routes and Settings. Wait for
changes instead of repeatedly reading records. Permit one delivery in flight.
Retain the 64 KiB frame ceiling and two-second frame-transfer budget; use five-second
heartbeats and a 15-second acknowledgement deadline. Keep persistent framing
buffers. Close malformed, timed-out, or failed subscriptions visibly.

The Tauri bridge also permits one frontend batch in flight. The frontend validates,
records dirty targets, and acknowledges before refetches settle. Flow control supplies
bounds; Tauri channels supply ordering.

Keep status and frontend-accepted feed cursors separate. Reconnect from the accepted
feed cursor. Missing, expired, future, or incompatible cursors require bounded
resync of active observations and retained pending actions. A stalled frontend stops
delivery visibly instead of accumulating channels/messages. Rust owns subscription
task cancellation and awaited cleanup. The frontend feed actor transitions through
idle/starting/live/stopping/stopped, retaining visible failures. Replacement waits
for outstanding setup and native stop/join, including late registration; uncertain
cleanup fences replacement. Stopping an actor alone is not physical cleanup.
Route changes preserve the subscription. Desktop loss does not stop supervisor-owned work.

### Query freshness

F04e completes repository/workspace observation migration, freshness and covered
polling replacement together; F04f does the same for inspection. Feature modules
own hint mapping, dirty generations and visible/retained-action interests. Feed
acceptance synchronously records invalidations there before advancing the accepted
cursor and acknowledging; it never owns a second feature freshness ledger. A small
composition sink fans typed hints to concrete feature modules, not a public plugin
registry. The existing feed controller can receive that sink in F04e until F04f
replaces its lifecycle implementation. Query is the sole observation cache.
Defaults: owner/target-scoped keys; networkMode always; automatic retries, focus and
network-reconnect refetches disabled; five-minute inactive retention; bounded pages.

Hints mark affected entries dirty without immediate refetch fan-out. Refresh currently
interested targets through the coordinator. Reads capture target dirty generations;
a hint during a read remains pending and causes a follow-up. Older results cannot
discard newer invalidations. Attach and process the baseline before initial reads.

Hint-triggered repository reads use refresh=false. Explicit re-probing is separate,
avoiding a Checking/refetch feedback loop. Remove each feature's covered 250 ms
progress polling in its F04e/F04f slice only after controlled push equivalence
passes. Retain the visible inspection's two-second external-Git refresh. Hidden views retain bounded dirty state; pending action
interests survive navigation. Bound retained dirty identities to 256 across feature
modules; overflow becomes a sampled global resync generation. Reads satisfy only
captured generations, including the global generation, so one refreshed interest
cannot clear other hidden targets' obligations. Owner replacement resets acceptance
and current-owner interests while preserving retained action identity/uncertainty.
Errors remain visible with last-good observations.
Query request status never substitutes for domain delivery/execution/completion.

### Inspection coordination

F04f replaces the existing useful reader implementation with one app-lifetime
XState actor, using feature Query observations and the shared physical-read module.
Keep summary-before-preview, exact comparison/file identities, paging and queued
explicit-refresh priority inside the inspection module. Actor context retains
coordination IDs/handles, not duplicated observation bodies. Target/visibility
changes drain the current physical sequence, then start only the latest eligible
interest. Preserve error pause, last-good data and the visible two-second external
Git refresh; hidden views and Settings pause reads without freeing active capacity.
View cleanup releases interest instead of stopping the actor. App-owned feed/reader
cleanup is distinct from the later full owned-Quit/provider milestone requirement.

### Zustand presentation context

F04e establishes the store; F05 completes allowlisted persistence and integration.
Use an app-scoped store for per-workspace selected-file/comparison context, reading
position, panel layout, and selected remembered navigation preferences. Router owns
current destination/view; remembered values only provide restoration defaults.
Persist an allowlist with current-format validation and visible storage failures. Keep
heavy previews, transcripts, credentials, pending execution authority, and
authoritative snapshots out. Retain context across workspace/settings switches;
revalidate comparison-specific context before restoration.

[Milestone 01 ticket 07 — Durable streamed chat and uncertain-send recovery](../milestone-01/issues/07-durable-streamed-chat-and-uncertain-send-recovery.md)
may extend subscription lifetime/framing for conversation observations.
Durable conversation events need their own cursor, retention, paging, and delivery
semantics; they do not belong in this invalidation journal.

## 4. Routes, Settings, and development tooling

### Route structure

Use file-based routes and browser history; memory history in deterministic tests.
Missing view defaults to Workspace. Invalid explicit view values preserve the URL
and show an error with valid-view links (user selection during F06).

- `/`: landing and one-time startup restoration.
- `/repository/$repositoryId`.
- `/repository/$repositoryId/workspace/$workspaceId?view=workspace|changes`.
- `/settings` redirects with replacement to `/settings/general`.
- `/settings/general`: single placeholder section.

Routes are the sole current-selection authority. Adapt Hooks to explicit targets;
remove independent selection ownership. Validate repository/workspace membership,
including records outside the current page.

Explicit destinations override remembered navigation. Ordinary startup restores the
last valid main destination/view. Restore current-format selection preferences
independently of pending-action records. Confirmed stale startup targets fall back home; temporary
connection failures retain navigation and show failure. Invalid explicit targets show
unavailable/retry guidance. Capture navigation generations for action completion;
Settings, Back/Forward, and A→B→A navigation invalidate redirect eligibility.

Keep the physical inspection reader above both layouts. Settings pauses inspection
reads while outstanding IPC drains.

### Settings

The footer gear opens full-page Settings directly. It replaces the workspace layout
and has its own navigation/main landmark. General has honest placeholder content,
without fake controls or account scope. Back/Escape returns to the preceding app
destination and restores context; nested overlays handle Escape first. Without a
usable preceding app destination, replace Settings with home. This follows the
[F07 user decision](issues/07-routed-settings.md#2026-11-14--return-behavior-agreed-during-f07-planning),
superseding the original remembered-destination fallback. Preserve keyboard,
focus, minimum-window usability, and existing inspection presentation.

[Milestone 01 ticket 06 — Connect Codex with actionable setup diagnostics](../milestone-01/issues/06-connect-codex-with-actionable-setup-diagnostics.md)
is the parent/index for the [connection feature's separate sessions](../codex-connection/spec.md#session-sequence).
The [shared connection journey](../codex-connection/spec.md#connection-journey)
owns Providers/Codex/disabled-Claude presentation, browser/CLI authentication and
progress return. Keep supervisor transport diagnostics separate.
The [provider-independent workspace prerequisite](../provider-independent-workspaces/spec.md)
now precedes connection 01. The connection feature's
[architecture constraints](../codex-connection/spec.md#architecture-and-extension-constraints)
extend the existing observation composition with one provider feature module;
they add no replacement cache, scheduler or frontend execution owner. Foundation
implementation and qualification evidence remain historical results.

Connection 01 extends Settings return provenance across General, Providers and tab
changes: Back/Escape still returns to the preceding app destination, with the
existing home fallback, workspace reading context and focus restoration. Use the
correct content heading for each main landmark; nested overlays and terminal
keyboard handling take precedence over page Escape. Route/view changes release
presentation interests without cancelling supervisor-owned setup. A compact
sign-in-progress status links back to Providers; cancellation and closing a running
login terminal are explicitly named actions requesting owned cleanup. Completion
does not navigate away automatically. Reopening or preloading a route retrieves
observations, never replays sign-in.

The [2026-11-15 session split](../milestone-01/issues/06-connect-codex-with-actionable-setup-diagnostics.md#2026-11-15--split-into-separate-connection-sessions)
preserves F07/F08 implementation evidence. Connection 01–03 each implement and
qualify their focused slice; connection 04 qualifies the combined journey. The
embedded official-login terminal and runtime-owned authentication remain successor
work, not implemented foundation features. API-key setup and general development
terminals remain deferred.

### Route preparation and Intent

Commit the generated tree; route files remain authoritative. One preparation owner
serves dev/check/build/Tauri/CI. Check drift before a build can repair it. Keep
checking/bundling/unit/packaging distinct, exclude generated code from manual
formatting, preserve sidecar provenance and target ownership.

Pin development-only Intent tooling and review Router and Query skills, including
applicable integration guidance discovered from installed packages. Map routing/cache
work to relevant guidance, preserve repository precedence, and review guidance on
dependency updates.

## 5. Delivery sequence and qualification

| Ticket                                                  | Deliverable                                                                                                                    | Blocked by             |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ---------------------- |
| F01–F04                                                 | Completed documentation, owner-bound contracts, admission and protocol-7 push implementation; retain original evidence         | Completed milestone 05 |
| [F04a](issues/04a-publish-library-refactor-plan.md)     | This documentation-only publication, ADR, research, guidance and session handoffs                                              | F04                    |
| [F04b](issues/04b-generate-contracts-and-validators.md) | Rust-owned declarations/schemas/validators and deterministic preparation/drift tooling                                         | F04a                   |
| [F04c](issues/04c-admission-and-framed-exchange.md)     | Semaphore and one controlled/tested internal async framed-exchange module                                                      | F04b                   |
| [F04d](issues/04d-migrate-native-transport.md)          | Protocol-8 async client/server/feed, runtime/task ownership, removal of synchronous mechanics                                  | F04c                   |
| [F04e](issues/04e-deepen-feature-observations.md)       | Physical-read module and complete repository/workspace Query, freshness and polling replacement; presentation store foundation | F04d                   |
| [F04f](issues/04f-actor-feed-and-inspection.md)         | App-lifetime feed/inspection actors, inspection Query/freshness/polling replacement, removal of old controllers                | F04e                   |
| [F04g](issues/04g-qualify-library-refactor.md)          | Focused actual socket/process/fixture-desktop and packaging qualification                                                      | F04f                   |
| [F05](issues/05-feature-observations-and-ui-context.md) | Retained presentation context, persistence/migration and final observation integration                                         | F04g                   |
| F06                                                     | Typed file routes, shared generation owner, restoration/membership/history checks                                              | F05                    |
| F07                                                     | Footer Settings, General, focus/return/context, ticket-6 seam                                                                  | F06                    |
| F08                                                     | Cumulative routed desktop/accessibility/packaging/performance/size evidence and fixes                                          | F07                    |

All foundation implementation and F08 qualification precede connection 01 under
Milestone 01 parent 06. The [connection sequence](../codex-connection/spec.md#session-sequence)
continues through 04 before storage research. Preserve completed foundation evidence.

### Storage research checkpoint

[Existing 06a](../milestone-01/issues/06a-evaluate-storage-before-durable-chat.md)
follows connection 04 and parent 06's aggregate before 07: evaluate rusqlite/SQLx inside Rust-owned
persistence; WAL/rollback configuration; connection scheduling; transcript batching
and history paging; synchronization/checkpoint/recovery/backup; atomicity across
records and transcript/attachment files; future disposable disk-cache separation;
dependency/binary/memory/runtime costs. Use supported-version sources and disposable
representative workloads, keeping real storage measurements separate from units.
Rust remains authoritative. Research recommends changes without automatically
implementing a driver, journal, or layout migration.

### External deep-link follow-up

Create a non-blocking ticket for macOS scheme registration, cold/warm delivery,
window activation, URL validation/route translation, missing targets/startup races,
and observation-only navigation. Scheme and external contract are future decisions.

### Verification and performance

Preserve existing scenarios. Add deterministic coverage for owner/target mismatch,
replacement before next UI snapshot, uncertainty after pre-send rejection, admission
and physical draining, discovery/no-replay, publication completeness, coalescing,
overflow/gaps/resync, changes during baseline/reads, acknowledgement/deadline/stall/
cleanup, route history/current-format preferences/membership/late navigation, context
restoration, and clean-checkout generated-route preparation.

Application units substitute dependencies. Actual sockets, Tauri delivery, process
cleanup and desktop behavior are separate private-fixture qualification. Compare
identical builds: initial/lazy/total raw/gzip/Brotli JS, packaged size, startup and
navigation against 100 ms, IPC/duplicate/outstanding/hidden-view activity, cache and
queue retention/memory after navigation/expiry. Follow repository measurement policy;
record source/versions/commands/timings/failures/unperformed checks.

F04g qualifies actual progress without covered polling, socket/frame bounds,
subscription/handler cleanup, stalled delivery, disconnect/reconnect, owner replacement,
private access, packaging and native/frontend physical limits before F05 resumes.
Use disposable repositories/private state; this allowance contains no provider work.
F08 retains cumulative qualification after route/Settings/context changes, including
nested-route reload and Settings accessibility; F04g does not complete it.
No live-provider submissions. Full owned Quit, streamed-chat/history performance,
and native-provider qualification remain their milestone tickets.

## Comments

### 2026-11-14 — implementation started

Saved before product changes. Existing unrelated `docs/IDEAS.md` modification and
`docs/research/open-source-acquisitions.md` are preserved. Acceptance is not claimed.

### 2026-11-14 — session scope corrected

The user clarified that this session implements F01 only. Initial Rust observation
edits were removed before F01 closure; product source remains at the baseline.
F02–F08 are published and unimplemented. Stop after F01, retaining later acceptance
items unchecked. Installed Query has no standalone Intent skill; reviewed Router/Query
composition and official Query guidance are configured explicitly.

### 2026-11-14 — F01 complete

[Evidence](../../docs/evidence/frontend-navigation/f01.md) records published research/ADR,
pinned preparation, reviewed Intent guidance, passing deterministic/static checks
and unchanged assets. F02–F08 and whole-foundation acceptance remain open.

### 2026-11-14 — F04a library refactor publication

The user approved refactoring completed mechanisms for robustness, locality and
reduced maintenance, including private protocols. Publish F04a only in this session;
implement F04b–F04g separately. Feature-specific interfaces share internal mechanisms
without duplicating cache/lifecycle authority. Complete each feature's freshness
and polling replacement in F04e/F04f; F05 now finishes presentation context and
integration. [ADR 0007](../../docs/adr/0007-library-backed-observation-modules.md),
[research](../../docs/research/frontend-navigation-library-refactor.md) and
[F04a evidence](../../docs/evidence/frontend-navigation/f04a.md) record the decision.
F01–F04 results remain historical; no new product or native qualification is claimed.

### 2026-11-14 — F05 complete

[F05 evidence](../../docs/evidence/frontend-navigation/f05.md) records validated,
allowlisted Zustand persistence/migration, visible storage errors, comparison-bound
reading context, retained panel/view defaults and independent pending identities.
F04e/F04f/F04g fulfill the original observation requirements; their evidence remains
linked and unchanged. Final verification passed 158 Rust + 40 tooling + 174 frontend
units; production packaging, execution measurements and focused packaged desktop
restoration passed. Review ended with Standards 0 / Spec 0 outstanding. Only F05
was implemented; F06 routing/migration, F07 Settings and F08 cumulative qualification
remain separate. No provider work or whole-foundation completion is claimed.

### 2026-11-14 — cumulative F08 fixture qualification

[F08 evidence](../../docs/evidence/frontend-navigation/f08.md) cross-checks the
foundation requirements and preserves controlled, native and routed desktop results
separately, including all failures and unperformed checks. Sidecar publication,
WebView reload subscription replacement and Settings hidden interests are fixed.
Current configured gates, separate production packaging, actual fixture recovery,
physical bounds, focus/return, off-page routes and real cache expiry pass. Material
bundle/fast-workload costs are measured and explicitly returned for decision.
Milestone 01 acceptance remains open; no provider allowance or successor work is
created by this checkpoint.
