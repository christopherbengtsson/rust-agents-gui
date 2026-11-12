# Ticket 02a — official Sidebar refactor

**Current result: pass for ticket 02a.** The refactor and branding fix passed
[actual Tauri requalification](#actual-tauri-requalification--2026-11-1213) on
2026-11-12–13. The implementation-time pending records below remain historical.

2026-11-12 · Implemented source; **actual Tauri requalification pending** at the
user's request. The completed desktop cohort and subsequent shell-boundary review
in [UI foundation evidence](milestone-01-ui-foundation.md) remain historical
evidence; neither qualifies this new source. No commit or provider submissions.

## Source and generated-component provenance

Baseline `90d140e96817b474b76283760487208e52b5b08d`, initially clean, plus this
uncommitted refactor. macOS 26.2 (25C56), arm64 Mac16,8 / Apple M4 Pro, 48 GB;
Node 26.13.1, pnpm 12.18.0. Existing installed dependencies and warm download/build
caches; frontend production build, no Cargo target/profile or sidecar staging.
React 19.3.1, Base UI 1.9.0, Vite 8.3.8, TypeScript 7.0.2, Tailwind 4.3.5,
React Compiler 1.0.0 and Vitest 5.1.0 remain unchanged.

Used official shadcn CLI **4.23.0**, configured `base-nova`, Base UI, lucide,
existing aliases and `src/style.css`. Inspected project context and installed
components with `corepack pnpm dlx shadcn@latest info --json`; ran
`docs sidebar` and fetched the official
[Sidebar documentation](https://ui.shadcn.com/docs/components/base/sidebar) and
[example](https://ui.shadcn.com/code/apps/v4/registry/bases/base/examples/sidebar-example.tsx).
Reviewed `add @shadcn/sidebar --dry-run`, `--diff button.tsx`,
`--view sidebar.tsx`, and `view @shadcn/sidebar` before composing the shell.
The exact registry response, CLI version and machine readings are retained in
[environment-and-registry.txt](milestone-01-ui-foundation/sidebar-refactor/environment-and-registry.txt).
Registry source: `https://ui.shadcn.com/r/styles/base-nova/sidebar.json`.

Installed using `corepack pnpm dlx shadcn@latest add @shadcn/sidebar --yes`,
answering **no** to overwriting `button.tsx`. The first noninteractive attempt
stopped at that prompt without generating files; the interactive retry completed.
The standalone `pnpm --version` launcher failed selecting the pinned binary
(`ENOEXEC`); a disposable PATH wrapper forwarded nested CLI calls to
`corepack pnpm`, which reports the pinned 12.18.0. No project tooling was changed.

Reviewed all seven added files: Sidebar and its registry dependencies Input,
Separator, Sheet, Skeleton, Tooltip and `use-mobile`. Imports resolve to existing
Base UI, lucide, cva and cn packages. The registry declares `cn`, already installed
at 0.5.2; **package.json, pnpm-lock.yaml and components.json are unchanged**.
Existing Button, Dialog, Alert, Badge and Empty sources/customizations are
untouched. Supporting registry components remain source dependencies of Sidebar;
the shell does not render their inputs, tooltips, skeletons or sheet.

Local changes to generated Sidebar: removed cookie writing and the global
Cmd/Ctrl+B shortcut; customized the group label to the existing 11 px muted,
medium-weight, letter-spaced section treatment. Other added source changes are
formatting only. No preset, theme replacement or additional component catalog.

## Composition and ownership

`App` composes `SidebarProvider`, `Sidebar collapsible="none"`, `SidebarHeader`,
`SidebarContent`, `SidebarGroup`, `SidebarGroupLabel`, `SidebarGroupContent` and
`SidebarFooter`. The existing named complementary landmark, named repository nav,
level-two repository heading, focusable main and skip-link target remain.
The label uses Base UI's `render` composition with the actual `h2`. Main remains
the existing workspace region; the inset variant is not used.

Retained the 204 px sidebar, 172 px at **max-width: 700 px inclusive**, 49 px
identity row, existing paddings, dark surfaces, borders, typography, empty copy,
connection summary and diagnostics. Sidebar semantic theme variables map to the
existing palette. Fixed-width utilities and structural flex layout replace the
old `.application-shell`, `.repository-sidebar`, `.repository-navigation` and
associated responsive CSS. Inspected the built CSS for both widths and the
inclusive breakpoint; this is source/build evidence, not rendered-layout proof.

`useSupervisorConnection` is unchanged and remains the single observation owner,
called only by `App`. `SupervisorConnection` is unchanged. Mount/refresh/retry
still retrieve only `supervisor_status` and `desktop_pid`; diagnostics receive the
same observation without new IPC. Its request identity, cleanup, pending guard,
error/stale presentation and Rust recovery ownership are unchanged.

No Ticket 03 behavior, navigation entries, selection, persistence, collapse
controls or drawer behavior is enabled. `collapsible="none"` returns the fixed
panel before the generated mobile sheet branch. The registry's responsive Hook
only observes viewport changes and cleans up its listener; it has no execution
or observation ownership.

## Verification and limits

Checks ran sequentially with installed dependencies/warm caches, no provider or
real-process fixtures. Vitest maximum workers: **1**. These are single correctness
runs, not new performance measurements; prior warm-up/five-run measurements remain
preserved in the original evidence.

| Command                        | Result                                                          | Wall time                 |
| ------------------------------ | --------------------------------------------------------------- | ------------------------- |
| `corepack pnpm test:ui`        | Exit 0; 1 file, **8 mocked-IPC units passed**                   | 1.343939 s; Vitest 1.11 s |
| `corepack pnpm frontend:build` | Exit 0; `tsc --noEmit` and production Vite build; 2,342 modules | 0.747679 s; Vite 385 ms   |

[Unit log](milestone-01-ui-foundation/sidebar-refactor/units.txt),
[build/typecheck log](milestone-01-ui-foundation/sidebar-refactor/frontend-build.txt).
The six prior units retain failure/explicit retry, diagnostics without replay,
pending/duplicate-read exclusion, stale observations, remount, empty rejection,
and Strict Mode late-success/late-failure coverage. Two new width scenarios
supply 600/900 px observations and check named landmarks, honest empty navigation,
only working actions, no global shortcut interception, cookie writes or drawer,
and exactly the two observation IPC reads. Escape return-focus is also asserted
in the existing error/diagnostics case. jsdom does not qualify CSS geometry,
visible focus or Tauri/WebKit behavior.

`corepack pnpm format:check`: **exit 0**, **0.582787 s** wall; repository
Prettier and rustfmt checks passed. [Formatting log](milestone-01-ui-foundation/sidebar-refactor/format-check.txt).
Final targeted documentation formatting and diff checks (including added files)
also passed; [log](milestone-01-ui-foundation/sidebar-refactor/final-checks.txt).
The initial added-file check wrapper stopped on `git diff --no-index --check`
exit 1 with no diagnostic output (the normal difference status for a new file).
The corrected wrapper accepts 0/1 only when no whitespace diagnostics are emitted;
this was a check-harness interpretation error, not a source failure.
Source hashes are retained in
[source-sha256.txt](milestone-01-ui-foundation/sidebar-refactor/source-sha256.txt).

Rust units/static checks, full `pnpm verify`, desktop packaging, actual Tauri
qualification, fixture-desktop workflows and native-provider qualification were
**not run** for this frontend-only refactor. No Rust, CSP, provider, authentication
or process lifecycle changes. The desktop recheck should cover ordinary/minimum
layout and widths, keyboard/skip-link/dialog focus, connected/pending/error/stale
diagnostics and explicit refresh/retry against this new source. Prior completed
milestone evidence remains preserved; the new source is not desktop-qualified.

## Branding review finding addressed — 2026-11-12

Addressed the shadcn P3 finding on `SidebarHeader.className`: moved
`app-identity` onto a branding-content row inside the header. Typography and the
existing icon treatment now belong to that content. The header uses only layout
utilities, including `border-b` in place of the removed CSS border declaration.
The 49 px row, 8 px icon/text gap, responsive horizontal padding, 12 px/600 text,
17 px icon and semantic border color are retained in source. No new component,
state, Effect, IPC, dependency or behavior was introduced; connection ownership
remains unchanged. Rechecked `corepack pnpm dlx shadcn@latest docs sidebar` and the
linked official documentation before the edit.

Same source baseline, machine, versions and warm installed-dependency/build
context as above; existing staged changes were preserved. This follow-up changes
only `src/App.tsx`, `src/style.css` and added evidence in the working tree. Prior
logs/hashes remain intact; the current touched-source hashes are in
[branding-source-sha256.txt](milestone-01-ui-foundation/sidebar-refactor/branding-source-sha256.txt).

Checks ran sequentially, once each; Vitest maximum workers 1. These are correctness
executions, not a new benchmark. No new test was added for this styling-only edit.

| Command                        | Result                                                      | Wall time                 |
| ------------------------------ | ----------------------------------------------------------- | ------------------------- |
| `corepack pnpm test:ui`        | Exit 0; all 8 mocked-IPC units passed                       | 1.189088 s; Vitest 955 ms |
| `corepack pnpm frontend:build` | Exit 0; TypeScript and production Vite build, 2,342 modules | 0.817537 s; Vite 380 ms   |

[Units](milestone-01-ui-foundation/sidebar-refactor/branding-units.txt),
[build/typecheck](milestone-01-ui-foundation/sidebar-refactor/branding-frontend-build.txt).
`corepack pnpm format:check` passed (exit 0, 0.542315 s); working-tree, staged
and added-file diff checks passed. Results are retained in
[branding-final-checks.txt](milestone-01-ui-foundation/sidebar-refactor/branding-final-checks.txt).

No desktop packaging, Rust verification, fixture-desktop or native checks were
run. Actual Tauri appearance/focus requalification remains pending; the source
change does not extend the historical desktop qualification. No commit or live
provider submissions.

## Actual Tauri requalification — 2026-11-12–13

**Pass for current ticket 02a source.** Rebuilt and qualified the actual packaged
Tauri/WebKit app after the official Sidebar refactor and branding fix. No concrete
product defect was found; no implementation, dependency, CSP, provider or runtime
change was needed. Other tickets, the parent spec and Milestone 01 remain open.
No live agent submissions or provider launches occurred. Previous evidence is
preserved and supplies no qualification claim for this cohort.

### Source, bundle and environment

Source HEAD **`46d8656483f19c19089dd3fa8c3a954f71235dcd`**, initially clean:
no staged, unstaged or added files were present. Qualification used working-tree
files directly, enumerating tracked and untracked inputs rather than building a
HEAD-only export. Subsequent dirty state contains only qualification evidence and
documentation. The index is unchanged and all changes remain uncommitted.
[Before](milestone-01-ui-foundation/sidebar-desktop-requalification/source-before.sha256)
and [after](milestone-01-ui-foundation/sidebar-desktop-requalification/source-after.sha256)
SHA-256 manifests cover 83 product/configuration/tooling/skill inputs and match.
[Source confirmation](milestone-01-ui-foundation/sidebar-desktop-requalification/source-confirmation.txt),
[initial Git state](milestone-01-ui-foundation/sidebar-desktop-requalification/git-before.txt)
and [index snapshot](milestone-01-ui-foundation/sidebar-desktop-requalification/index-before.txt)
are retained.

macOS **26.2 / 25C56**, arm64 Mac16,8, Apple M4 Pro (12 cores), 48 GB;
Node **26.13.1**, pnpm **12.18.0**, rustc/Cargo **1.99.1**. Installed dependencies,
warm worktree-default dev/test/release caches and target ownership marker;
no shared mutable target or fresh-target compilation. Stack remains React
19.3.1, Base UI 1.9.0, Tailwind 4.3.5, TypeScript 7.0.2, Vite 8.3.8, React
Compiler 1.0.0, Vitest 5.1.0, Tauri JS/CLI 2.12.4 and locked Rust Tauri 2.12.5.
[Environment readings](milestone-01-ui-foundation/sidebar-desktop-requalification/environment.txt)
retain an `xcodebuild -version` exit 1 because only Command Line Tools are selected;
release packaging succeeded with those tools. This is not a product failure.

`corepack pnpm build` prepared the release supervisor from Cargo's reported
executable, ran TypeScript/Vite and freshly compiled the desktop, then packaged
`target/release/bundle/macos/Agent Workspace.app`. No older bundle was used for
this cohort. Bundled desktop equals the release build output; bundled supervisor
equals the Cargo-reported executable, staged resource and provenance hash.
[Sidecar provenance](milestone-01-ui-foundation/sidebar-desktop-requalification/sidecar-provenance.json)
records source revision, dirty true (evidence only at build time), release profile,
opt-level 3, exact target and executable path. The supervisor's unchanged hash is
expected for unchanged Rust source; the new desktop hash differs from the older
qualification. [Bundle and asset hashes](milestone-01-ui-foundation/sidebar-desktop-requalification/bundle-hashes.json):

```text
Desktop:    d269e1f2afba38a93a4724193ae18cf24027ce33eab0abfe4394ea47d043056c
Supervisor: c6e672cf42ace6f4c749a1fa64b9386bbc58423ef6927786441b85a46a321abb
Assets:     index-8PWl-ozw.js, index-DzO0iUHb.css
```

Launched the exact packaged `Contents/MacOS/workspace-desktop` directly, with
`AGENT_WORKSPACE_SUPERVISOR` removed and `AGENT_WORKSPACE_STATE_DIR` pointing to
fresh mode-700 `/private/tmp/rag-02a-sidebar-5i4kosut`. Diagnostics' desktop PID
matches the launched process. The app ran at `tauri://localhost` under the
unchanged production CSP. Bundle declares macOS 13.3; oldest-OS execution remains
unperformed. [Session](milestone-01-ui-foundation/sidebar-desktop-requalification/session.json),
[process commands](milestone-01-ui-foundation/sidebar-desktop-requalification/process-commands.txt),
[bundle metadata](milestone-01-ui-foundation/sidebar-desktop-requalification/bundle-context.json)
and [CUA commands](milestone-01-ui-foundation/sidebar-desktop-requalification/cua-commands.txt)
retain exact launch/fault/interaction context.

Read AGENTS, verification policy, MVP, milestone, ticket/parent context and both
prior evidence records. Applied repository-local React and shadcn skills. Ran
`corepack pnpm dlx shadcn@latest docs sidebar dialog button` and reviewed the
linked official [Sidebar](https://ui.shadcn.com/docs/components/base/sidebar),
[Dialog](https://ui.shadcn.com/docs/components/base/dialog) and
[Button](https://ui.shadcn.com/docs/components/base/button) docs;
[lookup output](milestone-01-ui-foundation/sidebar-desktop-requalification/shadcn-docs.txt).
No component regeneration was necessary.

### Actual-desktop results

Screenshots below are unedited CUA JPEG bytes. Native captures include a **32 pt
(64 pixel) title bar at 2× Retina scale**. Capture 03 is **1800×1424 pixels**:
900×712 frame, exactly **900×680 content**. Capture 15 is **1200×1024 pixels**:
600×512 frame, exactly **600×480 content**. Initial and reopened 900×680 frames
have 648 pt content and are identified separately in the
[screenshot dimensions/hash manifest](milestone-01-ui-foundation/sidebar-desktop-requalification/screenshots.json).
Pixel inspection places the ordinary sidebar border at x=406–407 with workspace
starting at x=408 (**204 CSS px**), and minimum border at x=342–343 with workspace
starting at x=344 (**172 CSS px**). The 49 px branding/header rows and 17 px icon
remain visually consistent. [Pixel readings](milestone-01-ui-foundation/sidebar-desktop-requalification/geometry.txt)
and the retained read-only Swift decoder script corroborate the source widths.

| Check                                   | Actual observation                                                                                                                                                                                                                                                                                                                                                  | Screenshot(s)                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ordinary/minimum layout                 | Pass. Branding, repository empty state, workspace explanation, both controls and footer readable; no clipped content or horizontal overflow at either requested content size. Error copy wraps and stays readable.                                                                                                                                                  | [03](milestone-01-ui-foundation/sidebar-desktop-requalification/03-ordinary-resized.jpg), [15](milestone-01-ui-foundation/sidebar-desktop-requalification/15-content-600x480.jpg), [19](milestone-01-ui-foundation/sidebar-desktop-requalification/19-minimum-timeout-error.jpg), [23](milestone-01-ui-foundation/sidebar-desktop-requalification/23-ordinary-error.jpg)                                                                                                                                                                                                                                                                            |
| Fixed Sidebar                           | Pass. Sidebar stays visible at both sizes with no drawer, rail, collapse trigger or fabricated repository action. Cmd+B and Ctrl+B cause no sidebar/UI/focus change. Source has no global key handler; units independently assert neither event is prevented. `collapsible="none"` exits before the sheet branch.                                                   | [07](milestone-01-ui-foundation/sidebar-desktop-requalification/07-ordinary-shortcuts-unchanged.jpg), [16](milestone-01-ui-foundation/sidebar-desktop-requalification/16-minimum-shortcuts-fixed.jpg)                                                                                                                                                                                                                                                                                                                                                                                                                                               |
| Keyboard/visible focus                  | Pass. Tab reaches visible Skip to workspace; Enter focuses outlined workspace; Tab reaches Refresh/Retry, Shift-Tab reaches details. Visible focus on link, main, Refresh/Retry and details at both sizes.                                                                                                                                                          | [04](milestone-01-ui-foundation/sidebar-desktop-requalification/04-skip-focus.jpg), [05](milestone-01-ui-foundation/sidebar-desktop-requalification/05-workspace-focus.jpg), [06](milestone-01-ui-foundation/sidebar-desktop-requalification/06-refresh-focus.jpg), [11](milestone-01-ui-foundation/sidebar-desktop-requalification/11-details-focus.jpg), [33](milestone-01-ui-foundation/sidebar-desktop-requalification/33-minimum-skip-focus.jpg), [34](milestone-01-ui-foundation/sidebar-desktop-requalification/34-minimum-workspace-focus.jpg), [35](milestone-01-ui-foundation/sidebar-desktop-requalification/35-minimum-retry-focus.jpg) |
| Connected diagnostics/focus containment | Pass. Version, PIDs, owner and revision readable. Keyboard Enter opens diagnostics, repeated Tab and Shift-Tab remain on Close; background controls are absent from the modal AX tree. Escape closes and returns focus to details. Repeated at minimum size.                                                                                                        | [12](milestone-01-ui-foundation/sidebar-desktop-requalification/12-ordinary-diagnostics.jpg), [13](milestone-01-ui-foundation/sidebar-desktop-requalification/13-dialog-focus-contained.jpg), [14](milestone-01-ui-foundation/sidebar-desktop-requalification/14-dialog-return-focus.jpg), [17](milestone-01-ui-foundation/sidebar-desktop-requalification/17-minimum-diagnostics.jpg)                                                                                                                                                                                                                                                              |
| Pending/disabled refresh                | Pass. SIGSTOP of only qualification owner 88884, explicit Refresh: Checking connection… and disabled Checking… control. Diagnostics can open while pending and retain the prior observation marked may be stale.                                                                                                                                                    | [18](milestone-01-ui-foundation/sidebar-desktop-requalification/18-minimum-pending.jpg), [21](milestone-01-ui-foundation/sidebar-desktop-requalification/21-pending-diagnostics.jpg)                                                                                                                                                                                                                                                                                                                                                                                                                                                                |
| Timeout/stale/recovery                  | Pass. Two explicit observation attempts while stopped reach bounded read error (os error 35). Prior observation remains clearly stale. SIGCONT alone leaves error visible; explicit keyboard Retry clears it and retrieves the same owner/revision.                                                                                                                 | [19](milestone-01-ui-foundation/sidebar-desktop-requalification/19-minimum-timeout-error.jpg), [20](milestone-01-ui-foundation/sidebar-desktop-requalification/20-minimum-stale-diagnostics.jpg), [22](milestone-01-ui-foundation/sidebar-desktop-requalification/22-resumed-owner-error-retained.jpg), [24](milestone-01-ui-foundation/sidebar-desktop-requalification/24-explicit-retry-recovered.jpg), [25](milestone-01-ui-foundation/sidebar-desktop-requalification/25-owner-before-crash.jpg)                                                                                                                                                |
| Desktop-only crash/reopen               | Pass. SIGKILL desktop 88878; reopen as 89299 using same state. Supervisor **88884**, owner **88884-1794524236080516000**, revision **1** unchanged. Exact-state process count is one before and after reconnect. Only observation commands exist in the desktop; no execution action was replayed.                                                                  | [25](milestone-01-ui-foundation/sidebar-desktop-requalification/25-owner-before-crash.jpg), [26](milestone-01-ui-foundation/sidebar-desktop-requalification/26-crash-reconnect-same-owner.jpg)                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| Lost owner/failed retry                 | Pass. SIGTERM owner 88884; Refresh then explicit keyboard Retry both fail with connection refused (os error 61), retain stale identity and show reconnect/reopen/no-replay guidance. Exact-state supervisor count remains **zero** after failed retry; no replacement owner.                                                                                        | [27](milestone-01-ui-foundation/sidebar-desktop-requalification/27-lost-owner-error.jpg), [28](milestone-01-ui-foundation/sidebar-desktop-requalification/28-lost-owner-failed-retry.jpg), [29](milestone-01-ui-foundation/sidebar-desktop-requalification/29-lost-owner-stale-diagnostics.jpg)                                                                                                                                                                                                                                                                                                                                                     |
| Initial error/retry                     | Pass. Separate fresh mode-500 `/private/tmp/rag-02a-sidebar-initial-pmqt7jzx`, desktop 89346: permission denied and No successful observation yet. After mode 700 restoration, no owner exists and the error remains. Explicit keyboard Retry admits bundled owner **89374 / 89374-1794524475206736000**, revision 1, clears error and exposes current diagnostics. | [30](milestone-01-ui-foundation/sidebar-desktop-requalification/30-initial-error-minimum.jpg), [31](milestone-01-ui-foundation/sidebar-desktop-requalification/31-initial-error-no-observation.jpg), [32](milestone-01-ui-foundation/sidebar-desktop-requalification/32-permission-restored-error-retained.jpg), [36](milestone-01-ui-foundation/sidebar-desktop-requalification/36-initial-error-explicit-recovery.jpg), [37](milestone-01-ui-foundation/sidebar-desktop-requalification/37-recovered-current-observation.jpg)                                                                                                                     |

Matching `*-ax.txt` files retain full accessibility snapshots and focused-element
identity for every capture. No DOM injection, browser substitute, modified bundle
or test-only backend was used. Controlled OS faults targeted only verified full
executable paths/PIDs and the qualification-owned state.

### Unit, static and packaging results

Checks ran sequentially, one retained execution per complete gate, with Rust
harness threads 1, Node test concurrency 1 and Vitest maximum workers 1. This is a
correctness requalification, not a new benchmark. Prior warm-up/five-run
measurements remain historical; no fresh-target build or new performance claim.

| Command                | Result                                                                                                                                                                                                                                                                              | Wall time      |
| ---------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| `corepack pnpm verify` | Exit 0. Prettier/rustfmt, TypeScript, all-target/all-feature Clippy, **33 Rust + 12 tooling + 8 mocked-IPC frontend units passed**. Rust unit compilation separately 0.111708 s; execution 0.008229 s Rust, 0.102141 s tooling, 0.984977 s frontend. Clippy dev compilation 1.45 s. | **3.706598 s** |
| `corepack pnpm build`  | Exit 0. Release sidecar preparation, TypeScript/Vite build (2,342 modules, 390 ms), desktop release compilation (7.52 s), app bundle 10.40 MiB.                                                                                                                                     | **9.020347 s** |

[Complete gate log](milestone-01-ui-foundation/sidebar-desktop-requalification/verify.txt),
[packaging/typecheck log](milestone-01-ui-foundation/sidebar-desktop-requalification/build.txt).
No extra standalone UI/build runs were needed. Final documentation/JSON formatting,
tracked/staged/added-file whitespace, local evidence links, source/bundle hash and
unchanged-index checks are retained in
[final checks](milestone-01-ui-foundation/sidebar-desktop-requalification/final-checks.txt).

### Failures, cleanup and limits

No product failure was observed. Retained harness failures:

- Initial resize used logical-point coordinates; screenshot dimensions showed no
  resize. Corrected to screenshot-pixel coordinates and verified actual sizes.
- After Refresh disabled itself, focus moved to document; an initial batched
  keyboard sequence did not reach diagnostics. Captures 08–10 are retained under
  `early-attempts` and do **not** establish their filename's intended check.
  Re-derived focus, then completed checks in captures 11–17.
- Optional Pillow import failed (`ModuleNotFoundError`); no image was changed.
  `sips` supplied screenshot dimensions. Initial Swift decoder used an unsupported
  initializer; compiler output is in `geometry-failed.txt`. Corrected to the
  installed SDK's data initializer; read-only pixel inspection then passed.
- Cleanup recorder initially parsed the `ps` header as a PID after both final
  SIGTERM calls had succeeded. It stopped before deleting state. Corrected the
  parser and confirmed **all five owned PIDs absent** before removing either
  disposable state. [Cleanup log](milestone-01-ui-foundation/sidebar-desktop-requalification/cleanup.txt).
- The first added-file whitespace check caught trailing spaces emitted by the
  Swift compiler in the retained failed decoder log. Normalized that log's line
  endings, preserving its error text; the failed check remains in
  `final-checks-initial.txt`. Corrected checks passed. No product source changed.

Pre-existing debug supervisors and default-state supervisor 85891 were observed
before the run and left untouched, along with their state. Both qualification
states had private mode-600 socket/owner files; only these disposable directories
were removed after owned processes stopped. Manual cleanup is not qualification
of the later product-owned Quit behavior.

**Separate result categories:** unit/static/build **pass**; actual Tauri desktop
empty-shell/production supervisor cohort **pass**; full provider-fixture desktop
workflow **not run / not implemented**; native-provider cohort **not run**, live
submissions **0**. Standalone `pnpm compile`, inherited `pnpm qualify:process`,
full A1–A10 workflow, native streaming/approval/interrupt/resume, durable history,
active-work replay, owned Quit, oldest OS, light theme, navigation/history latency
and Ticket 03 behavior are unperformed. No allowance for native submissions was
requested or consumed. Empty-shell reconnect evidence does not qualify recovery
of future execution actions or full competing-owner/private-access/version cohorts.

The obsolete `App` finalizer bailout note does not apply: `App`,
`SupervisorConnection` and `ConnectionDetails` compile in the retained compiler
inspection; **`useSupervisorConnection` retains the unsupported-finalizer coverage
limit**. No new compiler transform, full-coverage or speedup claim is made here.
All current 02a acceptance items are verified; other tickets and milestone
acceptance remain open. No commit was created.
