# 03: Admit concurrent workspaces and atomic reservations

Status: ready-for-agent

Blocked by: [02 — SQLx storage and settlement](02-establish-sqlx-storage-and-settlement.md).

## What to build

Allow independent workspace operations to progress concurrently while registration,
first-creation continuation and readable saved locations remain atomic and immutable.
Large retained histories must not require load-all name/folder scans.

## Context and ownership

Read [admission](../spec.md#core-lifetime-and-task-ownership),
[reservation](../spec.md#atomic-bounded-reservation), [amendment A7](../amendments.md#a7--workspace-invariants-and-development-state),
[ADRs 0003](../../../docs/adr/0003-shared-local-repository-identity.md),
[0004](../../../docs/adr/0004-immediate-workspaces-with-durable-onboarding.md),
[0005](../../../docs/adr/0005-readable-workspace-reservations.md),
[MVP](../../../docs/MVP.md), [milestone](../../../docs/MILESTONE-01.md) and
[verification](../../../docs/agents/verification.md). Use rust-engineering.

Own workspace async admission/activity/reconciliation and its typed storage claims.
Continue 02's integration branch; 04 is the combined compile/qualification gate.
Do not reintroduce the old driver or global active-workspace authority. Preserve
other edits and original worktrees/branches.

## Acceptance

- [x] Await shared storage on the supplied runtime with no synchronous state lock across awaits. A short workspace decision gate covers duplicate ID/payload, cross-kind/pending capacity, generation/fence and atomic registration/first-creation intent, without holding across Git or joins.
- [x] Retained activity records replace the single active-workspace sentinel. Independent repository effects dispatch concurrently; actual common-Git-directory mutations serialize, with keyed exclusion retained through physical work/waiters.
- [x] Indexed bounded candidates and one typed atomic claim resolve global repository-folder and per-repository name collisions. Two unrelated repositories named app choose distinct valid folders; same-repository creations get distinct names without lost saved intent.
- [x] Bound attempts per turn and persist/yield a checked continuation under the same action; no history/cardinality cap or load-all reservations/numeric tiers. Bound startup/pending and filesystem enumeration too. No filesystem/Git wait inside database transactions.
- [x] A failed/unacknowledged claim follows writer settlement and same-action reconciliation before another candidate. Once committed, name/branch/path never change on retry; external collisions remain explicit recovery at the exact reserved location.
- [x] Capture and revalidate generation/identity around awaited work, including stale completion, Quit versus admission and activity retirement. Root guards settle/unwind without escaped operations or gate-order cycles.
- [x] Replace useful workspace coverage with substituted units for all above cases and original pinned-base, partial worktree recovery and no unrelated deletion behavior. Record which former format-only tests retire.
- [ ] Separately qualify real SQLx claims and concurrent disposable Git/worktree effects; observe bounded metadata through the incumbent desktop path when the combined branch is runnable. Final combined pass is 04, not a fictional standalone green checkpoint.

## Verification and limits

Q2/Q3 use controlled storage/filesystem/Git outcomes in application units; real
database/Git effects are separate qualification. Inspection remains read-only and
provider-independent. Record provisional limits, commands, source and open gates.
No provider submissions or user-state cleanup.

## Comments

2026-11-16: Published as the workspace checkpoint of 02–04; acceptance is unchecked.

2026-11-16: Implemented ticket 03 on `sqlx-storage-cutover`, runtime source `3ac0bc5`. [Evidence, coverage map and two-axis review](../../../docs/evidence/sqlx-architecture/03/README.md) retain 86 passing substituted Rust units, focused Clippy, eight actual disposable workspace/SQLx/Git checks, 14 actual SQLx cases, TypeScript and 51 tooling / 260 mocked-IPC UI tests. Both fresh explicitly selected GPT-6.1 (`gpt-6.1-sol`) High reviewers report zero unresolved findings after corrections. Failed probes, source/binary hashes and measurements remain recorded. The final mixed acceptance item remains unchecked because incumbent desktop observation is blocked by ticket 04's two provider-store callers; its real database/Git portion passed. Full combined compilation, generated drift completion, full-core units, desktop/package/native-host and remaining physical-fault qualification are not claimed. Q2/Q3, ticket 04, parent and milestone remain open. No live-provider, authentication, user-state reset, merge or later-ticket work occurred.
