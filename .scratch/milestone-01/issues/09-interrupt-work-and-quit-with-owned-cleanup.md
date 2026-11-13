# 09: Interrupt work and quit with owned cleanup

**What to build:** Interrupt the exact active turn and explicitly quit with trustworthy owned cleanup, durable flush, and supervisor shutdown observations.

**Blocked by:** 07: Durable streamed chat and uncertain-send recovery

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A7 and explicit-Quit portion of A8. Integration with pending approvals is verified in the complete workflow.

- [ ] Place exact-turn interrupt controls and their pending/outcome feedback in the established conversation UI. Keep explicit Quit discoverable and show stopping/flushing/shutdown progress or failure clearly while retaining recoverable context; use shared controls and status patterns.
- [ ] Send interruption against an explicit active conversation/turn identity, separately from runtime stop; delayed requests/replies cannot interrupt a later turn.
- [ ] Show provider-observed interrupted/completed/failed/unknown outcomes without treating silence or an unqualified exit alone as success.
- [ ] Verify owned tool work stops when cleanup is confirmed; expose failed/unacknowledged interruption and unconfirmed descendant cleanup with actionable recovery.
- [ ] Explicit Quit requests owned stop, durable flush, and supervisor shutdown before reporting success; storage/cleanup failures remain visible rather than falsely confirming a clean shutdown.
- [ ] Give explicit Quit one supervisor-owned lifecycle decision path. Fence new
      mutations before stopping admitted conversation/runtime work and existing
      workspace/Git workers. Confirm required persistence, owned cleanup and worker
      completion before releasing the owner and reporting successful shutdown.
- [ ] Preserve observable progress/failure and necessary ownership after cleanup,
      persistence or worker-completion failure. Repeated/lost Quit requests retrieve
      the same shutdown attempt's status/outcome. Socket disappearance, a client
      timeout or a saved PID establishes neither successful shutdown nor authority
      to start a competing owner. Destructor cleanup remains a fallback.
- [ ] Add typed lifecycle actions/observations through core, supervisor, Tauri and
      TypeScript validation, updating protocol compatibility when contracts change.
      Keep turn interruption distinct from whole-runtime/application stop; owning
      modules retain their own cleanup and recovery decisions.
- [ ] Hide/minimize and the agreed close-to-hide behavior retain admitted work. Desktop crash remains distinct from explicit Quit.
- [ ] Keep interruption mechanics/process ownership in runtimes and execution policy in conversations; saved PIDs alone grant no signaling authority.
- [ ] Unit-test exact-turn interruption, late replies, interruption failure and shutdown/flush decisions with controlled outcomes. Separately qualify actual owned fixture-tool cleanup and successful Quit/reopen through desktop controls; retain distinct unit/desktop evidence.
- [ ] Cover admission fencing, stop/flush ordering, partial failures, retained
      handles and late/repeated shutdown requests with controlled outcomes.
      Hand pending-approval-during-Quit qualification to ticket 12 after tickets
      08 and 09 converge; this ticket does not implicitly allow or deny approvals.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
