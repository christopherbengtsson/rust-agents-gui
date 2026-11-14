# Routed observations and retained presentation context

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
