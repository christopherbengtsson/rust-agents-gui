# 06: Make navigation route-authoritative

Status: ready-for-agent

Blocked by: [F05](05-feature-observations-and-ui-context.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#4-routes-settings-and-development-tooling) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

Use the exact route destinations and restoration rules in the parent plan. Router
becomes the only current-target owner; retaining old selection state and synchronizing
it bidirectionally would create competing authorities. Remembered UI context supplies
restoration defaults, not the active route.

File-route generation must work from a clean checkout before TypeScript, rather than
depending on whether Vite ran earlier. Keep source routes authoritative and generated
tree changes reviewable.

- Add file-based routes, typed registration, browser history and validated view search. Keep startup restoration one-time; explicit destinations and user navigation take precedence.
- Resolve exact details even off the listed page; validate repository/workspace membership. Preserve temporary failures and show invalid explicit targets instead of selecting a substitute.
- Migrate old selection preferences without deleting retained action IDs. Bind completion redirects to explicit-navigation generations, including A-B-A and history.
- Create one route preparation/drift-check responsibility reused by dev, standalone checking, frontend build, Tauri and CI. Check drift before bundling can repair it, and typecheck once per gate.
- Commit generated tree and exclude it from manual formatter ownership. Keep optional route views split and actual source imports analyzable; inspect emitted output rather than assuming installed dependencies are eagerly bundled.

Baseline source footholds:

- [src/App.tsx](../../../src/App.tsx) — shell/observation composition.
- [src/components/blocks/repository-sidebar.tsx](../../../src/components/blocks/repository-sidebar.tsx) — current two-selection navigation callbacks.
- [src/hooks/use-repositories.ts](../../../src/hooks/use-repositories.ts) — registration completion selection eligibility.
- [src/hooks/use-workspaces.ts](../../../src/hooks/use-workspaces.ts) — workspace completion/navigation generations.
- [vite.config.ts](../../../vite.config.ts) — compiler/plugin order and test selection.
- [package.json](../../../package.json) — dev/check/build/gate entry points.
- [.github/workflows/ci.yml](../../../.github/workflows/ci.yml) — asset preparation before Tauri checks.

## Required scenarios

- Memory-history cases cover direct/off-page targets, invalid search, wrong membership and unavailable observations.
- Startup restoration, explicit-route precedence, stale saved target and preference migration preserve pending actions.
- Delayed registration/creation cannot redirect after history, Settings-target navigation or A-B-A selection.
- A fresh-checkout generation/typecheck path and stale-tree failure are exercised through dependency-substituted tooling checks.

Use [repository verification policy](../../../docs/agents/verification.md) and
the existing gate. Preserve prior useful assertions; actual desktop/process evidence
remains separate from controlled units.

## Acceptance

- [ ] URL target/view is authoritative and no old independent selection owner competes with it.
- [ ] Typed routes, restoration, membership and generation-safe action completion pass.
- [ ] All frontend preparation paths use deterministic generated inputs in the documented order.
- [ ] Existing workspace/inspection UI still works; route changes preserve app-lifetime resources.

## Handoff boundary

F07 supplies the Settings screen at the agreed destination; keep this increment usable without implementing successor acceptance. Native URL registration remains the deferred follow-up.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.
