# 06a: Settings and retained workspace context

**What to build:** Open a full Settings screen from the sidebar and return to the
same workbench context through a small application navigation module.

**Blocked by:** [02a: UI foundation](02a-establish-incremental-ui-foundation.md),
[04: Workspaces](04-create-isolated-workspaces-with-recoverable-retries.md) and
[05: Change inspection](05-inspect-uncommitted-workspace-changes.md) (complete).

**Status:** ready-for-agent

**Parent:** [06: Codex setup](06-connect-codex-with-actionable-setup-diagnostics.md).

## Start here

Read the [accepted plan](../codex-setup-plan.md),
[navigation research](../../../docs/research/settings-navigation.md),
[MVP](../../../docs/MVP.md), [Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[component guide](../../../src/components/README.md). Use repository-local React,
shadcn and Rust skills where applicable. Inspect the current branch and predecessor
evidence; follow the plan's branch/session rules.

Before product edits, create and switch to `feature/m01-codex-setup` from the
current `main` preparation baseline. If it already exists, inspect and continue it
without resetting or recreating it. Commit this child on that branch; 06b–06d
continue on the same branch.

## Acceptance

- [x] An accessible Settings gear opens a full Settings screen replacing the
      workbench/sidebar visually. It has Settings, Back, category navigation and
      Agent runtimes first, with Codex as the only current runtime surface.
- [x] This first slice truthfully explains that no setup diagnostic has run; do
      not fabricate success or expose an inert Check/Save action. 06b supplies the
      working runtime controls and observations.
- [x] A small navigation module near App owns screen transitions, return/focus and
      explicit-navigation eligibility. Keep no router, mirrored selected identity,
      general history engine or generic view cache. Existing hooks retain
      canonical repository/workspace selections.
- [x] Settings entry prevents earlier registration/creation completion from
      redirecting selection; the correct action still settles and updates records.
      Cover entry during pending work and navigation away/back to the same ID.
- [x] One stable retained workbench preserves tab, selected file, panel
      configuration and reading position. Keep the physical reader lane; hidden
      inspection effects pause and shared app observations remain outside Activity.
- [x] Opening Settings focuses its heading; Back returns appropriate focus.
      Main/skip-target identity, keyboard navigation, hidden disclosures and portal
      cleanup remain correct. Navigation executes no retry or provider command.
- [x] Compose installed Base UI/shadcn controls, compact variants, tokens and
      existing CSP integration. Document new blocks and preserve props/refs and
      native interactive semantics.
- [x] Controlled DOM checks cover transitions, retained state, late action
      completion, shared observation ownership, effect cleanup/reveal and focus
      intent. Relevant checks and review pass; qualification limits are recorded.

## Completion and handoff

Commit the working increment and add a dated comment linking evidence, checks and
remaining qualification. Record the actual navigation module, action eligibility
rule, retained-view composition and reader lifetime for 06b. Leave actual WebView
qualification unclaimed until 06d supplies evidence.

Stop after this child. [06b](06b-check-codex-setup-end-to-end.md) owns runtime
adapters, executable persistence and Check setup. Parent 06 remains open. No live
agent submission, provider process, login change or runtime configuration change
is authorized by this child.

## Comments

### 2026-11-13 — 06a implemented and controlled checks passed

Implemented on `feature/m01-codex-setup` from preparation baseline
`fbf6e7361af3046945d396d602536b3c6beffb94`; the commit containing this comment
contains this increment. See [implementation, verification and handoff evidence](../../../docs/evidence/milestone-01-settings.md)
and its source manifest/logs. The complete gate passed formatting, lint, TypeScript,
Clippy and **110 Rust + 25 tooling + 99 UI** checks. Required warm measurements
and the production frontend build passed. Independent code-review Standards and
Spec axes reported **0 unresolved findings** each.

06b continues on this branch. `src/use-app-navigation.ts` owns screen/focus intent
and shared generation-based selection eligibility. Existing domain hooks remain
canonical selection owners. A logical action keeps its original permission across
retry; Settings/explicit navigation invalidate redirect permission while exact
action settlement and records still update. Initial-workspace handoff carries the
same generation. App keeps shared observations outside one stable workbench
Activity; WorkspacePanel retains the physical reader above workspace-keyed
inspection. Hide pauses inspection effects and reveal drains old IPC before
observing again. Tab/file/panel/reading state remains with workspace presentation.

Checked acceptance is scoped to 06a's implementation and controlled DOM evidence.
Actual WebView accessibility/layout/responsiveness, fixture-desktop/process and
packaged-desktop qualification are **not run for this source** and remain 06d.
Native-provider qualification is **not run**; zero provider submissions/processes,
login changes or runtime configuration changes. 06b owns real setup controls and
observations. Parent 06 and later children remain open. Stop after this child.
