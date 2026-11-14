# Shared mechanisms inside deep observation modules

Accepted 2026-11-14; implementation is F04b–F04g, not this documentation publication.
Amends the transport and observation implementation choices in
[ADR 0006](0006-routed-observation-presentation.md); its Router/Query/Zustand and
Rust-authority decisions remain valid.

Use maintained established libraries inside cohesive modules: Tokio/Semaphore and
one internal framed-exchange module for native I/O; Rust-generated declarations
and standalone structural validators; Query plus shared physical scheduling for
feature observations; XState for feed/inspection coordination; Zustand for allowed
presentation memory. Feature interfaces hide mechanics and retain domain decisions
locally. Shared code eliminates repeated mechanisms without requiring callers to
configure a generic observation host.

Choose protocol-8 length-prefixed frames and explicit async runtime/task ownership
to replace handwritten polling, framing and thread coordination. Maintenance and
robustness justify this change without a demonstrated throughput problem. This
costs new dependencies, generation tooling, migration and actual qualification;
existing unit evidence establishes neither their compatibility nor performance.
Feed acceptance owns delivery/cursor ordering, while feature modules own dirty
generations and Query freshness. Actor coordination, Query records and Zustand
preferences have distinct ownership, with no mirrored caches.

Keep owner matching, action uncertainty, authoritative persistence/reconciliation
and execution in the owning Rust modules. Preserve physical work through logical
cancellation. Qualify actual sockets and fixture desktop behavior before F05, then
qualify the cumulative routed foundation in F08. The
[spec](../../.scratch/frontend-navigation/spec.md) owns requirements/session order;
[research](../research/frontend-navigation-library-refactor.md) owns source evidence
and rejected alternatives. Historical F01–F04 results remain unchanged.
