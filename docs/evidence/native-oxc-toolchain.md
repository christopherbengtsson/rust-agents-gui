# Native Oxc toolchain migration

2026-11-13. Branch `chore/native-oxc-toolchain`, separate worktree from main
`6ded0f24e7cd26f8b8dc877ccb51e69c707dce1d`. Existing registration work in the
original worktree remains untouched. This does not complete Milestone 01.

## Configuration and behavior

Pinned Oxlint 1.93.0, Oxfmt 0.78.0 and oxc-transform-react 0.156.0 replace eight
direct ESLint/Babel/Prettier dependencies. Vite 8.3.8, plugin-react 6.3.0,
React 19.3.1, TypeScript 7.0.2 and rustfmt remain. No JS bridge is used.
The native compiler pin satisfies plugin-react's `>=0.152.0 <0.157.0` peer range.
Oxfmt still brings Prettier transitively; its existing minimum-release-age
exception must remain. Removing it initially failed pnpm policy verification.

The 14 applicable native Hooks/compiler rules and 31 accessibility rules retain
explicit severities/options. Fixed native lint compiler options make the old
config/gating checks inapplicable. Expanded toolkit override patterns and native
import restrictions cover static/dynamic/type imports and re-exports. The narrow
table-cell gridcell exception retains that accepted case; native lint still flags
cell redundancy and interactive-to-noninteractive role conversion.

The observation Hook separates initial reads from explicit Refresh/Retry feedback.
A typed IPC outcome helper settles failures before presentation commits; a promise
callback makes those commits asynchronous. Admission, per-request symbols,
cleanup invalidation and finally release remain. No new Hook suppression was added.
The first simpler async refactor still triggered native set-state-in-effect and
was replaced. Existing mobile/heading exceptions retain their reasons.

Compiler diagnostics are visible; the Hook retains its finalizer bailout. A
successful build is not full compiler coverage. No performance improvement is
claimed by this implementation cohort.

## Commands and results

Apple Silicon Mac16,8, 48 GiB, macOS 26.2, Rust 1.99.1; default private worktree
Cargo target, debug static checks/test units and optimized release packaging.
Commands ran sequentially within this worktree; no borrowed target or sidecar.

| Command                                                           | Result                                                                                                                                                        |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `node --test scripts/frontend-lint.test.mjs` before configuration | Failed as expected; missing native configuration.                                                                                                             |
| Focused lint checks during implementation                         | Exposed unused-directive diagnostics without a rule code and an additional native cell-role diagnostic; assertions corrected to the actual explicit contract. |
| `corepack pnpm verify` initially                                  | Failed: fresh worktree static Tauri build required a staged sidecar.                                                                                          |
| `corepack pnpm build`                                             | Passed; staged release sidecar and bundled app.                                                                                                               |
| `corepack pnpm verify` after staging                              | Passed under shell Node 24.21.0; retained separately, not the supported cohort.                                                                               |
| `corepack pnpm verify` with Node 26.13.1 first on PATH            | Passed: formatting, lint, TypeScript/Clippy, 33 Rust units, 19 tooling checks, 12 mocked-IPC presentation tests.                                              |
| `corepack pnpm build` with Node 26.13.1 first on PATH             | Passed; release app 10.40 MiB; same JS/CSS filenames and reported sizes as the Node 24 cohort.                                                                |
| `corepack pnpm peers check`                                       | Passed.                                                                                                                                                       |
| `corepack pnpm install --frozen-lockfile`                         | Passed.                                                                                                                                                       |

Formatter write/check passes were repeated after documentation: hash checks
confirmed idempotence and non-writing check mode on source, scripts and configs.

Full logs, test-profile compilation/execution times, asset/source hashes and
release executable hashes are retained in [the artifact directory](native-oxc-toolchain/verify-node26.log).
Hashes were captured after release qualification; this is not a frozen before/after
source cohort. These are correctness executions; no cold/warm comparative benchmark
was run.
The added CLI tooling checks are an explicitly approved narrow exception to the
application unit-test policy. No new testing framework or application-process test
was added to the default gate.

## Actual Tauri qualification

Launched the fresh release executable with private disposable state recorded in
[desktop identity](native-oxc-toolchain/desktop.json). Actual Computer Use observed:

- Ordinary 900x680 and minimum 600x480 logical layouts, readable content and no
  horizontal overflow. Retina captures were inspected in the conversation.
- Tab/Enter skip-link navigation; Shift-Tab/Enter diagnostics; focus inside the
  modal; Escape returning focus to its trigger.
- Verified supervisor PID 11026 and desktop PID 10875 against executable paths
  and the exact private state path before signaling.
- Pausing only that supervisor produced disabled Checking feedback; its read
  timed out visibly. Diagnostics labeled the retained observation potentially
  stale. Resuming it and explicitly retrying restored Connected.
- Killing only the desktop and reopening the same private state produced desktop
  PID 11690 while preserving supervisor PID 11026 and its exact owner identity.
- Native Quit closed the desktop. Supervisor cleanup remains a future product
  capability; the qualification stopped its verified private owner explicitly.

Development mode built successfully with Node 26.13.1. Computer Use cannot select
its unbundled executable, so a disposable app bundle wrapped the same debug binary
and connected to this worktree's Vite server. Initial attempts exposed a missing
dev supervisor override and mode-755 test-state directory; setting the normal dev
supervisor environment variable and private mode 700 resolved both. A second server
attempt correctly rejected occupied port 1420; checks used the original owned
server instead.

A temporary diagnostics text marker appeared in the actual development WebView.
Removing it updated the open dialog back to its original text without closing the
dialog. The source was restored byte-for-byte. Development Retry connected to its
private supervisor. Verified qualification servers, desktops and supervisors were
stopped; no personal application state or provider process was used.

## Review and limits

Independent Standards and Spec reviews against the starting main SHA reported
zero findings. Final documentation and formatting checks follow those reviews.
Actual checks qualify the shell/status workflow, not the full milestone fixture
workflow, every supported OS, screen-reader output, latency, or native providers.
Native-provider submissions and authentication changes: none.

Rollback the native compiler independently by restoring the former Babel Vite
entry/dependencies. Lint/format rollback restores their former configuration,
commands, dependency pins and test adapter. Historical evidence remains intact.

Primary sources: [native React compiler](https://oxc.rs/docs/guide/usage/transformer/react-compiler),
[compiler lint categories](https://oxc.rs/blog/2026-08-18-react-compiler-support),
[native plugins](https://oxc.rs/docs/guide/usage/linter/plugins),
[formatter settings](https://oxc.rs/docs/guide/usage/formatter/unsupported-features).
