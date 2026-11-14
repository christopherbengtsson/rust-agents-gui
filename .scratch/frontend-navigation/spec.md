# Frontend and observation architecture foundation

Status: ready-for-agent

Agreed 2026-11-14. Saved before implementation. Branch:
`feature/m01-frontend-navigation`; baseline `99adb6366ef7eed1cc72d310f75b6fbec361d436`.

Authority: [MVP](../../docs/MVP.md), [Milestone 01](../../docs/MILESTONE-01.md),
[verification policy](../../docs/agents/verification.md), and
[component guide](../../src/components/README.md). Those owners remain authoritative.

## 1. Purpose, scope, and publication

Build the foundation for our own Conductor-style application before starting
Milestone 01 ticket 6:

- TanStack Router for typed navigation and future deep-link destinations.
- TanStack Query for shared asynchronous observations and cache freshness.
- Zustand for retained presentation context.
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
   [ADR 0006](../../docs/adr/0006-routed-observation-presentation.md). F01's actual
   dependency/verification state is in [its evidence](../../docs/evidence/frontend-navigation/f01.md).
4. Use each ticket's source pointers as baseline footholds; follow definitions moved
   by predecessors. Read current scripts/configuration to select checks. Prove new
   tests are selected by the gate; choosing a new filename alone does not do that.
5. Finish a working increment, run relevant configured checks, and append a dated
   comment linking source identity, evidence, and remaining qualification. Check
   only verified acceptance. Keep successor tickets and foundation acceptance open
   until their own gates pass; report the next ticket without starting it.

F01 prepared dependencies and guidance only. At that handoff, selection Hooks,
native protocol, request mutex, progress polling, and inspection ownership were
still the baseline implementation. Inspect predecessor changes before treating
those baseline footholds as the current implementation. New tickets must replace their stated responsibilities
while preserving existing regression scenarios. No transcript of this chat is
required: this spec owns decisions; tickets own their completion criteria; research
owns evidence and rationale. Update those owners when implementation reveals a
necessary refinement instead of silently relying on chat context.

## 2. Ownership and module structure

| Module | Responsibility |
| --- | --- |
| Rust domain modules | Durable state, execution, admission, reconciliation, recovery |
| Rust connection module | Discovery, owner binding, bounded exchanges, transport outcomes, subscription lifecycle |
| Feature observation modules | Runtime decoding, Query definitions, target-specific freshness, observation interests |
| Router | Current destination, repository/workspace identity, active navigable view, history |
| Zustand | Lightweight remembered reading context and selected presentation preferences |
| Shared UI foundation | Controls and primitives independent of feature transport |

Keep repository, workspace, and inspection observation modules cohesive. Their
interfaces hide decoding, target validation, cache keys, and freshness rules from
screens. Preserve existing component layers. Shared pure guards import neither
Tauri, preferences, nor feature hooks. Avoid broad re-export barrels and
per-command forwarding modules that add interfaces without useful depth.

Create app-scoped Query, Zustand, connection-observation, and inspection-reader
instances above routed layouts. Their lifetimes survive navigation; individual
views subscribe narrowly.

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

Retain semantic guards if generated declarations arrive later. Do not stamp feature
results with a separately sampled global revision suggesting an atomic snapshot.

### Native request scheduling

Retain synchronous socket exchanges. Acquire native admission before spawning
blocking work.

| Lane | Maximum active |
| --- | ---: |
| Observation exchanges | 2 |
| Intent/control exchanges | 1 |
| Status/discovery | 1 |
| Dedicated subscription | 1 |

Five desktop connections fit below the current eight-client supervisor ceiling.
Use short connection-state locks; perform socket I/O outside them. Keep permits
until physical completion, including after presentation disposal. Use owned guards
for server admission cleanup.

The frontend observation coordinator allows two active reads, coalesces identical
pending targets, bounds queued targets to 64, discards obsolete queued work after
navigation/owner changes, and drains active reads while rejecting obsolete results.
Intent submissions remain explicit. Discovery stays Rust-owned; established
connection loss grants no authority to start another supervisor or replay actions.

## 3. Push observations, Query, and Zustand

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
delivery visibly instead of accumulating channels/messages. Cancellation shuts down
the socket, wakes waits, and joins owned work off rendering. Route changes preserve
the subscription. Desktop loss does not stop supervisor-owned work.

### Query freshness

Migrate repository/workspace list and detail reads, removing replaced cache/read state.
Defaults: owner/target-scoped keys; networkMode always; automatic retries, focus and
network-reconnect refetches disabled; five-minute inactive retention; bounded pages.

Hints mark affected entries dirty without immediate refetch fan-out. Refresh currently
interested targets through the coordinator. Reads capture target dirty generations;
a hint during a read remains pending and causes a follow-up. Older results cannot
discard newer invalidations. Attach and process the baseline before initial reads.

Hint-triggered repository reads use refresh=false. Explicit re-probing is separate,
avoiding a Checking/refetch feedback loop. Remove covered 250 ms progress polling
only after push equivalence passes. Retain the visible inspection's two-second
external-Git refresh. Hidden views retain bounded dirty state; pending action
interests survive navigation. Errors remain visible with last-good observations.
Query request status never substitutes for domain delivery/execution/completion.

### Zustand presentation context

Use an app-scoped store for per-workspace selected-file/comparison context, reading
position, panel layout, and selected remembered navigation preferences. Router owns
current destination/view; remembered values only provide restoration defaults.
Persist an allowlist with validation/migration and visible storage failures. Keep
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

- `/`: landing and one-time startup restoration.
- `/repository/$repositoryId`.
- `/repository/$repositoryId/workspace/$workspaceId?view=workspace|changes`.
- `/settings` redirects with replacement to `/settings/general`.
- `/settings/general`: single placeholder section.

Routes are the sole current-selection authority. Adapt Hooks to explicit targets;
remove independent selection ownership. Validate repository/workspace membership,
including records outside the current page.

Explicit destinations override remembered navigation. Ordinary startup restores the
last valid main destination/view. Migrate old selection preferences without touching
pending-action records. Confirmed stale startup targets fall back home; temporary
connection failures retain navigation and show failure. Invalid explicit targets show
unavailable/retry guidance. Capture navigation generations for action completion;
Settings, Back/Forward, and A→B→A navigation invalidate redirect eligibility.

Keep the physical inspection reader above both layouts. Settings pauses inspection
reads while outstanding IPC drains.

### Settings

The footer gear opens full-page Settings directly. It replaces the workspace layout
and has its own navigation/main landmark. General has honest placeholder content,
without fake controls or account scope. Back/Escape returns to the preceding app
destination and restores context; nested overlays handle Escape first. Direct entry
falls back to the remembered valid main destination, then home. Preserve keyboard,
focus, minimum-window usability, and existing inspection presentation.

[Milestone 01 ticket 06 — Connect Codex with actionable setup diagnostics](../milestone-01/issues/06-connect-codex-with-actionable-setup-diagnostics.md)
adds Settings → Providers → Codex; workspace status links there. Keep
supervisor transport diagnostics separate.

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

| Ticket | Deliverable | Blocked by |
| --- | --- | --- |
| F01 | Publish full plan, research, architecture decision, baseline, dependencies and Intent | Completed 05 |
| F02 | Owner-bound contracts, snapshot validation, transport outcomes, sticky uncertainty, corpus | F01 |
| F03 | Bounded native/frontend scheduling, short locks, physical completion/cleanup | F02 |
| F04 | Push publication, journal, subscription, acknowledgements, resync, bridge | F03 |
| F05 | Query migration, freshness, covered polling removal, reader lifetime, Zustand, feature imports | F04 |
| F06 | Typed file routes, generation, restoration, membership/history/generation checks | F05 |
| F07 | Footer Settings, General, focus/return/context, ticket-6 seam | F06 |
| F08 | Regression, desktop/process/accessibility/packaging/performance/size evidence and fixes | F07 |

All eight precede existing Milestone 01 ticket 6. Update its blockers and settings
requirement; correct stale milestone summaries without rewriting completed evidence.

### Storage research checkpoint

Add a separate ticket after 6 and before 7: evaluate rusqlite/SQLx inside Rust-owned
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
cleanup, route history/preference migration/membership/late navigation, context
restoration, and clean-checkout generated-route preparation.

Application units substitute dependencies. Actual sockets, Tauri delivery, process
cleanup and desktop behavior are separate private-fixture qualification. Compare
identical builds: initial/lazy/total raw/gzip/Brotli JS, packaged size, startup and
navigation against 100 ms, IPC/duplicate/outstanding/hidden-view activity, cache and
queue retention/memory after navigation/expiry. Follow repository measurement policy;
record source/versions/commands/timings/failures/unperformed checks.

Qualify actual progress without 250 ms polling, desktop disconnect/reconnect,
overflow, stalled delivery, owner replacement, nested-route reload, and Settings
accessibility. No live-provider submissions. Full owned Quit, streamed-chat/history
performance, and native-provider qualification remain their milestone tickets.

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
