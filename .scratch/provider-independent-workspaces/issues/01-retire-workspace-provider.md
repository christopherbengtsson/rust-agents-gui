# 01: Retire workspace-level provider ownership

**What to build:** Create, reopen, inspect and recover workspaces without assigning
them to Codex or another provider. Update the current record/contract/desktop path
directly, retaining current-build persistence and visible recovery state.

**Blocked by:** [Milestone 02a — UI foundation](../../milestone-01/issues/02a-establish-incremental-ui-foundation.md);
[F08 — Frontend qualification](../../frontend-navigation/issues/08-qualify-frontend-foundation.md)

**Status:** ready-for-agent

## Start here

Read the [spec](../spec.md), [MVP ownership](../../../docs/MVP.md#starting-architecture),
[Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[observation guidance](../../../docs/agents/observations.md). Inspect actual
workspace allocation, stored-record encoding, generated contracts, frontend
semantic checks and consumers before editing. Use the applicable repository-local
Rust/React skills and installed Intent guidance; consult the component guide for
any presentation changes. Confirm prerequisite evidence and preserve its limits.
Apply the milestone's [development compatibility policy](../../../docs/MILESTONE-01.md#development-compatibility-policy);
earlier development records/contracts need no migration or backward support.

## Acceptance

- [x] Remove workspace-level provider identity from the current domain model, new/reconciled record writes and exported observations. Remove the hardcoded Codex allocation and frontend Codex-only semantic restriction. Introduce no replacement workspace provider selector/list or speculative provider framework.
- [x] Update the current stored record and generated/native contracts together through the existing preparation owner. Use fresh private application state for incompatible development changes and record reset/restart steps; add no old-record migration, compatibility reader or dual format. Incompatible running peers fail explicitly, retaining supervisor ownership and physical-cleanup rules.
- [ ] Creating and reopening a workspace, listing it, changing selection and viewing Changes work through the production supervisor/desktop path without provider availability or authentication. Display path/branch as workspace facts; show any later provider identity with its conversation. Explicit milestone setup links may still target Codex without deriving that target from a workspace provider field.
- [x] Preserve creation admission/reconciliation, name/folder reservations, original-checkout protection and pending/failed/uncertain recovery behavior. Provider defaults remain feature admission/presentation choices. Add no conversation, sign-in, provider probe/process or runtime configuration mutation.
- [x] Passing deterministic units exercise current record serialization/decoding, allocation/recovery invariants and workspace observation decoding with controlled storage/filesystem/Git outcomes. Retain real serializers, validation and owning policy; identify relevant replaced assertions. Run generated preparation/drift and relevant static checks separately under repository verification policy. Old-format compatibility-only assertions need no replacement; retain coverage of still-required application decisions.
- [ ] Separately qualify actual fixture-desktop new creation, current-build quit/reopen, change inspection and a retained pending/failed recovery case through the production path. Use fresh disposable state/checkouts; confirm current-build identities, branch/path and reservations survive restart/reconciliation. Record actual persistence/desktop limits separately from units and leave failed/unperformed acceptance open.
- [x] Append dated source identity, exact commands/results, evidence and any required development reset/restart steps here. Hand off the provider-independent workspace contract to connection 01 and conversation ticket 07 without implementing either successor. Preserve historical evidence and stop after this ticket.

## Handoff boundary

This prefactor gates [connection 01](../../codex-connection/issues/01-provider-readiness-and-diagnostics.md)
and [durable chat 07](../../milestone-01/issues/07-durable-streamed-chat-and-uncertain-send-recovery.md).
The [connection spec](../../codex-connection/spec.md#architecture-and-extension-constraints)
owns provider connection architecture. The later 06a ticket still researches
storage choices; this ticket changes only the scoped record/contract meaning.

## Comments

2026-11-15: Published after the user agreed the provider architecture and requested
documentation of the hardcoded Codex cleanup. All implementation acceptance remains
unchecked; no code, storage, generated artifact or runtime state changed here.

2026-11-15: Removed prior-format record compatibility and migration requirements
following the user's milestone-wide early-development policy. Qualify the current
model using fresh state; retain current-build persistence, recovery and cleanup.
This amendment resets no development data and changes no application code.

### 2026-11-15 — implementation and automated qualification; desktop deferred

Implemented on `main` from `aba401363fac07d88799f2b62c4e089c6b63b51a`.
[Evidence and separate-session handoff](../../../docs/evidence/provider-independent-workspaces/01.md)
record exact commands/results, source/artifact hashes, fixture provenance, coverage
mapping, generated preparation/drift, warm unit measurements, production build and
independent Standards/Spec review. Workspace records/observations have no provider,
frontend validation and the hardcoded Codex badge are removed, and protocol 9
rejects incompatible peers before dispatch. No connection/chat successor work,
provider process, authentication or runtime configuration mutation was introduced.

The user explicitly instructed: “Defer actual desktop qualification to a separate
session. Finish implementation, automated checks, build, review and commit now.
Leave desktop acceptance unchecked and document the handoff.” Actual fixture-desktop
creation/reopen/Changes and retained recovery therefore remain unperformed and their
acceptance items are unchecked. The selected follow-up case is failed base
resolution after name/branch/path reservation (ambiguous local main/master with no
remote default), quit/reopen, then explicit reconciliation of the same workspace
after fixing the disposable repository's base ambiguity. A worktree-command failure
produces uncertain status and is not the selected failed case. The evidence provides reset/restart steps: fresh private protocol-9 state,
matching owner cleanup and no old-record migration. Earlier evidence is preserved.
This ticket and its downstream connection 01/conversation 07 qualification gate
remain open until that separate desktop session passes. Stop after this ticket.
