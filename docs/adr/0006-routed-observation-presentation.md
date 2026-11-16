# Routed observations and retained presentation context

## Native core amendment — 2026-11-16

The [accepted native-core/SQLx spec](../../.scratch/sqlx-architecture/spec.md) and
[supersession amendments A4](../../.scratch/sqlx-architecture/amendments.md) govern
separate host, supervisor-owner and socket/peer-specific decisions.
They replace conflicting instructions below for subsequent work. Remaining
policy and recorded checkpoint results retain their scope; publication claims
no production change or new qualification pass.

Use TanStack Router for current navigation, Query for shared asynchronous observations,
and Zustand for retained presentation context. Keep feature decoding/freshness behind
cohesive feature interfaces and app-lifetime resource ownership above routed layouts.
This supports Conductor-style workspace/settings navigation without multiplying
selection authorities or tying pending actions to visible pages.

The supervisor's Rust core remains the persistence/execution/reconciliation owner.
Targeted exchanges bind to the caller-observed supervisor owner. Bounded native
request lanes and an acknowledged invalidation journal provide push progress with
snapshot recovery; the journal is not durable conversation history. This adds
transport/cache coordination in exchange for explicit ownership and bounded work.
Conductor demonstrates the library combination, not our Rust ownership contract.

The user chose Zustand over Jotai for workspace-keyed presentation records and
explicit actions; no measured performance advantage is established. External native
deep links are deferred. Storage research follows milestone ticket 06 and precedes
07, without authorizing frontend SQL access or a database migration. See the
[complete agreed plan](../../.scratch/frontend-navigation/spec.md) and
[research](../research/conductor-frontend-architecture.md). F01 prepares documentation
and dependencies only; F02–F08 implement/qualify the decision.

## Amendment — 2026-11-14

[ADR 0007](0007-library-backed-observation-modules.md) amends transport and observation
implementation choices after F04: async library mechanisms, generated contracts,
feature-local freshness and scoped XState coordination. The original product and
ownership rationale remains. F04a publishes the amendment; F04b–F04g implement and
qualify it before revised F05. Earlier completion statements describe their recorded
checkpoint, not current dependency or implementation state.
