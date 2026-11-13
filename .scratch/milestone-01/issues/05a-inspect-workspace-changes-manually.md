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
- [x] Manual Refresh shows text modifications, new files and deletions against
      resolved HEAD, with visible comparison identity and staged/unstaged status.
      Known unsupported/unsafe content has honest labels from this first slice.
- [x] The private inspection module within `workspaces` owns comparison and
      failure policy. Existing workspace requests/replies carry its observations;
      Tauri forwards them and TypeScript validates their shapes and targets.
- [x] The existing worker admits bounded, coalesced inspection work with fair job
      selection and owned shutdown. Observation requests remain responsive while
      jobs run; Git/filesystem operations execute outside observation/storage locks.
- [x] File-list paging and the single selected-file preview obey the accepted
      count/serialized-byte limits. Limited previews are labeled; Git failure/output
      overflow cannot masquerade as a valid prefix. There is no diff pagination.
- [x] Clean, loading, unsupported and failed inspection remain distinct. A failed
      manual refresh retains and labels its last-good result rather than appearing
      clean. No saved workspace identity or original-checkout files are mutated by
      inspection, and no agent starts.
- [ ] The inactive heavy tab content unmounts. One observation owner sits above
      the tab content; ordinary tab switching retains selected-file reading context.
      Scroll Area works under the existing CSP without broad policy relaxation.
- [x] Deterministic units cover the manual path, exact targets, text parsing and
      numbering, basic unsupported/error outcomes, preview bounds, fairness and
      command/environment construction using supplied dependency outcomes.
- [x] Relevant checks pass, new product contracts are documented in the component
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

## Comments

### 2026-11-13 — manual inspection implemented and reviewed

Implementation commit: `9790e461cc14a3374058b4441a2995d82d460528` on
`feature/m01-change-inspection`.
[Verification and handoff evidence](../../../docs/evidence/milestone-01-change-inspection-manual.md)
records commands, versions/hardware, source hashes, counts, timings, development
failures, review findings/fixes and qualification limits.

The complete configured gate passed 96 Rust + 25 tooling + 55 UI checks. Two
post-review Rust regressions initially failed, were fixed, and final controlled
verification passes all 98 Rust units; all-target Clippy and final warm Rust
measurements pass. Independent Standards and Spec review follow-ups report zero
unresolved findings. Frontend production compilation passed with its retained
bundle-size warning. Commit hooks passed format/lint.

The concrete observation interface is `WorkspaceRequest::{Changes,ChangePreview}`
and the corresponding `WorkspaceReply` variants in
[`workspaces/creation.rs`](../../../crates/core/src/workspaces/creation.rs).
The private [`inspection` owner](../../../crates/core/src/workspaces/inspection.rs)
and [`TypeScript validators`](../../../src/lib/changes.ts) own the actual contracts.
Use `refresh=false` / `load=false` for observation lookup; pending jobs settle via
250 ms polling without repeat admission. The evidence documents cache/acquisition
limits, stable raw-path file IDs across refreshed comparisons, last-good preview
comparison identity, and the remaining 05b work.

The two unchecked acceptance items include actual desktop presentation/CSP
behavior for which this slice has implementation and controlled DOM evidence but
no actual WebView qualification. Packaging, actual desktop/process inspection and
native-provider checks were not performed; 05c owns that qualification. No live
provider submissions occurred. Stop after 05a: 05b, 05c, parent 05 and Milestone 01
remain open. Completed predecessor evidence is preserved.
