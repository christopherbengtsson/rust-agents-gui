# 04d: Migrate native transport and subscription ownership

Status: ready-for-agent

Blocked by: [F04c](04c-admission-and-framed-exchange.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read the
[owning requirements](../spec.md#native-request-scheduling) and predecessor evidence. Read the MVP,
milestone, verification policy and applicable repository engineering guidance.
[Observation guidance](../../../docs/agents/observations.md) owns shared engineering
pitfalls; the parent spec owns behavior and limits. This ticket is one session;
successors remain separate. Preserve unrelated edits and completed evidence.

## Module and caller interface

Connection owns owner binding, discovery, physical work and subscription resources.
Request and subscription callers share F04c's internal framed-exchange module;
Tauri forwards typed targeted work without learning framing or recovery policy.
Rust domain interfaces remain authoritative and independent of Tauri.

## Implementation and removal

- Move desktop client, status/discovery exchanges, supervisor listener/handlers and
  feed together to protocol 8. Update application corpus/framing compatibility checks
  while retaining prior corpus/evidence. Reject incompatible peers before dispatch;
  preserve visible incompatibility and no restart/replay on established connection loss.
- Supervisor main continuously drives one current-thread Tokio runtime with I/O/time;
  desktop uses Tauri's existing runtime. Core constructs no runtime. Keep exclusive
  discovery and private-access checks behind substituted effects and off rendering.
- Bound blocking filesystem/Git/SQLite/domain work before submission; permits remain
  in physical workers through cancellation. Replace detached connection threads with
  owned task completion. Keep short state locks outside I/O and consistent lock order.
- Replace native subscription thread/Condvar/clone-shutdown mechanics with owned
  async task, CancellationToken and awaited cleanup. Cancellation wakes all relevant
  waits. Keep exact acknowledgements, deadlines and one in-flight delivery.
- Adapt journal notification without losing publication-before-wait or resync changes;
  retain the concrete replay journal and its bounded semantics. Subscription stop is
  idempotent and exact-ID targeted; account for outstanding setup before replacement.
- Remove old socket polling, synchronous framing and superseded worker/control code.
  Keep necessary blocking domain/process implementations and test dependency seams.

Footholds: `connection.rs`, `connection/subscription.rs`, `observations.rs`,
`crates/supervisor/src/main.rs`, `src-tauri/src/main.rs`, development-state/sidecar
scripts and explicitly gated supervisor process qualification callers. Follow the
predecessor's new module rather than duplicating framing in client and feed.

## Required scenarios

- Wrong-owner/version requests produce no domain effect; status alone learns owner;
  discovery is exclusive; old-owner loss cannot start a competitor or replay an action.
- Partial/lost replies followed by Busy/owner/local rejection retain uncertainty.
- Stalled exchanges leave control/status capacity; dropped waiters retain permits;
  handlers/subscriptions join after stop, cancellation, errors and late attachment.
- Exact ack, malformed/oversized frame, absolute deadline, sink failure, EOF,
  publication-before-wait, journal overflow/resync and accepted-vs-status cursor cases.
- Units substitute I/O/time/process/storage. Update process qualification without
  moving it into units. Preserve protocol-specific development state and binary provenance.

## Acceptance

- [ ] Both peers and feed use the shared protocol-8 module; obsolete synchronous transport mechanics are removed.
- [ ] Explicit runtime/task ownership and bounded blocking dispatch preserve owner/delivery/publication guarantees.
- [ ] Relevant configured static/unit/corpus/tooling gates and measurements pass with replacement coverage mapped.
- [ ] Evidence separates controlled results from actual qualification still required in F04g.

Frontend contracts stay compatible with F04b; no provider or storage migration.
Full owned Quit remains later milestone work.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.
