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

- [x] Before changing an adapter/fixture, inspect Codex 0.167.2's official schema
      at source `ce21072c90d32c40af7a55b60b2fb35c84a43a6a` and retained evidence.
      Record version, source/schema hashes and captured/synthetic provenance beside
      fixtures. Retain exact identifier types and approval matching assumptions.
- [x] Core hosts a concrete runtimes owner beside workspaces. It owns setup
      admission, sequencing, protocol interpretation, observations and process
      cleanup through private effect seams. Add no generic provider framework or
      workspace/Git-worker runtime jobs.
- [x] Concrete stdio and deterministic fixture adapters use the same execution
      seam and shared protocol interpretation. Retain production policy, parsing
      and command construction under controlled tests.
- [x] Typed runtime requests/replies traverse Core, supervisor, existing client
      transport and Tauri to TypeScript validation. Bind owner/runtime/configuration
      revision/check identity, preserve structured runtime errors, and update wire
      version/compatibility checks. Shared exchange refactoring remains deferred.
- [x] Persist executable save/reset through narrow supervisor-owned SQLite
      storage. Override, PATH and standard-location discovery follow the plan;
      execute directly without a shell or silent invalid-override fallback.
- [x] Save/check work is serialized in Rust and presented accurately. Save starts
      no Codex process; Check uses saved configuration and is disabled for unsaved
      edits. Pending Save results retain edits typed later. Definite rejection and
      uncertain commit/reply differ; authoritative reads settle uncertainty before
      further setup work, without replay.
- [x] Explicit Check setup performs metadata-only initialization, non-forced-refresh
      account/configuration reads and owned cleanup. Mount/navigation/reconnect
      only observe; repeated admitted-check identity returns status. Preserve
      identity/recovery evidence before effects.
- [x] A bounded runtime worker keeps admission/status short, uses short locks and
      owns handles, output draining, frame/queue/output bounds, deadlines and
      bounded shutdown. Cleanup uncertainty is visible and blocks conflicting
      replacement. No known unsafe lifecycle behavior is deferred to 06c.
- [x] Settings shows truthful pending/result/timestamp, executable/version,
      auth/provider/billing evidence or unknown, plus missing executable,
      incompatible version/protocol, unavailable auth and other actionable errors.
      Transport failures retain stale evidence rather than invent execution failure.
- [x] Official external instructions and copyable commands work with visible
      copy/open failures and scoped application-owned documentation destinations.
      Preserve Field labels/errors, Base UI semantics/refs, tokens and CSP.
- [x] Diagnostics never gate normal Send/Resume or require authentication each app
      launch. Successful check results are session observations; persist only
      preferences and required recovery evidence. Invoke no login/logout, credential
      extraction/editing, config writes, thread or turn.
- [x] Controlled Rust/DOM checks cover the working setup path, primary error states,
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

## Comments

### 2026-11-13 — 06b implemented, reviewed and controlled checks passed

Implemented on `feature/m01-codex-setup` from 06a's completed
`c5130794eabf8273c1650e7cb3f52b958f30fd1a`. Final runtime source:
`77f84c81291d44299f705103d380debc8f5114b5`; the commit containing this comment
adds the final evidence. See [contracts, budgets, provenance, review and results](../../../docs/evidence/milestone-01-codex-setup.md)
and [private-state supervisor fixture instructions](../../../docs/evidence/milestone-01-codex-setup/fixture-instructions.md).

Core now hosts the concrete runtime owner with one bounded worker, shared
production/fixture protocol interpretation, persisted executable save/reset,
configuration/action/check identity, wire protocol 6 and SQLite schema 4. Settings
provides real Save/reset/check controls, precise pending/error/stale/recovery
observations and scoped official-document opening/copy feedback. Unsaved drafts
discard on leaving Settings; retained action identity survives navigation and
reconnect. Rust resolves durable action identity before fresh admission and can
reconstruct conflicting-payload rejection after restart without replay.

The final configured gate passed **140 Rust + 25 tooling + 113 UI** checks,
formatting, lint, TypeScript and Clippy. Required warm measurements and production
app packaging passed. Independent final re-review: **Standards 0, Spec 0 unresolved**;
initial/follow-up findings and their fixes are preserved in evidence.

Checked acceptance describes implementation and controlled decision/DOM evidence.
Actual fixture desktop/process, WebView accessibility/CSP/responsiveness, SQLite
durability/migration, kernel ownership/lease/descendant cleanup and native-provider
qualification are **not run for this source**; 06d owns those results. Native
provider processes/submissions, login changes and runtime configuration changes:
**zero**. Parent 06, 06c–06d and the milestone remain open. Stop after this child.


### 2026-11-13 — Scoped recovery corrections after independent review

Addressed the three missed 06b cases: lease contention without a quarantined
handle, confirmed cleanup followed by final persistence failure, and cleanup
reconciliation targeting mutable active observations. The runtime worker now
retains exact check ownership through cleanup and completion persistence, with
readback before completion-only retry and distinct cleanup/storage evidence.
Controlled regressions also retain queued-request admission and bounded shutdown
error classification. See the appended [06b correction evidence](../../../docs/evidence/milestone-01-codex-setup.md#06b-recovery-correction--2026-11-13).

The correction preserves public contracts/schema and predecessor results. The
unrelated research file is excluded from the fix. Broader 06c coverage and all 06d
qualification remain open; no native processes or submissions were performed.
Final checks and source identity accompany the correction evidence.


Final correction gate: **149 Rust + 25 tooling + 113 UI**, all configured checks
passed. Required warm measurements and production app packaging passed.
Independent final re-review: **Standards 0, Spec 0 unresolved**. Exact source
hashes, command timings, logs and staged-sidecar provenance are linked in the
correction evidence. Actual desktop/process/native qualification remains unperformed.
