# Provider-independent workspaces

Status: ready-for-agent

Agreed 2026-11-15 during the Codex connection architecture discussion. This
publication documents the cleanup; it implements no product changes.

## Problem statement

A workspace is shared file/branch context, but its current record stores a
provider string, creation hardcodes Codex and frontend semantic validation rejects
other values. That accidental ownership conflicts with several provider-bound
conversations sharing one workspace. Retire it before connection work extends the
model, preserving current-build creation/recovery behavior and historical
qualification evidence. Earlier development application state needs no migration.

## Solution

Keep workspace creation, reopening, inspection and recovery provider-independent.
Show path and branch as workspace facts; provider identity belongs to a future
conversation or an explicitly targeted setup destination. Keep the milestone's
Codex default in admission/presentation instead of workspace identity.

## User stories

1. As a developer, I want workspaces created with the current model to reopen with
   their identity, files and recovery status intact.
2. As a developer, I want newly created workspaces to remain usable by separate
   conversations from different providers without recreating the checkout.
3. As a developer, I want workspace navigation and change inspection to work
   independently of any provider's installation or authentication state.
4. As a maintainer, I want provider restrictions enforced by the selected feature's
   admission policy rather than persisted workspace ownership.

## Implementation decisions

- `workspaces` retains creation/reconciliation policy and identity; `storage` owns
  current record access. Remove the provider field from the current workspace
  model, newly written records and exported observation contract. Do not replace
  it with an enabled-provider list or another workspace-level provider selector.
- Apply the global [MVP development compatibility policy](../../docs/MVP.md#development-compatibility-policy):
  change the current record/contract directly, using fresh private application
  state and documenting reset/restart steps where required. Add no old-format
  reader, migration or dual representation. Current-build reopen/recovery still
  retains repository/workspace/action identities, pinned bases, names, branches,
  paths, reservations, partial artifacts, status and cleanup uncertainty.
- Remove the Codex-only workspace semantic check through the generated-contract
  preparation owner and existing feature-observation seam. Provider-specific
  setup links may explicitly target Codex for this milestone; they do not assign
  the workspace to it. Future `conversations` owns provider/native mappings.
- Keep current Git/base/naming/storage-driver decisions. This scoped record/contract
  cleanup precedes and does not consume
  [06a storage evaluation](../milestone-01/issues/06a-evaluate-storage-before-durable-chat.md).

## Verification and completion

Follow [verification policy](../../docs/agents/verification.md) and
[observation guidance](../../docs/agents/observations.md). Deterministic units cover
current record decoding, creation/recovery decisions and workspace observation
validation with controlled storage/filesystem/Git outcomes. Contract generation
and read-only drift are separate preparation gates. Separately qualify new
creation and current-build restart/reconciliation in the actual fixture desktop
through the production supervisor with fresh private disposable state. Record
failed/unperformed checks; start no provider process, login, thread or turn.

## Ticket and downstream gates

The single [implementation ticket](issues/01-retire-workspace-provider.md) depends
on milestone 02a and foundation F08. It gates
[connection 01](../codex-connection/issues/01-provider-readiness-and-diagnostics.md)
and [durable chat 07](../milestone-01/issues/07-durable-streamed-chat-and-uncertain-send-recovery.md).
The connection feature retains its four separately selected sessions.

## Out of scope

Provider implementations, login or chat, enable switches, multiple-account/profile
management, a provider registry/framework, storage-driver/journal redesign and live
provider qualification. Read-only inspection of archived evidence does not grant
permission to touch archived state.

## Comments

2026-11-15: The user clarified that all Milestone 01 work is early development;
backward compatibility with previous development builds is unnecessary. Removed
old-record migration/retention acceptance. Current-build recovery and external
provider compatibility requirements remain; no state was reset by this publication.
