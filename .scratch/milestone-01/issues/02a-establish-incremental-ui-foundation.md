# 02a: Establish the incremental UI foundation

**What to build:** Replace the centered supervisor diagnostics page with the first lasting desktop application shell, preserving the working supervisor status connection. Establish a compact dark Conductor-inspired visual baseline that repository navigation, workspace chat and change inspection can extend incrementally.

**Blocked by:** 02: Reliable supervisor discovery and connection

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), [milestone](../../../docs/MILESTONE-01.md), [testing policy](../../../docs/agents/verification.md), [React decision](../../../docs/adr/0001-react-presentation-with-vite.md), and [component foundation decision](../../../docs/adr/0002-shadcn-base-ui-presentation-components.md). Use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy. The [Conductor UI research](../../../docs/research/conductor-incremental-ui-foundation.md) provides official visual/workflow references.

**Coverage:** Presentation foundation for A1 and subsequent A3–A10 views; preserves the completed ticket 02 connection behavior. This inserted ticket runs before ticket 03 without renumbering existing tickets.

- [x] Adopt React with TypeScript in the existing Vite/pnpm/Tauri setup. Add official shadcn/ui components backed by Base UI, Tailwind and shared theme variables only as this working shell needs them; record resolved stable versions and generated-source provenance and retain lockfiles.
- [x] Establish compact dark typography, spacing, surfaces, borders, focus and semantic status styling using Conductor's documented desktop workspace as the visual reference. Keep common styles and repeated controls consistent without a general component-framework or docking-system project.
- [x] Provide a lasting application shell with repository navigation and main-content regions plus a compact supervisor connection indicator. Structure the composition so later tickets can add workspace identity, chat/composer and change inspection in their intended locations.
- [x] Show an honest initial empty state and only working actions. Preserve supervisor status retrieval, refresh/retry feedback, error details and reconnection behavior through existing typed actions/observations; keep process/version/owner diagnostics available without making them the main product composition.
- [x] Keep execution, storage ownership, retries and reconciliation in their existing Rust modules. React owns presentation concerns such as selection, layout and focus; mounting or remounting views must not gain authority to repeat uncertain work.
- [x] Create and retain three project-specific visual-reference artifacts (screenshots or static mockups) showing this application's empty shell, selected-workspace arrangement, and conversation with change inspection. Each artifact must show the proposed application composition and shared visual treatment; Conductor source screenshots alone do not satisfy this criterion. Record repository-relative artifact paths in the ticket's completion evidence.
- [x] Label each visual-reference artifact as implemented or proposed, and identify planned content within mixed views. Future workspace/chat/diff content is design-only in this ticket; repository registration remains ticket 03 and later functional capabilities remain in their owning tickets.
- [x] Check keyboard focus, readable error/connection states and layout at the application's configured minimum and ordinary window sizes in the actual Tauri desktop under its existing CSP. Record screenshots and observed results separately from unit checks; do not assume browser/library claims qualify the webview composition.
- [x] Keep automated checks proportional and follow the unit-only testing policy. Where presentation behavior needs regression coverage, supply mocked IPC/dependency observations; introduce no automated integration suite or real-process unit dependencies. Preserve the existing verification commands and avoid unnecessary reruns.
- [x] Record build/static/unit results and desktop qualification evidence with source/version/profile context. Do not claim streaming/history performance targets pass before representative workloads exist, and do not make live agent submissions.

**Out of scope:** Repository registration/storage, workspace creation, functional chat/diff/approval controls, light-theme qualification, a full component catalog, terminal/PR/cloud features, and general-purpose UI infrastructure. These do not prevent the shell and tokens from being extended later.

## Comments

The user confirmed a compact dark Conductor-inspired baseline and a separate UI-foundation ticket before repository registration. Tickets 01–02 remain completed evidence; ticket 03's already completed verification-gate item is preserved.

2026-11-12 · Implemented the React/Vite/shadcn Base UI shell and mocked-IPC checks;
see [implementation and qualification evidence](../../../docs/evidence/milestone-01-ui-foundation.md).
Actual Tauri layout, focus, connection/error screenshots and the corresponding
qualification record remain open. The user deferred them to a follow-up task from
the ChatGPT app because per-app Computer Use approval was unavailable in the
current Codex CLI session. This is not a completed ticket. No live submissions.
Tickets 01–02 and their evidence are preserved; other tickets and the parent spec
remain open.

Project-specific visual references (repository-relative paths):

- `docs/evidence/milestone-01-ui-foundation/implemented-empty-shell.svg` — implemented composition, static reference rather than desktop screenshot.
- `docs/evidence/milestone-01-ui-foundation/proposed-selected-workspace.svg` — proposed; repository/workspace/conversation entries are planned.
- `docs/evidence/milestone-01-ui-foundation/proposed-conversation-changes.svg` — proposed; transcript, composer and diff are design-only.

2026-11-12 · Completed the deferred actual-Tauri qualification against a fresh
release bundle from `4420328f2ab2fdf97e0524817d0b94fbafbd99db`, including the
current React Compiler configuration. See the appended
[desktop qualification results](../../../docs/evidence/milestone-01-ui-foundation.md#desktop-qualification-follow-up--2026-11-12)
for screenshots, source/binary hashes, build/static/unit results, controlled
permission/timeout/lost-owner errors, explicit retry, keyboard focus, ordinary and
minimum sizes, and desktop-only crash/reconnect to the same supervisor owner.
All 02a acceptance items are verified; no product source fix was needed. The
compiler's unsupported `App` try/finally remains an explicit coverage limit.
No live agent submissions or commits. Prior evidence is preserved; other tickets,
the parent spec and the milestone remain open.

2026-11-12 · Reviewed the shell against the React/shadcn guidance for a follow-up
amendment. Kept the fixed sidebar pending real repository navigation in ticket 03;
extracted one connection-observation Hook and the diagnostics composition, made
connection states explicit, and covered empty-error rendering and late replies
after Effect cleanup/re-setup. Six mocked-IPC units and the production frontend
build pass. `App` now compiles; the Hook retains the known finalizer bailout.
See [review evidence](../../../docs/evidence/milestone-01-ui-foundation.md#shell-boundary-review--2026-11-12).
Actual Tauri/fixture-desktop and native qualification were not rerun for this new
source; the earlier completed cohort remains preserved evidence. No commit or
amendment was performed by this follow-up.

2026-11-12 · Replaced the custom sidebar layout with official `@shadcn/sidebar`
structural components in the existing Base UI/base-nova setup, using
`collapsible="none"`. Preserved the fixed widths, accessible landmarks, empty
navigation and single `useSupervisorConnection` owner. Removed replaced sidebar
CSS and generated cookie/global-shortcut behavior; no Ticket 03 functionality.
See [Sidebar refactor evidence](../../../docs/evidence/milestone-01-sidebar-refactor.md)
for the reviewed generated dependencies and frontend checks. Actual Tauri desktop
requalification is deferred by the user; the new source is not qualified by the
retained older cohort. Changes remain uncommitted.

2026-11-13 · Requalified the current official Sidebar refactor and branding fix in
an actual fresh release Tauri bundle from
`46d8656483f19c19089dd3fa8c3a954f71235dcd`. See
[requalification evidence](../../../docs/evidence/milestone-01-sidebar-refactor.md#actual-tauri-requalification--2026-11-1213)
for exact 900×680 / 600×480 content sizes, measured 204 / 172 px sidebar widths,
fixed-sidebar/shortcut checks, visible keyboard and modal focus, all connection
states, explicit recovery, desktop-only crash/reconnect to the same owner and
lost-owner failed retry without replacement. Production bundled supervisor, fresh
private disposable state, before/after source hashes, sidecar provenance, commands,
37 captures (including retained unsuccessful harness attempts), results and
cleanup are retained. All owned processes stopped before state removal.
`pnpm verify` passed 33 Rust, 12 tooling and 8 mocked-IPC frontend units plus
format/static checks; release build/typecheck passed. No product fix was needed.
Compiler limitation remains in `useSupervisorConnection`, superseding the obsolete
historical App bailout note. All current 02a criteria are satisfied. Full provider
fixture/native workflows remain unperformed; live submissions 0. Index preserved,
changes uncommitted; other tickets, parent spec and milestone remain open.
