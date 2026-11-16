# Milestone 01 implementation spec — one usable Codex desktop workflow

## Native core amendment — 2026-11-16

The [accepted native-core/SQLx spec](../sqlx-architecture/spec.md) and
[supersession amendments A2/A3/A8](../sqlx-architecture/amendments.md) govern
host/lifecycle, one-active admission, concurrency deferral, acceptance and ticket dependencies.
They replace conflicting instructions below for subsequent work. Remaining
policy and recorded checkpoint results retain their scope; publication claims
no production change or new qualification pass.

Status: ready-for-agent

Implementation baseline: tickets 01–05, including 02a, are complete for their recorded fixture scope. Tickets 06–13 and whole-milestone qualification remain incomplete. The [frontend foundation spec](../frontend-navigation/spec.md) defines the prerequisite implementation and qualification sequence.

## Problem Statement

Christopher needs a local desktop workspace for coding with his existing Codex runtime and authentication. The first usable workflow must isolate code changes, show agent activity and requested approvals, retain readable history, and recover from failures without silently repeating uncertain work.

## Solution

Build one Apple Silicon macOS desktop workflow: register an existing Git repository, create a separate branch/worktree at an automatically identified local default commit, chat with Codex, allow or deny an exact requested operation, inspect uncommitted changes, interrupt work, quit/reopen, and explicitly resume the same native conversation.

Use a Tauri desktop connected to a separate on-demand local supervisor hosting the Rust core. The supervisor owns execution and authoritative storage. The desktop submits explicitly targeted actions and displays observations. Viewing history and reconnecting never start or replay a turn.

Deliver this as one implementation spec with session-sized implementation tickets. Begin with a verified dependency/build/start baseline, then extend the running application with narrow, demonstrable behaviors. The product contract and milestone acceptance/evidence requirements remain authoritative.

## User Stories

1. As a developer, I want a reproducible development setup using compatible stable tools, so that I can build and run the application reliably.
2. As a developer, I want the initial desktop window to connect to a separate supervisor and receive status, so that the intended process arrangement is proven before adding product behavior.
3. As a developer, I want to register an existing Git repository without changing its files or starting Codex, so that adding a project is safe and predictable.
4. As a developer, I want invalid or unavailable repository paths to produce actionable errors, so that I can correct the problem without retaining a broken entry.
5. As a developer, I want registered repository identity restored after reopening, so that I can return to my projects.
6. As a developer, I want workspace creation to use and display a pinned local default commit automatically, so that I can create workspaces without a setup dialog.
7. As a developer, I want a provider-independent branch/worktree with a visible path and branch, so that conversations can share isolated files without changing my original checkout.
8. As a developer, I want partially failed workspace creation to show what already exists and reconcile the same operation, so that retrying neither duplicates nor destroys work.
9. As a developer, I want workspace identity restored after restart, so that I return to the same code state.
10. As a developer, I want to use the supported installed Codex runtime with its existing authentication/configuration or explicitly sign in through its official browser flow, so that I can connect without giving the app ownership of credentials or silently changing billing.
11. As a developer, I want missing-runtime, incompatible-protocol, and unavailable-authentication errors to explain recovery, so that I can fix setup within the workflow.
12. As a developer, I want to send multiline text and see streamed replies and tool activity, so that I can direct and follow coding work.
13. As a developer, I want every message and event attached to its exact conversation and turn, so that activity cannot appear under the wrong work.
14. As a developer, I want locally saved, provider-received, and uncertain sends distinguished, so that I do not accidentally submit the same work twice.
15. As a developer, I want navigation and interrupt controls to remain responsive during sustained output, so that streaming does not prevent control.
16. As a developer, I want the actual requested operation and available approval scope shown before answering, so that I understand what I authorize.
17. As a developer, I want allow-once and deny to apply to the exact pending request, so that an answer cannot authorize different work.
18. As a developer, I want stale or conflicting approval answers rejected, so that delayed interactions cannot change a later request.
19. As a developer, I want a lost approval reply to remain uncertain, so that connection failure cannot broaden permission.
20. As a developer, I want to inspect uncommitted text edits, new files, and deletions with an identified comparison, so that I can understand workspace changes.
21. As a developer, I want refreshed changes after further edits, so that inspection reflects current work.
22. As a developer, I want unsupported content labeled and Git/read failures displayed alongside the last good result, so that a failure cannot look like a clean workspace.
23. As a developer, I want to interrupt the exact active turn and see its observed outcome, so that I can stop unwanted work without affecting a later turn.
24. As a developer, I want failed or unacknowledged interruption and unconfirmed cleanup visible, so that I know whether owned tool work may still be running.
25. As a developer, I want explicit Quit to stop owned activity, flush durable records, and shut down the supervisor before reporting success, so that shutdown is trustworthy.
26. As a developer, I want readable local history and native conversation mapping restored without starting work, so that reopening is observational.
27. As a developer, I want a crashed desktop to reconnect to the existing supervisor and ongoing work, so that presentation failure does not duplicate or abandon execution.
28. As a developer, I want supervisor failure to preserve the durable history prefix and show interrupted or uncertain execution, so that recovery does not invent a successful outcome.
29. As a developer, I want surviving runtime ownership reconciled before conflicting execution is admitted, so that recovery cannot create competing owners.
30. As a developer, I want competing supervisor ownership and incompatible client versions rejected visibly, so that one application state has one authoritative owner.
31. As a developer, I want an explicit Resume to continue the same native conversation and recall earlier context, so that I can continue prior work intentionally.
32. As a developer, I want unavailable native history to leave local history readable and explain resume failure, so that loss of provider history does not erase application history.
33. As a developer, I want starting a new conversation to be a separate, clearly identified action, so that it cannot silently replace failed exact resume.
34. As a developer, I want storage and shutdown failures to remain actionable in the application, so that apparent success does not conceal lost records or running work.
35. As a maintainer, I want fixture and native qualification results recorded separately, so that deterministic recovery coverage cannot be mistaken for proven provider compatibility.
36. As a maintainer, I want later conversations, providers, and clients to extend the established ownership and identity model, so that temporary milestone limits do not require replacing its foundation.

## Implementation Decisions

Apply the global [MVP development compatibility policy](../../docs/MVP.md#development-compatibility-policy)
to every ticket and successor milestone. Retired application compatibility is
excluded from engineering and acceptance; implement only the current design with
fresh private state and documented reset/restart steps. Carry its clean-state
completion gate through final MVP qualification. Preserve current-build recovery,
ownership/cleanup and selected external-provider qualification; historical
compatibility results remain evidence only.

### Bootstrap and delivery

- Use one spec for the complete milestone and later divide it into tickets with explicit blocking dependencies. Product behavior tickets should deliver narrow end-to-end behavior through the layers they exercise.
- The first ticket is the agreed bootstrap exception: install required dependencies, build the initial application, open a minimal Tauri window, launch a separate supervisor, and successfully exchange connection/status information. Repository registration, persistence, chat, and recovery behavior follow afterward.
- Select the latest compatible stable tool/dependency versions when executing bootstrap, record resolved toolchain versions, and commit dependency lockfiles. Record the generator version and selections if scaffolding is used. The scaffold command does not determine execution ownership.
- Use React with TypeScript and Vite, pnpm for frontend tooling, and Cargo for Rust tooling. Use official shadcn/ui source components backed by Base UI, with Tailwind and shared theme variables. See the [React decision](../../docs/adr/0001-react-presentation-with-vite.md) and [component foundation decision](../../docs/adr/0002-shadcn-base-ui-presentation-components.md).
- Establish reproducible build, formatting, static checking, and startup verification commands. Bootstrap performs no live agent submissions.

### Incremental UI foundation

- Extend the shell established by completed [ticket 02a](issues/02a-establish-incremental-ui-foundation.md); retain its [desktop qualification](../../docs/evidence/milestone-01-sidebar-refactor.md) and subsequent [shell block refactor evidence](../../docs/evidence/shell-blocks-refactor.md).
- Use Conductor's compact dark desktop composition as the initial reference, with consistent typography, spacing, surfaces, focus and semantic state styling. Ticket 02a defines the project-specific visual-reference artifacts and how to distinguish proposed content from implemented behavior.
- Add components as working screens need them. Repository/workspace navigation, change inspection, chat and recovery extend the same shell in their existing tickets; full desktop qualification remains ticket 12.
- Before extending or reviewing frontend components, read the [component guide](../../src/components/README.md) and use the [React skill](../../.agents/skills/react-engineering/SKILL.md). They own component placement, styling contracts, and observation lifecycles. `App` coordinates shared observations and composes `RepositorySidebar` and `WorkspacePanel`; `SupervisorConnection` owns supervisor status and diagnostics. Keep feature observations and frequent streaming updates near their consumers rather than accumulating them in `App`.
- For common controls, forms, overlays and status presentation, use the established shadcn/Base UI components and shared tokens under the [repository-local shadcn skill](../../.agents/skills/shadcn/SKILL.md). Keep feature composition and exact operation/identity semantics application-owned; add components only when the working feature needs them.
- Each behavior ticket delivers its relevant visual states within the shared shell, including empty/loading/error/uncertain states where applicable. Retain feature-level desktop evidence as required by its ticket; ticket 12 qualifies the accumulated UI for consistency, keyboard use and window sizing rather than first composing it. Automated coverage and actual-desktop qualification remain separate under the repository testing policy.
- Keep presentation state separate from authoritative Rust execution/recovery policy. Bound streaming/history views as those capabilities arrive and qualify responsiveness with representative actual-desktop workloads; framework selection is not performance evidence.

### Modules and ownership

- Keep the Rust core independent of Tauri and presentation state. The supervisor is its sole execution and authoritative storage host.
- `workspaces` owns repository/worktree identity, selected base commit, workspace creation, creation reconciliation, and workspace-wide admission.
- `conversations` owns application/native conversation mapping, turns, durable action policy, pending sends, approval validity, exact resume, and reconciliation of uncertain conversation actions.
- `runtimes` contains the deep provider-connection module owning readiness, setup/login attempts, admission, reconciliation and owned process cleanup. Concrete adapters keep protocol/authentication interpretation local; small internal process/stream seams supply controlled outcomes. Later execution mechanics and interruption extend the owning modules without moving durable conversation policy out of `conversations`.
- `storage` owns authoritative SQLite metadata and durable transcript writes outside project files. Local history remains available independently of native provider history. Execution policy remains in its owning core module.
- Tauri forwards typed actions and presents typed snapshots/events. Recovery and retry policy remain in Rust; the desktop reconnects to retrieve authoritative observations and recorded action status.
- Add concrete implementations when exercised behavior needs them. Keep interfaces small and ownership clear; later capabilities extend these modules. Introduce no generic workflow engine, speculative provider framework, or placeholder implementation for deferred features.

### Identity, durability, and recovery

- Retain separate repository, workspace, application conversation, native conversation, turn, action, and runtime-process identities. A process lifetime is distinct from a conversation or turn lifetime.
- Keep provider availability, inspected provider tab, login attempt and conversation provider binding separate. A workspace has no exclusive provider; retire its current hardcoded Codex field through the [provider-independent workspace prerequisite](../provider-independent-workspaces/spec.md) before connection 01. Multiple provider targets remain representable while this milestone's supported-provider/conversation limits stay in admission and presentation.
- Admit one active conversation application-wide for this slice. Enforce this as an admission/presentation limit, keeping storage and identity capable of representing multiple conversations per workspace. Reject duplicate ownership of a native conversation.
- Persist action identity and intent before effects. A repeated action ID retrieves its recorded status or outcome. Preserve the distinction between local durability, provider receipt, and uncertain outcome without promising exactly-once provider execution.
- Reconcile pending creation, send, resume, and approval outcomes before any potentially duplicate effect. Connection loss alone grants no authority to repeat work or start a competing owner.
- Restore durable history after supervisor failure and expose interrupted/uncertain execution according to available evidence. An uncertain surviving runtime blocks conflicting execution until reconciled. Saved PIDs alone do not establish authority to signal a process.
- Treat storage failures as visible failures. Report success only when the durability required for that action is established.

### Supervisor and desktop lifecycle

- Start or discover one supervisor on demand for the selected application state. Use a user-private local connection with exclusive owner admission, protocol/version compatibility checks, bounded output, and snapshot/event catch-up.
- Desktop crash or disconnection leaves already admitted work with the supervisor. Reopening reconnects to the same owner and retrieves observations without resubmission.
- Explicit Quit requests owned stop, durable flush, and supervisor shutdown, and reports cleanup or persistence failures visibly. Hide/minimize keeps work running; retain the product contract's recommended close-to-hide behavior.
- Keep blocking Git, process, and disk work away from rendering. Bound streams and page history rather than loading unbounded output into the desktop.

### Repository, workspace, and inspection

- Validate an existing Git repository before saving a usable registration. Registration performs no project-file changes and starts no provider work.
- Create a separate branch/worktree from an automatically identified and resolved local default commit. Adding a new repository requests its first workspace; repository + immediately requests another. Defer the composer and base picker. Generate names/branches/managed folders. Use usable local origin/HEAD, otherwise exactly one usable local main/master; fail visibly when unavailable or ambiguous. Retain the operation's identity and partial creation evidence for reconciliation. Preserve unrelated branches/folders and the original checkout.
- Display workspace path and branch; identify the provider with its conversation or explicitly targeted setup status. Provide a usable composition of navigation, chat, pending approval, changed-file list, and text diff.
- The milestone comparison covers uncommitted workspace changes, including new files and deletions. Identify the comparison and refresh after edits. Label binary/unsupported content and preserve the last good result with a visible error if refresh fails.

### Codex and human control

- Use the installed official Codex runtime with its existing authentication/configuration, or initiate official-runtime sign-in on explicit user action. Authentication and credentials remain runtime-owned: extract/store no subscription tokens, silently change no login state, and silently substitute no billing route. Recognize existing API-key configuration but omit API-key setup UI and credential-storage changes in this milestone.
- [Parent 06](issues/06-connect-codex-with-actionable-setup-diagnostics.md) indexes the current connection sequence 01 -> 02 -> 02a -> 04; 03 is deferred. The [connection spec](../codex-connection/spec.md#connection-journey) owns the shared Providers journey, diagnostic recovery, browser sign-in presentation, pending-progress continuity, disabled Claude actions and accessibility requirements. Its [ownership/protocol section](../codex-connection/spec.md#ownership-and-protocol) owns the setup-attempt and browser-process lifecycle requirements under Rust runtime ownership.
- Recognize existing authentication automatically and offer one official-runtime browser sign-in action. Authentication completion is rechecked authoritatively and starts no agent work. Installation remains user-managed. Connection 01, 02 and 02a retain their focused evidence before cumulative 04; separate CLI login, PTY/xterm and command-output viewers are deferred by the [scope amendment](../codex-connection/spec.md#authentication-scope-amendment--2026-11-15).
- Before adapter or fixture implementation, inspect the supported runtime version's official protocol/schema and retained evidence. Record runtime version, schema/source revision, and captured-versus-synthetic provenance beside fixtures. Metadata/schema inspection does not authorize agent sessions.
- Put the real Codex adapter and deterministic fixture adapter at the same execution seam. Preserve provider-specific evidence and qualify runtime outcomes against the supported protocol; silence is not completion evidence.
- Apply the [connection architecture](../codex-connection/spec.md#architecture-and-extension-constraints): one lifecycle owner, concrete provider adapters and one provider-observation feature module on the existing Query/feed/physical-scheduling foundation. Prove provider/configuration isolation with synthetic targets; add no provider framework, second cache or Claude/OpenCode implementation to this slice.
- Bind approvals to the exact pending provider request, conversation, turn, requested operation, and offered scope. Keep strict operation matching; reject malformed, stale, or conflicting answers. Lost replies remain uncertain rather than widening approval.
- Interrupt an explicitly identified active turn, separately from stopping the runtime. A delayed interrupt cannot target a later turn. Preserve failed/unknown interruption and cleanup evidence visibly, including owned tool work that cannot be confirmed stopped.
- Resume only on explicit action using the saved native conversation mapping. An unavailable native conversation leaves local history readable and produces a clear exact-resume failure. Starting a new conversation is a separate action.

## Testing Decisions

Follow [testing and verification](../../docs/agents/verification.md) for automated test design and skill constraints. Use the current configured verification gate in `package.json` under that policy. The A1–A10 table below lists behaviors to cover; it does not choose dependency realism. Actual desktop and native qualification follow the milestone's separate stages. Apply this policy from the first tests of each ticket.

| Acceptance               | Required coverage                                                                                                                                                                                                             |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A1 — Register            | Valid and invalid/unavailable repositories; no original-file changes or provider start; restored identity after reopening.                                                                                                    |
| A2 — Workspace           | Automatic pinned local default, separate branch/worktree, untouched original checkout, persistent identity, partial creation failure, and repeat-action reconciliation without duplication or unrelated deletion.             |
| A3 — Connect             | Existing authentication first; explicit browser sign-in; separate CLI/terminal login deferred; actionable setup errors; no API-key setup or Claude effects; duplicate/stale/lost results, navigation/progress return, cancellation/physical cleanup, and browser completion without verified authentication. Keyboard/focus/landmarks and window sizing need desktop evidence. Fixture, real-authentication and native-agent compatibility results remain separate. |
| A4 — Chat                | Multiline sends, correctly routed streaming/tool activity, saved/received/uncertain delivery, repeated action IDs, late/duplicate events, and usable control during sustained output.                                         |
| A5 — Approval            | Exact operation/request/turn matching, allow effect, denied non-effect, malformed payloads, stale/conflicting answers, and uncertain lost replies without broadened permission.                                               |
| A6 — Diff                | Text modifications, additions, deletions, comparison identity, refresh, unsupported content, and visible Git/read failure retaining the last good result.                                                                     |
| A7 — Interrupt           | Exact-turn targeting, late response isolation from later turns, observed outcome, failed/unacknowledged interruption, and owned tool cleanup or explicit uncertainty.                                                         |
| A8 — Restart/history     | Owned stop/flush/shutdown on Quit, visible failures, history restored without execution, desktop-only crash/reconnect without replay, and durable-prefix recovery after supervisor failure.                                   |
| A9 — Exact resume        | Saved native mapping, earlier-marker recall, reconciliation after uncertain resume, unavailable native history with readable local history, and distinct new-conversation action.                                             |
| A10 — Ownership/recovery | Actual process owner exclusivity, private connection access, version mismatch, disconnect/catch-up, repeat-action results, uncertain surviving work blocking conflicts, and no signaling authority inferred from a saved PID. |

Verification follows the milestone's order:

1. **Deterministic checks:** use the testing policy for application decisions and mocked effect outcomes. Ground protocol inputs in the supported schema/evidence and preserve strict identity/approval matching. Record unit results separately from actual desktop/process and native qualification; synthetic faults do not establish provider behavior.
2. **Actual desktop with fixtures:** complete the usable workflow, deny approval, interrupt, quit/restart, crash/reconnect the desktop, recover from supervisor failure, and resume exactly. Record working/error-state screenshots, commands, and results. Run build, formatting, static checks, and relevant tests. Measure navigation feedback against 100 ms and recent-history opening against one second on the test Mac under this single-conversation workload; these measurements do not qualify deferred concurrency.
3. **Separately authorized native cohort:** before any real agent run, agree exact scenarios, permitted submissions, model/effort, time/usage limits, and stop conditions. Use disposable test checkouts/private application state while authentication stays runtime-owned. Record runtime/schema versions, effective configuration/MCP startup exposure, and source hashes before/after. There are no automatic scenario retries, larger-model fallbacks, or inherited archived allowances; no established hard dollar cap is assumed.
4. **Evidence closure:** demonstrate native streaming, allow/deny, interrupt/owned cleanup, restart, and exact resume through the desktop. Record non-secret subscription/account/billing evidence when available. API-key execution remains separately unqualified unless included in the allowance. Retain failures and unperformed checks explicitly; fixture success does not substitute for native success.

A3 read-only metadata/schema checks must not change authentication. Agree and
record real authentication checks separately from fixture-desktop and native-agent
qualification; explicit runtime-managed sign-in starts no agent thread or turn.
Do not silently sign out or replace existing credentials to create test states.
Ticket 06 records a hands-on connection trial when feasible; source inspection and
controlled UI checks do not establish actual usability or successful native login.

## Out of Scope

- Claude authentication/readiness/execution integration (only its informational tab and disabled connection actions are included); multiple active chats and concurrent-work qualification; multi-project coordination.
- Attachments/images and file-reference UX; steering; model/effort selection UI; Plan Mode; full skills/hooks/MCP/configuration compatibility qualification.
- Remote-base fetching and project setup commands; full workspace/base comparisons; inline diff comments sent to chat; editor/terminal handoff.
- Archive commits/restoration and configurable branch cleanup; turn-level code rewind.
- Built-in PR creation, checks, feedback, merge, PR inbox, automated review preparation, and human handoffs.
- API-key setup UI and automatic runtime installation/bundling/distribution; cloud execution/collaboration; separate CLI login, embedded authentication/development terminals, command-output viewers, general shells, workspace terminal management and run/test controls; extension manager; standalone CLI/TUI; additional independent clients.
- Scheduling, login-time startup, and continuing work after explicit Quit; remaining full-MVP onboarding/performance qualification.

These are sequencing limits. Retain the product contract's ownership and identity model, including the separate supervisor, while implementing only behavior this milestone exercises.

## Further Notes

- Authority: [MVP product contract](../../docs/MVP.md) defines product scope and architecture; [Milestone 01](../../docs/MILESTONE-01.md) defines A1–A10, verification order, completion gate, and retained results. This spec elaborates implementation without replacing those authorities.
- Preparation confirmed one spec, separate-session tickets, bootstrap before product behavior, a clear usable desktop, stable reproducible versions, and lasting modules without speculative abstractions. Automated coverage follows the repository testing policy; desktop and native qualification remain separate.
- Published to the configured local Markdown tracker with `ready-for-agent` status. Implementation tickets are separate files with explicit blockers under this feature's `issues/` directory.
- Exact stable dependency versions and the supported Codex runtime/schema cohort are selected and recorded during their respective implementation gates. The agreed presentation stack is recorded above; no transport library, message schema, or provider compatibility result is implied by that choice.
- Tickets 01–02 are complete and predate the testing policy. Tickets 03–05 are complete for their recorded fixture scope; see [05 qualification](../../docs/evidence/milestone-01-change-inspection-qualification.md). Connection 01, 02 and 02a have recorded fixture evidence; connection 03 is deferred and cumulative 04/parent 06 acceptance remains open. Milestone 07–13 remain unimplemented. The [frontend foundation](../frontend-navigation/spec.md) precedes the active connection sessions under parent 06, and [storage research](issues/06a-evaluate-storage-before-durable-chat.md) follows connection 04 and the parent aggregate before 07. The ticket 03 verification gate is in place, with [policy-alignment evidence](../../docs/evidence/milestone-01-verification-policy-alignment.md). Record implementation revision, hardware, versions, effective test configuration, reproducible commands, screenshots, A1–A10 pass/fail/not-run results, fixture/native distinction, live allowance/actual usage, and unresolved limits in the milestone evidence.
- Mark the milestone Passed only when its deterministic, desktop, and required native checks succeed. If native checks are unauthorized or fail, retain completed evidence and leave the milestone incomplete. Preserve the completed milestone when defining the next one.

## Workspace-flow decision — 2026-11-13

The user accepted immediate workspace creation after adding a new repository and
from its + action, generated names/folders, and automatic local-default selection.
Registration remains separate and read-only; successful new registration records
its first-creation continuation atomically. Re-registering an existing repository
does not create another. No composer, base picker, fetch, setup or agent launch.
See [research](../../docs/research/ticket-04-conductor-workspace-flow.md).

## Readable workspace follow-up

The approved [readable-workspaces plan](readable-workspaces-plan.md) refines ticket
04 naming and development startup. UUIDs remain internal identity; human labels
use the curated footballer catalog with permanent reservations and numeric tiers.
This does not expand creation into a composer, base picker or provider launch.
See [evidence](../../docs/evidence/readable-workspaces.md) for qualification and limits.

### 2026-11-14 — frontend library refactor handoff

The [amended frontend foundation](../frontend-navigation/spec.md) inserts F04a–F04g
before F05: this session publishes documentation only; separate sessions introduce
generated contracts, async owned transport and complete feature observation/actor
modules. F04g actual qualification precedes F05 retained context/integration, then
F06–F08 routes/Settings/cumulative qualification. Milestone 06 still waits for F08.
Original completed ticket results remain unchanged. Later Codex/chat work consumes
the resulting interfaces and does not assume a synchronous transport or recreate
frontend observation owners. Durable conversation delivery stays separate from the
invalidation journal and its accepted cursor; storage research sequencing is unchanged.

### 2026-11-15 — provider connection documentation published

Ticket 06 now specifies a Conductor-style shared Providers page, explicit
official-runtime browser/embedded CLI sign-in, preserved setup progress and
disabled Claude connection actions. API-key setup and the general development
terminal remain deferred. This supersedes the former no-login-changes requirement
and separate Providers → Codex successor wording; credentials and execution policy
remain runtime/Rust-owned. See the
[source research and limits](../../docs/research/conductor-provider-connection.md).
This publication changes documentation only; ticket 06 implementation, acceptance
and provider qualification remain open, with existing blockers and 06 → 06a → 07
sequencing retained.

### 2026-11-15 — connection work split for fresh sessions

The [connection feature spec](../codex-connection/spec.md) now holds the shared
requirements previously in 06. Its tickets deliver readiness/diagnostics (01),
browser sign-in (02), embedded CLI login (03) and cumulative qualification (04).
Each is selected separately, includes focused checks and leaves a dated handoff.
Parent 06 is the milestone index/aggregate; existing 06a stays storage research and
now explicitly waits for connection 04 before durable chat 07. This supersedes
single-session execution of 06 without changing the approved behavior, completed
evidence or native-agent allowance. All new implementation acceptance stays open.
