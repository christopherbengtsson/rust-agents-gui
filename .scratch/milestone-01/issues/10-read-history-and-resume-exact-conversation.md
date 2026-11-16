# 10: Read history and explicitly resume the exact conversation

## Native core amendment — 2026-11-16

Apply the [accepted native-core/SQLx spec](../../sqlx-architecture/spec.md) and
[amendments A2/A3/A8](../../sqlx-architecture/amendments.md) before this ticket.
They replace separate-supervisor/native-crash continuation and single-active
instructions with native ownership, WebView continuity, durable native-failure
recovery and the concurrent three-conversation scenario. Preserve exact controls,
remaining milestone scope and historical evidence. Acceptance remains unchecked;
this publication authorizes no live-provider or authentication action.

**What to build:** Reopen the application to readable local history without starting work, then explicitly resume the saved native conversation or receive a clear exact-resume failure.

**Blocked by:** 09: Interrupt work and quit with owned cleanup

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A9, history/restart portion of A8, and resume reconciliation in A10.

- [ ] Restore readable history in the established navigation/transcript layout with bounded history-loading controls and visible conversation identity. Clearly distinguish observing saved history, explicitly resuming it, failed/unavailable native resume and the separate new-conversation action, using shared controls and status patterns.
- [ ] After normal Quit/reopen, restore repository/workspace identity, readable durable messages, turn/action evidence, and native conversation mapping without starting or replaying a turn.
- [ ] Read recent history through a bounded/paged interface independent of native provider history availability.
- [ ] An explicit Resume targets the saved native conversation and fixture evidence demonstrates recall of an earlier marker.
- [ ] Persist resume intent, return recorded results for repeated action IDs, and reconcile uncertain/lost resume outcomes before any potentially duplicate effect.
- [ ] Unavailable/deleted native history leaves local history readable and explains why exact resume failed. Starting a new conversation is a distinct explicitly identified action, never an automatic fallback.
- [ ] Unit-test restored records, no-effect history reads, exact mapping, unavailable native history, lost resume replies and storage failures. Separately demonstrate desktop restart/resume states and fixture marker recall; retain distinct unit/desktop evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

2026-11-16: Reconciled with the native-core/SQLx spec and amended A8/A10/concurrency
acceptance. Existing dependency edges remain; milestone 07 now inherits SQLx 08.
Qualification stays open and no production implementation is claimed.
