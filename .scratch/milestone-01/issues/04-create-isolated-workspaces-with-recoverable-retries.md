# 04: Create isolated workspaces with recoverable retries

**What to build:** Create a separate branch/worktree at an automatically resolved and pinned local default commit, retain its identity, and recover a partially completed creation without duplicating or deleting work.

**Blocked by:** 03: Register repositories and restore navigation

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A2 and workspace-creation portion of A10.

- [x] Extend `RepositorySidebar` with workspace entries and `WorkspacePanel` with selected-workspace context. Coordinate selection above its consumers while preserving observation owners across selection changes. Add repository creates its first workspace and repository + creates another immediately, without a composer dialog or base picker. Present the automatic pinned base and creation actions using the established controls, keeping branch, path and provider readable.
- [x] Show creation progress, known partial artifacts and failed/uncertain outcomes in the same workspace flow with explicit recovery actions reflecting core observations; retain selection/context so a failed operation remains understandable.
- [x] Resolve usable local origin/HEAD, otherwise exactly one usable local main/master, to a pinned commit; report missing or ambiguous defaults actionably; create the separate branch/worktree and display workspace path, branch, and provider.
- [x] Persist workspace identity, pinned base, action intent, and sufficient partial-creation evidence before effects; retain identity across restart.
- [x] Preserve original checkout files and unrelated branches/folders on success, failure, and recovery.
- [x] Expose branch/folder artifacts from partial failure and offer explicit recovery. Repeating the same action ID returns recorded status/outcome; reconcile uncertainty before another creation effect.
- [x] Keep creation and retry/reconciliation policy in workspaces. New workspace creation does not automatically start Codex.
- [x] Unit-test successful creation, failures before/after recorded effects, lost replies, restored records, repeated/conflicting action IDs and storage failures with controlled outcomes. Prove intent precedes mutation, failed intent blocks mutation, replay retrieves status, and only explicit reconciliation continues uncertain work.
- [x] Demonstrate desktop success and actionable recovery states; record commands/results.
- [x] Cover timeout/output-limit and unconfirmed cleanup responses without real deadline waits. Use controlled events to verify pending status remains readable during an effect, and supplied records/serialization for capacity boundaries and recovery reserve. Keep admission limits separate from storage identity.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

2026-11-13: User revised the flow during implementation planning: automatically
create the first workspace after new registration, use repository + for additional
workspaces, generate names/branches/folders, and defer the composer and base
picker. Registration and creation retain separate durable actions. The automatic
local default policy replaces the earlier manual-selection requirement.

2026-11-13: Implemented and qualified ticket 04. Final behavior gate passed 68 Rust,
19 tooling and 39 UI units. Production packaging, actual Tauri creation/recovery,
minimum-window keyboard recovery, desktop/supervisor restart, schema-1 migration
and the real inherited-execution-lease crash scenario passed. Independent Standards
and Spec review findings were fixed with red/green regressions and confirmed
resolved. [Evidence](../../../docs/evidence/milestone-01-workspaces.md) records
source/binary identities, commands, timings, screenshots, failures and limitations.
No live-provider submissions occurred; remaining tickets and the milestone stay open.

### 2026-11-13 — readable names and development follow-up

Implemented the [approved follow-up](../readable-workspaces-plan.md).

- [x] 500 attributed footballer slugs, permanent per-repository reservations,
      numeric fallback and readable repository/workspace locations.
- [x] Pending identity before inspection; atomic final reservations before mutation;
      immutable recovery targets, batch collision checks and existing race guards.
- [x] Schema 3 / protocol 4, stable clean legacy labels and explicit saved locations;
      meaningful-label rule in the component guide.
- [x] Persistent compatible dev-state isolation, side-effect-free protocol metadata,
      explicit overrides preserved and old owner untouched; unavailable navigation guidance.
- [x] Compiler-compatible async settlement with lifecycle guards/returns preserved.
- [x] 76 Rust + 25 tooling + 44 UI units, full static/format gate, production package,
      real Git/SQLite/lease/migration/reconnect and actual desktop/dev evidence; catalog
      cost measurements and independent two-axis review with its one finding resolved.

See [follow-up evidence](../../../docs/evidence/readable-workspaces.md) for commands,
results, failed attempts and limits. No provider submissions; the parent milestone
remains incomplete. Previous ticket cohorts remain historical evidence.
