# 06b: Check Codex setup end to end

**What to build:** Explicitly check installed Codex setup from Settings through a
deep Rust runtime module, with persisted executable selection and actionable results.

**Blocked by:** [06a: Settings and retained context](06a-settings-and-retained-workspace-context.md).

**Status:** ready-for-agent

**Parent:** [06: Codex setup](06-connect-codex-with-actionable-setup-diagnostics.md).

## Start here

Read the [accepted plan](../codex-setup-plan.md), 06a's completion comment,
[MVP](../../../docs/MVP.md), [Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[component guide](../../../src/components/README.md). Apply repository-local Rust,
React and shadcn skills. Inspect implemented contracts before extending them.

Switch to `feature/m01-codex-setup`, created by 06a, and continue from its completed
commit. Keep this child's commits on that branch; do not create a new branch or
implement on `main`.

## Acceptance

- [ ] Before changing an adapter/fixture, inspect Codex 0.167.2's official schema
      at source `ce21072c90d32c40af7a55b60b2fb35c84a43a6a` and retained evidence.
      Record version, source/schema hashes and captured/synthetic provenance beside
      fixtures. Retain exact identifier types and approval matching assumptions.
- [ ] Core hosts a concrete runtimes owner beside workspaces. It owns setup
      admission, sequencing, protocol interpretation, observations and process
      cleanup through private effect seams. Add no generic provider framework or
      workspace/Git-worker runtime jobs.
- [ ] Concrete stdio and deterministic fixture adapters use the same execution
      seam and shared protocol interpretation. Retain production policy, parsing
      and command construction under controlled tests.
- [ ] Typed runtime requests/replies traverse Core, supervisor, existing client
      transport and Tauri to TypeScript validation. Bind owner/runtime/configuration
      revision/check identity, preserve structured runtime errors, and update wire
      version/compatibility checks. Shared exchange refactoring remains deferred.
- [ ] Persist executable save/reset through narrow supervisor-owned SQLite
      storage. Override, PATH and standard-location discovery follow the plan;
      execute directly without a shell or silent invalid-override fallback.
- [ ] Save/check work is serialized in Rust and presented accurately. Save starts
      no Codex process; Check uses saved configuration and is disabled for unsaved
      edits. Pending Save results retain edits typed later. Definite rejection and
      uncertain commit/reply differ; authoritative reads settle uncertainty before
      further setup work, without replay.
- [ ] Explicit Check setup performs metadata-only initialization, non-forced-refresh
      account/configuration reads and owned cleanup. Mount/navigation/reconnect
      only observe; repeated admitted-check identity returns status. Preserve
      identity/recovery evidence before effects.
- [ ] A bounded runtime worker keeps admission/status short, uses short locks and
      owns handles, output draining, frame/queue/output bounds, deadlines and
      bounded shutdown. Cleanup uncertainty is visible and blocks conflicting
      replacement. No known unsafe lifecycle behavior is deferred to 06c.
- [ ] Settings shows truthful pending/result/timestamp, executable/version,
      auth/provider/billing evidence or unknown, plus missing executable,
      incompatible version/protocol, unavailable auth and other actionable errors.
      Transport failures retain stale evidence rather than invent execution failure.
- [ ] Official external instructions and copyable commands work with visible
      copy/open failures and scoped application-owned documentation destinations.
      Preserve Field labels/errors, Base UI semantics/refs, tokens and CSP.
- [ ] Diagnostics never gate normal Send/Resume or require authentication each app
      launch. Successful check results are session observations; persist only
      preferences and required recovery evidence. Invoke no login/logout, credential
      extraction/editing, config writes, thread or turn.
- [ ] Controlled Rust/DOM checks cover the working setup path, primary error states,
      save uncertainty/draft races, no replay, bounds and cleanup decisions. Apply
      configured checks/review and document concrete contracts and evidence limits.

## Completion and handoff

Commit and record actual module/transport/storage contracts, worker ownership,
named budgets/deadlines, protocol cohort/provenance, tests and review. Supply
reproducible private-state fixture instructions for setup success and primary
failures through the production supervisor path. Unit mocks do not qualify desktop
or native behavior. Preserve failed/unperformed results.

Stop after this child. [06c](06c-harden-codex-protocol-and-process-recovery.md)
extends adversarial coverage; [06d](06d-qualify-codex-setup.md) owns qualification.
Parent 06 remains open. Metadata/schema inspection authorizes no live agent
submission or inherited native allowance.
