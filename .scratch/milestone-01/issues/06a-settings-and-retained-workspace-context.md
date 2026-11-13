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

- [ ] An accessible Settings gear opens a full Settings screen replacing the
      workbench/sidebar visually. It has Settings, Back, category navigation and
      Agent runtimes first, with Codex as the only current runtime surface.
- [ ] This first slice truthfully explains that no setup diagnostic has run; do
      not fabricate success or expose an inert Check/Save action. 06b supplies the
      working runtime controls and observations.
- [ ] A small navigation module near App owns screen transitions, return/focus and
      explicit-navigation eligibility. Keep no router, mirrored selected identity,
      general history engine or generic view cache. Existing hooks retain
      canonical repository/workspace selections.
- [ ] Settings entry prevents earlier registration/creation completion from
      redirecting selection; the correct action still settles and updates records.
      Cover entry during pending work and navigation away/back to the same ID.
- [ ] One stable retained workbench preserves tab, selected file, panel
      configuration and reading position. Keep the physical reader lane; hidden
      inspection effects pause and shared app observations remain outside Activity.
- [ ] Opening Settings focuses its heading; Back returns appropriate focus.
      Main/skip-target identity, keyboard navigation, hidden disclosures and portal
      cleanup remain correct. Navigation executes no retry or provider command.
- [ ] Compose installed Base UI/shadcn controls, compact variants, tokens and
      existing CSP integration. Document new blocks and preserve props/refs and
      native interactive semantics.
- [ ] Controlled DOM checks cover transitions, retained state, late action
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
