# 04: Implement push delivery

Status: ready-for-agent

Blocked by: [F03](03-bound-request-scheduling.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#rust-invalidation-journal) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

The current coarse revision is not a complete changefeed. Registration and workspace
admission and some fault-overlay changes do not increment it. Publish target hints
from the owning Rust transitions, including ambiguous potentially committed writes;
a hint requests observation and never asserts action success.

Use the plan's bounded in-memory journal and acknowledged subscription. Its cursor
orders invalidation hints; it does not stamp separately read feature data as atomic
or serve as durable conversation history.

- Implement typed target tags and audit publication for every transition enumerated in the parent plan, not only existing change() calls.
- Keep publication under owning-state protection with one lock order; subscribers never call domain modules under the journal lock, and delivery performs no I/O under those locks.
- Implement owner-bound subscribe/catch-up, sampled watermarks, ResyncRequired, exact subscription/batch/cursor acknowledgements, persistent framing buffers, and separate frame/heartbeat/ack deadlines.
- Keep one native subscription and at most one frontend batch in flight. Frontend acceptance records dirty state before acknowledgement; acknowledgement does not wait for refetches.
- Separate status cursor from frontend-accepted feed cursor. Implement stop/socket shutdown/wake/join and generation-safe late completion; stalled delivery stops visibly.

Baseline source footholds:

- [crates/core/src/lib.rs](../../../crates/core/src/lib.rs) — owner/snapshot and core composition.
- [crates/core/src/workspaces.rs](../../../crates/core/src/workspaces.rs) — registration, availability and registration faults.
- [crates/core/src/workspaces/creation.rs](../../../crates/core/src/workspaces/creation.rs) — admission, stages/results and workspace faults.
- [crates/core/src/workspaces/inspection.rs](../../../crates/core/src/workspaces/inspection.rs) — inspection job/preview transitions.
- [crates/core/src/connection.rs](../../../crates/core/src/connection.rs) — subscription protocol and bounded framing.
- [src-tauri/src/main.rs](../../../src-tauri/src/main.rs) — desktop channel and subscription resource ownership.

## Required scenarios

- Publication covers admissions, ambiguous writes, availability, creation/recovery stages, faults and inspection/preview transitions.
- Missing/expired/future cursor, journal/batch overflow and changes during resync recover without silently dropping invalidation.
- No lost wakeups, exact acknowledgement matching, one in-flight batch, controlled deadlines, EOF/cancellation/late-attach cleanup.
- Reconnect uses the accepted cursor; owner replacement never starts a competing owner or replays actions.

Use [repository verification policy](../../../docs/agents/verification.md) and
the existing gate. Preserve prior useful assertions; actual desktop/process evidence
remains separate from controlled units.

## Acceptance

- [x] The journal and delivery enforce every parent-plan count/byte/time bound.
- [x] Repository/workspace progress is the first exercised workload through the native/Tauri subscription seam.
- [x] Snapshot/catch-up and stalled-consumer behavior preserve bounded work and visible uncertainty.
- [x] Controlled lifecycle/protocol/publication checks pass; new desktop qualification remains explicitly pending.

## Handoff boundary

Preserve working baseline observations until F05 proves push equivalence and removes polling. Keep this journal in memory;
[Milestone 01 ticket 07 — Durable streamed chat and uncertain-send recovery](../../milestone-01/issues/07-durable-streamed-chat-and-uncertain-send-recovery.md)
later adds separate durable conversation delivery semantics, not tokens in this journal.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.


### 2026-11-14 — F04 implemented, reviewed and verified

Confirmed F03's checked acceptance and dated evidence at
`b7c9b8907f8f06e7cd43a17a9328c15e0f7ba61b`, continuing the existing feature branch.
The supervisor now publishes typed invalidation hints to the bounded in-memory journal;
protocol 7 supplies owner-bound catch-up, sampled resync, exact acknowledgements,
persistent framing, independent deadlines and owned cancellation/cleanup. Tauri admits
one native subscription and one frontend batch. The app records dirty state before
acknowledgement and reconnects from its accepted feed cursor, independently of status.

The user selected dirty-state acceptance as this ticket's frontend boundary. Existing
feature reads and polling remain for F05. No Query/Zustand/routing migration or provider
work is included. The unrelated `docs/IDEAS.md` edit remains preserved/excluded.

[Evidence](../../../docs/evidence/frontend-navigation/f04.md) records source identity,
publication coverage, shared corpus/selected tests, failed attempts, independent
Standards/Spec review and corrected cleanup findings. The complete gate passed
140 Rust + 25 tooling + 119 frontend checks. Required measured runs and separate
frontend bundling passed. Repository/workspace progress traversed the controlled
subscription and mounted mocked-Tauri channel seam; no actual desktop result is claimed.

Actual socket/process/fixture-desktop/accessibility/latency, packaging and native-provider
qualification remain unperformed and separate. No provider submissions. Prior milestone
evidence is preserved. F05 is next and remains unimplemented; stop after F04.
