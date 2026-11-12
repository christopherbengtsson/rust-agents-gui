# 08: Answer exact pending approvals

**What to build:** Inspect an actual pending operation and answer allow-once or deny from the desktop, with strict matching and visible uncertainty after a lost reply.

**Blocked by:** 07: Durable streamed chat and uncertain-send recovery

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A5 and approval-related A10. Native allow/deny success remains a separate qualification requirement.

- [ ] Show the actual requested operation and available scope before answering; bind answers to the exact provider request, conversation, turn, operation, and scope.
- [ ] Reject malformed payloads, stale requests, and conflicting answers while keeping exact operation matching strict.
- [ ] Persist approval action intent before the effect; repeated action IDs return recorded status/outcome and uncertain replies trigger core reconciliation without widened permissions.
- [ ] Keep approval validity/reconciliation in conversations and protocol mechanics in runtimes. The desktop never substitutes a different request or retries uncertain answers on its own.
- [ ] In desktop qualification with grounded protocol fixtures, demonstrate an allowed operation actually occurs and a denied operation does not; label synthetic recovery faults separately from captured native behavior.
- [ ] Unit-test approval decisions for delayed/duplicate requests, stale/conflicting answers, lost replies and storage failures; separately demonstrate desktop approval/error states and retain evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
