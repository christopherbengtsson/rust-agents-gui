# 05: Inspect uncommitted workspace changes

**What to build:** Inspect current uncommitted workspace changes from the desktop, including additions and deletions, without mistaking failed inspection for a clean workspace.

**Blocked by:** 04: Create isolated workspaces with recoverable retries

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A6. Full changes against the workspace base and inline feedback remain deferred.

**Execution:** Follow the [accepted change-inspection plan](../change-inspection-plan.md)
on `feature/m01-change-inspection`, in order:
[05a: manual inspection](05a-inspect-workspace-changes-manually.md) →
[05b: reliable refresh](05b-refresh-workspace-changes-reliably.md) →
[05c: qualification and review](05c-qualify-workspace-change-inspection.md).
This ticket remains the acceptance parent; child completion alone does not check
its original acceptance items.

- [ ] Add change inspection as a focused product block within `WorkspacePanel`, with keyboard-reachable file selection, a readable selected-file diff and visible comparison identity; preserve access to workspace navigation.
- [ ] Apply the shared empty/loading/error and unsupported-content patterns to the changes region. Keep long diffs/previews bounded and readable, and label a retained last-good result when refresh fails.
- [ ] Show a changed-file list and readable text diffs for modifications, new files, and deletions in the selected workspace.
- [ ] Identify the comparison shown and refresh it after further edits, including fixture/agent edits once chat is integrated.
- [ ] Label binary and unsupported content honestly rather than presenting an incomplete text rendering as complete.
- [ ] Preserve the last good result with a visible error when Git or file reads fail; distinguish stale/failed inspection from a clean comparison.
- [ ] Keep blocking Git/file operations away from rendering and attach observations to the correct workspace.
- [ ] Unit-test workspace/comparison selection, parsing, classification, refresh and last-good retention with supplied Git bytes and file metadata/read outcomes. Cover newline, binary/non-UTF-8, symlink/submodule/size and preview-bound cases; test command/environment construction without running Git. Separately demonstrate desktop changed, clean, unsupported, refreshed and failed states with disposable fixture state; retain distinct evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

### 2026-11-13 — accepted plan and session split

The user accepted one central Changes tab with a collapsible/resizable right-hand
flat file list, unified bounded previews, Scroll Area and a narrow-window Sheet.
Keep the full agreed presentation; simplify through one inspection owner within
`workspaces` and bounded selected-file previews instead of diff pagination.
Inactive heavy tab content unmounts while one observation owner retains reading
context above it.

The linked plan records decisions and dependencies for two implementation sessions
and a separate actual-desktop qualification/review session, all on the same feature
branch. This preparation implements no product behavior and claims no A6
qualification; the acceptance items above remain unchecked.
