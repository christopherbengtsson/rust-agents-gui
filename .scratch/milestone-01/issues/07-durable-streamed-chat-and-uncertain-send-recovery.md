# 07: Durable streamed chat and uncertain-send recovery

**What to build:** Send multiline text in a workspace, follow streamed replies/tool activity in the correct conversation, and reconnect after desktop loss without replaying uncertain work.

**Blocked by:** 04: Create isolated workspaces with recoverable retries; 06: Connect Codex with actionable setup diagnostics

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A4; active-work desktop reconnect in A8; creation/send identity, admission, and reconciliation in A10.

- [ ] Build the transcript, tool-activity presentation and multiline composer within the established workspace layout using appropriate shadcn composition patterns and shared tokens. Keep the selected workspace/conversation and saved/received/uncertain delivery states legible.
- [ ] Keep composer input and workspace navigation usable during streaming. Localize changing-response updates, bound mounted history as needed, and preserve readable scrolling/catch-up feedback without rebuilding unrelated shell views on every text update.
- [ ] Provide a usable desktop chat/composer and streamed text/tool/error observations through the core interface and real supervisor connection.
- [ ] Persist separate workspace, application/native conversation, turn, action, and process identities; retain native mapping and durable transcript history outside project files.
- [ ] Admit one active conversation application-wide as a temporary policy, without permanent one-conversation-per-workspace storage restrictions; reject duplicate native-conversation ownership.
- [ ] Save action identity/intent before effects and distinguish local durability, provider receipt, and unknown outcome in authoritative observations and the desktop.
- [ ] Reconcile uncertain initial conversation creation and sends before potentially duplicate effects. Repeated action IDs retrieve recorded status/outcome; desktop reconnection retrieves status rather than resubmitting.
- [ ] Route duplicate/late events to exact conversation/turn identities and preserve provider-specific outcome evidence; never infer completion from silence.
- [ ] Bound output, page history, and keep navigation/control responsive under sustained fixture output. Provide the targeted action interface later interrupt behavior uses.
- [ ] Unit-test send/routing/recovery decisions for delayed or lost replies, duplicate/late events, storage failures and reconnect observations. Distinguish non-delivery from admitted work with an unavailable reply; verify intent-before-send and catch-up without resubmission. Separately demonstrate active-work desktop disconnect/reconnect through the production supervisor path and retain unit/desktop evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
