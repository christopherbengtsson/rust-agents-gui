# 05: Establish feature observations and UI context

Status: ready-for-agent

Blocked by: [F04](04-push-observation-delivery.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#query-freshness) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

Existing repository/workspace Hooks mix bounded reads with selection and retained
action observations. Migrate list/detail read/cache responsibility to Query without
moving action reconciliation out of Rust. Feature modules hide decoding, query keys
and freshness from screens. Current navigation remains working until F06 makes it
route-authoritative.

Create app-scoped instances above future layouts. The stable physical inspection
reader must survive both route replacement and workspace presentation resets.
Zustand retains lightweight reading context, while Query owns fetched observations.

- Use reviewed installed Router/Query composition and official Query references under repository policy. Apply the parent-plan local-IPC cache defaults and owner/target keys.
- Translate typed hints into feature-local affected keys; mark stale without refetch fan-out. Refresh only visible/current or retained-action interests through F03.
- Attach and process a baseline before initial reads; keep per-target dirty generations so in-flight reads cannot consume later invalidations. Bound dirty state; overflow becomes one active-interest resync.
- Remove covered 250 ms availability/action/progress and inspection-pending polls only after replacement scenarios pass. Retain visible external-Git capture cadence and explicit refresh distinction.
- Retain last-good/error observations and pending action identities across Settings/selection changes. Hint-driven repository reads never request explicit re-probing.
- Keep pure shared guards independent of Tauri/preferences; keep feature modules cohesive and shared toolkit layers independent of transport.

Baseline source footholds:

- [src/App.tsx](../../../src/App.tsx) — single current observation owners/composition.
- [src/hooks/use-repositories.ts](../../../src/hooks/use-repositories.ts) — repository reads, registration settlement and pending polling.
- [src/hooks/use-workspaces.ts](../../../src/hooks/use-workspaces.ts) — workspace reads/actions and progress polling.
- [src/components/blocks/workspace-panel.tsx](../../../src/components/blocks/workspace-panel.tsx) — current physical-reader lifetime.
- [src/lib/workspace-changes-reader.ts](../../../src/lib/workspace-changes-reader.ts) — drain, visibility/error pause and selected-preview sequencing.

## Required scenarios

- A hint before baseline, during the initial read and during a follow-up is eventually observed; acknowledge before refetch settlement.
- Hint storms coalesce; unrelated targets do not read; owner change cannot reuse earlier-owner data as current.
- Hidden details pause reads; pending actions and reader physical drain survive navigation/disposal.
- Removed polling no longer runs; visible external-edit refresh and error/last-good behavior still pass.
- Restored per-workspace file/comparison/scroll context remains validated and preference errors remain visible.

Use [repository verification policy](../../../docs/agents/verification.md) and
the existing gate. Preserve prior useful assertions; actual desktop/process evidence
remains separate from controlled units.

## Acceptance

- [ ] Query replaces list/detail cache/read state without duplicating authoritative snapshots in Zustand.
- [ ] One app-lifetime subscription and physical inspection reader serve narrowly subscribed views.
- [ ] Push-driven freshness, queue bounds and failure/visibility behavior have passing controlled coverage.
- [ ] Presentation context persists only the agreed preference allowlist; no provider/storage policy moves to JavaScript.

## Handoff boundary

F06 owns final route selection authority and preference-to-route migration. F07 owns Settings UI. Keep heavy diffs/transcripts out of Zustand and leave full desktop qualification to F08.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.
