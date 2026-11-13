# Local agent workspace — MVP product contract

2026-11-12 · Product contract. Milestone documents track implementation and qualification against this scope.

## User, promise, and decision status

The first user is Christopher, coordinating coding work on his own Mac. The product is a desktop workspace for application-owned chat, Git workspaces, concurrent conversations, change inspection, and durable history.

**“Bring your own subscriptions and keys.”** Official Claude Code and Codex runtimes execute the work. Authentication stays with those runtimes; providers bill the user directly. The app does not extract subscription tokens or silently substitute API billing.

**Confirmed here:** the product promise and integrated-runtime approach; Rust core + Tauri with a separate local supervisor from Milestone 01; first release for personal use; PR inbox and automated review preparation belong later.

**Full product vision:** bring Conductor's local-development workflow into this app: runtime installation/bundling and agent setup, workspaces, integrated terminals and run/test actions, change inspection and feedback, commit/push, PR creation, checks, review responses, and merge. Add incoming review preparation and editable human handoffs. Current MVP reductions are sequencing choices; cloud execution and collaboration remain outside the vision. This sets direction, not an automatic commitment to every future Conductor feature.

**Recommended launch defaults:** Apple Silicon macOS first and a TypeScript web UI. The MVP runs locally with Claude Code and Codex. RAT remains an untouched, read-only reference; its terminal architecture and ticket hierarchy do not govern this product.

## One daily workflow

Open an existing repository and create a workspace for a bug fix. The app creates a separate branch/worktree, runs optional project setup, and opens a Codex chat. Give it the bug and a screenshot. While it works, start an independent feature in another workspace. Return when approval is needed, inspect the exact operation, and approve once or deny.

Open a Claude conversation in the bug workspace for a second opinion. Both chats share the files and one workspace diff. Inspect changed files, send corrections, and run tests through the agent or an external terminal. Quit; reopen tomorrow to read both conversations and explicitly resume the selected provider conversation. Archive finished work without losing history. Use existing Git tools to commit/push/open a PR during this MVP.

This follows Conductor's documented [workspace loop](https://www.conductor.build/docs/first-workspace), [shared versus independent work](https://www.conductor.build/docs/concepts/parallel-agents), and [review/archive workflow](https://www.conductor.build/docs/concepts/workflow). Worktree isolation separates code state; it is not a system security sandbox. Conductor's private implementation and provider agreements are unknown.

## MVP scope and observable acceptance

| Capability | Acceptance evidence |
| --- | --- |
| Repository and workspace navigation | Add existing repositories without changing original-checkout files or starting agents. Adding a new repository also requests its first isolated workspace as a separate durable operation; repository + requests another immediately. Generate names and managed folders. Fetch the configured remote before creating a branch/worktree from its configured base; pin the resolved commit. Fetch failure is visible and requires an explicit choice to use an identified local commit. Repositories without a remote use a selected local base. Setup failure retains the workspace and offers explicit recovery without duplicate creation or automatic launch. |
| Integrated chat | With either configured provider, send multiline text, file references, and uploaded images; see streamed text, tool activity, errors, and supported model/effort choices. Include Plan Mode: inspect a readable plan and explicitly approve implementation or send feedback. Plan approval and tool approval are distinct. Qualify each provider's actual mode support. Provider changes create separate conversations. Retain uploaded content for later history. |
| Concurrent work | Demonstrate two active chats sharing one workspace and another active workspace. Switching views stops none of them. Events, sends, approvals, and completion remain attached to the exact chat/turn. Show one shared diff and explain overlapping-edit risk. Reject duplicate ownership of a native conversation. |
| Human control | Show running, waiting, completed, interrupted, failed, or unknown based on runtime evidence. Approve once/deny the exact pending operation; reject stale approvals. Interrupt a turn separately from stopping a runtime. Follow-ups steer where supported and distinguish pending from received. |
| Change inspection | Show uncommitted changes and the workspace's full changes against its base, including new files. Refresh after agent edits. Write/edit comments on changed lines and attach them to the chosen chat for explicit sending, retaining file/line and comparison context. Changed comparisons require revalidation rather than silently relocating feedback. Open the workspace in an external editor/terminal for testing and ordinary Git operations. |
| Durable history and recovery | After restart, restore workspace/chat identity, readable messages, attachment content, and provider IDs. Explicit resume recalls the same conversation. Lost replies cannot silently cause duplicate turns. Crash/transport loss exposes uncertainty and reconciles ownership before restart; viewing history never starts work. Storage errors are visible. |
| Runtime configuration | Use existing runtime authentication and API-key configuration. Identify the billing route when available; otherwise show unknown. Exercise project/user instructions, skills, hooks, and MCP with recorded results. Show unsupported features honestly; preserve external access to native tools. |
| Safe lifecycle and responsiveness | Archive only after accounting for all affected chats and confirming owned stop. Save code state, including uncommitted changes, as a local archive commit and retain chat history. Restore the saved workspace/code/history even if the original branch has changed. Local branch deletion on archive is configurable and its consequence is visible; snapshot/storage failure retains the original work and reports failure. Under the concurrency scenario, target navigation feedback within 100 ms and opening recent history within one second on the test Mac; record measurements and versions. |

No product account or GitHub login is needed for local coding. Initially use user-installed official runtimes with version checks. Bundled runtime distribution is deferred; packaging the Claude SDK bridge still needs qualification.

Deliberate reductions from Conductor: no cloud/collaboration, built-in PR creation/merge/checks, full embedded terminal, extension manager, or turn-level code rewind. External editor/terminal handoff keeps the coding loop usable. A full TUI is a later independent client, not an MVP prerequisite. Provider child-agent execution remains runtime-owned; rich child-agent inspection/control is deferred, and cleanup must still account for owned child work.

Workspace defaults follow Conductor's [fetch-before-create behavior](https://www.conductor.build/docs/concepts/workspaces-and-branches#branches-and-review); Plan Mode and local diff comments follow its [agent controls](https://www.conductor.build/docs/concepts/agent-modes) and [diff feedback](https://www.conductor.build/docs/reference/diff-viewer). Archive commits and restoration follow its [archive-save behavior](https://www.conductor.build/changelog/0.49.0-conductor-allegro-gpt-5-5), [restoration guarantees](https://www.conductor.build/changelog/0.33.5-better-archiving), and [configurable branch cleanup](https://www.conductor.build/docs/reference/settings/reference). Exact folder-removal mechanics are not established by these sources; preserving/restoring saved work governs implementation choices.

## Starting architecture

**Confirmed stack:** Rust application core + Tauri. **Recommendation:** TypeScript web presentation, with the Rust core independent of Tauri and UI state.

**Foundation principle:** each milestone adds a usable capability to the intended product architecture. Build production-intended implementations that later milestones extend. Limit capabilities temporarily while retaining the intended ownership, persistence, and execution seams. Avoid known architectural shortcuts that require replacement in the next milestone; allow refactoring when implementation evidence justifies it. Add concrete implementations as needed rather than speculative frameworks.

Keep one Rust core with concrete modules:

- `workspaces` owns repository/worktree identity, base/setup, and workspace-wide admission and archive across every affected conversation.
- `conversations` owns durable execution policy: application/provider identities, turns, pending sends, approval validity, exact resume, and reconciliation after lost replies.
- `runtimes` owns execution mechanics: provider connections, process ownership, cancellation and cleanup evidence. Use concrete Codex stdio app-server and official Claude SDK bridge adapters; credentials remain runtime-owned.
- `storage` owns authoritative writes for SQLite metadata and durable transcripts/attachments, outside project files and independent of provider history availability. Execution policy stays with its owning module.

Tauri requests actions against explicit conversation/turn identities and presents observations. Save action identity and intent before runtime effects; distinguish local durability, provider receipt, and unknown outcome. Retrying the same action retrieves its recorded status or outcome; an uncertain result triggers reconciliation before any potentially duplicate effect. Approvals bind to the exact pending operation. Conversation, turn, and runtime-process lifetimes remain distinct. Archive fences new work before reconciling all affected conversations. This provides recovery without promising exactly-once provider execution.

Expose typed actions and snapshots/events; keep blocking Git/process/disk work away from rendering, bound streams, and page history. Preserve provider-specific evidence. Test the owning Rust modules under the [testing policy](agents/verification.md), including lost replies, stale approvals, restart decisions, and archive racing with a send. A UI-independent core supports later clients without shared UI components. A small CLI means open/status commands; a full TUI adds its own presentation and interaction work. Defer both until useful.

**Process ownership from Milestone 01:** Tauri desktop → separate local supervisor hosting the Rust core → official agent runtimes. The supervisor is the sole execution and storage owner; Tauri forwards typed actions and presents observations. Run official runtimes as supervised local processes, preserving their authentication and agent loops. A process may host several conversations. Manage owned processes and cleanup explicitly rather than assuming parent exit stops them.

Start or reconnect to one supervisor on demand. Use a user-private local connection with version compatibility checks, exclusive owner admission, bounded output, and snapshot/event catch-up. Reopening the desktop reconnects to the existing owner; a lost connection grants no authority to start a competing owner or repeat work. Qualify this actual process seam in the first milestone desktop workflow; automated test mechanics follow the [testing policy](agents/verification.md).

**Lifecycle:** desktop crash/disconnection leaves already admitted work with the supervisor; reconnect recovers observations and action status without resubmitting turns. Supervisor failure restores durable history with interrupted/unknown execution and reconciles surviving work before replacement. Explicit Quit requests owned stop, durable flush, and supervisor shutdown; failed cleanup or persistence remains visible. Hide/minimize keeps work running. Recommend the window close button hides the window; Conductor's exact last-window-close behavior is undocumented. Conductor describes [local sessions ending on app close](https://www.conductor.build/docs/cloud/faq) and [hidden/minimized operation continuing](https://www.conductor.build/changelog); surviving desktop failure is our explicit ownership requirement, not a claim about its internals.

**Main tradeoff:** the supervisor adds startup/discovery, local transport, access control, version coordination, and recovery work now, so later milestones reuse the execution owner in its intended location. Scheduling, login-time startup, running after explicit Quit, and independent clients are deferred capabilities. This is a local backend, with no hosted server or cloud execution. The Claude SDK bridge adds executable packaging and its own failure cases. See [Tauri processes](https://v2.tauri.app/concept/process-model/) and [sidecars](https://v2.tauri.app/develop/sidecar/).

## Evidence: keep four categories separate

**1. Accepted requirement:** subscription-backed and API-key-backed official runtimes with application-owned chat. Incomplete qualification does not reopen this requirement.

**2. Documented behavior:** [Codex app-server](https://learn.chatgpt.com/docs/app-server) supports rich clients, streaming, approvals, history, and runtime-managed ChatGPT authentication. [Anthropic's subscription guidance](https://support.claude.com/en/articles/15036540-use-the-claude-agent-sdk-with-your-claude-plan) explicitly covers signed-in SDK/`claude -p` subscription usage. Documentation is not installed-version qualification.

**3. Observed standalone proof, 2026-11-12:** macOS 26.2 ARM64; Codex 0.167.1 and Claude Code 2.1.322 / SDK 0.3.322. Both streamed, retained distinct chat identities, resumed the exact saved conversation, and recognized an image. Claude demonstrated approve-once, deny, steering, and interrupt. Codex completed subscription-backed turns with ChatGPT Pro metadata; deny left fixture files absent. These clients did not qualify an integrated desktop, durable app storage, or production cleanup.

**4. Unfinished checks and distribution conditions:**

- Claude subscriber test: the maintainer has no Claude subscription; existing Claude runs do not establish subscription compatibility. Use a subscriber tester, without requiring a purchase.
- Codex allow approval failed the harness command/CWD validator; payload evidence was insufficient to diagnose it. Preserve strict matching. Steering remains unverified; interrupt returned `-32600`; `thread/read` returned `-32603` after two starts. Two accepted starts are not proof of overlapping execution.
- Both providers: actual concurrent active work, lost-reply/reconnect and stale-approval reconciliation; configuration/skills/hooks/MCP and child-agent behavior; model/effort controls, app latency, attachment durability, and escaped-descendant cleanup. Claude fixture CWD needs canonical-path comparison. Claude result subtype/exit code alone cannot distinguish completion from interrupted streaming.
- Claude: [legal guidance](https://code.claude.com/docs/en/legal-and-compliance) permits unmodified-binary subscription sign-in under product conditions: Commercial Terms, published binary, preserved authentication methods, user-owned credentials/direct billing, and no resale/intermediation. The [SDK quickstart](https://code.claude.com/docs/en/agent-sdk/quickstart) retains a prior-approval restriction. Their overlap for this exact distributed wrapper remains unresolved. This is narrower than whether subscriptions technically work.
- Codex: current [auth guidance](https://learn.chatgpt.com/docs/app-server#auth-endpoints) allows continuation for local/open-source apps but excludes commercial/hosted services from that auth path and points to Sign in with ChatGPT. Continuation wording is not blanket approval for every new distributed product. Personal development can proceed; revisit the appropriate route before broader distribution.

Qualification also needs a frozen version/source cohort: the proof captured hashes after its runs, not before and after. Both subscription and API-key paths need application-level checks. These primary sources were checked for this baseline. The desktop architecture research's older broad Claude warning is superseded by the targeted subscription findings; its Codex distribution caveat remains relevant. No contradictory evidence here requires abandoning subscriptions.

## Later: PR inbox and prepared reviews

The full product keeps implementation → PR creation → addressing feedback → checks → merge inside the app, alongside incoming requested reviews. These remain later scope; external Git/editor/terminal handoff serves the MVP.

Future `pull_requests` owns remote identity, checks/feedback, and action reconciliation independently of any chat or workspace. Future `reviews` owns findings, human edits/selections, and durable handoffs. Both use the existing core; provider adapters execute agents rather than own GitHub publication. Add these modules when their features enter scope, without a generic workflow engine or placeholder implementation now.

**Review context direction:** automatic preparation uses independent reasoning context with explicitly supplied requirements, diff/revision, repository guidance, relevant checks, and prior findings. It reuses core execution and recovery without requiring a visible implementation chat. Independence concerns model inputs, not a mandatory second tab, process, or checkout. Qualify the provider path before promising history isolation. Keep implementer self-checks distinct; return accepted findings to the implementation conversation for fixes.

Recommended later controls cover reviewer/model/effort, review instructions, accepted automatic triggers, scope, and selected additional evidence. Defer a full-transcript-inheritance option until demonstrated need. During provider qualification, inspect actual context inheritance: Claude's [fresh-review guidance](https://code.claude.com/docs/en/best-practices#give-claude-a-check-to-run) and [local review modes](https://code.claude.com/docs/en/code-review-local) differ in their descriptions; Codex distinguishes [fresh threads from history forks](https://learn.chatgpt.com/docs/app-server#review). A review command or separate tab alone does not establish independence.

With separately accepted background usage, prepare an editable local handoff bound to repository/PR, base/head SHAs, comparison identity, draft version, and included evidence. Preserve human edits and mark changed revisions stale. Preview the exact revision, outcome, and selected comments, then require explicit human submission. Reconcile uncertain GitHub writes before retrying. Preparation grants no publication authority; remote pending reviews are not local draft storage.

## Smallest next milestone and remaining decisions

The first working slice is defined in [MILESTONE-01.md](MILESTONE-01.md). Read it when implementing or verifying that slice. It contains its scope, acceptance criteria, verification order, and results; this document remains authoritative for product scope and architecture. Keep completed milestones as evidence and create the next numbered milestone after the preceding one passes.

No additional product decision blocks starting that slice. Public distribution and a full TUI can be decided later. Qualification gaps are engineering evidence to obtain, not questions for the user to guess.

Local sources: [subscription research](../../rust-agent-terminal/docs/research/subscription-chat-integration.md), [Claude source reconciliation](../../rust-agent-terminal/docs/research/claude-subscription-answer-check.md), [accepted RAT chat decisions](../../rust-agent-terminal/.scratch/integrated-agent-chat/decisions.md), [proof and retained artifact inventory](../../rust-agent-terminal/.scratch/integrated-agent-chat/proof-results.md), [ADR 0008](../../rust-agent-terminal/docs/adr/0008-integrated-chat-over-official-agent-runtimes.md), and [desktop/CLI architecture research](../../agent-workspace-research/desktop-cli-architecture.md). These are evidence links, not imported implementation obligations.

## Workspace creation sequencing decision — 2026-11-13

Milestone 01 defers the composer dialog and base picker. Use the locally available
origin/HEAD commit, otherwise an unambiguous local main/master commit; missing or
ambiguous defaults fail visibly. No fetch occurs in this milestone. Registration
and its first-workspace continuation retain separate durable identities. Creation
never starts Codex. This updates the earlier manually selected-base requirement;
full-MVP configured-base fetching remains required. See
[creation research](research/ticket-04-conductor-workspace-flow.md).
