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

- [x] Publish source-linked findings with realistic write/read workload and measured tradeoffs.
- [x] Compare dependency/binary/memory/runtime costs and define what each durability result establishes.
- [x] Recommend driver, connection and journal/checkpoint choices, including whether a cache database is currently justified.
- [x] Record current-build durability/recovery implications, required development reset/restart steps and unresolved qualification; propose implementation tickets if changes are warranted. Earlier development formats need no migration or compatibility support.

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
This ticket keeps its existing 06a identifier, research scope and position before 07. The split completes no storage research or migration.

### 2026-11-15 — research and disposable qualification completed

The [source-linked evaluation](../../../docs/research/milestone-01-storage-evaluation.md)
and [reproducible evidence](../../../docs/evidence/milestone-01-storage/README.md)
record baseline `24390a1e7ba108ca88704fbf73deac92915d3ab6`, supported engines,
frozen workloads, exact commands, resource costs, raw results, failed construction
attempts and qualification limits. Both standalone driver harnesses completed
86 short workload samples plus two queue/completion probes; all reported integrity
checks passed with zero SQL errors. The final native cohort passed 24 sampled
process-kill points, busy-commit/rollback, checkpoint/backup, file-publication and
synthetic shared-context/continuation checks. These exercise real storage
primitives and synthetic transaction shapes, not newly implemented application
recovery or native-provider behavior.

Retain rusqlite and recommend WAL/FULL/fullfsync, intentional bounded writer
ownership, time/size-bounded grouping of streamed events, indexed database history,
initial database attachment content and no separate cache database. Global
16-event/25 ms grouping sustained the offered 500 events/second across 25 synthetic
chats in the short cohorts; these measurement points impose no product cap and
do not establish long-soak capacity. Different bundled SQLite versions remain an
explicit driver-comparison limit. Power loss, hardware flush honesty, actual
production responsiveness and the selected implementation remain unqualified.

[Proposed 06b](06b-prepare-storage-for-durable-chat.md) needs triage and a separately
selected implementation session; settle its sequencing with 07. No production
storage, schema, dependencies or contracts changed, so this research requires no
application reset/restart. Later incompatible changes use fresh private state
after owned stop/join, with no old-format migration requirement. No application
units, full application checks/build/package, fixture desktop, authentication or
provider submissions were run. Existing evidence and the pre-existing
`docs/IDEAS.md` edit were preserved; ticket 07 remains unimplemented.

### 2026-11-16 — genuine async and hierarchical workload follow-up

The user separately selected the
[bounded async architecture comparison](../../../docs/research/storage-async-architecture-comparison.md)
after the source-selection research and subagent workload discussion. It compares
maintained async SQLx and rusqlite implementations, including synthetic workspace,
root/child streams and future transaction/read probes. Both accepted the baseline;
the original heavy grouping policy overloaded both. A single larger-group pair
improved results without fully accepting hot-workspace output and is not repeated
capacity qualification. Exact raw evidence, source-backed maintenance differences
and failed/unperformed adoption gates remain explicit.

That follow-up selects bundled rusqlite with maintained tokio-rusqlite workers,
preserving current opening behavior. [06b](06b-prepare-storage-for-durable-chat.md)
is now prepared for its own implementation session before 07, with no production
migration or new provider allowance in this research. Earlier checked research
acceptance and evidence remain their historical cohorts.
