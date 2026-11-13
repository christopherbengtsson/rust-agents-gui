# Milestone 01 — reliable workspace change refresh (05b)

2026-11-13. Implementation and controlled verification for 05b on
`feature/m01-change-inspection`, starting from 05a's recorded handoff at
`029e2fb4a9a5f428ffe666cd87057d014c6a59c6`. Parent 05, actual desktop qualification,
packaging (05c), and Milestone 01 remain open. No provider submissions occurred.
Implementation commit: `50685d5cd8325d595a8868ce3b768638e6806f91`.
The 05b completion comment links this evidence and the qualification handoff.

## Implemented behavior and ownership

WorkspacePanel owns one concrete presentation reader above workspace-keyed views.
WorkspaceInspection retains one observation hook above its tab panels, with
explicit preview-active and summary-visible inputs. Workspace/owner/request
and selection generations discard obsolete successes and failures. Disposing a
keyed view disconnects its publisher without releasing physically pending IPC;
new targets wait for that read to drain. The current view then observes only its
explicit target. No workspace creation, provider submission or durable retry is
part of these reads.

Visible summaries refresh two seconds after settlement, with one sequence in
flight. Pending Rust jobs settle through 250 ms observation-only reads rather
than repeat admission. Previews load only in Changes. Expanded desktop file panel,
open below-880-px Sheet and Changes count as visible surfaces. Hidden documents
or both hidden surfaces pause automatic reads, as do disconnection and failures.
Returning/reconnect first retrieves observations. Explicit manual Refresh remains
usable after failures, including transport failure while a job was pending.

The Rust inspection owner retains last-good observations, creates fresh comparison
IDs and preserves file IDs for surviving raw paths. Successful refresh explicitly
invalidates old preview observations and removes queued old-comparison previews
before effects. Preview completion checks current comparison as well as its exact
target. HEAD/status/selected-file evidence is revalidated around acquisition.
Same-status content edits replace the selected preview. A comparison refresh
resets list paging to First; old continuations expire explicitly.

The existing Rust request/reply contracts, protocol 5, shared worker, Git runner,
no-follow file acquisition, queue/cache bounds and storage schema are retained.
The new hook visibility input and required stable reader prop are internal
presentation interfaces. The component guide documents their ownership.

The single unified preview remains bounded to 200 complete rows and 48 KiB encoded
reply; lists remain at most 32 entries and 48 KiB. Acquisition is bounded to
256 KiB/4096 inventory entries. Unsupported content, valid limited previews, empty
and metadata-only changes, cancellation, complete clean inventories and failures
stay distinct. Last-good preview staleness is visible even for a same-comparison
read failure. Removed selections stay named; another file is never substituted.
Tabs unmount heavy content while retaining reading position; a different selected
file resets it. Resizable, Scroll Area, Sheet, compact foundation and existing CSP
remain in use. No highlighting, virtualization or manual memoization was added.
The configured React Compiler handles the presentation build.

## Controlled coverage

Rust units use typed supplied Git/filesystem outcomes through the existing private
inspection seam, keeping real parsing, policy, admission, bounds and command
construction. No Git/worktrees, filesystem/storage access, supervisor/provider
process or production-length wait enters behavior units. Git porcelain/diff and
fault inputs are synthetic; they do not qualify native Git or OS behavior.

Added coverage includes same-status edits with stable file IDs/new comparison IDs;
queued preview invalidation; HEAD/status/file-evidence changes before/after reads;
read and Git/output-budget failures retaining old comparison content and clearing
on success; inventory capture failure/recovery; CRLF/LF; missing final newline;
empty additions/deletions; raw/unusual/non-UTF-8 paths/content; binary content;
symlinks, submodules, conflicts and special files without text reads/diffs;
metadata-only changes; staged/unstaged cancellation; oversized acquisition and
valid-hunk row limits. Retained 05a units cover byte/frame limits, individually
oversized rows, escaped failure envelopes, list paging/expiry and worker fairness.

Mocked-IPC frontend units exercise controlled timers, two-second refresh and slow
reads; summary/preview observation-only polling; document/surface visibility;
disconnect/reconnect; workspace/owner/selection generations; A→B→A late preview
responses; queued manual refresh preservation; failure retention/recovery;
unmount/rejected-IPC cleanup; actual WorkspacePanel keyed navigation across
A→B→none→B; 200-row rendering, long paths, selection/scroll reset; inactive renderer
unmount and retained reading context; Sheet selection/dismissal/focus return;
arrow-key focus plus Enter activation; and keyboard separator resizing with supplied
layout/ResizeObserver outcomes. DOM checks provide partial accessibility/layout
evidence, not actual WebView rendering or OS qualification.

## Verification environment and results

[Source identity](milestone-01-change-inspection-refresh/source-identity.json)
records the baseline, final dirty-source hashes, unit executable/hash, versions,
profile and target ownership. Apple M4 Pro / Mac16,8, 12 cores, 48 GiB; macOS 26.2
ARM64 (25C56). Rust/Cargo 1.99.1, Git 2.50.0, Node 26.13.1, pnpm 12.18.0.
Dependency versions remain locked: React 19.3.1, Base UI 1.9.0, Vite 8.3.8,
TypeScript 7.0.2 and react-resizable-panels 4.21.0. The worktree-default owned
`target/`, warm caches, test profile, one Rust harness thread and one UI worker
were used. Verification/build commands ran sequentially. Reviewers ran no checks.

| Command                                                                                          | Result                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm verify`                                 | Exit 0, **8.79 s** wall. Format, lint, TypeScript, all-target/all-feature Clippy; **108 Rust + 25 tooling + 70 UI** checks before the review correction. [Log](milestone-01-change-inspection-refresh/verify.log). |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm test:measure`                           | Exit 0, **34.58 s** wall. Compilation **0.078904 s** separately, then one warm-up + five warm executions per harness. [Log](milestone-01-change-inspection-refresh/unit-measurements.log).                         |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm frontend:build`                         | Initial exit 0, **0.60 s** wall, Vite **208 ms**; superseded for changed frontend by the final build below. [Initial log](milestone-01-change-inspection-refresh/frontend-build.log).                              |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm test:ui src/workspace-changes.test.tsx` | Post-review exit 0, **22 focused UI units**, **1.39 s** wall. [Log](milestone-01-change-inspection-refresh/review-ui.log).                                                                                         |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm exec tsc --noEmit`                      | Post-review exit 0, **0.26 s** wall. [Log](milestone-01-change-inspection-refresh/review-typescript.log).                                                                                                          |

The complete gate compiled units in **0.741179 s** separately and executed
Rust/tooling/UI in **0.896195 / 0.852865 / 4.613412 s**. Initial warm medians/ranges:
Rust **0.387310 / 0.383644–0.388670 s**; tooling **0.839059 /
0.834407–0.850227 s**; UI **4.484269 / 4.458140–4.559894 s**. Rust/tooling source
was unchanged by review; these passing checks/measurements are retained. Final
frontend measurements, production build and static rechecks are recorded below.
Fresh-private-target compilation was not measured. Standalone compile-only and
packaging commands were not run; Clippy's compilation, unit compilation/execution,
frontend compilation and packaging are separately identified here.

### Final affected frontend checks after review

The measurement wrapper is summarized in the command block below; it executes
`node node_modules/vitest/vitest.mjs run --maxWorkers=1` once for warm-up and five
warm runs under `fnm exec --using 26.13.1 python3 -`, measuring each subprocess with
`time.perf_counter()` and failing immediately on nonzero status. All six runs pass
**71 UI units** in six files. Warm median **4.502980 s**, range
**4.473797–4.575267 s**; wrapper wall **27.20 s**, exit 0.
[Final UI measurements](milestone-01-change-inspection-refresh/review-ui-measurements.log).
No Rust/tooling measurements were repeated for this frontend-only correction.

```python
import statistics, subprocess, time
command = ["node", "node_modules/vitest/vitest.mjs", "run", "--maxWorkers=1"]
durations = []
for n in range(6):
    start = time.perf_counter()
    result = subprocess.run(command)
    elapsed = time.perf_counter() - start
    if result.returncode:
        raise SystemExit(result.returncode)
    if n:
        durations.append(elapsed)
print(statistics.median(durations), min(durations), max(durations))
```

Final `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm frontend:build`
passes, exit 0, **0.59 s** wall; Vite **208 ms**. Final JavaScript is **566.81 kB**
(**180.21 kB** gzip); CSS **98.72 kB** (**16.29 kB** gzip). The existing chunk-size
warning remains, with no React Compiler diagnostic.
[Final build log](milestone-01-change-inspection-refresh/review-frontend-build.log).
This is frontend compilation, not desktop packaging.

Final `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm format:check`
passes, exit 0, **0.21 s** wall ([log](milestone-01-change-inspection-refresh/review-format.log));
`/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm lint` passes, exit 0,
**0.09 s** wall ([log](milestone-01-change-inspection-refresh/review-lint.log)).
Existing commit hooks retain format/lint checks without repeating full units.

## Review and retained failures

The code-review skill reviewed the actual uncommitted increment against pinned
`029e2fb`, including new files; a committed-only three-dot diff would omit it.
Independent Standards and Spec reviewers received repository authority, engineering
skills/references, accepted plan, requested slice and qualification limits.

Standards found one P2: actual WorkspacePanel navigation keyed away the reader,
so a new view could overlap old physically pending IPC despite generation guards.
A red/green regression through the actual panel reproduced two calls instead of
one before drain. WorkspacePanel now owns the reader above keyed presentation;
publisher cleanup retains the IPC slot while preventing obsolete view updates.
The regression includes an intervening no-workspace view. Standards follow-up:
**0 unresolved**. Initial Spec and follow-up: **0 unresolved**. No other product
scope or native qualification is inferred from this review.

Expected development failures and harness corrections were retained in this
session's tool output and are summarized here:

- The first timer regression expected two reads and observed one: 05a had no
  automatic two-second refresh. The surface-collapse regression observed four
  calls instead of one until visibility was wired through the component.
- The Rust invalidation regression found an obsolete preview job still executable
  after successful refresh. Rust now drops that queued work and marks its old
  observation expired while preserving last-good content.
- A manual recovery regression expected ready and stayed failed: an old failed
  preview blocked polling of an explicitly refreshed queued comparison. Pending
  comparison settlement now proceeds; only a failed preview for the ready current
  comparison pauses subsequent reads.
- The old owner-change test assumed overlapping reads could produce the new
  observation before an old promise settled. Its expectation now requires the
  physical lane to drain first, while still asserting obsolete replies are ignored.
- Narrow-window tests now open a changes surface before expecting observations;
  both surfaces initially hidden correctly produce no automatic reads.
- The first keyboard test assumed arrow focus automatically activated a tab.
  Installed Base UI defaults to explicit Enter/Space activation; the test now
  exercises that contract. A resize test initially supplied no measured group
  layout and raised “Previous layout not found”; controlled dimensions and a
  ResizeObserver delivery corrected the harness without product changes.
- Production frontend compilation retains Vite's existing >500 kB chunk warning.
  Bundle size is not evidence that navigation latency passes.

## Handoff and unperformed qualification

[Disposable fixture instructions](milestone-01-change-inspection-refresh/fixture-instructions.md)
cover changed, clean, unsupported, limited, refreshed and failed states. They show
synthetic permission and Git-identity faults through the production inspection
path and explicit restoration/manual recovery. They were prepared, not executed;
unit mocks are not desktop evidence.

Actual Git/filesystem/SQLite/process/desktop/CSP/keyboard/focus/layout/latency
qualification and production packaging are **not run** in 05b and remain 05c.
Native-provider qualification is **not run**, and no live submissions are
authorized. Existing completed milestone evidence is preserved. Parent 05 and
Milestone 01 remain open.
