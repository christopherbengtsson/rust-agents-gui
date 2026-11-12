# 04: Create isolated workspaces with recoverable retries

**What to build:** Create a separate branch/worktree at an explicitly selected local commit, retain its identity, and recover a partially completed creation without duplicating or deleting work.

**Blocked by:** 03: Register repositories and restore navigation

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A2 and workspace-creation portion of A10.

- [ ] Resolve the selected local base to an identified commit; create the separate branch/worktree and display workspace path, branch, and provider.
- [ ] Persist workspace identity, pinned base, action intent, and sufficient partial-creation evidence before effects; retain identity across restart.
- [ ] Preserve original checkout files and unrelated branches/folders on success, failure, and recovery.
- [ ] Expose branch/folder artifacts from partial failure and offer explicit recovery. Repeating the same action ID returns recorded status/outcome; reconcile uncertainty before another creation effect.
- [ ] Keep creation and retry/reconciliation policy in workspaces. New workspace creation does not automatically start Codex.
- [ ] Verify successful creation, failures at meaningful partial-effect points, lost replies, restart, repeated action IDs, and storage failures through the core interface and actual supervisor seam with temporary real Git repositories.
- [ ] Demonstrate desktop success and actionable recovery states; record commands/results.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
