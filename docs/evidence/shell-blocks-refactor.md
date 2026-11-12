# Shell block refactor — 2026-11-13

## Source and change

Baseline HEAD: `75bc36faa080e54c6a6ed543819d3d1e2c0a27fa`, with pre-existing
uncommitted frontend convention/lint work preserved. The refactor extracts
RepositorySidebar and WorkspacePanel, moves SupervisorConnection to blocks,
centralizes repeated shell values, and adds keyboard-only dialog regression
coverage. App retains the single observation Hook. No Rust or provider behavior,
toolchain dependency, or generated UI source changed.

The [source and binary hashes](shell-blocks-refactor/source-and-binary.sha256)
identify the checked implementation and fresh release bundle. The
[sidecar provenance](shell-blocks-refactor/sidecar-provenance.json) records the
release supervisor's Cargo target, revision, dirty state and hash. Hashes were
recorded after qualification; this is not a before/after frozen-source cohort.

## Static, unit, and packaging results

Apple M4 Pro, arm64, macOS 26.2; Node 26.13.1, pnpm 12.18.0; pinned Rust 1.99.1.
Default owned `target/`, warm cache; dev Clippy, test-profile units, release bundle.

| Check                                   | Result                                                                                    |
| --------------------------------------- | ----------------------------------------------------------------------------------------- |
| `/usr/bin/time -p corepack pnpm verify` | Exit 0, wall 5.18 s. Formatting, ESLint, TypeScript/Clippy and unit gate each run once.   |
| Core units                              | 33 passed, 1 thread; unit compilation 0.104322 s, compiled execution 0.006619 s.          |
| Tooling units                           | 16 passed, Node concurrency 1; execution 0.455496 s.                                      |
| Mocked-IPC frontend units               | 9 passed, Vitest maxWorkers 1; execution 1.223261 s.                                      |
| `/usr/bin/time -p corepack pnpm build`  | Exit 0, wall 9.77 s; includes production TypeScript/Vite build and release app packaging. |
| `git diff --check`                      | Passed.                                                                                   |

The new frontend unit exercises Tab navigation to the diagnostics trigger, Enter
activation, focus within the modal, Escape dismissal, focus return, and no extra
status reads. Existing units retain empty/error/pending/stale/retry, remount,
duplicate-read and Strict Mode late-response coverage. Timings are single-run
observations, not performance benchmarks; warm medians and fresh-target compilation
were not measured.

## Actual Tauri desktop qualification

Launched the fresh bundle executable with `AGENT_WORKSPACE_STATE_DIR` pointing to
a new private disposable directory. Desktop PID 93561; bundled supervisor PID 93566. Inspected the native app through Computer Use, not a browser mock.

- Ordinary window: 900×680 logical points (1800×1360 Retina capture, including
  native title bar). Fixed sidebar appears at the expected 204 px width;
  branding, empty navigation/content, connection footer and workspace footer
  remain readable with no horizontal overflow. [Screenshot](shell-blocks-refactor/ordinary.png).
- Tab reveals the skip link; Enter moves focus to the outlined main region.
  Shift-Tab reaches diagnostics; Enter opens the named modal with focus on Close.
  Escape returns focus to the diagnostics trigger. Confirmed in native AX state.
- Diagnostics retain current owner/process/version information.
  [Screenshot](shell-blocks-refactor/diagnostics.png),
  [AX state](shell-blocks-refactor/diagnostics-ax.txt).
- Resized through the native window corner to 600×480 logical points
  (1200×960 capture, including title bar). Sidebar contracts to the expected
  172 px width; navigation, workspace content and controls remain readable.
  Widths are corroborated visually against Retina captures, not DOM measurements.
  [Minimum shell](shell-blocks-refactor/minimum.png),
  [minimum diagnostics](shell-blocks-refactor/minimum-diagnostics.png).
- Sent SIGTERM only to the supervisor whose executable and private state path were
  verified for this run. Refresh showed an actionable connection error and Retry
  connection. Long error text wraps, workspace content scrolls, and navigation
  remains visible. No horizontal overflow observed.
  [Minimum error](shell-blocks-refactor/minimum-error.png),
  [ordinary error](shell-blocks-refactor/ordinary-error.png).
- Diagnostics retain the prior observation explicitly labeled potentially stale.
  [Screenshot](shell-blocks-refactor/stale-diagnostics.png),
  [AX state](shell-blocks-refactor/stale-diagnostics-ax.txt).
- Quit through the native app shortcut; confirmed desktop and the already stopped
  supervisor no longer exist before removing disposable state. Retained the
  [desktop log](shell-blocks-refactor/desktop.log).

This is shell/status desktop qualification, not the full milestone fixture workflow.
No native-provider submissions, screen-reader output qualification, chat/history
performance workload, or complete crash/restart cohort ran. Earlier milestone
results remain unchanged and preserved.
