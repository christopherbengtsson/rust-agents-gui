# 11: Recover from native failure and surviving runtimes

## Native core amendment — 2026-11-16

Apply the [accepted native-core/SQLx spec](../../sqlx-architecture/spec.md) and
[amendments A2/A3/A8](../../sqlx-architecture/amendments.md) before this ticket.
They replace separate-supervisor/native-crash continuation and single-active
instructions with native ownership, WebView continuity, durable native-failure
recovery and the concurrent three-conversation scenario. Preserve exact controls,
remaining milestone scope and historical evidence. Acceptance remains unchecked;
this publication authorizes no live-provider or authentication action.

**What to build:** Recover authoritative history and execution observations after native Tauri failure, reconciling surviving runtimes before admitting conflicting work.

**Blocked by:** 08: Answer exact pending approvals; 10: Read history and explicitly resume the exact conversation

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** Native-failure portion of amended A8 and surviving-owner/action reconciliation in A10; regressions across A4, A5, A7, and A9. The filename is a retained tracker link, not a supervisor requirement.

- [ ] Present interrupted/uncertain ownership, blocked execution and storage/recovery failures as persistent, readable shell/conversation states with core-authorized recovery actions. Keep local history accessible where available; a transient toast alone must not be the only place to inspect unresolved recovery state.
- [ ] In desktop qualification, kill the actual native Tauri process during meaningful fixture execution/action/commit windows. Reopen to a new core incarnation and restore the durable prefix with interrupted/uncertain outcomes; distinguish this from WebView replacement attaching to the still-live incarnation. No uninterrupted native-crash execution promise.
- [ ] Reconcile surviving runtime ownership and pending sends, approvals, and resumes inside the owning Rust modules. Keep unknown outcomes visible; never invent receipt, completion, permission, or safe cleanup.
- [ ] Block conflicting execution while surviving work is uncertain; reconnect/status retrieval grants no authority to repeat effects or signal a process based only on saved PIDs.
- [ ] Verify repeated action IDs still retrieve authoritative recorded status/outcomes across recovery; preserve operation-level workspace creation reconciliation already established.
- [ ] Expose recoverable/fenced states and storage/recovery failures in the desktop, keeping local history readable to the extent of its durable prefix.
- [ ] Unit-test failure before/after recorded writes and runtime effects, stale ownership evidence and failed persistence using retained mocked records/outcomes. Separately qualify surviving fixture runtime/tool behavior through actual native-hosted Tauri with disposable repositories/private state and the three-conversation topology. Saved PIDs and leader exit alone do not authorize signaling or prove descendant cleanup.
- [ ] Retain reproducible unit regression checks and separate desktop recovery evidence; synthetic survival/failure behavior is not native Codex qualification.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

2026-11-16: Reconciled with the native-core/SQLx spec and amended A8/A10/concurrency
acceptance. Existing dependency edges remain; milestone 07 now inherits SQLx 08.
Qualification stays open and no production implementation is claimed.
