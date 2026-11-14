# 07: Add routed Settings

Status: ready-for-agent

Blocked by: [F06](06-route-authoritative-navigation.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#settings) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

Settings removes inspection interest from the app-lifetime actor while physical
work drains; it never disposes the actor, feed or Query client. Restore context
through F05's validated presentation store. Consume feature interfaces without
queue/ack bookkeeping; verify native physical cleanup with the established seam.
The current UI guide distinguishes the F04 baseline reader from this target.

Implement the agreed footer gear and full-page Settings layout. Conductor 0.93.0
opens routed Settings from a top account menu; our footer placement and lack of
account scope are deliberate. General is the only section and contains honest
placeholder content.

Settings replaces the workspace layout while app-lifetime observation/action owners
and the inspection reader remain alive. Heavy workspace presentation unmounts and
inspection activity pauses; returning restores validated lightweight context.

- Compose existing Base UI/shadcn controls with one interactive target and accessible Settings name. Follow installed-version composition and retain semantic tokens.
- Add independent settings navigation/main landmark at /settings/general; redirect /settings with replacement. Use the existing empty/placeholder patterns without fake controls.
- Back/Escape uses preceding app navigation; when no usable preceding app destination exists, replace with home. Nested overlays handle Escape first and focus returns appropriately. This follows the F07 user decision below, superseding the original remembered-destination fallback.
- Preserve per-workspace selected file, comparison context, scroll and panel layout through Settings and workspace switches; revalidate stale comparison context.
- Document changed block contracts. Link original milestone 06 to Settings → Providers → Codex without implementing provider setup or merging supervisor diagnostics.

Baseline source footholds:

- [src/components/blocks/repository-sidebar.tsx](../../../src/components/blocks/repository-sidebar.tsx) — footer entry alongside separate supervisor diagnostics.
- [src/components/README.md](../../../src/components/README.md) — component contracts and layers.
- [src/components/blocks/workspace-changes.tsx](../../../src/components/blocks/workspace-changes.tsx) — reading context and existing responsive inspection behavior.
- [src/style.css](../../../src/style.css) — shared semantic tokens and shell dimensions.

## Required scenarios

- Gear keyboard activation, General active navigation, Back/Escape and focus return.
- Direct settings entry with and without remembered valid destination; nested-overlay Escape.
- Slow inspection across Settings return maintains one physical lane and no hidden automatic reads.
- Reading context and existing tabs/file selection/Sheet/resizing remain usable at ordinary and minimum sizes.

Use [repository verification policy](../../../docs/agents/verification.md) and
the existing gate. Preserve prior useful assertions; actual desktop/process evidence
remains separate from controlled units.

## Acceptance

- [x] Footer gear opens a full-page routed General placeholder directly.
- [x] Return, keyboard/focus and retained-context behavior pass controlled checks.
- [x] Main and settings layouts share one observation/action lifetime with inspection paused when hidden.
- [x] Component guide and milestone-06 extension pointers match the actual composition.

## Handoff boundary

No account menu, provider settings implementation or native deep-link handling is added here. F08 must qualify real WebView layout, focus, reload and responsiveness.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.

### 2026-11-14 — F04a handoff amendment

Updated predecessor consumption for the library refactor; implementation and
qualification acceptance remain unchecked. Original dated comments are retained.

### 2026-11-14 — return behavior agreed during F07 planning

The user simplified direct-entry handling: normal launch does not open Settings,
and this ticket adds no reload/deep-link or reconnect machinery. Back/Escape uses
the preceding app destination when available; otherwise replace with home, even
when a remembered destination exists. This explicit decision supersedes the older
remembered-destination fallback in this ticket and the parent Settings section.
Actual cumulative WebView qualification remains F08.

### 2026-11-14 — F07 implemented and verified

Confirmed F06 checked acceptance and dated evidence at
`4be1962e37a1d6c97a6918ca8e95c9b972ac2b8b`; initial tree/index were clean. Continued
the current feature branch and preserved all predecessor evidence.

[Evidence](../../../docs/evidence/frontend-navigation/f07.md) records the footer gear,
full-page routed General placeholder, history/Back/Escape/focus behavior, overlay
precedence and retained workspace reading/layout context. The user-selected
absent-origin home fallback above is implemented; no reconnect, account or provider
setup was added. App-lifetime owners remain shared and hidden inspection drains.

Final configured gate passed **158 Rust + 49 tooling + 213 frontend units**, plus
production packaging and one warm-up/five-run measurements. The established
controlled native physical-completion checks passed. Failed red/intermediate
attempts, exact commands/timings, source/package hashes, independent review
(**Standards 0 / Spec 0**) and qualification limits are retained. No live-provider
submissions or actual desktop/process cohort was performed for F07. F08 retains
cumulative real WebView layout/focus/reload/responsiveness qualification and remains
unimplemented. The containing commit owns F07. Stop after F07.
