# 04f: Deepen feed and inspection coordination with actors

Status: ready-for-agent

Blocked by: [F04e](04e-deepen-feature-observations.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read the
[owning requirements](../spec.md#inspection-coordination) and predecessor evidence. Read the MVP,
milestone, verification policy and applicable repository engineering guidance.
[Observation guidance](../../../docs/agents/observations.md) owns shared engineering
pitfalls; the parent spec owns behavior and limits. This ticket is one session;
successors remain separate. Preserve unrelated edits and completed evidence.

## Module and caller interface

Use XState 5.33.4 and @xstate/react 6.1.0 inside two app-lifetime modules, constructed
once above layouts. Views subscribe narrowly through feature interfaces and send
interest/visibility/selection/refresh changes. They never drive ack/queue bookkeeping
or stop app-lifetime actors. Preserve existing useful module depth rather than adding
another manager that forwards to an actor.

## Implementation and removal

- Feed actor owns idle/starting/live/stopping/stopped coordination, exact batch
  validation, accepted cursor, ack ordering and watchdog. Its synchronous acceptance
  sink records feature invalidations before cursor advancement/ack; feature modules
  own target generations and freshness. Refetch completion never delays ack.
- Owner/configuration changes record latest desired context while stopping. Await
  outstanding setup and exact native stop/join, including late registration, before
  replacement. Failed cleanup stays visible and fenced until native reconciliation.
  Actor disposal/promise cancellation alone is not native cleanup.
- Inspection actor owns one summary→preview sequence, desired target, visibility,
  queued explicit-refresh priority and drain states. Hold physical work in its owner
  until settlement; target changes drain before starting the latest eligible target.
- Complete inspection's Query definitions, validation, interests, dirty generations,
  baseline and push freshness in the same module migration. Remove covered inspection
  pending/progress polls after parity passes; retain visible two-second external-Git
  refresh, explicit refresh, error pause and last-good comparison/preview handling.
- Query owns observation bodies; actor context retains coordination IDs/handles.
  Zustand owns selected reading context; do not mirror actors into it. Keep paging,
  opaque file IDs and comparison revalidation inside feature coordination.
- Replace old serial promise, manual watchdog/epoch/reading/queued machinery and
  duplicated publication/cache state once new module scenarios pass. Keep a typed
  transition table beside each machine; do not leave two lifecycle authorities.

Footholds: `lib/observation-feed.ts`, `hooks/use-observation-feed.ts`,
`lib/workspace-changes-reader.ts`, `hooks/use-workspace-changes.ts`, current
inspection blocks and F04e's app composition, feature modules and physical gate.

## Required scenarios

- Dirty-before-ack, accepted-vs-status cursor, rapid owner changes, next batch before
  ack promise settles, stalled delivery, late setup, lost stop reply and setup failure.
- During stopping, new configuration changes desired context without starting a
  competitor; cleanup failure remains fenced and recoverable by observed status.
- Summary/preview target change, A→B→A, hide/unmount during I/O, explicit-refresh
  priority, paging/comparison change, stale preview errors and restored file context.
- Hidden/Settings-style interest release starts no reads while actual work drains.
  Actors/Query use controlled request/time adapters; retain relevant presentation tests.

## Acceptance

- [x] XState replaces both lifecycle implementations with one app-lifetime owner each and narrow feature interfaces.
- [x] Inspection Query/freshness replacement passes and covered progress polling is removed; external-Git cadence remains.
- [x] Old controller machinery/duplicate caches are removed with passing mapped replacement scenarios.
- [x] Static/unit gates, actor selection/render behavior and evidence pass; actual native qualification remains F04g.

No routed Settings implementation, durable conversation delivery or full owned-Quit
qualification is included. F05 later completes presentation persistence.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.


### 2026-11-14 — F04f implementation complete

Confirmed F04e checked acceptance and linked evidence at `1fc3d936`; continued
`feature/m01-frontend-navigation` from that tip with a clean tree/index. Preserved
predecessor evidence and all successor tickets.

XState 5.33.4 and @xstate/react 6.1.0 now coordinate one app-lifetime feed owner and
one inspection owner. Feed acceptance records invalidations before cursor/ack;
replacement awaits setup and exact native stop/join, with visible fenced cleanup
failure. Inspection Query/freshness, baseline gating, single-sequence physical
draining and Zustand selected-file context replace the old lifecycle/cache loop.
Covered pending polling is removed; visible two-second external-Git cadence,
explicit-refresh priority, paging and last-good/error handling remain.

[Evidence](../../../docs/evidence/frontend-navigation/f04f.md) records source hashes,
coverage replacement, failures/red-green checks, exact commands, versions,
measurements and limits. Final configured verification passed **158 Rust + 40
tooling + 151 frontend** units. One-warm-up/five-run measurements and separate
frontend bundling passed. Independent review resolved one Standards P2 and two
Spec P2s; focused follow-up reports **Standards 0 / Spec 0 outstanding findings**.

Actual sockets/process, fixture desktop/WebView, packaging and native-provider
qualification were not run and remain F04g/later. No provider submissions occurred.
Durable reading-context persistence remains F05. The containing commit owns this
ticket's changes and evidence. Stop after F04f; F04g and all successors remain open
and were not implemented in this session.
