# 04g: Qualify the library refactor before F05

Status: ready-for-agent

Blocked by: [F04f](04f-actor-feed-and-inspection.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read the
[owning requirements](../spec.md#verification-and-performance) and predecessor evidence. Read the MVP,
milestone, verification policy and applicable repository engineering guidance.
[Observation guidance](../../../docs/agents/observations.md) owns shared engineering
pitfalls; the parent spec owns behavior and limits. This ticket is one session;
successors remain separate. Preserve unrelated edits and completed evidence.

## Qualification scope

This is the focused actual transport/observation gate before F05. Deterministic
checks do not establish socket behavior, OS access control, native cleanup, packaged
resources or real WebView delivery. Use private disposable application state and
repositories, production core/supervisor/Tauri paths, and application-owned fixture
faults. No real provider submissions. Preserve completed evidence and failed runs.

## Work and removal

- Map F04b–F04f requirements to controlled and actual evidence. Remove remaining
  superseded synchronous mechanisms, duplicate caches/lifecycle systems or temporary
  migration adapters before closure, with passing coverage replacements first.
- Run relevant configured static/unit gates and required warm-up/five-run measurements;
  record compilation separately, then packaging/provenance separately. Record exact
  versions, source identity, profile/cache, commands, statuses and elapsed times.
- Qualify actual protocol-8 framing/limits/deadlines, private access, exclusive owner
  admission, incompatible peers, bounded client/server work, stopped/late peers,
  cancellation and joined subscription/handler completion. Keep intentional blocking
  work's physical admission until completion.
- Demonstrate repository/workspace/inspection progress without removed 250 ms polls,
  explicit availability reprobe, visible external-Git refresh, last-good errors,
  overflow/resync, stalls, feed ack flow control and accepted-cursor reconnect.
- Exercise desktop disconnect/reconnect, replacement owner rejection, rapid selection,
  hidden views/interest changes and no duplicate intents. Retained actions remain
  exact and uncertain until authoritative matching status settles them.
- Compare bundle/raw/gzip/Brotli, packaged size, representative navigation against
  100 ms, retained memory/queue/IPC counts and hidden activity against comparable
  F04 baseline conditions. Record maintenance/removal benefits without inventing
  a line-count target or treating unit timings as desktop latency. F04a's baseline
  records existing F04 evidence only; recreate `7dc4eac` in an isolated checkout
  and private Cargo target for matched native measurements still unperformed there.
  Never reset the active branch or share mutable target/application state.

Source footholds: predecessor evidence and new modules; existing sidecar/development
state scripts; original supervisor process qualification; original inspection fixture
qualification methods. Keep process/desktop harnesses explicit and outside units;
add no integration-test framework. Report unsupported/unperformed checks honestly.

## Acceptance

- [x] Requirement-to-evidence and removed-to-replacement coverage matrices are complete; temporary migration mechanisms are gone.
- [x] Relevant deterministic/static gates and separate production packaging pass with source/sidecar provenance.
- [x] Focused actual socket/process/fixture-desktop recovery, bounds, access and cleanup checks pass before F05.
- [x] Costs/benefits, failures, commands and qualification limits are recorded; material regressions are fixed or returned for decision.
- [x] Future handoffs match actual definitions and F05 remains limited to presentation context/integration.

This does not complete F08's later route/Settings/accessibility/cumulative performance
qualification, milestone native-provider work or full owned Quit. A failed required
actual check leaves this gate incomplete; F05 cannot start until it passes.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.

### 2026-11-14 — F04g implemented and qualified

Confirmed F04f checked acceptance and evidence at
`5140ecee8a938c996cd4759917c8355eebc3ac2e`; continued the current feature branch
with a clean tree/index. Preserved historical evidence and successor tickets.

[Evidence](../../../docs/evidence/frontend-navigation/f04g.md) records the
requirement/removal matrices, source/artifact hashes, versions, commands, timings,
failed attempts, limits and independent review. Actual WebView qualification found
and fixed pending/error-summary retry storms with retained selected previews;
new actor regressions failed before their fixes and pass in the configured gate.

Final `corepack pnpm verify` passed **158 Rust + 40 tooling + 153 frontend units**;
separate production packaging, **19 actual socket scenarios**, and **9 actual
process scenarios** passed. Warm-up/five-run measurements are separate from
compilation. Matched baseline `7dc4eac` used an isolated checkout/private target.
Actual rendered navigation feedback medians were **32 / 45 ms**; queue bounds,
physical draining, hidden pause, cache expiry, explicit reprobe, last-good errors,
accepted-cursor reconnect, real lost admitted reply and desktop-only crash recovery
passed. One retained creation/action/worktree effect was observed with no replay.
All fixture apps/owners exited, faults/endpoints were restored, and original
checkout files were unchanged.

Standards and Spec review resolved all findings; final signoff is **0 / 0
outstanding**. The containing commit owns this ticket's changes/evidence. F05 is
limited to presentation context/integration; it and all successors remain open.
No provider submissions, F08 cumulative qualification or full owned-Quit claim is
included. Stop after F04g.
