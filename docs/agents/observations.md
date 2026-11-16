# Observation and transport engineering

## Native core amendment — 2026-11-16

The [accepted native-core/SQLx spec](../../.scratch/sqlx-architecture/spec.md) and
[supersession amendments A4/A8](../../.scratch/sqlx-architecture/amendments.md) govern
native hosting, core incarnation, application transport deletion and preparation applicability.
They replace conflicting instructions below for subsequent work. Remaining
policy and recorded checkpoint results retain their scope; publication claims
no production change or new qualification pass.

Read this when changing or reviewing native IPC, generated contracts, feature observations or
feed/inspection lifetime. The [frontend spec](../../.scratch/frontend-navigation/spec.md)
owns behavior/limits/session order; [ADR 0007](../adr/0007-library-backed-observation-modules.md)
records rationale; [verification policy](verification.md) owns test substitution and
qualification. This guidance supports those owners rather than duplicating them.

## Current versus planned

The historical F04 product at `7dc4eac` used protocol 7, synchronous transport,
manual read/feed controllers and baseline polling. F04a published documentation.
F04b–F04f implemented the selected libraries; [F04g evidence](../evidence/frontend-navigation/f04g.md)
records focused actual transport/fixture-desktop qualification. F05 remains
presentation context/integration and F08 retains cumulative routed qualification. Check predecessor evidence and actual files before assuming a library,
command, generated artifact or actor exists. Land dependencies with first use;
recheck installed Intent guidance and supported-version sources on upgrades.

## Deep modules and shared mechanisms

Keep feature-specific interfaces for repository, workspace and inspection
observations. Hide Query keys, validators, dirty generations and interests from
screens. Shared framing, physical reads, preparation and common assertions are
implemented once; feature rules remain local. Composition constructs app-scoped
owners and adapters without becoming a second policy owner. Avoid a public
observation framework or one forwarding module per library/command.

The deletion test checks depth: removing a useful module redistributes its decisions
among callers. Keep existing inspection/feed module responsibilities while replacing
their internals. The interface includes ordering, errors and lifetimes, not merely
method names. Use owning-module internal seams where tests need controlled outcomes;
the repository permits them even with only one production adapter. Replace useful
coverage before removing old orchestration tests and record the mapping.

## Contract preparation

Rust Serde wire types are authoritative. Generate declarations and structural
validators from them; keep named contextual assertions beside feature observations.
An optional application `codegen` feature enables the exporters. Select pure
contract checks and actual generation/drift gates through the
[verification policy](verification.md#generated-contracts-and-preparation).

F04b creates one preparation owner, `scripts/prepare-generated.mjs`, with explicit
`--write` and `--check` modes exposed as `generated:write` and `generated:check`.
These are planned commands, not present at F04a. The check computes expected managed
outputs without repairing committed files. Use it before dev/check/build/Tauri/CI
can generate or consume assets; F06 adds routes to the same owner. Avoid repeating
application typechecking or full verification inside preparation. Preserve Cargo
target claims and sidecar provenance; generated files have no manual formatter owner.

Use explicit ts-rs Config and Schemars serialization settings for reply schemas.
Ajv2020 and registered UUID formats match the chosen dialect. Generate standalone
ESM at tooling time, preserving Tauri CSP. Do not equate JSON Schema character
length with UTF-8 bytes, or Rust u64 with JavaScript-safe integers. Check flatten,
`Result`, defaults/omissions/null and custom deserialization against actual serialized
corpus shapes. Schema validity does not establish current owner or target authority.

## Physical completion and cancellation

Native Semaphore admission precedes task creation/blocking submission; owned permits
stay in actual work through completion. Core domain/process/storage decisions remain
behind their effect seams. Tokio's started blocking work is not stopped by aborting
an async waiter. Runtime ownership belongs to supervisor main or Tauri, not a newly
created runtime per core request. State locks stay short and outside socket I/O.

One internal framed-exchange module hides persistent buffering, strict EOF and
absolute deadlines. Use the Rust skill's [framed-I/O reference](../../.agents/skills/rust-engineering/references/framed-io.md)
for codec/buffer mechanics and cancellation reasoning. Keep owner/delivery/reconciliation policy in connection/domain
modules. A cancelled or timed-out partial transmission closes the affected connection
and retains uncertainty; restarting a future is not safe replay. Strict EOF must cover
a complete length header followed by no payload, not just a nonempty trailing buffer.

Query or actor cancellation can settle logical work before Tauri IPC ends. Keep the
small unsettled physical registry below them; it holds work identity/promises rather
than observation results. With p-queue, forward abort only while waiting and detach
it at task start. Running abort/timeouts can otherwise free a slot too early. Physical
keys include operation flags; cache keys describe the observation and comparison,
avoiding separate permanent caches for observing versus explicitly refreshing it.

## Cache, actor and presentation ownership

Query alone owns validated observations and last-good data. Feature modules own
freshness generations and visible/pending-action interests. Feed acceptance records
invalidations synchronously before acknowledging, independent of refetch settlement.
Resync and hidden-target obligations survive refreshing only one active interest.
Use [TanStack guidance](tanstack.md) for local IPC defaults and Router integration.

Feed and inspection actors own coordination, IDs and resource handles, not copies
of Query observation bodies. Views select narrow snapshots through feature interfaces;
Zustand holds allowlisted presentation context rather than mirrored actor snapshots.
An actor's stopping/draining state awaits outstanding setup/physical cleanup before
replacement. View cleanup releases an interest rather than stopping the app owner.
Keep UI errors and cleanup uncertainty visible. Full owned Quit and durable
conversation delivery remain separately qualified milestone responsibilities.

Use the upstream [XState v5 skill](../../.agents/skills/xstate-v5/SKILL.md) for
machine modeling and API guidance. Apply its store-selection and snapshot-persistence
examples within the ownership above: these actors remain ephemeral and presentation
persistence stays allowlisted. The repository's verification policy governs required
checks even when upstream discusses tests only on request.

XState [actions](https://stately.ai/docs/xstate/v5/actions), including entry/exit
actions, are fire-and-forget; returning a promise does not delay a transition.
Model awaited setup and draining as [invoked work](https://stately.ai/docs/invoke)
with completion/failure transitions while the coordinating machine is still active.
Callback disposers and actor stop are not asynchronous completion barriers; retain
physical handles/promises with their owner until actual cleanup settles.

## Source and qualification discipline

Use the selected library's official supported-version docs/source, not generic
latest examples. [Research](../research/frontend-navigation-library-refactor.md)
records fit/maintenance evidence and pitfalls. A maintained library can justify
removing complex infrastructure without a performance bottleneck; measure app
performance and dependency costs before claiming an improvement. F04g qualifies
actual transport/fixture desktop behavior before F05. F08 qualifies the cumulative
routed foundation. Unit, actual fixture and native-provider results remain separate.
