# 05a: Inspect workspace changes manually

**What to build:** Select a ready workspace, refresh its uncommitted changes,
select a changed file and read its bounded unified diff in the desktop.

**Blocked by:** [04: Create isolated workspaces with recoverable retries](04-create-isolated-workspaces-with-recoverable-retries.md) (complete).

**Status:** ready-for-agent

**Parent:** [05: Inspect uncommitted workspace changes](05-inspect-uncommitted-workspace-changes.md).

## Start here

Read the [accepted plan](../change-inspection-plan.md),
[MVP](../../../docs/MVP.md), [Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[component guide](../../../src/components/README.md). Apply repository-local Rust,
React and shadcn skills. Continue on `feature/m01-change-inspection` from the
ticket 04 baseline; inspect current edits before starting.

This is a complete manual inspection slice through Rust, supervisor, Tauri and
React, not a backend-only preparation ticket. It establishes the intended module
ownership and presentation for 05b to extend.

## Acceptance

- [ ] A ready workspace has working Workspace/Changes tabs and the agreed
      independently collapsible/resizable right file list, Scroll Area and
      narrow-window Sheet. Selection opens the single Changes tab; existing
      navigation, details and recovery remain reachable.
- [ ] Manual Refresh shows text modifications, new files and deletions against
      resolved HEAD, with visible comparison identity and staged/unstaged status.
      Known unsupported/unsafe content has honest labels from this first slice.
- [ ] The private inspection module within `workspaces` owns comparison and
      failure policy. Existing workspace requests/replies carry its observations;
      Tauri forwards them and TypeScript validates their shapes and targets.
- [ ] The existing worker admits bounded, coalesced inspection work with fair job
      selection and owned shutdown. Observation requests remain responsive while
      jobs run; Git/filesystem operations execute outside observation/storage locks.
- [ ] File-list paging and the single selected-file preview obey the accepted
      count/serialized-byte limits. Limited previews are labeled; Git failure/output
      overflow cannot masquerade as a valid prefix. There is no diff pagination.
- [ ] Clean, loading, unsupported and failed inspection remain distinct. A failed
      manual refresh retains and labels its last-good result rather than appearing
      clean. No saved workspace identity or original-checkout files are mutated by
      inspection, and no agent starts.
- [ ] The inactive heavy tab content unmounts. One observation owner sits above
      the tab content; ordinary tab switching retains selected-file reading context.
      Scroll Area works under the existing CSP without broad policy relaxation.
- [ ] Deterministic units cover the manual path, exact targets, text parsing and
      numbering, basic unsupported/error outcomes, preview bounds, fairness and
      command/environment construction using supplied dependency outcomes.
- [ ] Relevant checks pass, new product contracts are documented in the component
      guide, and a dated comment records the implementation commit, checks and
      unperformed desktop/native qualification.

## Completion and handoff

Run relevant configured checks under repository policy and commit the working
increment on the same branch. Record the concrete observation interface and any
implementation-specific constraints needed by 05b in the completion comment or a
linked maintained source. Point to actual code rather than copying its contracts
into a second specification. Leave acceptance unchecked where evidence is absent.

Stop after this slice. [05b](05b-refresh-workspace-changes-reliably.md) adds automatic
refresh and complete edge-case coverage. Parent 05 remains open; packaging,
desktop qualification and final branch review belong to
[05c](05c-qualify-workspace-change-inspection.md). This ticket authorizes no live
provider submissions.
