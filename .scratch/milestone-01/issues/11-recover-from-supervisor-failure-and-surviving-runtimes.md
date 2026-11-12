# 11: Recover from supervisor failure and surviving runtimes

**What to build:** Recover authoritative history and execution observations after supervisor failure, reconciling surviving runtimes before admitting conflicting work.

**Blocked by:** 08: Answer exact pending approvals; 10: Read history and explicitly resume the exact conversation

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** Supervisor-failure portion of A8 and surviving-owner/action reconciliation in A10; regressions across A4, A5, A7, and A9.

- [ ] Kill the actual supervisor during meaningful fixture execution/action windows and restore the durable history prefix with interrupted/uncertain observations matching available evidence.
- [ ] Reconcile surviving runtime ownership and pending sends, approvals, and resumes inside the owning Rust modules. Keep unknown outcomes visible; never invent receipt, completion, permission, or safe cleanup.
- [ ] Block conflicting execution while surviving work is uncertain; reconnect/status retrieval grants no authority to repeat effects or signal a process based only on saved PIDs.
- [ ] Verify repeated action IDs still retrieve authoritative recorded status/outcomes across recovery; preserve operation-level workspace creation reconciliation already established.
- [ ] Expose recoverable/fenced states and storage/recovery failures in the desktop, keeping local history readable to the extent of its durable prefix.
- [ ] Use actual supervisor and surviving fixture runtime/tool processes, temporary repositories, and private state. Cover failure before/after durable writes and runtime effects, stale ownership evidence, and failed persistence without touching unrelated processes.
- [ ] Retain reproducible process/core regression checks and desktop recovery evidence; synthetic survival/failure behavior is not native Codex qualification.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
