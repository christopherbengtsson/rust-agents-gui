# 07: Durable streamed chat and uncertain-send recovery

**What to build:** Send multiline text in a workspace, follow streamed replies/tool activity in the correct conversation, and reconnect after desktop loss without replaying uncertain work.

**Blocked by:** [06a: Storage evaluation](06a-evaluate-storage-before-durable-chat.md); 04: Create isolated workspaces with recoverable retries; [connection 04: Qualified connection journey](../../codex-connection/issues/04-qualify-connection-journey.md); [provider-independent workspaces 01](../../provider-independent-workspaces/issues/01-retire-workspace-provider.md)

**Status:** ready-for-agent

The workspace-provider prerequisite is also inherited through connection 01.
Consume its dated current-contract
handoff alongside the connection and 06a evidence before editing.

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A4; active-work desktop reconnect in A8; creation/send identity, admission, and reconciliation in A10.

- [ ] Build the transcript, tool-activity presentation and multiline composer within the established workspace layout using appropriate shadcn composition patterns and shared tokens. Give conversation observations and frequent streaming updates a focused owner near the chat UI. Keep the selected workspace/conversation and saved/received/uncertain delivery states legible.
- [ ] Keep composer input and workspace navigation usable during streaming. Localize changing-response updates, bound mounted history as needed, and preserve readable scrolling/catch-up feedback without rebuilding unrelated shell views on every text update.
- [ ] Provide a usable desktop chat/composer and streamed text/tool/error observations through the core interface and real supervisor connection.
- [ ] Persist separate workspace, application/native conversation, turn, action, and process identities; retain native mapping and durable transcript history outside project files.
- [ ] Bind each conversation to its provider and native mapping independently of the workspace and inspected Providers tab. Reuse the agreed provider-connection module and concrete adapter seam; do not restore a workspace provider field or infer execution targeting from setup presentation. Keep multiple provider-bound conversations representable while enforcing this milestone's Codex/one-active-conversation limits in admission/presentation.
- [ ] Admit one active conversation application-wide as a temporary policy, without permanent one-conversation-per-workspace storage restrictions; reject duplicate native-conversation ownership.
- [ ] Save action identity/intent before effects and distinguish local durability, provider receipt, and unknown outcome in authoritative observations and the desktop.
- [ ] Reconcile uncertain initial conversation creation and sends before potentially duplicate effects. Repeated action IDs retrieve recorded status/outcome; desktop reconnection retrieves status rather than resubmitting.
- [ ] Route duplicate/late events to exact conversation/turn identities and preserve provider-specific outcome evidence; never infer completion from silence.
- [ ] Bound output, page history, and keep navigation/control responsive under sustained fixture output. Provide the targeted action interface later interrupt behavior uses.
- [ ] Unit-test send/routing/recovery decisions for delayed or lost replies, duplicate/late events, storage failures and reconnect observations. Distinguish non-delivery from admitted work with an unavailable reply; verify intent-before-send and catch-up without resubmission. Separately demonstrate active-work desktop disconnect/reconnect through the production supervisor path and retain unit/desktop evidence.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

### 2026-11-14 — library refactor handoff

Consume the foundation's qualified async transport, generated-contract preparation,
feature observations and app-lifetime coordination. Extend owning modules for
conversation observations rather than duplicating a scheduler/cache/lifecycle.
Durable conversation events require their own retention, paging and accepted cursor;
the in-memory invalidation journal and its feed cursor cannot substitute for them.
Rust conversation policy owns durable sends/reconciliation; XState remains presentation
coordination and no frontend library retries uncertain actions automatically.

### 2026-11-15 — connection session dependency clarified

[Parent 06](06-connect-codex-with-actionable-setup-diagnostics.md) now indexes
connection 01–04 rather than a single implementation session. Confirm connection
04's checked acceptance, parent aggregate and dated adapter/cohort evidence along
with 06a's storage recommendation before starting this ticket. Reuse those real
and fixture adapter interfaces for chat; their setup/authentication evidence does
not qualify native-agent sends, approvals or resume. This updates dependencies
only; durable chat remains unimplemented.

### 2026-11-15 — provider-independent conversation handoff

Consume the [connection architecture](../../codex-connection/spec.md#architecture-and-extension-constraints)
and completed workspace-provider cleanup. Connection setup readiness/login cleanup
stay with the provider-connection owner; durable sends, turns, approvals and exact
resume stay with `conversations`. Additional provider implementations remain future
work; this amendment implements no chat or multi-provider execution.
