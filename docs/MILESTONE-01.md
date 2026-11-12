# Milestone 01 — one usable Codex desktop workflow

Status: In progress — tickets 01–02 are complete. Product behavior in tickets 03–13 is not implemented; ticket 03's verification gate is in place. Complete workflow and native qualification remain incomplete.

## Outcome and authority

A developer can register a repository, create an isolated workspace, use Codex through desktop chat, answer an approval, inspect changes, interrupt work, restart the app, read history, and explicitly resume the exact conversation.

Build the first lasting part of the product. [MVP.md](MVP.md) is the product contract; its [Starting architecture](MVP.md#starting-architecture) owns the foundation principle, module responsibilities, and process/lifecycle design. This milestone limits capabilities while building those architectural seams in their intended form. Later milestones extend this implementation; evidence-driven refactoring remains possible. Completion requires the observable criteria below and recorded verification evidence.

## Scope

One local Apple Silicon macOS workflow using an installed official Codex runtime and its existing authentication. Support multiline text chat and one active conversation at a time in this slice. Persist separate repository, workspace, conversation, turn, and action identities; do not encode a permanent one-conversation-per-workspace limit.

Include the minimal on-demand local supervisor and one Tauri desktop client from the start, following MVP.md. Establish the real process connection and reconnection path; keep scheduling, login-time startup, and additional independent clients deferred.

Create a branch/worktree from an explicitly identified local base commit. This is a temporary milestone limitation; the full MVP fetches its configured remote before creation. Show workspace path, branch, and provider. Provide a usable desktop composition with navigation, chat, pending approval, changed-file list, and text diff. Controls represent working behavior; failures remain actionable within the app.

Implement only the code this workflow exercises. Follow the Rust-core, thin-Tauri, exact-identity, runtime-owned-authentication, and durable-storage constraints in [MVP.md](MVP.md#starting-architecture). Keep the real Codex adapter and deterministic fixture adapter at the same execution seam. Reuse the archived [proof results and artifact inventory](../../rust-agent-terminal/.scratch/integrated-agent-chat/proof-results.md) as evidence; preserve its files, ledgers, credentials, and native conversations. Before further RAT inspection, follow its AGENTS.md and required documentation.

## Implementation planning

Prepare one implementation spec for this milestone, then split execution into tickets sized for separate coding sessions. Keep the product contract and this milestone's acceptance/evidence requirements authoritative.

The [implementation spec](../.scratch/milestone-01/spec.md) captures the agreed build and testing decisions and is published in the local Markdown tracker.

The first ticket installs dependencies and establishes a verified build/start baseline before product behavior is implemented. Completion requires a minimal Tauri window, a separately launched supervisor, and a successful connection/status exchange between them. Repository registration, persistence, chat, and recovery behavior follow in subsequent tickets.

Select the latest compatible stable versions when executing the bootstrap ticket. Record toolchain versions and commit dependency lockfiles so the baseline is reproducible. Record the supported Codex runtime/schema version separately before adapter implementation; bootstrap verification does not authorize live agent sessions.

The desktop should have a clear, usable layout for navigation, chat, approvals, and change inspection. Build lasting, reusable modules with clear ownership under the MVP architecture; introduce abstractions when exercised behavior needs them. Later capabilities should extend these first building blocks.

## Observable acceptance

| ID | Required behavior, including failure/recovery |
| --- | --- |
| A1 · Register | Register an existing Git repository without changing its files or starting Codex. Reopening restores its identity. Invalid paths and unavailable repositories produce an actionable error rather than a broken saved entry. |
| A2 · Workspace | Create a separate branch/worktree at the selected commit and retain its identity across restart. Original checkout files remain untouched. Creation failures identify any branch/folder already created; retry reconciles that operation instead of creating duplicates or deleting unrelated work. |
| A3 · Connect | Connect to the supported installed Codex version using its existing authentication/configuration. Missing executable, unsupported protocol, or unavailable authentication gives setup/retry guidance. No app-owned token extraction, login changes, or silent switch of billing route. |
| A4 · Chat | Send multiline text and see streamed replies/tool activity in the correct conversation. Keep saved, provider-received, and uncertain delivery distinguishable. Navigation and interrupt remain usable during sustained output. Preserve supported provider evidence instead of inferring completion from silence. |
| A5 · Approval | Show the actual requested operation and available scope before allow-once or deny. Match the pending provider request to its conversation/turn; reject stale or conflicting answers. Verify an allowed fixture operation occurs and a denied operation does not. A lost approval reply retains uncertainty rather than broadening permission. |
| A6 · Diff | Inspect uncommitted text edits, new files, and deletions in the workspace. Identify the comparison shown and refresh after further edits. Binary/unsupported content is labeled honestly; Git/read errors stay visible without replacing the last good result with an apparent clean workspace. |
| A7 · Interrupt | Interrupt the exact active turn and display the provider's observed outcome. A late reply cannot interrupt a later turn. Failed or unacknowledged interruption remains failed/unknown, preserving history and offering recovery. Demonstrate that owned tool work stops; unconfirmed cleanup remains visible. |
| A8 · Restart/history | Explicit Quit stops owned activity, flushes durable records, and shuts down the supervisor before reporting success. Reopening restores readable history, workspace identity, and native conversation mapping without starting a turn. Killing/reopening the desktop alone reconnects to the same supervisor and existing work without replay. Supervisor failure retains the durable prefix and restores interrupted/uncertain state; report storage/shutdown failures visibly. |
| A9 · Exact resume | An explicit Resume continues the same saved native conversation and recalls an earlier marker. Unavailable/deleted native history leaves local history readable; explain why exact resume failed. Any new-conversation action is separate and clearly identified. |
| A10 · Ownership/recovery | Start or discover exactly one supervisor for the application state; reject a competing owner and incompatible client/supervisor versions visibly. Desktop connection loss cannot create another execution/storage owner. Reconcile pending creation/send/resume outcomes after lost replies or restart. Repeating an action ID returns its recorded status or outcome. An uncertain result triggers reconciliation before any potentially duplicate effect. An uncertain surviving runtime blocks conflicting execution until reconciled; saved PIDs alone grant no signaling authority. |

## Verification order

1. **Deterministic unit checks first.** Follow the [testing policy](agents/verification.md) for automated coverage of A1–A10 decisions. Ground protocol inputs in the supported Codex schema and record captured-versus-synthetic provenance. Exercise routing, intent-before-effect, lost replies, duplicate/late events, malformed approvals, connection loss and storage/recovery failures with substituted outcomes. Unit success establishes application decisions; actual OS ownership, durability, cleanup and provider compatibility remain separate qualification claims. Apply this policy from the first tests of each ticket and record evidence for the implemented behavior.
2. **Desktop workflow with fixtures.** Qualify the actual process/desktop behavior separately from unit tests, using the production core/supervisor path and private disposable state. Include A10 owner-exclusivity, private-access and version checks. Complete the full sequence in the actual app, including denied approval, interrupt, normal quit/restart, desktop-only crash/reconnect, supervisor failure/recovery, and exact resume. Record screenshots of the working states and errors, plus navigation/history-opening measurements against the MVP targets under this single-conversation workload. Run the implemented repository's build, formatting, static checks, and relevant tests; record exact commands and results.
3. **Bounded live-provider qualification.** Before any real run, agree the exact scenarios, permitted submissions, selected model/effort, time/usage limits, and stop conditions. Record runtime/schema versions, effective configuration and MCP startup exposure, and source hashes before/after the cohort. Use disposable test checkouts and private application state while leaving authentication with the runtime. No automatic scenario retries, larger-model fallback, or reuse of archived proof allowances. Codex has no established hard dollar cap in the existing proof.
4. **Close against evidence.** Demonstrate native streaming, allow/deny, interruption/owned cleanup, restart, and exact resume in the desktop app. Observe subscription-backed execution through the existing account and record non-secret account/billing evidence when available. API-key compatibility remains separately unqualified unless included in the agreed run. Failed or unperformed native checks remain open; fixture success cannot substitute for them.

## Deferred work

Claude integration; multiple active chats and concurrency qualification; complete skills/hooks/MCP/configuration compatibility; attachments/images and file-reference UX; steering; model/effort selection UI and Plan Mode; multi-project coordination; full workspace/base diffs and inline diff comments sent to chat; remote-base fetching; project setup commands; archive commits/restoration and configurable branch cleanup; editor/terminal handoff; and remaining MVP onboarding/performance qualification. These remain full-MVP requirements where specified in MVP.md; their deferral here applies only to Milestone 01.

PR inbox, automated review preparation/handoffs, PR creation/feedback/merge controls, and review configuration remain later product scope under MVP.md. Add their implementations when needed, using the supervisor/core established here. General scheduling, login-time startup, running after explicit Quit, and additional independent clients remain deferred; their deferral does not defer the supervisor itself.

## Results and next milestone

Ticket 01 established the TypeScript/pnpm + Rust/Tauri baseline, a separate supervisor and a successful desktop status exchange; see [bootstrap evidence](evidence/milestone-01-bootstrap.md). Ticket 02 established private owner admission, versioned snapshots and bundled desktop kill/reconnect; see [discovery evidence](evidence/milestone-01-discovery.md). These remain the completed starting point.

Tickets 01–02 predate the testing policy. Current repository alignment brought forward ticket 03's unit-only verification gate while retaining the earlier process/desktop evidence. See [policy-alignment evidence](evidence/milestone-01-verification-policy-alignment.md) for the coverage mapping, separate process qualification and measurements. Repository registration and the remaining product behavior are still unimplemented.

Product behavior in tickets 03–13 remains unimplemented. Remaining acceptance items stay open until the required unit, desktop and separately authorized native evidence is recorded.

During qualification, record:

- Implementation revision or source identity, tool/runtime versions, hardware, and effective test configuration.
- A1–A10 pass/fail/not-run status with reproducible commands, fixture/native distinction, screenshots, and evidence paths.
- Live allowance and actual submissions/usage; failures, unresolved behavior, and explicit qualification limits.
- Overall outcome and which deferred capabilities the evidence suggests tackling next.

Mark Passed only after required acceptance and deterministic/desktop/native checks succeed. If live checks are not authorized or cannot pass, record the completed work and leave the milestone incomplete. After it passes, add MILESTONE-02.md with its own narrow outcome and acceptance criteria; preserve this milestone's results rather than repurposing the document.
