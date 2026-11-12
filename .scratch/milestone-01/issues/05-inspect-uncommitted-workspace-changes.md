# 05: Inspect uncommitted workspace changes

**What to build:** Inspect current uncommitted workspace changes from the desktop, including additions and deletions, without mistaking failed inspection for a clean workspace.

**Blocked by:** 04: Create isolated workspaces with recoverable retries

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A6. Full changes against the workspace base and inline feedback remain deferred.

- [ ] Show a changed-file list and readable text diffs for modifications, new files, and deletions in the selected workspace.
- [ ] Identify the comparison shown and refresh it after further edits, including fixture/agent edits once chat is integrated.
- [ ] Label binary and unsupported content honestly rather than presenting an incomplete text rendering as complete.
- [ ] Preserve the last good result with a visible error when Git or file reads fail; distinguish stale/failed inspection from a clean comparison.
- [ ] Keep blocking Git/file operations away from rendering and attach observations to the correct workspace.
- [ ] Verify through the core interface with temporary real Git repositories; demonstrate desktop states for changed, clean, unsupported, refreshed, and failed inspection and retain evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
