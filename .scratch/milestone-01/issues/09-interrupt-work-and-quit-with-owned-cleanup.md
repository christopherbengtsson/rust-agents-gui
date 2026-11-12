# 09: Interrupt work and quit with owned cleanup

**What to build:** Interrupt the exact active turn and explicitly quit with trustworthy owned cleanup, durable flush, and supervisor shutdown observations.

**Blocked by:** 07: Durable streamed chat and uncertain-send recovery

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A7 and explicit-Quit portion of A8. Integration with pending approvals is verified in the complete workflow.

- [ ] Send interruption against an explicit active conversation/turn identity, separately from runtime stop; delayed requests/replies cannot interrupt a later turn.
- [ ] Show provider-observed interrupted/completed/failed/unknown outcomes without treating silence or an unqualified exit alone as success.
- [ ] Verify owned tool work stops when cleanup is confirmed; expose failed/unacknowledged interruption and unconfirmed descendant cleanup with actionable recovery.
- [ ] Explicit Quit requests owned stop, durable flush, and supervisor shutdown before reporting success; storage/cleanup failures remain visible rather than falsely confirming a clean shutdown.
- [ ] Hide/minimize and the agreed close-to-hide behavior retain admitted work. Desktop crash remains distinct from explicit Quit.
- [ ] Keep interruption mechanics/process ownership in runtimes and execution policy in conversations; saved PIDs alone grant no signaling authority.
- [ ] Unit-test exact-turn interruption, late replies, interruption failure and shutdown/flush decisions with controlled outcomes. Separately qualify actual owned fixture-tool cleanup and successful Quit/reopen through desktop controls; retain distinct unit/desktop evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
