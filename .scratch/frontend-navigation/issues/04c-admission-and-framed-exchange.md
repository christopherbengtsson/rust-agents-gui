# 04c: Deepen admission and framed exchange

Status: ready-for-agent

Blocked by: [F04b](04b-generate-contracts-and-validators.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read the
[owning requirements](../spec.md#shared-framed-exchange) and predecessor evidence. Read the MVP,
milestone, verification policy and applicable repository engineering guidance.
[Observation guidance](../../../docs/agents/observations.md) owns shared engineering
pitfalls; the parent spec owns behavior and limits. This ticket is one session;
successors remain separate. Preserve unrelated edits and completed evidence.

## Module and caller interface

Keep the public connection module's owner/discovery/delivery responsibilities.
One private framed-exchange module hides codec configuration, persistent buffering,
first-byte/absolute deadline policy and terminal cancellation for both short exchanges
and subscriptions. Its interface exposes complete frame outcomes, not raw poll or
codec configuration. Do not move domain routing/reconciliation into framing.

## Implementation and removal

- Replace ConnectionSlots' atomic counter/drop implementation with Tokio Semaphore
  and owned permits. Keep nonwaiting acquisition before physical task creation,
  exact lane budgets and Busy/NotSent behavior. Preserve the admitted-work seam.
- Establish the internal async module using tokio-util length-delimited framing:
  four-byte big-endian u32 prefix, 65,532-byte maximum payload. Delegate buffers
  and ordinary encoding/decoding to the crate, with a small strict-EOF/progress adapter.
- Centralize two-phase absolute read deadlines, bounded writes and cancellation.
  Buffered data begins transfer immediately; fragments never reset the budget.
- Accept controlled byte-stream/time adapters. Keep parsers and deadline decisions
  real in tests. Use minimal explicit Tokio/tokio-util features and already locked
  compatible support crates when direct imports require them.

Footholds: `crates/core/src/connection/{admission.rs,tests.rs}`, framing helpers at
the end of `connection.rs`, and persistent callers in `connection/subscription.rs`.
The old production framing remains until F04d. The new module is the tested kernel
for that immediate migration, not a second configurable transport architecture.
F04d must remove superseded polling/framing after both callers migrate.

## Required scenarios

- Existing lane isolation, error/unwind/unstarted-drop cleanup and abandoned-waiter
  cases retain physical admission until completion.
- Pure codec checks cover exact/over ceiling, oversized advertised length before
  body allocation, split/coalesced frames, partial header, complete nonzero header
  with zero body, partial body and clean frame-boundary EOF.
- Controlled async streams/time cover buffered next frames, first-byte transition,
  original acknowledgement cap, slow fragments, partial writes and terminal cancel.
  Close instead of restarting incomplete I/O. No actual sockets or real waits in units.

## Acceptance

- [x] Semaphore replaces custom permit counting while all admission decisions remain covered.
- [x] One private module hides frame/deadline mechanics and passes controlled scenarios through its owning interface.
- [x] Production protocol-7 behavior still works pending F04d; temporary old/new code is explicitly mapped for removal there.
- [x] Relevant static/unit gates and required measurements pass with failures/limits recorded.

No runtime host, production protocol switch, actor or Query migration is included.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.


### 2026-11-14 — F04c implementation complete

Confirmed F04b's checked acceptance and linked evidence at `b96b743`, continued
`feature/m01-frontend-navigation` with a clean working tree/index, and preserved
prior evidence and both protocol corpora. Replaced custom admission accounting
with Tokio owned semaphore permits while retaining nonwaiting physical-work
admission and lane budgets. Added one private, normally compiled framed-exchange
kernel with codec-owned buffers, strict EOF, absolute two-phase read/write bounds
and terminal cancellation/abandoned-write cleanup. Production remains protocol 7;
F04d's caller migration and old-code removal are explicitly mapped and unstarted.

[Evidence](../../../docs/evidence/frontend-navigation/f04c.md) records pinned
versions/features, source hashes, red/green regressions, gate selection, retained
coverage, failures/limits and warm measurements. Final `corepack pnpm verify`
passed: preparation/static checks and 162 Rust + 40 tooling + 126 frontend units.
The 18 new kernel checks use controlled in-memory streams and paused time.
Independent review found Standards 0 / Spec 0 findings. Only generated contract
manifest provenance changed; declarations/schemas/validators remain identical.

Actual sockets, process/fixture desktop, packaging, hosted CI and native-provider
qualification were not performed; no provider submissions occurred. F04d and all
successors, the foundation and whole milestone remain open. The containing commit
owns this ticket's changes and evidence. Stop after F04c.
