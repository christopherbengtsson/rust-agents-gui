# 07: Add routed Settings

Status: ready-for-agent

Blocked by: [F06](06-route-authoritative-navigation.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#settings) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

Implement the agreed footer gear and full-page Settings layout. Conductor 0.93.0
opens routed Settings from a top account menu; our footer placement and lack of
account scope are deliberate. General is the only section and contains honest
placeholder content.

Settings replaces the workspace layout while app-lifetime observation/action owners
and the inspection reader remain alive. Heavy workspace presentation unmounts and
inspection activity pauses; returning restores validated lightweight context.

- Compose existing Base UI/shadcn controls with one interactive target and accessible Settings name. Follow installed-version composition and retain semantic tokens.
- Add independent settings navigation/main landmark at /settings/general; redirect /settings with replacement. Use the existing empty/placeholder patterns without fake controls.
- Back/Escape uses preceding app navigation; direct entry falls back to remembered valid main destination, then home. Nested overlays handle Escape first and focus returns appropriately.
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

- [ ] Footer gear opens a full-page routed General placeholder directly.
- [ ] Return, keyboard/focus and retained-context behavior pass controlled checks.
- [ ] Main and settings layouts share one observation/action lifetime with inspection paused when hidden.
- [ ] Component guide and milestone-06 extension pointers match the actual composition.

## Handoff boundary

No account menu, provider settings implementation or native deep-link handling is added here. F08 must qualify real WebView layout, focus, reload and responsiveness.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.
