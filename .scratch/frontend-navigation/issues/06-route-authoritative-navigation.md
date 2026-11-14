# 06: Make navigation route-authoritative

Status: ready-for-agent

Blocked by: [F05](05-feature-observations-and-ui-context.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#4-routes-settings-and-development-tooling) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

Consume F04e/F04f feature interfaces and app-lifetime actors plus F05's validated
presentation store. Route loaders express observation interest through those
interfaces; they do not construct Query keys, decode IPC or stop actors. Extend
F04b's generated-input preparation owner for routes rather than creating a second
pipeline. Keep drift-before-repair and disable competing loader/preload freshness
(defaultPreloadStaleTime: 0 when Query is the external cache). Existing source
footholds are historical; follow predecessor definitions.

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

- [x] URL target/view is authoritative and no old independent selection owner competes with it.
- [x] Typed routes, restoration, membership and generation-safe action completion pass.
- [x] All frontend preparation paths use deterministic generated inputs in the documented order.
- [x] Existing workspace/inspection UI still works; route changes preserve app-lifetime resources.

## Handoff boundary

F07 supplies the Settings screen at the agreed destination; keep this increment usable without implementing successor acceptance. Native URL registration remains the deferred follow-up.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.

### 2026-11-14 — F04a handoff amendment

Updated predecessor consumption for the library refactor; implementation and
qualification acceptance remain unchecked. Original dated comments are retained.


### 2026-11-14 — F06 implemented and verified

Confirmed F05 checked acceptance and dated evidence at initial `3fd878a`; the
external F05 amendment to `98cbbc9` added only `docs/IDEAS.md` and was preserved.
Continued the current feature branch without resetting existing history or edits.

[Evidence](../../../docs/evidence/frontend-navigation/f06.md) records typed file
routes and URL authority, exact off-page membership checks, once-only restoration,
validated preference migration, generation-safe completion, shared resource
lifetimes and deterministic contract/route preparation. Invalid explicit views
preserve the URL and show valid-view guidance; missing views default to Workspace,
as selected by the user. Legacy action IDs and uncertainty remain independent.

Final configured verification passed **158 Rust + 49 tooling + 199 frontend units**;
release packaging, warm-up/five-run measurements and focused final-package private
fixture desktop smoke passed. Failed attempts, source/artifact hashes, exact
commands, timing, launcher limitations and cleanup are retained. Actual WebView
registration/first-workspace navigation, Changes URL/diff and retained workspace
context were observed. Native providers were not exercised. F08 retains cumulative
history/reload, accessibility and responsiveness qualification.

Independent Standards and Spec review, including the final explicit-view/default
regression fix, ended with **0 / 0 outstanding**. The containing commit owns F06.
F07 supplies the gear and full Settings interaction; F07 and all successors remain
unimplemented. Stop after F06.
