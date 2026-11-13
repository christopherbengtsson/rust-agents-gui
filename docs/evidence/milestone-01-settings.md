# 06a — Settings and retained workspace context

2026-11-13. Implemented on `feature/m01-codex-setup` from the clean preparation
baseline `fbf6e7361af3046945d396d602536b3c6beffb94`. The commit containing this report
contains the implementation. The [source identity](milestone-01-settings/source-identity.json)
records file hashes and environment; existing milestone/predecessor evidence is
preserved. Parent 06 and children 06b–06d remain open.

## Behavior and 06b handoff

- `src/use-app-navigation.ts` owns Settings/workbench transitions, focus intent,
  disclosure visits and the shared `SelectionEligibility` contract. Domain hooks
  retain canonical selections; no router, mirrored identity, history engine or
  workspace view cache was added.
- Registration and workspace actions capture a navigation generation once per
  logical action. Settings entry, Back and explicit selection advance eligibility.
  Direct replies and status lookups settle the exact action and update records;
  they redirect selection only while the original generation remains eligible.
  Retrying the same uncertain action does not renew permission. Registration's
  initial-workspace handoff carries the generation into workspace lookup and
  application. Desktop preference/wire/storage representations are unchanged.
- Repository default selection is restored only during the first successful
  observation with its initial navigation permission. Subsequent list refreshes
  cannot select a newly registered record indirectly after its redirect was
  fenced. Observation of an already-selected workspace still updates that same
  record while Settings is visible.
- App retains its one connection, repository and workspace observation owner above
  one stable React Activity containing RepositorySidebar and WorkspacePanel.
  WorkspacePanel retains the physical WorkspaceChangesReader above its existing
  workspace-keyed view. Hide cleans up inspection listeners/timers and invalidates
  replies without cancelling the underlying IPC. Reveal observes through the same
  lane after outstanding IPC drains. No reader implementation change was needed.
- WorkspaceInspection retains the active tab, selected file, reading position and
  requested resizable layout. Activity restores DOM; the resizable library needs
  its requested layout supplied again through `defaultLayout`/`onLayoutChanged`.
  The layout API is grounded in the installed 4.21.0 types. See also
  [React Activity](https://react.dev/reference/react/Activity).
- SettingsScreen receives its heading ref and Back callback. It shows Settings,
  Agent runtimes and Codex, explicitly saying “No setup diagnostic has run.”
  No Check, Save, executable input, unavailable runtime or fabricated readiness is
  exposed. 06b should extend this block with its real observation/control contract.
- A Settings gear uses the compact Button in the sidebar footer. Entry focuses the
  Settings heading; Back focuses the retained trigger or workspace-main fallback.
  Skip links target unique active-main IDs. Transient dialogs/Sheet reset on a new
  visit; popup focus permission is bound to the original visible visit, so an old
  popup cannot restore focus after a later Back. Existing Base UI composition,
  compact controls, tokens and CSP integration remain in place. The component
  guide documents the new contracts. Base UI's
  [Dialog focus API](https://base-ui.com/react/components/dialog) supplies the
  `finalFocus` callback; no foundation regeneration or dependency change was made.

## Controlled checks

The agreed frontend seams use actual components, domain hooks, navigation and
reader code with mocked IPC/native-picker outcomes and controlled events/time.
Application Git, storage, supervisor and runtime effects remain substituted in the
unit gate. No integration-test framework or application process fixture was added.

Coverage includes:

- Keyboard Settings/Back/category actions, heading/return/fallback focus, unique
  IDs, one exposed main landmark, active skip target and truthful runtime content.
- Registration completion during Settings, after Back, after selection away/back
  to the same ID, and from an initially empty workbench; newly recorded rows still
  update. A shared App-owned status poll continues while the workbench is hidden,
  settles the exact registration and submits no second registration.
- Creation direct replies and action lookups after Settings and same-ID navigation;
  retry of the same uncertain action; initial-workspace handoff before/during a
  delayed lookup; polling updates to the same selected workspace while hidden.
- Diff DOM identity, selected tab/file, panel width/collapse and scroll position
  across repeated visits, including Strict Mode. Controlled ResizeObserver
  deliveries represent layout changes; they do not qualify WebView geometry.
- Inspection visibility-listener cleanup and timer pause, stale success/failure
  results settling while hidden or after reveal, draining the physical read lane,
  observation-only reveal and resumed polling.
- Open registration/connection dialog and changed-file Sheet dismissal, no hidden
  focus trap/reopened portal, obsolete popup focus suppression and late picker
  responses after a new visit.

The five changed frontend test files passed **96 checks** together. The complete
unit entry point additionally retains the existing unaffected checks.

## Verification and measurements

Apple M4 Pro, 48 GiB, macOS 26.2 / 25C56 ARM64; Node 26.13.1, pnpm 12.18.0,
Rust/Cargo 1.99.1. React 19.3.1, Base UI 1.9.0, TypeScript 7.0.2, Vite 8.3.8,
Vitest 5.1.0. Lockfiles are unchanged. Commands ran sequentially in this worktree's
owned default `target/`, with existing warm caches; Rust units use the test profile
and one harness thread, UI units one worker, tooling tests concurrency one.
No fresh-private-target cold compilation measurement was performed.

| Check                                                                      | Result                                                                                                                                                                                                                                                                                                                                                                                |
| -------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Focused units                                                              | `corepack pnpm test:ui src/App.test.tsx src/repositories.test.tsx src/workspaces.test.tsx src/workspace-changes.test.tsx src/use-app-navigation.test.tsx`: exit 0; 96 passed; Vitest 4.77 s. TypeScript and lint also passed during implementation.                                                                                                                                   |
| Complete gate                                                              | `corepack pnpm verify`: exit 0; **9.324522 s** wall; format, lint, TypeScript, all-target/all-feature Clippy; **110 Rust + 25 tooling + 99 UI** passed. Rust harness compilation **0.135911 s**, execution **0.395052 s** separately. [Log](milestone-01-settings/verify.log).                                                                                                        |
| Frontend production build                                                  | `corepack pnpm frontend:build`: exit 0; **0.659072 s** wall. Vite reports a JS chunk above 500 kB; no compiler diagnostic. This is frontend asset construction, not desktop packaging. [Log](milestone-01-settings/frontend-build.log).                                                                                                                                               |
| Warm unit measurements                                                     | `corepack pnpm test:measure`: exit 0; **39.080765 s** wall; one warm-up and five warm runs per harness. Compilation **0.070460 s**, separately. Median/range (s): Rust **0.387843 / 0.384658–0.388920**; tooling **0.784674 / 0.781674–0.813938**; UI **5.337488 / 5.194771–5.528799**. All six runs passed 110 + 25 + 99 checks. [Log](milestone-01-settings/unit-measurements.log). |
| Actual fixture desktop/process, WebView accessibility and packaged desktop | **Not run for 06a.** Reserved for 06d.                                                                                                                                                                                                                                                                                                                                                |
| Native provider                                                            | **Not run. Zero submissions and no provider process.** No login/runtime configuration change.                                                                                                                                                                                                                                                                                         |

## Development failures retained as results

These are development attempts, not claims against final source:

- The first Settings DOM test failed because the Settings button did not exist.
  The registration navigation parameter test then reproduced three failures
  (same-ID return, Settings and Settings/Back); the empty-workbench regression
  independently failed with heading `one` instead of `Workspace`.
- Retention tests reproduced panel width resetting from 37% to 32% on reveal.
  Retaining the requested layout fixed both normal and Strict Mode cases.
- The initial inspection-navigation test harness omitted SidebarProvider and
  produced a React Compiler refs diagnostic from accessing a mixed navigation
  object after passing its ref. Supplying the provider and destructuring the
  navigation result fixed the harness. Production build has no such diagnostic.
- The collapse fixture initially omitted its ResizeObserver delivery; reporting
  the controlled zero-width resize made its geometry consistent with collapse.
- An initial Strict Mode reader check incorrectly required exactly one reveal
  observation despite development effect replay (three calls instead of two).
  Exact production-lifecycle counts are checked without Strict Mode; separate
  Strict Mode checks retain state and resource cleanup coverage.
- TypeScript identified three mock `InvokeArgs` narrowing errors. Using the
  existing `object` guard fixed them; the subsequent typecheck and final gate pass.
- Existing shell assertions initially omitted the new Settings button/tab stop;
  they were updated to the new accessible keyboard order.
- The final diff whitespace check found a trailing space emitted by Vite in the
  captured build log. That log whitespace was normalized; command results are
  unchanged.

## Review

The code-review skill ran independent read-only Standards and Spec reviewers over
`git diff --cached fbf6e7361af3046945d396d602536b3c6beffb94`, including new files,
before the final commit. No source changes followed those reviews.

### Standards

No documented standards violations or substantive baseline design smells found.
Shared observations remain outside Activity; domain hooks retain selection;
navigation generations fence late selection without replay; transient disclosures
have deliberate reset/focus contracts. Component behavior is documented in the
guide. Repository policies and relevant React/shadcn guidance were inspected.

### Spec

No findings. Settings fences registration/creation selection while permitting exact
settlement and observations. Initial-workspace handoff is checked before lookup
and when applying results. One retained Activity preserves presentation and the
reader lane; focus, skip links and disclosure reset match the accepted contract.
No 06b runtime work or unrelated product scope was added.

**Unresolved findings: Standards 0; Spec 0.**

## Qualification limits

DOM checks establish application decisions, state and focus intent. They do not
establish actual WKWebView geometry, accessibility output, CSP enforcement,
responsiveness, real process behavior, durable storage or native compatibility.
The existing predecessor desktop evidence is preserved, not extended to this
source. Actual WebView qualification and packaging remain 06d; runtime adapter,
configuration persistence and Check setup remain 06b. Stop after this child.
