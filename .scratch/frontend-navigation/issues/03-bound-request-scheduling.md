# 03: Bound request scheduling

Status: ready-for-agent

Blocked by: [F02](02-owner-bound-observation-contracts.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#native-request-scheduling) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

Tauri currently holds one connection mutex during independent socket exchanges.
Removing that lock alone would leave unbounded blocking work and initialization
races. The baseline workspace Hook also fans list reads out per repository; native
fail-fast admission must ship with a frontend read coordinator so normal navigation
is not converted into Busy errors.

Use the plan's finite native lanes and frontend queue bounds. Native admission is
nonwaiting and precedes spawn_blocking; the frontend coordinator waits/coalesces
observation work. Keep request permits until physical completion, not until the
view or promise disappears.

- Keep immutable exchange configuration/client handles outside short state-lock scopes; only status/discovery commits connection cursor state.
- Admit two observations, one intent/control and one status operation; reserve the later subscription separately. Return F02 transport Busy rather than a domain Capacity/Conflict result.
- Route all automatic feature reads, including inspection reads, through the shared bounded coordinator; retain inspection sequencing inside its existing reader.
- Drop obsolete queued reads and coalesce identical targets. Keep explicit intent submission outside automatic retrying queues.
- Preserve existing exclusive discovery/no-restart-on-established-disconnect rules and release server slots through owned guards.

Baseline source footholds:

- [src-tauri/src/main.rs](../../../src-tauri/src/main.rs) — shared mutex and blocking task forwarding.
- [crates/core/src/connection.rs](../../../crates/core/src/connection.rs) — Rust discovery/client ownership.
- [src/hooks/use-workspaces.ts](../../../src/hooks/use-workspaces.ts) — per-repository list fan-out.
- [src/lib/workspace-changes-reader.ts](../../../src/lib/workspace-changes-reader.ts) — physical inspection lane to preserve.

## Required scenarios

- Stall observation exchanges: status and an explicit intent can still start; native limits and frontend queue bounds hold.
- Dispose a presentation request while physical I/O is stalled: admission remains occupied until completion.
- Error/unwind/abandoned-response paths release admission; concurrent initialization admits one discovery.
- Busy after earlier uncertainty retains the action; disconnect and queue disposal never replay it.

Use [repository verification policy](../../../docs/agents/verification.md) and
the existing gate. Preserve prior useful assertions; actual desktop/process evidence
remains separate from controlled units.

## Acceptance

- [ ] Socket I/O holds no shared connection-state lock.
- [ ] Admission is bounded before blocking tasks start, with no native waiting queue.
- [ ] Frontend reads are bounded/coalesced and preserve the one physical inspection lane.
- [ ] Controlled scheduling/cleanup/discovery regressions pass; no new async runtime migration is introduced.

## Handoff boundary

F04 supplies the dedicated subscription. F05 later replaces covered observation scheduling with push. Actual latency/socket qualification belongs to F08; unit scheduling uses controlled effects.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.
