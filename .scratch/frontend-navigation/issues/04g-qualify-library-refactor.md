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

- [ ] Requirement-to-evidence and removed-to-replacement coverage matrices are complete; temporary migration mechanisms are gone.
- [ ] Relevant deterministic/static gates and separate production packaging pass with source/sidecar provenance.
- [ ] Focused actual socket/process/fixture-desktop recovery, bounds, access and cleanup checks pass before F05.
- [ ] Costs/benefits, failures, commands and qualification limits are recorded; material regressions are fixed or returned for decision.
- [ ] Future handoffs match actual definitions and F05 remains limited to presentation context/integration.

This does not complete F08's later route/Settings/accessibility/cumulative performance
qualification, milestone native-provider work or full owned Quit. A failed required
actual check leaves this gate incomplete; F05 cannot start until it passes.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.
