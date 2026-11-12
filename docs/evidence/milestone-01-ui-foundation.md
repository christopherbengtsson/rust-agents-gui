# Milestone 01 ticket 02a — incremental UI foundation

Current source: the [official Sidebar refactor and branding fix](milestone-01-sidebar-refactor.md#actual-tauri-requalification--2026-11-1213)
passed actual Tauri requalification on 2026-11-12–13. Earlier cohorts below remain
historical evidence and do not qualify subsequent source changes.

Historical result: **02a complete**. The [desktop follow-up below](#desktop-qualification-follow-up--2026-11-12)
closes the deferred gate. The original implementation/deferred-check record is
preserved here as historical evidence. Other tickets and the milestone remain open.

2026-11-12 · Implementation and unit/build evidence. Desktop qualification deferred
by the user to a follow-up from the ChatGPT app; ticket completion is not claimed. No live agent
submissions, authentication changes or provider/adapter changes occurred. Tickets
01–02 and their completed evidence remain intact. The parent spec, milestone and
other tickets remain open.

## Source and stack

Baseline: `31dca30e86d88d13d280733a8ec1234c3f22add9`, plus this ticket's changes.
macOS 26.2 (25C56), arm64, Mac16,8 / Apple M4 Pro, 48 GB; Node 26.13.1,
pnpm 12.18.0, rustc 1.99.1, Cargo 1.99.1. Worktree-default Cargo target with
warm release caches for packaging; a fresh private unit target is described below.

React/React DOM **19.3.1**, Vite **8.3.8**, React plugin **6.3.0**, TypeScript
**7.0.2**, Tailwind / Vite plugin **4.3.5**, Base UI **1.9.0**, lucide-react
**1.72.0**, class-variance-authority **0.7.1**, cn **0.5.2**, tw-animate-css
**1.4.0**. Tauri JS API/CLI **2.12.4**; locked Rust Tauri **2.12.5**.
Presentation units use Vitest **5.1.0**, jsdom **31.0.0**, Testing Library React
**16.3.3** and user-event **14.6.9**. Resolved stable registry versions were
checked at implementation time; pnpm and Cargo lockfiles are retained.

Official shadcn CLI **4.23.0** generated `components.json` with Base UI, nova,
neutral, lucide, CSS variables, TSX and no RSC. Command:
`corepack pnpm dlx shadcn@latest init --base base --preset nova --yes --no-monorepo`.
The skill's `base-nova` CLI preset spelling was rejected; `nova` is the accepted
CLI spelling, yielding registry style `base-nova`. Initial Tailwind preflight
failed until its import was installed. Init then encountered an older shell
`pnpm` (store v10 versus pinned pnpm 12's v11), and stopped at dependency install.
No lockfile from that older command was adopted. A temporary PATH wrapper invoked
`corepack pnpm` for the generator's nested commands. The official command
`corepack pnpm dlx shadcn@latest add button badge empty alert dialog --yes`
generated exactly the five used component modules from
`https://ui.shadcn.com/r/styles/base-nova/{name}.json`. Their imports and composition
were inspected; source is retained in `src/components/ui`. Dependencies were
installed explicitly through pinned pnpm and retained in the lockfile. Local
changes to generated source are formatting only. Theme and shell CSS are
application-owned. No font/network runtime dependency or full component catalog
was added.

## Presentation and architecture

`src/App.tsx` owns shell composition and transient connection presentation. The
repository sidebar and main workspace region establish the compact dark layout;
there are no fabricated repository entries or inactive product buttons. The
empty state explains that repository registration is forthcoming. Shared theme
roles cover surfaces, text, borders, focus, errors and connected state. System
fonts, compact controls and restrained violet/green status accents follow the
[retained Conductor research](../research/conductor-incremental-ui-foundation.md).

The existing typed `supervisor_status` and `desktop_pid` IPC commands are the only
requests. Initial mount and explicit Refresh/Retry retrieve observations. An
in-flight request disables duplicate refresh; diagnostics opening performs no IPC.
Errors remain visible and retain the Rust guidance verbatim as text. Retained
observations are explicitly labeled stale after failed refresh. No polling,
automatic retry, turn, registration, chat, diff or execution policy was added.
Rust execution, storage, admission and reconciliation ownership is unchanged.

## Vite / webview compatibility check

Checked the [official Tauri Vite 8 guide](https://v2.tauri.app/start/frontend/vite/)
following the user's additional request. The server uses the matching fixed
`127.0.0.1:1420` URL and strict port; `clearScreen: false` preserves Rust errors.
Vite excludes `**/src-tauri/**`, `**/crates/**`, and `**/target/**`, so Rust changes,
sidecar provenance/staging and Cargo output do not trigger frontend reloads.
`frontendDist` remains `../dist`, with the existing Tauri hooks and CSP.
Mobile host/HMR and Windows configuration are outside this macOS-only slice.

The guide's example Safari 13 JavaScript target does not make Tailwind 4 CSS
compatible with that WebKit version. [Tailwind's official browser floor](https://tailwindcss.com/docs/compatibility)
is Safari 16.4. The Vite target is therefore explicitly `safari16.4`, and the
bundle declares macOS **13.3**, the Ventura generation of WebKit 16.4. See
[WebKit's release overview](https://webkit.org/blog/13966/webkit-features-in-safari-16-4/)
and [Tauri bundle minimum configuration](https://v2.tauri.app/reference/config/#minimumsystemversion-1).
Debug builds disable minification and produce source maps using Tauri's debug
flag. The packaged Info.plist was inspected and reports `LSMinimumSystemVersion`
**13.3**. This is a declared compatibility baseline, not an oldest-OS test.
Actual qualification is limited to macOS 26.2 on this machine.

## Three visual-reference artifacts

All are project-specific static SVG compositions, with labels embedded in each
artifact. SVG sources are the retained artifacts, inspected after rasterizing
with resvg 2.6.2 in a disposable directory; no rendering dependency was added to
the application. Proposed content is not shipped as an interactive app view.

| Repository-relative artifact                                                 | Status and meaning                                                                                                                                                                      |
| ---------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `docs/evidence/milestone-01-ui-foundation/implemented-empty-shell.svg`       | **Implemented composition**, static reference drawn from source; not a desktop screenshot. Empty shell, navigation, connection and working status controls.                             |
| `docs/evidence/milestone-01-ui-foundation/proposed-selected-workspace.svg`   | **Proposed** selected workspace. Shell/theme reuse implemented treatment; all repository/workspace entries, branch/path/provider and conversation selection are planned (03–04, 06–07). |
| `docs/evidence/milestone-01-ui-foundation/proposed-conversation-changes.svg` | **Proposed** conversation/change inspection. Sample transcript, composer, changed file and diff are illustrative only (05, 07); no agent was run.                                       |

## Unit checks and measurements

The ticket explicitly authorizes mocked-IPC presentation coverage; that is the
TDD seam used here. Repository policy governs the retained private Rust tests
and unit-only gate. No integration suite or
real-process unit dependency was added. Existing checks remain; `pnpm test` now
also runs the presentation units, and `pnpm test:ui` selects those alone.

Initial tracer check failed because `App` did not exist, then passed after the
shell implementation. All three presentation checks passed during development,
with TypeScript checks after implementation and after additional coverage:
error text/explicit retry and observational diagnostics; pending refresh blocks
duplicates and retained diagnostics become stale; remount performs status reads
only and exposes no future product actions. These establish presentation decisions
under supplied IPC outcomes, not OS ownership or actual webview accessibility.

`/usr/bin/time -p env CARGO_TARGET_DIR=/private/tmp/rag-ui-02a-unit-target corepack pnpm test:measure`:
exit **0**, whole command **8.59 s**. Fresh-private-target unit compilation
**3.668322 s**. Test profile, warm download cache, Rust harness threads 1, Node
concurrency 1, Vitest maximum workers 1. One warm-up plus five execution runs:

| Harness      | Passed each run | Warm-up    | Warm median | Warm range          |
| ------------ | --------------- | ---------- | ----------- | ------------------- |
| Rust core    | 33              | 0.313267 s | 0.005040 s  | 0.004209–0.005576 s |
| Tooling      | 12              | 0.100631 s | 0.090166 s  | 0.087041–0.091621 s |
| Presentation | 3               | 0.712319 s | 0.640551 s  | 0.638296–0.647520 s |

[Raw measured unit log](milestone-01-ui-foundation/logs/units-measured.txt).

## Build and desktop qualification

- Initial `/usr/bin/time -p corepack pnpm build`: exit **0**, **9.54 s**;
  [log](milestone-01-ui-foundation/logs/build-initial.txt). This preceded the
  explicit webview target and minimum-OS correction; not the qualified bundle.
- Corrected `/usr/bin/time -p corepack pnpm build`: exit **0**, **17.49 s**;
  [log](milestone-01-ui-foundation/logs/build.txt). Release desktop compilation
  **16.42 s**, bundle **10.38 MiB**, TypeScript/Vite build passed. Build caches
  partly invalidated by the deployment target change. Staged supervisor
  [provenance](milestone-01-ui-foundation/sidecar-provenance.json) records baseline
  revision, dirty true, target/profile and binary hash.
- Actual packaged desktop launched with fresh mode-700 disposable state and no
  supervisor executable override. Computer Use refused app access before UI
  inspection: `Computer Use was not approved to use Agent Workspace`.
  Keyboard, sizing, connection/error screenshots and reconnect remain pending
  that follow-up. The user confirmed the current client is Codex CLI and explicitly
  deferred desktop qualification to a follow-up task from the ChatGPT app. No
  AppleScript fallback was run. Launch alone is not qualification.
- Standalone `pnpm compile` and inherited `pnpm qualify:process` were not rerun;
  unchanged Rust policy retains prior separate process evidence. The native
  agent cohort is **not run** and no live submissions are authorized or made.
- Streaming/history performance targets are **not measured**: representative
  workloads do not exist in this slice. No full-milestone acceptance is claimed.

## Review and remaining gate

Applied the code-review skill with two fresh-context agents reviewing the initial
HEAD baseline plus the working diff/untracked source against repository standards
and ticket/spec requirements. Both found **no implementation defects or scope
creep**. Both identified one documentation inconsistency: README described the
02a shell within the completed baseline despite pending desktop qualification.
Corrected it to distinguish implemented source from completed qualification.
Review totals before correction: Standards 1 documentation consistency note;
Spec 1 matching documentation finding, resolved. Both reviewers rechecked the
correction and reported **0 remaining findings**. The known actual-desktop gate
remains open by user instruction, separately from source correctness.

The ChatGPT app follow-up should use the packaged production supervisor path
with fresh private state, qualify the 900×680 ordinary and 600×480 minimum content
sizes, keyboard traversal/visible focus/dialog Escape and return focus, loading
and connection/error details, Refresh/retry, desktop-only kill/reconnect to the
same supervisor and lost-owner guidance without replacement. Retain screenshots
and observed results under this evidence directory. Keep current CSP; no live
agent submissions. This remains ticket 02a work, not a claim that ticket 12 or
the parent milestone is complete.

Final complete verification gate is recorded below.

## Final verification gate

`/usr/bin/time -p corepack pnpm verify`: exit **0**, **3.27 s** wall, existing
worktree-default target, warm dev/test caches. Prettier/rustfmt passed; TypeScript
and all-target/all-feature Clippy passed (Cargo check **1.42 s**). All **33 Rust +
12 tooling + 3 presentation** unit checks passed. Unit compilation **0.104544 s**;
execution **0.049438 s** Rust, **0.094087 s** tooling, **0.667026 s** presentation.
Each check ran once in the complete gate. [Raw gate log](milestone-01-ui-foundation/logs/verify.txt).
Afterward only evidence/ticket text and milestone wording were updated; no
implementation change required another full run. `git diff --check` passed.

Source hashes are retained in
[source-sha256.txt](milestone-01-ui-foundation/source-sha256.txt) for the packaged
implementation and verification entry points. The staged and bundled supervisor
hash match the retained provenance. Oldest-macOS execution, actual desktop focus,
sizing, errors and reconnect remain unperformed as described above.

The temporary desktop and supervisor were stopped. The disposable-state cleanup
harness initially matched a nonexistent `--state-dir` option instead of the
supervisor's positional state argument, so it removed the test directory before
stopping that test-owned supervisor. This was corrected by matching its exact
bundled executable and unique positional state path and stopping it. No other
application state was touched. This cleanup error supplies no evidence for the
product's owned Quit/recovery guarantees; those remain later qualification.

## Desktop qualification follow-up — 2026-11-12

**Pass for ticket 02a.** Actual packaged Tauri/WebKit checks covered ordinary and
minimum layout, keyboard navigation and visible focus, diagnostics, connected,
pending, initial-error and stale-error presentation, explicit Refresh/Retry, and
desktop-only crash/reconnect. No product implementation changes were necessary.
No providers, adapters, authentication or CSP were changed; live submissions: **0**.
Ticket 12, other tickets, the parent spec and the milestone remain open.

### Cohort and retained artifacts

Source: `4420328f2ab2fdf97e0524817d0b94fbafbd99db`; initially clean checkout,
then dirty only with this qualification's evidence and documentation. The
application source remains unchanged. Same macOS 26.2 (25C56), Apple M4 Pro,
Mac16,8, 48 GB, arm64 and toolchain as the original record; exact readings are in
[environment.txt](milestone-01-ui-foundation/desktop-qualification/environment.txt).
The current manifest adds Babel 8.1.2, Rolldown Babel plugin 0.3.1 and React
Compiler 1.0.0 to the original stack. Locked Tauri remains JS/CLI 2.12.4 and Rust
2.12.5. Release profile, worktree-default `target`, warm caches.

The first preliminary UI checks used the retained older bundle. Hash inspection
showed that it predated the current compiler configuration. Those captures are
retained under `desktop-qualification/pre-rebuild/`, with
[preliminary-session.json](milestone-01-ui-foundation/desktop-qualification/preliminary-session.json);
they are not the final qualification cohort. Rebuilt current source, launched it
with a second fresh private state, and repeated the checks below. Early capture
and focus attempts, including a transient incomplete capture and a CUA
`noWindowsAvailable` binding error, are retained under
`desktop-qualification/early-current-captures/`; rebinding to the exact app path
and inspecting stable output resolved the harness issue. Final screenshots are
numbered **24–42**. They are unedited CUA JPEG bytes with matching `.jpg` suffixes.

[Current source hashes](milestone-01-ui-foundation/desktop-qualification/current-source-sha256.txt),
[sidecar provenance](milestone-01-ui-foundation/desktop-qualification/current-sidecar-provenance.json),
[session and binary hashes](milestone-01-ui-foundation/desktop-qualification/session.json),
and [screenshot dimensions/hashes](milestone-01-ui-foundation/desktop-qualification/screenshots.json)
identify the cohort. Desktop SHA-256:
`44700e9fc5527331e303343d7658195c47f6aef2536a8acde37a8a5f8a62d678`;
bundled supervisor SHA-256:
`c6e672cf42ace6f4c749a1fa64b9386bbc58423ef6927786441b85a46a321abb`.
The supervisor matches its staging provenance. Production assets include
`index-DcNlVMhu.js`, matching the retained compiler adoption build.

### Actual desktop observations

Native captures include a 32-point title bar and use Retina 2× pixels. Checked the
configured **900×680** and minimum **600×480** window frames (1800×1360 and
1200×960 captures). Also checked **900×680 and 600×480 web content** by increasing
frame height by 32 points (1800×1424 and 1200×1024 captures). This distinguishes
the window frame from content size rather than silently counting the title bar
as application content. Native resize stopped at the configured minimum.

All links in the following table are actual desktop screenshots from the current
release bundle, under the unchanged production CSP and `tauri://localhost`.

| Check                             | Observed result                                                                                                                                                                                                                                     | Retained screenshot                                                                                                                                                                                                                                                                                   |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ordinary connected diagnostics    | Pass. Shell, working status control, version, both PIDs, owner and revision readable.                                                                                                                                                               | [24](milestone-01-ui-foundation/desktop-qualification/24-current-ordinary-diagnostics.jpg)                                                                                                                                                                                                            |
| Keyboard entry and skip link      | Pass. Shift-Tab from diagnostics reaches visible Skip to workspace; Enter focuses the workspace with a visible outline.                                                                                                                             | [25](milestone-01-ui-foundation/desktop-qualification/25-current-skip-focus.jpg), [26](milestone-01-ui-foundation/desktop-qualification/26-current-workspace-focus.jpg)                                                                                                                               |
| Refresh and dialog keyboard       | Pass. Tab reaches Refresh with a visible ring; Enter refreshes. Shift-Tab reaches diagnostics; Enter opens it. Tab/Shift-Tab remain on its sole Close control; Escape closes and returns focus to the trigger.                                      | [27](milestone-01-ui-foundation/desktop-qualification/27-current-refresh-focus.jpg), [24](milestone-01-ui-foundation/desktop-qualification/24-current-ordinary-diagnostics.jpg)                                                                                                                       |
| Minimum layout and modal          | Pass. Navigation, empty explanation, status, controls and footer fit; diagnostic fields wrap without clipping or horizontal overflow. Keyboard Close focus stays visible.                                                                           | [28](milestone-01-ui-foundation/desktop-qualification/28-current-minimum-connected.jpg), [29](milestone-01-ui-foundation/desktop-qualification/29-current-minimum-diagnostics.jpg)                                                                                                                    |
| Content dimensions                | Pass. Both requested content sizes render legibly; sidebar narrows at the small size and error text wraps.                                                                                                                                          | [30](milestone-01-ui-foundation/desktop-qualification/30-current-content-600x480.jpg), [35](milestone-01-ui-foundation/desktop-qualification/35-current-content-900x680-error.jpg), [36](milestone-01-ui-foundation/desktop-qualification/36-current-content-900x680-recovered.jpg)                   |
| Pending status                    | Pass. Pausing only the private-state owner displays Checking connection… and a disabled Checking… button.                                                                                                                                           | [31](milestone-01-ui-foundation/desktop-qualification/31-current-minimum-checking.jpg)                                                                                                                                                                                                                |
| Timeout and stale details         | Pass. Bounded request fails visibly; Rust error and reconnect/reopen guidance remain readable at both sizes. Dialog retains prior PID/owner and explicitly says may be stale.                                                                       | [32](milestone-01-ui-foundation/desktop-qualification/32-current-minimum-timeout-error.jpg), [33](milestone-01-ui-foundation/desktop-qualification/33-current-minimum-stale-diagnostics.jpg), [34](milestone-01-ui-foundation/desktop-qualification/34-current-ordinary-timeout-error.jpg)            |
| Explicit retry                    | Pass. Resuming the owner alone leaves the error visible. Tab/Enter on Retry retrieves status, clears the error and returns Connected to the same owner.                                                                                             | [36](milestone-01-ui-foundation/desktop-qualification/36-current-content-900x680-recovered.jpg)                                                                                                                                                                                                       |
| Desktop crash/reconnect           | Pass. SIGKILL desktop 78366 only; reopened desktop 78599 shows supervisor **78396**, unchanged owner **78396-1794521017360729000**, revision **1**. Exactly one owner for the private state.                                                        | [37](milestone-01-ui-foundation/desktop-qualification/37-current-crash-reconnect.jpg), [process evidence](milestone-01-ui-foundation/desktop-qualification/current-crash-reconnect.txt)                                                                                                               |
| Lost supervisor                   | Pass for presentation/connection scope. After stopping that test owner, Refresh and another Retry report connection refused, preserve stale observations and instruct reopening. No replacement owner appears.                                      | [38](milestone-01-ui-foundation/desktop-qualification/38-current-lost-owner-error.jpg), [39](milestone-01-ui-foundation/desktop-qualification/39-current-lost-owner-stale-details.jpg), [process evidence](milestone-01-ui-foundation/desktop-qualification/current-lost-owner.txt)                   |
| Initial error without observation | Pass. Separate mode-500 disposable state produces Permission denied and No successful observation yet. Restoring mode 700 does not clear it automatically; explicit keyboard Retry starts/connects the bundled owner and shows current diagnostics. | [40](milestone-01-ui-foundation/desktop-qualification/40-current-initial-permission-error.jpg), [41](milestone-01-ui-foundation/desktop-qualification/41-current-no-observation-diagnostics.jpg), [42](milestone-01-ui-foundation/desktop-qualification/42-current-initial-error-retry-recovered.jpg) |

Retained accessibility observations corroborate
[minimum diagnostics](milestone-01-ui-foundation/desktop-qualification/29-current-minimum-diagnostics-ax.txt),
[lost-owner state](milestone-01-ui-foundation/desktop-qualification/39-current-lost-owner-ax.txt),
and [initial-error recovery](milestone-01-ui-foundation/desktop-qualification/42-current-recovered-ax.txt).
The CUA app connection sometimes needed a fresh binding or coordinate click;
results above require observed state changes, not attempted input alone.

### Reproduction and check results

Build command: `/usr/bin/time -p corepack pnpm build`, exit **0**, **9.52 s** wall;
release desktop compilation **8.01 s**, Vite **311 ms**, app bundle **10.40 MiB**.
[Build log](milestone-01-ui-foundation/desktop-qualification/build-current.txt).
This rebuild includes TypeScript and the current compiler-enabled production
frontend; no implementation refactor was made to force compiler coverage.

For each desktop cohort, create a disposable directory under `/private/tmp` with
mode 700; launch the packaged `Contents/MacOS/workspace-desktop` directly with
`AGENT_WORKSPACE_STATE_DIR` set to that directory and
`AGENT_WORKSPACE_SUPERVISOR` absent. Use CUA for resizing, Tab/Shift-Tab, Enter,
Escape and explicit Refresh/Retry. Match the test process's full bundled
executable and unique positional state path before signaling it. Pending/timeout:
`kill -STOP 78396`, click Refresh, inspect pending and then error; `kill -CONT
78396`, inspect retained error, explicitly Retry. Crash: `kill -KILL 78366`,
relaunch with the same state, compare diagnostics and `ps`. Lost-owner:
`kill -TERM 78396`, Refresh and Retry, confirm no process for that state.
Initial-error check uses a separate mode-500 directory, then `chmod 700` and an
explicit Retry. Exact state paths and process identities are in the session log.
These are manual controlled OS faults, not a new automated integration suite or
native-provider fixtures.

`/usr/bin/time -p corepack pnpm verify`: exit **0**, **3.30 s** wall. Prettier,
rustfmt, TypeScript and all-target/all-feature Clippy passed; Cargo static check
**1.46 s**. All **33 Rust + 12 tooling + 3 mocked-IPC presentation** units passed.
Existing worktree-default target, warm dev/test caches; Rust harness threads 1,
Node test concurrency 1, Vitest maximum workers 1. Unit compilation **0.105445 s**;
execution **0.003881 s** Rust, **0.090973 s** tooling, **0.713599 s** presentation.
Each retained check ran once in the complete gate.
[Raw gate log](milestone-01-ui-foundation/desktop-qualification/verify-current.txt).
The earlier measured warm-up/five-run series remains preserved; these gate times
are single correctness executions, not a new performance benchmark. After this
gate, only this result paragraph was updated; targeted formatting/link checks and
`git diff --check` passed. Product source still matches the qualified hashes.

Read the repository-local React/shadcn skills. Reviewed installed composition and
the official [button](https://ui.shadcn.com/docs/components/base/button),
[dialog](https://ui.shadcn.com/docs/components/base/dialog),
[alert](https://ui.shadcn.com/docs/components/base/alert),
[badge](https://ui.shadcn.com/docs/components/base/badge),
[empty](https://ui.shadcn.com/docs/components/base/empty) and
[Base UI dialog](https://base-ui.com/react/components/dialog.md) documentation;
the CLI documentation lookup is retained in
[shadcn-docs.txt](milestone-01-ui-foundation/desktop-qualification/shadcn-docs.txt).
No generated components were replaced or added.

All qualification-owned processes were stopped before removing their three
disposable state directories; see [cleanup](milestone-01-ui-foundation/desktop-qualification/cleanup.txt).
Initial CUA app lookup also launched a new default-state desktop/supervisor; both
were stopped before qualification and that default state was left on disk.
Pre-existing debug supervisors were untouched. This cleanup supplies no claim
about later product-owned Quit behavior.

### Limits and closure

Unit results establish presentation/core decisions; the actual-desktop results
above establish this empty shell and production connection path on this Mac.
There is no provider fixture workflow yet, and no native-provider cohort was
run. No agent submissions, runtime launches, repository registration, chat,
streaming/history performance measurements, owned Quit qualification, oldest-OS
execution or whole-milestone completion are claimed. Standalone `pnpm compile`
and inherited `pnpm qualify:process` were not rerun; prior evidence is retained.

**Compiler coverage limit:** `App` is still skipped for its unsupported
`try/finally`; `ConnectionDetails` and local UI components have compiler output.
The earlier [compiler diagnostics](react-compiler-adoption.md#compiler-coverage-inspection)
remain authoritative for that limit. Successful desktop behavior does not imply
full compiler coverage or a measured speedup.

All remaining **02a** acceptance items are satisfied. Other tickets and parent
acceptance stay open. No commit was created.

## Shell boundary review — 2026-11-12

Follow-up against `f28da71` with uncommitted frontend changes intended for the
user's amendment to ticket 02a. The prior desktop cohort above remains historical
evidence; it does not qualify this new source.

Kept the fixed sidebar and existing theme/markup. Inspected the project's
Base UI / base-nova registry item with `corepack pnpm dlx shadcn@latest info --json`,
`docs sidebar dialog button alert empty badge`, and `view @shadcn/sidebar`, then
read the official component documentation. The official
[Sidebar](https://ui.shadcn.com/docs/components/base/sidebar) is a reasonable
candidate when ticket 03 introduces real navigation entries. Its default
collapsible behavior switches to a sheet on small screens; `collapsible="none"`
can retain a fixed layout. The registry also adds input, separator, sheet,
skeleton, tooltip and a mobile Hook alongside the existing button. Neither menu
composition nor collapse behavior is exercised by this empty navigation yet, so
adding it now supplies little benefit. Reconsider it with working repository
navigation rather than committing to a permanent custom navigation toolkit.

Extracted `useSupervisorConnection` as the single observation owner called by
`App`, and `SupervisorConnection` as the indicator/diagnostics composition. The
Hook retains typed IPC reads, pending exclusion, retained stale observations and
`finally` cleanup. Its discriminated union requires an observation when connected
and an error when failed. Failure rendering now checks the state, so even an
empty rejection string exposes the alert and retry. Request identity invalidates
late replies after cleanup/re-setup; an obsolete finalizer cannot clear a newer
request. Cleanup suppresses UI updates without claiming to cancel backend work.
The stable refresh callback is an explicit Effect-dependency integration use of
`useCallback`, not a measured rendering optimization. Rust ownership is unchanged.

### Follow-up checks and limits

macOS 26.2 (25C56), arm64 Apple M4 Pro; Node 26.13.1, pnpm 12.18.0. Installed
dependencies/warm caches; existing React 19.3.1, Compiler 1.0.0, Vite 8.3.8 and
TypeScript 7.0.2. Checks ran sequentially, without Cargo compilation or sidecar
staging. Each behavioral/build check ran once; these are correctness checks, not
latency benchmarks.

| Command                                    | Result                                                                                                                                                                            | Duration                                                               |
| ------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `corepack pnpm test:ui`                    | Exit 0; 1 file, 6 mocked-IPC units, maximum workers 1. Existing failure/retry, stale/pending and remount checks plus empty-error and Strict Mode late-success/late-failure cases. | Vitest reports 0.953 s; process wall time was not separately captured. |
| `corepack pnpm frontend:build`             | Exit 0; TypeScript and production build, 2,253 modules.                                                                                                                           | 0.589 s tool wall time; Vite reports 342 ms.                           |
| In-memory Babel diagnostic transform below | Exit 0; `App` compiles (27 slots), `SupervisorConnection` compiles (22), `ConnectionDetails` compiles (24). Hook retains the known unsupported-finalizer bailout.                 | 0.095 s tool wall time.                                                |

Compiler inspection used the following command; this supersedes the previous
`App` bailout location for this source, without claiming a measured speedup:

```sh
node --input-type=module <<'JS'
import { transformFileAsync } from '@babel/core';
import compiler from 'babel-plugin-react-compiler';
for (const file of ['src/App.tsx', 'src/components/supervisor-connection.tsx', 'src/hooks/use-supervisor-connection.ts']) {
  const events = [];
  await transformFileAsync(file, {
    babelrc: false,
    configFile: false,
    parserOpts: { plugins: ['jsx', 'typescript'] },
    plugins: [[compiler, { logger: { logEvent(filename, event) {
      events.push({ filename, event });
    } } }]],
  });
  console.log(JSON.stringify(events, null, 2));
}
JS
```

Source SHA-256 values for the checked frontend:

```text
a622ec700de2c6cca69bc370b6a583509228f96c0108d2cf269649d471dfb50d  src/App.tsx
69a4c086a34a5ab068ca98a2a316f1d333f32079ab49e2cf5fa36a3f141fad3f  src/App.test.tsx
3643da889b18a118abfcb96de4ccb791e8c04aeeed142e975f7816f058d46969  src/components/supervisor-connection.tsx
2ef991901d2ed52a4ceb0c382eabba9a5d7b1ef65b3510c1ad0c1d4caa3f6eaf  src/hooks/use-supervisor-connection.ts
```

Manifest, lockfile and Vite configuration hashes still match the retained compiler
cohort. Targeted `corepack pnpm exec prettier --check` over the four frontend
files, this evidence, the compiler-coverage research and the 02a ticket passed
(exit 0, 0.090 s tool wall time). `git diff --check` passed. Rust/full verification,
packaging, actual Tauri requalification,
fixture-desktop and native-provider checks were not run for this follow-up. No
provider submissions or commits. Prior qualification artifacts are preserved.
