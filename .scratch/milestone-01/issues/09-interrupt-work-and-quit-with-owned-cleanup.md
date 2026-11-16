# 09: Interrupt work and quit with owned cleanup

## Native core amendment — 2026-11-16

Apply the [accepted native-core/SQLx spec](../../sqlx-architecture/spec.md) and
[amendments A2/A3/A8](../../sqlx-architecture/amendments.md) before this ticket.
They replace separate-supervisor/native-crash continuation and single-active
instructions with native ownership, WebView continuity, durable native-failure
recovery and the concurrent three-conversation scenario. Preserve exact controls,
remaining milestone scope and historical evidence. Acceptance remains unchecked;
this publication authorizes no live-provider or authentication action.

**What to build:** Interrupt an exact active turn independently of other conversations and explicitly quit with trustworthy whole-core cleanup and durable flush observations.

**Blocked by:** 07: Durable streamed chat and uncertain-send recovery

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A7 and explicit-Quit portion of A8. Integration with pending approvals is verified in the complete workflow.

- [ ] Apply the native-core spec's retained whole-core shutdown: close root admission, fence workspace/provider starts, settle admitted-start guards, drain every affected conversation/process, settle storage while open, close pools/feed/host and release the lease last. Collect all stage failures; repeated Quit/deadline observes the same continuing owner and cannot fabricate success. Keep future restart/update behind completed drain.
- [ ] Target interrupt independently across two conversations sharing a workspace and a third elsewhere, with one hot stream and a slow writer. Reserved control submission alone is not receipt/completion. Qualify observed interrupt/stop or explicit pending/unknown cleanup without silently stopping unrelated execution; retain exact overload failure scope for final ticket 12.

- [ ] Place exact-turn interrupt controls and their pending/outcome feedback in the established conversation UI. Keep explicit Quit discoverable and show stopping/flushing/shutdown progress or failure clearly while retaining recoverable context; use shared controls and status patterns.
- [ ] Send interruption against an explicit active conversation/turn identity, separately from runtime stop; delayed requests/replies cannot interrupt a later turn.
- [ ] Show provider-observed interrupted/completed/failed/unknown outcomes without treating silence or an unqualified exit alone as success.
- [ ] Verify owned tool work stops when cleanup is confirmed; expose failed/unacknowledged interruption and unconfirmed descendant cleanup with actionable recovery.
- [ ] Explicit Quit requests owned stop, durable flush, and complete native core shutdown before allowing Tauri exit; storage/cleanup failures remain visible rather than falsely confirming a clean shutdown.
- [ ] Hide/minimize and the agreed close-to-hide behavior retain admitted work. Desktop crash remains distinct from explicit Quit.
- [ ] Keep interruption mechanics/process ownership in runtimes and execution policy in conversations; saved PIDs alone grant no signaling authority.
- [ ] Unit-test exact-turn interruption, late replies, interruption failure and shutdown/flush decisions with controlled outcomes. Separately qualify actual owned fixture-tool cleanup and successful Quit/reopen through desktop controls; retain distinct unit/desktop evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

2026-11-16: Reconciled with the native-core/SQLx spec and amended A8/A10/concurrency
acceptance. Existing dependency edges remain; milestone 07 now inherits SQLx 08.
Qualification stays open and no production implementation is claimed.
