# 04: Create isolated workspaces with recoverable retries

**What to build:** Create a separate branch/worktree at an explicitly selected local commit, retain its identity, and recover a partially completed creation without duplicating or deleting work.

**Blocked by:** 03: Register repositories and restore navigation

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A2 and workspace-creation portion of A10.

- [ ] Extend `RepositorySidebar` with workspace entries and `WorkspacePanel` with selected-workspace context. Coordinate selection above its consumers while preserving observation owners across selection changes. Present base-commit selection and creation actions using the established controls, keeping branch, path and provider readable.
- [ ] Show creation progress, known partial artifacts and failed/uncertain outcomes in the same workspace flow with explicit recovery actions reflecting core observations; retain selection/context so a failed operation remains understandable.
- [ ] Resolve the selected local base to an identified commit; create the separate branch/worktree and display workspace path, branch, and provider.
- [ ] Persist workspace identity, pinned base, action intent, and sufficient partial-creation evidence before effects; retain identity across restart.
- [ ] Preserve original checkout files and unrelated branches/folders on success, failure, and recovery.
- [ ] Expose branch/folder artifacts from partial failure and offer explicit recovery. Repeating the same action ID returns recorded status/outcome; reconcile uncertainty before another creation effect.
- [ ] Keep creation and retry/reconciliation policy in workspaces. New workspace creation does not automatically start Codex.
- [ ] Unit-test successful creation, failures before/after recorded effects, lost replies, restored records, repeated/conflicting action IDs and storage failures with controlled outcomes. Prove intent precedes mutation, failed intent blocks mutation, replay retrieves status, and only explicit reconciliation continues uncertain work.
- [ ] Demonstrate desktop success and actionable recovery states; record commands/results.
- [ ] Cover timeout/output-limit and unconfirmed cleanup responses without real deadline waits. Use controlled events to verify pending status remains readable during an effect, and supplied records/serialization for capacity boundaries and recovery reserve. Keep admission limits separate from storage identity.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
