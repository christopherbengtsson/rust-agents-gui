# 07: Durable streamed chat and uncertain-send recovery

**What to build:** Send multiline text in concurrent workspace conversations, follow exactly routed replies/tool activity, preserve native work through WebView replacement, and recover the durable prefix after native failure without replay.

**Blocked by:** [SQLx 08 — Qualified native/storage foundation](../../sqlx-architecture/issues/08-qualify-foundation-and-publish-chat-handoff.md); [04 — Isolated workspaces](04-create-isolated-workspaces-with-recoverable-retries.md); [connection 04 — Qualified connection journey](../../codex-connection/issues/04-qualify-connection-journey.md) and [parent 06 aggregate](06-connect-codex-with-actionable-setup-diagnostics.md); [provider-independent workspaces 01](../../provider-independent-workspaces/issues/01-retire-workspace-provider.md).

**Status:** ready-for-agent

## Current architecture handoff — 2026-11-16

The [final native-core/SQLx spec](../../sqlx-architecture/spec.md) and
[amendments](../../sqlx-architecture/amendments.md) supersede older supervisor,
bridge and single-active instructions, including dated comments below. Read its
requirements, process failure domain, chat writer policy, provisional settings and
Q6 gates. SQLx 08 must provide checked current implementation/evidence before this
ticket starts; publication alone is not a storage handoff. 06a/research remain
historical inputs; 06b is retired, not completed. Preserve old evidence.

Use [verification policy](../../../docs/agents/verification.md): substitute effects
in application units and qualify real SQLx/process/Tauri separately. Inspect the
supported chat schemas/runtime and record fixture provenance before extending the
adapter. Readiness fixtures reject thread/turn/approval methods; they cannot simply
be relabelled chat qualification. This ticket authorizes no live submissions.

The workspace-provider prerequisite is also inherited through connection 01.
Consume its dated current-contract
handoff alongside the connection and 06a evidence before editing.

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A4; WebView continuity and native-prefix recovery in amended A8; exact concurrent creation/send identity, admission and reconciliation in amended A10.

- [ ] Build the transcript, tool-activity presentation and multiline composer within the established workspace layout using appropriate shadcn composition patterns and shared tokens. Give conversation observations and frequent streaming updates a focused owner near the chat UI. Keep the selected workspace/conversation and saved/received/uncertain delivery states legible.
- [ ] Keep composer input and workspace navigation usable during streaming. Localize changing-response updates, bound mounted history as needed, and preserve readable scrolling/catch-up feedback without rebuilding unrelated shell views on every text update.
- [ ] Provide a usable desktop chat/composer and streamed text/tool/error observations through direct native-hosted core actions and bounded validated observations.
- [ ] Persist separate workspace, application/native conversation, turn, action, and process identities; retain native mapping and durable transcript history outside project files.
- [ ] Bind each conversation to its provider/native mapping independently of workspace and inspected Providers tab. Reuse provider-connection readiness/credential policy and concrete adapter seams. Codex is this milestone's execution provider; no single-active conversation rule or fixed repository/workspace/conversation/agent cap is permitted.
- [ ] Demonstrate two active conversations sharing workspace A while another runs in B. Retain exact action/turn/event targeting and duplicate native-conversation exclusion; show shared-file overlap risk. Use workspace-owned activity reservations and generation/fence rules, never a global active sentinel.
- [ ] Save action identity/intent before effects and distinguish local durability, provider receipt, and unknown outcome in authoritative observations and the desktop.
- [ ] Reconcile uncertain initial conversation creation and sends before potentially duplicate effects. Repeated action IDs retrieve recorded status/outcome; desktop reconnection retrieves status rather than resubmitting.
- [ ] Route duplicate/late events to exact conversation/turn identities and preserve provider-specific outcome evidence; never infer completion from silence.
- [ ] Bound output, page history, and keep navigation/control responsive under sustained fixture output. Provide the targeted action interface later interrupt behavior uses.
- [ ] Consume the completed [SQLx 08 handoff](../../sqlx-architecture/issues/08-qualify-foundation-and-publish-chat-handoff.md). Persist transcript events in the same authoritative database as action/turn metadata, advancing indexed per-conversation durable positions with confirmed event batches. Keep presented/received/durable/feed positions distinct; reconciliation follows physical writer settlement and a fresh snapshot. Local history remains independent of provider history availability.
- [ ] Implement the spec's small private writer policy: per-conversation FIFO, nested ready-conversation/workspace rotation with bounded batches and alternating urgent intent/output while both wait. Count/byte/time defaults are provisional. Bound queued/in-flight/frame/decode/driver/result copies explicitly; do not introduce per-historical-workspace queues or a generic scheduler.
- [ ] Use one Codex app-server process per independently active native conversation, no process for history viewing. Keep protocol routing separate from bulk persistence waits and bound its control/frame capacity. On saturation stop/fence that connection with explicit gap/uncertainty and durable prefix; retain exact cleanup ownership. Shared storage failure identifies/fences all affected work. Do not claim safe selective backpressure or independent control from a reserved send slot alone.
- [ ] Qualify provider-output pressure through the production conversation/storage path: use bounded backpressure where supported, otherwise expose capacity failure and the retained durable prefix through the owning module's stop/reconciliation policy. Preserve exact conversation/turn routing and complete accounting; silently dropping already-admitted output is failure. Record grouping/budget defaults, offered/admitted/committed/failed/outstanding work and navigation/history/control latency under sustained and overloaded fixtures. The measured 16-event policy and single 64-event diagnostic are starting evidence, not accepted defaults or a capacity guarantee.
- [ ] Before enabling sends, record the supported runtime/schema and concrete frame/item/input limits with oversized-item behavior and fixture provenance. Unit-test delayed/lost replies, duplicate/late events, worker panic, storage faults, admission/fence races, capacity failure and routing under controlled time/events. Distinguish rejected admission from unavailable replies after admission.
- [ ] Separately qualify actual SQLx/process/fixture desktop for three-conversation streaming, quiet-conversation progress in both workspaces, slow writer, overload/prefix/gap, WebView replacement without replay and native failure/reopen. Record navigation/history/control metrics and offered/admitted/committed/failed/outstanding work. This is preliminary Q6 streaming evidence; full approvals/interrupt/exact-resume pressure qualification waits for 08–12 and actual provider topology waits for separately authorized 13.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

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

### 2026-11-16 — bounded async storage comparison handoff

The [completed comparison](../../../docs/research/storage-async-architecture-comparison.md)
selects a maintained async rusqlite bridge, explicit physical-completion ownership
and WAL/FULL. Implement and qualify [06b](06b-prepare-storage-for-durable-chat.md)
before starting this ticket. Consume its actual storage interface rather than
recreating worker/pool/queue mechanics in conversations or Tauri. Hierarchical
synthetic streams do not qualify provider turn/subagent mappings; grouping/fairness
and real output backpressure still require their owning-module implementation and
qualification. All chat acceptance remains unchecked.

### 2026-11-16 — storage architecture review handoff clarified

06b now explicitly owns shared storage composition, physical completion/outcome
retention, writer-settlement ordering and whole-core shutdown qualification. This
ticket consumes those guarantees and owns concrete conversation event grouping,
fairness, accepted durable positions and provider-output pressure behavior. Before
qualifying chat, declare the sustained fixture trace and acceptance targets against
the [MVP responsiveness requirements](../../../docs/MVP.md#mvp-scope-and-observable-acceptance).
Report overload handling separately; rejected offered synthetic events do not
qualify dropping output from an already-admitted provider execution. Retain the
comparison's original results and limits. This amendment implements no chat or
storage code and authorizes no live-provider submission.

### 2026-11-16 — accepted native-core and SQLx handoff

The current acceptance and blockers above replace supervisor hosting, maintained
rusqlite-bridge and one-active policies in earlier comments. The final spec fixes
transaction retirement, reservation/admission ordering, per-active-conversation
process isolation and private bounded writer policy. SQLx 08 supplies the actual
qualified foundation; 07 supplies the first streaming pressure evidence; 12/13
close cumulative fixture/native concurrency separately. All acceptance remains
unchecked. No production change or provider allowance is claimed.
