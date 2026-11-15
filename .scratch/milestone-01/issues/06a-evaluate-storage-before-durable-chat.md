# 06a: Evaluate storage before durable chat

Status: ready-for-agent

Blocked by: [Connection 04 — Qualify the complete connection journey](../../codex-connection/issues/04-qualify-connection-journey.md)

## Start here

Perform this research after connection 01–04 and their
[parent 06 aggregate](06-connect-codex-with-actionable-setup-diagnostics.md) complete, and before
[07: Durable streamed chat](07-durable-streamed-chat-and-uncertain-send-recovery.md).
Read the [storage checkpoint](../../frontend-navigation/spec.md#storage-research-checkpoint),
[MVP](../../../docs/MVP.md), [verification policy](../../../docs/agents/verification.md)
and [Conductor comparison](../../../docs/research/conductor-frontend-architecture.md#comparison-with-our-process-architecture).
Rust-owned persistence remains an accepted contract.
Consume the earlier [workspace-provider cleanup handoff](../../provider-independent-workspaces/issues/01-retire-workspace-provider.md)
when assessing the current model. That bounded record/contract cleanup is a prerequisite
to connection 01, not a driver/journal redesign deferred to this research. Preserve
provider-independent workspace identity and conversation-owned provider mappings in
any recommendation.
Apply the milestone's [development compatibility policy](../../../docs/MILESTONE-01.md#development-compatibility-policy):
recommend the best current storage design without requiring migrations or readers
for earlier development formats. Use fresh disposable state for experiments.

## Investigation

Inspect the current [storage module](../../../crates/core/src/storage.rs) and the
actual supported SQLite/driver versions. Compare rusqlite and SQLx inside the Rust
core, WAL and rollback journaling, connection scheduling/contention, transcript
batching/history paging, sync/checkpoint/recovery/backup, and atomicity across
metadata and transcript/attachment files. Assess future disposable disk-cache
separation against a concrete workload rather than copying a database count.

Use official supported-version sources and representative disposable data. Keep
real storage experiments/benchmarks separate from deterministic units and record
versions, profiles, commands, durability semantics, costs and qualification limits.
Installed Conductor code may supply configuration evidence; user database contents,
credentials and provider history are outside this ticket.

## Acceptance

- [ ] Publish source-linked findings with realistic write/read workload and measured tradeoffs.
- [ ] Compare dependency/binary/memory/runtime costs and define what each durability result establishes.
- [ ] Recommend driver, connection and journal/checkpoint choices, including whether a cache database is currently justified.
- [ ] Record current-build durability/recovery implications, required development reset/restart steps and unresolved qualification; propose implementation tickets if changes are warranted. Earlier development formats need no migration or compatibility support.

## Handoff boundary

Research does not authorize a driver/journal/layout migration, frontend SQL access,
a live-provider submission or ticket-07 implementation. Record the recommendation
before starting durable chat so it can govern that ticket's persistence design.

## Comments

2026-11-14: Agreed checkpoint after 06 and before 07. Foundation implementation
publishes this research ticket without changing storage behavior.

2026-11-15: Parent 06 was split into the separate
[connection feature sessions](../../codex-connection/spec.md#session-sequence).
Connection 04 now explicitly gates this research and records the parent aggregate.
This ticket keeps its existing 06a identifier, research scope and position before
07. The split completes no storage research or migration.
