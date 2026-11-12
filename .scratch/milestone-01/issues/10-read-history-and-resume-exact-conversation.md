# 10: Read history and explicitly resume the exact conversation

**What to build:** Reopen the application to readable local history without starting work, then explicitly resume the saved native conversation or receive a clear exact-resume failure.

**Blocked by:** 09: Interrupt work and quit with owned cleanup

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A9, history/restart portion of A8, and resume reconciliation in A10.

- [ ] After normal Quit/reopen, restore repository/workspace identity, readable durable messages, turn/action evidence, and native conversation mapping without starting or replaying a turn.
- [ ] Read recent history through a bounded/paged interface independent of native provider history availability.
- [ ] An explicit Resume targets the saved native conversation and fixture evidence demonstrates recall of an earlier marker.
- [ ] Persist resume intent, return recorded results for repeated action IDs, and reconcile uncertain/lost resume outcomes before any potentially duplicate effect.
- [ ] Unavailable/deleted native history leaves local history readable and explains why exact resume failed. Starting a new conversation is a distinct explicitly identified action, never an automatic fallback.
- [ ] Unit-test restored records, no-effect history reads, exact mapping, unavailable native history, lost resume replies and storage failures. Separately demonstrate desktop restart/resume states and fixture marker recall; retain distinct unit/desktop evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
