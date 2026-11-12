# Milestone 01 implementation spec — one usable Codex desktop workflow

Status: ready-for-agent

Implementation baseline: tickets 01–02 are complete. Tickets 03–13 are not implemented.

## Problem Statement

Christopher needs a local desktop workspace for coding with his existing Codex runtime and authentication. The first usable workflow must isolate code changes, show agent activity and requested approvals, retain readable history, and recover from failures without silently repeating uncertain work.

## Solution

Build one Apple Silicon macOS desktop workflow: register an existing Git repository, create a separate branch/worktree at a selected local commit, chat with Codex, allow or deny an exact requested operation, inspect uncommitted changes, interrupt work, quit/reopen, and explicitly resume the same native conversation.

Use a Tauri desktop connected to a separate on-demand local supervisor hosting the Rust core. The supervisor owns execution and authoritative storage. The desktop submits explicitly targeted actions and displays observations. Viewing history and reconnecting never start or replay a turn.

Deliver this as one implementation spec with session-sized implementation tickets. Begin with a verified dependency/build/start baseline, then extend the running application with narrow, demonstrable behaviors. The product contract and milestone acceptance/evidence requirements remain authoritative.

## User Stories

1. As a developer, I want a reproducible development setup using compatible stable tools, so that I can build and run the application reliably.
2. As a developer, I want the initial desktop window to connect to a separate supervisor and receive status, so that the intended process arrangement is proven before adding product behavior.
3. As a developer, I want to register an existing Git repository without changing its files or starting Codex, so that adding a project is safe and predictable.
4. As a developer, I want invalid or unavailable repository paths to produce actionable errors, so that I can correct the problem without retaining a broken entry.
5. As a developer, I want registered repository identity restored after reopening, so that I can return to my projects.
6. As a developer, I want to select an identified local base commit for a new workspace, so that I know which code the work starts from.
7. As a developer, I want a separate branch/worktree with a visible path, branch, and provider, so that agent edits leave my original checkout untouched.
8. As a developer, I want partially failed workspace creation to show what already exists and reconcile the same operation, so that retrying neither duplicates nor destroys work.
9. As a developer, I want workspace identity restored after restart, so that I return to the same code state.
10. As a developer, I want to use the supported installed Codex runtime with its existing authentication/configuration, so that I retain control of credentials and billing.
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

### Bootstrap and delivery

- Use one spec for the complete milestone and later divide it into tickets with explicit blocking dependencies. Product behavior tickets should deliver narrow end-to-end behavior through the layers they exercise.
- The first ticket is the agreed bootstrap exception: install required dependencies, build the initial application, open a minimal Tauri window, launch a separate supervisor, and successfully exchange connection/status information. Repository registration, persistence, chat, and recovery behavior follow afterward.
- Select the latest compatible stable tool/dependency versions when executing bootstrap, record resolved toolchain versions, and commit dependency lockfiles. Record the generator version and selections if scaffolding is used. The scaffold command does not determine execution ownership.
- Use a TypeScript web presentation with pnpm for frontend tooling and Cargo for Rust tooling. The frontend framework choice is left to bootstrap implementation; this spec does not depend on a particular framework.
- Establish reproducible build, formatting, static checking, and startup verification commands. Bootstrap performs no live agent submissions.

### Modules and ownership

- Keep the Rust core independent of Tauri and presentation state. The supervisor is its sole execution and authoritative storage host.
- `workspaces` owns repository/worktree identity, selected base commit, workspace creation, creation reconciliation, and workspace-wide admission.
- `conversations` owns application/native conversation mapping, turns, durable action policy, pending sends, approval validity, exact resume, and reconciliation of uncertain conversation actions.
- `runtimes` owns the concrete Codex stdio app-server connection, provider protocol translation, process ownership, interruption mechanics, and cleanup evidence.
- `storage` owns authoritative SQLite metadata and durable transcript writes outside project files. Local history remains available independently of native provider history. Execution policy remains in its owning core module.
- Tauri forwards typed actions and presents typed snapshots/events. Recovery and retry policy remain in Rust; the desktop reconnects to retrieve authoritative observations and recorded action status.
- Add concrete implementations when exercised behavior needs them. Keep interfaces small and ownership clear; later capabilities extend these modules. Introduce no generic workflow engine, speculative provider framework, or placeholder implementation for deferred features.

### Identity, durability, and recovery

- Retain separate repository, workspace, application conversation, native conversation, turn, action, and runtime-process identities. A process lifetime is distinct from a conversation or turn lifetime.
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
- Create a separate branch/worktree from an explicitly selected and resolved local commit. Retain the operation's identity and partial creation evidence for reconciliation. Preserve unrelated branches/folders and the original checkout.
- Display workspace path, branch, and provider. Provide a usable composition of navigation, chat, pending approval, changed-file list, and text diff.
- The milestone comparison covers uncommitted workspace changes, including new files and deletions. Identify the comparison and refresh after edits. Label binary/unsupported content and preserve the last good result with a visible error if refresh fails.

### Codex and human control

- Use the installed official Codex runtime with its existing authentication/configuration. Do not extract tokens, change login state, or silently substitute a different billing route. Missing executable, unsupported protocol, and unavailable authentication produce setup/retry guidance.
- Before adapter or fixture implementation, inspect the supported runtime version's official protocol/schema and retained evidence. Record runtime version, schema/source revision, and captured-versus-synthetic provenance beside fixtures. Metadata/schema inspection does not authorize agent sessions.
- Put the real Codex adapter and deterministic fixture adapter at the same execution seam. Preserve provider-specific evidence and qualify terminal outcomes against the supported protocol; silence is not completion evidence.
- Bind approvals to the exact pending provider request, conversation, turn, requested operation, and offered scope. Keep strict operation matching; reject malformed, stale, or conflicting answers. Lost replies remain uncertain rather than widening approval.
- Interrupt an explicitly identified active turn, separately from stopping the runtime. A delayed interrupt cannot target a later turn. Preserve failed/unknown interruption and cleanup evidence visibly, including owned tool work that cannot be confirmed stopped.
- Resume only on explicit action using the saved native conversation mapping. An unavailable native conversation leaves local history readable and produces a clear exact-resume failure. Starting a new conversation is a separate action.

## Testing Decisions

Follow [testing and verification](../../docs/agents/verification.md) for automated test design and skill constraints. The A1–A10 table below lists behaviors to cover; it does not choose dependency realism. Actual desktop and native qualification follow the milestone's separate stages. Apply this policy from the first tests of each ticket.

| Acceptance               | Required coverage                                                                                                                                                                                                             |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A1 — Register            | Valid and invalid/unavailable repositories; no original-file changes or provider start; restored identity after reopening.                                                                                                    |
| A2 — Workspace           | Selected commit, separate branch/worktree, untouched original checkout, persistent identity, partial creation failure, and repeat-action reconciliation without duplication or unrelated deletion.                            |
| A3 — Connect             | Supported protocol/configuration and existing authentication; missing executable, incompatible protocol, unavailable authentication, and actionable setup errors. Native compatibility is separately qualified.               |
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
3. **Separately authorized native cohort:** before any real run, agree exact scenarios, permitted submissions, model/effort, time/usage limits, and stop conditions. Use disposable test checkouts/private application state while authentication stays runtime-owned. Record runtime/schema versions, effective configuration/MCP startup exposure, and source hashes before/after. There are no automatic scenario retries, larger-model fallbacks, or inherited archived allowances; no established hard dollar cap is assumed.
4. **Evidence closure:** demonstrate native streaming, allow/deny, interrupt/owned cleanup, restart, and exact resume through the desktop. Record non-secret subscription/account/billing evidence when available. API-key execution remains separately unqualified unless included in the allowance. Retain failures and unperformed checks explicitly; fixture success does not substitute for native success.

## Out of Scope

- Claude integration; multiple active chats and concurrent-work qualification; multi-project coordination.
- Attachments/images and file-reference UX; steering; model/effort selection UI; Plan Mode; full skills/hooks/MCP/configuration compatibility qualification.
- Remote-base fetching and project setup commands; full workspace/base comparisons; inline diff comments sent to chat; editor/terminal handoff.
- Archive commits/restoration and configurable branch cleanup; turn-level code rewind.
- Built-in PR creation, checks, feedback, merge, PR inbox, automated review preparation, and human handoffs.
- Runtime bundling/distribution; cloud execution/collaboration; full embedded terminal; extension manager; standalone CLI/TUI; additional independent clients.
- Scheduling, login-time startup, and continuing work after explicit Quit; remaining full-MVP onboarding/performance qualification.

These are sequencing limits. Retain the product contract's ownership and identity model, including the separate supervisor, while implementing only behavior this milestone exercises.

## Further Notes

- Authority: [MVP product contract](../../docs/MVP.md) defines product scope and architecture; [Milestone 01](../../docs/MILESTONE-01.md) defines A1–A10, verification order, completion gate, and retained results. This spec elaborates implementation without replacing those authorities.
- Preparation confirmed one spec, separate-session tickets, bootstrap before product behavior, a clear usable desktop, stable reproducible versions, and lasting modules without speculative abstractions. Automated coverage follows the repository testing policy; desktop and native qualification remain separate.
- Published to the configured local Markdown tracker with `ready-for-agent` status. Implementation tickets are separate files with explicit blockers under this feature's `issues/` directory.
- Exact stable dependency versions and the supported Codex runtime/schema cohort are selected and recorded during their respective implementation gates. No new framework choice, transport library, message schema, or provider compatibility result is implied by this document.
- Tickets 01–02 are complete; tickets 03–13 remain unimplemented. Record implementation revision, hardware, versions, effective test configuration, reproducible commands, screenshots, A1–A10 pass/fail/not-run results, fixture/native distinction, live allowance/actual usage, and unresolved limits in the milestone evidence.
- Mark the milestone Passed only when its deterministic, desktop, and required native checks succeed. If native checks are unauthorized or fail, retain completed evidence and leave the milestone incomplete. Preserve the completed milestone when defining the next one.
