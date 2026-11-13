# 05c — workspace change inspection qualification

2026-11-13. Parent 05 / Milestone 01 A6 fixture qualification passed after fixes.
Tested implementation/qualification revision: `23fb5e8e3a02886c44bde3f7989a8ec8510c6b16`.
Native-provider qualification was **not run**; **zero provider submissions**.
Milestone 01 and its unrelated tickets remain open.

## Source and environment

Branch `feature/m01-change-inspection`; starting revision `504bd5047232978070440b3730ada6fa7ee8eaf2`;
full feature review base `8aa37abe7a7ab466b8321869bb280fbaad571695`.
The product working tree started clean. An unrelated `docs/IDEAS.md` appeared during
qualification and was preserved, excluded from this work.

[Source identity](milestone-01-change-inspection-qualification/source-identity.json),
[final source/artifact hashes](milestone-01-change-inspection-qualification/final-source-hashes.json)
and [desktop context](milestone-01-change-inspection-qualification/desktop-context.json)
retain toolchain, target, profile, source hashes, executable paths, sidecar provenance,
private fixture paths and binary hashes. MacBook Pro Mac16,8, Apple M4 Pro (12 cores),
48 GB, macOS 26.2 / 25C56; Node 26.13.1, pnpm 12.18.0, Rust/Cargo 1.99.1,
Git 2.50.0. React 19.3.1, Base UI 1.9.0, Tauri CLI/API 2.12.4; lockfiles unchanged.
WKWebView identifies AppleWebKit 605.1.15 in its captured user agent.

The initial staged sidecar was stale at `8aa37ab`; it was rebuilt before use.
Debug qualification used Cargo-reported fresh executables in a private bundle,
then production-built frontend assets served locally without HMR. The packaged
smoke check used the release app and its bundled release supervisor with no
supervisor override, at `tauri://localhost`. Only the private copy's bundle name/id
changed to distinguish it from unrelated installed apps; executable/resource hashes
were preserved. Final frontend identity is `index-BH_usD4q.js` / `index-BSp6w5We.css`.

## Fixes and review

- Tabs used registry orientation shorthand absent from this stylesheet, stretching
  the strip vertically. Explicit `data-[orientation=…]` selectors restore horizontal
  layout; the orientation prop now reaches Base UI. Compare [before](milestone-01-change-inspection-qualification/02-clean.png)
  and [after](milestone-01-change-inspection-qualification/03-tabs-layout-fixed.png).
- Scroll Area had the same selector mismatch. Explicit selectors restore track
  sizing. The final actual WebView reports vertical tracks **10 px wide** and the
  horizontal track **10 px high** in [its computed styles](milestone-01-change-inspection-qualification/final-assets-console-ax.txt).
  Both scrollbars, CSP restrictions and compact toolkit customizations remain.
- Large replies failed with macOS `EINVAL`: changing `SO_RCVTIMEO` after peer closure
  rejects even buffered data. Readiness polling now preserves the absolute frame
  deadline without changing socket options between fragments. Initial socket timeout
  setup, frame limits, error propagation and no-replay behavior remain. The private
  `FrameStream` seam gained `wait_readable`; no public IPC/schema/storage change.

The controlled large-frame regression failed with exactly `EINVAL` before the fix:
[red](milestone-01-change-inspection-qualification/transport-red.log),
[green connection checks](milestone-01-change-inspection-qualification/transport-final-units.log).
A supplied readiness timeout also verifies no read, second exchange or leaked stream.
The [real OS comparison](milestone-01-change-inspection-qualification/transport-native-comparison.log)
reads the same production preview with the rebuilt client, reproduces failure with
its retained old client, and passes again with the rebuilt client.
The [socketpair probe](milestone-01-change-inspection-qualification/transport-probes.log)
is opt-in OS evidence, excluded from units.

The code-review skill ran independent Standards and Spec reviewers over the entire
feature against `8aa37ab`, followed by fixes. Standards found the scrollbar selector
issue; its final follow-up reports **0 unresolved findings**. Spec reports
**0 unresolved findings**. No unrelated refactor or new provider behavior was added.
The component guide records the toolkit deviations.

## Checks and measurements

Commands ran sequentially in the worktree's real, owned default `target/`, using
warm caches and the test profile for Rust units. No fresh-private-target compilation
was measured. Unit execution needs no staged sidecar and substitutes application
Git/filesystem/storage/process outcomes. No application integration test was added.

| Check                  | Exact command                                         | Result                                                                                                                                     |
| ---------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| Initial complete gate  | `fnm exec --using 26.13.1 corepack pnpm verify`       | Exit 0; 7.26 s; 108 Rust + 25 tooling + 71 UI; formatting, lint, TypeScript, all-target/all-feature Clippy passed.                         |
| Final complete gate    | `fnm exec --using 26.13.1 corepack pnpm verify`       | Exit 0; 8.21 s; **110 Rust + 25 tooling + 71 UI**; all static checks passed. Rust harness compilation 0.118665 s, separate from execution. |
| Warm unit measurements | `fnm exec --using 26.13.1 corepack pnpm test:measure` | Exit 0; 36.67 s; one warm-up + five warm runs per harness; compilation 0.080655 s separately.                                              |
| Initial packaging      | `fnm exec --using 26.13.1 corepack pnpm build`        | Exit 0; 32.70 s; superseded by final fixes.                                                                                                |
| Final packaging        | `fnm exec --using 26.13.1 corepack pnpm build`        | Exit 0; **43.91 s**; release app 14.31 MiB; actual packaged smoke passed.                                                                  |

Warm execution median / range: Rust **0.409184 / 0.404289–0.412434 s**;
Node tooling **0.895329 / 0.890657–0.909006 s**;
UI **4.760636 / 4.751386–4.790851 s**. Rust and Node use one test thread;
Vitest uses one worker. The existing frontend chunk-size warning remains; it is
not a test failure. [Machine-readable command summary](milestone-01-change-inspection-qualification/check-summary.json),
[final gate](milestone-01-change-inspection-qualification/verify-final.log),
[warm runs](milestone-01-change-inspection-qualification/unit-measurements.log),
[packaging](milestone-01-change-inspection-qualification/packaging-final.log).

Targeted commands, failed red attempts, frontend builds and Cargo artifact JSON
are retained beside these logs. Final documentation formatting and commit hooks
are recorded separately; unchanged passing suites were not repeated for prose.

## Actual desktop fixture cohort

[Executable preparation/observation helper](milestone-01-change-inspection-qualification/fixture.py)
and [05b's instructions](milestone-01-change-inspection-refresh/fixture-instructions.md)
use two disposable repositories and private application state. Product actions
were performed in the actual app; no mocked IPC or provider fixture supplied UI
observations. A private Git wrapper logged commands and optionally delayed status
by 1.5 s. Permission denial, hidden `.git`, and supervisor SIGSTOP/SIGCONT were
**synthetic faults** against real production adapters.

Each numbered screenshot has a matching `-ax.txt` accessibility capture.

| Scenario                                          | Result and evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Complete clean comparison and HEAD                | Pass: [02](milestone-01-change-inspection-qualification/02-clean-ax.txt), [03](milestone-01-change-inspection-qualification/03-tabs-layout-fixed.png), second workspace [37](milestone-01-change-inspection-qualification/37-beta-clean-settled-ax.txt). Idle/loading after fresh owner is [20](milestone-01-change-inspection-qualification/20-fresh-owner-ax.txt); no false clean state.                                                                                                                                                                                                                                                                                     |
| Modification, addition, deletion                  | Pass: unified source numbers/content [04](milestone-01-change-inspection-qualification/04-modified.png), keyboard-selected addition [05](milestone-01-change-inspection-qualification/05-added-keyboard-ax.txt), deletion [06](milestone-01-change-inspection-qualification/06-deleted-keyboard-ax.txt).                                                                                                                                                                                                                                                                                                                                                                       |
| Same-status further edits                         | Pass after a settled preview: [10](milestone-01-change-inspection-qualification/10-same-status-ready-edit-ax.txt). Editing during acquisition also produced an honest expired-comparison error with retained content [07](milestone-01-change-inspection-qualification/07-same-status-refreshed-ax.txt); explicit recovery preserved the target.                                                                                                                                                                                                                                                                                                                               |
| Unsupported/empty/newline                         | Pass: binary [11](milestone-01-change-inspection-qualification/11-binary-ax.txt), non-UTF-8 [12](milestone-01-change-inspection-qualification/12-non-utf8-ax.txt), symlink [13](milestone-01-change-inspection-qualification/13-symlink-ax.txt), read limit [14](milestone-01-change-inspection-qualification/14-read-limit-ax.txt), individual-row limit [15](milestone-01-change-inspection-qualification/15-row-limit-ax.txt), missing newline [16](milestone-01-change-inspection-qualification/16-no-newline-ax.txt), empty addition [17](milestone-01-change-inspection-qualification/17-empty-added-ax.txt).                                                            |
| Limited preview                                   | Initial transport failure retained [18](milestone-01-change-inspection-qualification/18-preview-transport-failure-ax.txt); fixed actual desktop [21](milestone-01-change-inspection-qualification/21-large-preview-fixed.png), final packaged preview [48](milestone-01-change-inspection-qualification/48-packaged-inspection.png). Maximum workload: 200 rows / 48,989-byte reply, 49,027-byte envelope; [budget observation](milestone-01-change-inspection-qualification/maximum-preview-budget.json).                                                                                                                                                                     |
| Read/Git failures retain last-good results        | Pass: permission denial [30](milestone-01-change-inspection-qualification/30-read-failure-last-good-ax.txt), recovered [32](milestone-01-change-inspection-qualification/32-read-recovery-settled-ax.txt); Git-identity fault [33](milestone-01-change-inspection-qualification/33-git-failure-last-good-ax.txt), later successful target [40](milestone-01-change-inspection-qualification/40-reconnected-same-target-ax.txt). [Read-failure pause](milestone-01-change-inspection-qualification/read-failure-pause.json) showed no inspections across three seconds.                                                                                                         |
| Exact targets during pending navigation/reconnect | Pass: delayed Alpha refresh [35](milestone-01-change-inspection-qualification/35-alpha-pending-ax.txt), switch to Beta [36](milestone-01-change-inspection-qualification/36-switched-to-beta-ax.txt), Beta clean [37](milestone-01-change-inspection-qualification/37-beta-clean-settled-ax.txt). Paused supervisor produces stale errors [38](milestone-01-change-inspection-qualification/38-disconnected-last-good-ax.txt), connection error [39](milestone-01-change-inspection-qualification/39-connection-error-stale-ax.txt), reconnect to the same owner/file and edited content [40](milestone-01-change-inspection-qualification/40-reconnected-same-target-ax.txt). |
| Removed selection                                 | Pass: [41](milestone-01-change-inspection-qualification/41-selection-removed-ax.txt) retains its name/last-good preview and asks for explicit replacement.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     |
| Keyboard/focus/resizing                           | Pass: Enter/Space file selection [05–06], keyboard diff scrolling [22](milestone-01-change-inspection-qualification/22-diff-keyboard-scrolled.png), arrow/Enter tab activation plus retained reading position [23](milestone-01-change-inspection-qualification/23-tab-keyboard-reading-retained.png), keyboard splitter from 68% to 53% [24](milestone-01-change-inspection-qualification/24-keyboard-resize-ax.txt), independent collapse [25](milestone-01-change-inspection-qualification/25-panel-collapsed-ax.txt).                                                                                                                                                      |
| Minimum window, Sheet and long path               | Pass at 600×480 [26](milestone-01-change-inspection-qualification/26-minimum-window.png); Sheet with accessible title/initial focus [27](milestone-01-change-inspection-qualification/27-sheet-minimum-ax.txt); Escape returns focus to trigger [28](milestone-01-change-inspection-qualification/28-sheet-escape-focus-return-ax.txt); long-path selection closes Sheet, opens Changes and preserves full path [29](milestone-01-change-inspection-qualification/29-long-path-minimum.png). Default 900×680 layout [21]; breakpoint crossed both directions.                                                                                                                  |
| Visibility pause/return                           | Pass: both surfaces hidden [42](milestone-01-change-inspection-qualification/42-surfaces-hidden-ax.txt), no inspections across three seconds [counts](milestone-01-change-inspection-qualification/hidden-surfaces-pause.json), edited content on return [43](milestone-01-change-inspection-qualification/43-returned-after-hidden-edit.png). Native minimization without an intervening automation observation also paused [counts](milestone-01-change-inspection-qualification/minimized-without-observation.json); restored [44](milestone-01-change-inspection-qualification/44-restored-after-minimize.png).                                                            |

The final private database retains exactly **2 repositories, 2 registration actions,
2 workspaces and 2 workspace actions**, including after reconnect and packaged
restart. Original checkout file hashes remain unchanged. No provider process was
started. [Final observation](milestone-01-change-inspection-qualification/final-private-observation.json)
and [cleanup](milestone-01-change-inspection-qualification/cleanup.json) record this.

## Navigation feedback

One warm-up cycle and five measured cycles switched Workspace → Changes and back
with a retained 200-row, near-48-KiB preview. All native click events were trusted.
Final production frontend ran in the actual debug Tauri WKWebView with the
production supervisor path; Inspector reduced the content viewport to **900×398 CSS px**.
The app's default/minimum full-window layouts were qualified separately above.

The retained [instrumentation](milestone-01-change-inspection-qualification/navigation-instrumentation.js)
observes clicks and selected-tab mutations, marks two animation frames, and records
WebKit's Screenshots/Layout & Rendering timeline. Measurement is a conservative
feedback bound: input timestamp marker to the first captured composite at/after
those two animation frames. Every retained frame was visually checked for the new
content. It is not a hardware-input-to-display scanout measurement, and does not
wait for the tab color transition to finish. JS timestamps have approximately
1 ms precision; native trace timestamps supply the composite timing. Inspector,
screenshot capture and automatic inspection remained active during measurement.

| Navigation          | Five-run median | Range          | 100 ms target |
| ------------------- | --------------- | -------------- | ------------- |
| Changes → Workspace | **34.71 ms**    | 33.84–34.96 ms | Pass          |
| Workspace → Changes | **56.73 ms**    | 56.50–58.54 ms | Pass          |

[Raw samples/results](milestone-01-change-inspection-qualification/navigation-results.json),
[asset-bound observations](milestone-01-change-inspection-qualification/navigation-observations.json),
[analyzer](milestone-01-change-inspection-qualification/analyze-navigation.py),
[importable compressed trace](milestone-01-change-inspection-qualification/navigation-timeline.json.gz),
[trace provenance/pruning](milestone-01-change-inspection-qualification/navigation-trace-identity.json).
All non-screenshot trace records remain; twelve full feedback screenshots remain
both separately and in the trace. The huge lossless full export is retained in the
private fixture directory with hashes. WebKit's [timeline reference](https://webkit.org/web-inspector/timelines-tab/)
documents composite screenshots and export. These results qualify tab feedback
under this inspection workload; full-MVP concurrent-chat/history latency is deferred.

## Failed attempts and limits

- CUA could not select the unbundled dev executable. The private debug bundle was
  used instead. Terminating the fnm parent left descendants; the exact owned tree
  was explicitly stopped. No unrelated supervisor was signaled.
- Automation observations after Quit could relaunch a private bundle without its
  private environment, yielding an incompatible existing default-owner response.
  No actions were admitted there. Final cleanup closed the fixture processes;
  retained user state/owners were not changed. Screen locking temporarily prevented
  input/screenshots; the user unlocked the Mac and qualification resumed.
- HMR and cached earlier frontend assets complicated early captures. The final
  timing cohort used a fresh URL and verified final bundle filenames/computed
  scrollbar sizing. The superseded [timing attempt](milestone-01-change-inspection-qualification/navigation-initial-attempt/README.md)
  is retained, not used for acceptance. Its first-post-mutation screenshot method
  could precede content feedback; the final conservative method corrected this.
- An initial socketpair probe blocked on filling an unread socket buffer; it was
  stopped and reduced to 1,000 bytes. Its result did not establish product behavior.
  An initial real-client probe failed to compile with a nonexistent ID constructor;
  the retained probe uses the public tuple identity. Pillow for visual contact-sheet
  QA was installed only in a private temporary venv, not repository dependencies.
- An initial minimize probe immediately observed/restored the window, so reads
  continued. The retained second probe omitted that observation and measured pause.
- Conflict, submodule, special-file, unusual raw-path and paging boundary mechanics
  retain controlled-unit evidence; they were not all recreated in the OS desktop.
  Git did not inventory the prepared FIFO, so no desktop special-file claim is made.
  AX names/focus and keyboard behavior were checked; VoiceOver speech and a separate
  formal contrast audit were not performed.
- Packaged Quit was observed. Whole supervisor shutdown-on-Quit belongs to ticket 09;
  the qualifier explicitly stopped its exact private owner afterwards. No power-loss,
  oldest-supported-macOS or native-provider qualification is claimed.

Raw AX captures and command logs retain their original whitespace. A whole-staged
`git diff --check` reports trailing whitespace/blank EOFs in those raw artifacts;
[its output is retained](milestone-01-change-inspection-qualification/raw-evidence-whitespace-check.log).
The separately scoped code/prose diff check passes. Raw evidence was not rewritten
to remove those tool-produced bytes.

## Parent 05 cross-check

| Original acceptance item                                                     | Evidence                                                             |
| ---------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Focused product block, keyboard selection, readable diff/identity/navigation | 04–06, 21–29, 35–37, 48                                              |
| Empty/loading/error/unsupported, bounded preview, labeled last-good          | 02–03, 11–21, 30–33, 38–41                                           |
| Modification/addition/deletion list and text previews                        | 04–06                                                                |
| Comparison identity and refresh after further edits                          | 10, 40, 43; fixture edits only; chat integration is later            |
| Honest binary/unsupported rendering                                          | 11–17                                                                |
| Git/read failures retain last-good without apparent clean                    | 30–33, 38–40                                                         |
| Blocking effects away from rendering and exact workspace target              | Controlled ownership units; 35–40; measured navigation frames        |
| Deterministic content/identity/budget tests plus separate desktop states     | 05a/05b coverage retained; final 110 Rust/71 UI; actual cohort above |

All eight parent items have current-scope evidence. This completes parent 05's
fixture A6 slice, not Milestone 01 or native-provider acceptance.
