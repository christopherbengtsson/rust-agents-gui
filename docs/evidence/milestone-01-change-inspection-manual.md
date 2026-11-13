# Milestone 01 — manual workspace change inspection (05a)

2026-11-13. Implementation and controlled verification for ticket 05a on
`feature/m01-change-inspection`. Parent 05, automatic refresh (05b), desktop
qualification/packaging (05c), and Milestone 01 remain open. No provider work was
submitted. The dated ticket comment identifies the implementation commit.

## Source and implemented behavior

The [source manifest](milestone-01-change-inspection-manual/source-identity.json)
records SHA-256 identities for the reviewed final implementation relative to
`6ecf36b31a70dc2171a4949dc7c25d2c9bf0418e`. Ticket 04's product baseline remains
`8aa37ab`; the intervening commit records the accepted plan.

- `crates/core/src/workspaces/inspection.rs` owns ephemeral comparisons, admission,
  classifications, bounded last-good results, list pages and one selected preview.
  Its private tests substitute all application dependencies. `git/inspection.rs`
  supplies production mechanics through the shared bounded Git runner; it checks
  saved Git identity and opens regular files through held directory descriptors
  with no-follow traversal, type checks and bounded reads.
- The existing worker fairly rotates creation, registration/probes and inspection
  jobs. Effects run outside observation/storage locks. Its existing owned thread
  and shutdown path remain in use. Eight pending inspection jobs and eight cached
  workspace observations bound retention; queued/active records cannot be evicted.
- `WorkspaceRequest::Changes` and `ChangePreview` use the existing supervisor/Tauri
  transport. Protocol **5** rejects protocol 4 visibly. No SQLite migration or
  durable inspection-action ledger is added. The TypeScript shape/target validators
  live in `src/lib/changes.ts`; `src/lib/workspaces.ts` forwards through the existing
  `workspace_request` command.
- `WorkspaceInspection` owns one observation hook above the Workspace/Changes
  panels, the independently collapsible/resizable right file list and below-880-px
  Sheet. The Code Block/Scroll Area diff primitive uses explicit Git source numbers.
  Inactive heavy content unmounts while lifted selection/reading position survive.
  Existing workspace details, recovery and navigation stay reachable.

## Verification environment and results

Apple M4 Pro, Mac16,8, 12 cores, 48 GiB; macOS 26.2 ARM64 (25C56). Rust/Cargo
1.99.1; Git 2.50.0; Node 26.13.1; pnpm 12.18.0. React 19.3.1, Base UI 1.9.0,
Vite 8.3.8, TypeScript 7.0.2, react-resizable-panels 4.21.0 and Shiki 4.6.0
(lockfile). Optional highlighting source is installed but unused by this feature.
The worktree-default owned `target/` and warm caches were used; test profile,
one Rust harness thread and one UI worker. Checks/builds were run sequentially.
The complete-gate log retains a brief Cargo build-directory lock wait; no stale
binary contradiction or demonstrated build contamination occurred.

| Check                                                                                                             | Result                                                                                                                                                                                                                                                                        |
| ----------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm verify`                                                  | Exit 0, **8.67 s** wall. Format, lint, TypeScript, all-target/all-feature Clippy; **96 Rust + 25 tooling + 55 UI** checks. [Log](milestone-01-change-inspection-manual/verify.log). This was the complete gate before review corrections.                                     |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm test:measure`                                            | Exit 0, **30.36 s** wall. Compilation **0.070266 s**, then one warm-up and five warm runs of each harness. [Log](milestone-01-change-inspection-manual/unit-measurements.log).                                                                                                |
| `/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm frontend:build`                                          | Exit 0, **0.74 s** wall; Vite **340 ms**. TypeScript and production frontend compilation passed. [Log](milestone-01-change-inspection-manual/frontend-build.log). This is not desktop packaging.                                                                              |
| `/usr/bin/time -p node scripts/cargo.mjs test --locked -p workspace-core --lib inspection -- --test-threads=1`    | Post-review exit 0, **18 focused Rust units**, **1.40 s** wall; compilation **0.70 s**. [Log](milestone-01-change-inspection-manual/review-rust.log).                                                                                                                         |
| `/usr/bin/time -p node scripts/cargo.mjs clippy --locked --workspace --all-targets --all-features -- -D warnings` | Post-review exit 0, **0.82 s** wall. [Log](milestone-01-change-inspection-manual/review-clippy.log).                                                                                                                                                                          |
| Final compiled Rust harness, one warm-up + five warm executions with `--test-threads=1`                           | All **98 units** pass after review corrections; compilation **0.157599 s** separately. Median **0.300699 s**, range **0.298518–0.303226 s**. Exact compile command, executable and hash are in the [log](milestone-01-change-inspection-manual/review-rust-measurements.log). |

Initial complete-gate unit compilation was **1.465618 s** separately; its measured
Rust/tooling/UI executions were **0.703599 / 0.788081 / 4.156910 s**. Before review
corrections, warm medians/ranges were Rust **0.287165 / 0.286127–0.297016 s**,
tooling **0.789365 / 0.781145–0.809471 s**, UI **3.952283 / 3.924012–3.997533 s**.
Review changed only Rust code/tests. The affected Rust checks and warm measurements
were repeated; unchanged passed frontend/tooling checks were retained. Commit
hooks separately check formatting and lint without rerunning the full suite.
Fresh-private-target compilation was not measured.

Controlled coverage includes ready-target admission, unchanged saved identities,
manual refresh/coalescing, observation availability during effects, fair job
selection, text modifications/additions/deletions, empty/metadata-only results,
status/path parsing, old/new numbers, missing-newline markers, basic unsupported
content, changed evidence, last-good lists/previews, page expiry, count/escaped-byte
bounds and Git/environment construction. Mocked-IPC UI units cover exact targets,
refresh submission versus pending polling, owner changes, failure retention,
removed selections, inactive-content unmount, retained scroll context, Sheet
selection and focus return. These establish application decisions, not actual
filesystem durability, no-follow OS behavior or WebView responsiveness.

## Review and retained failures

The code-review skill reviewed the staged increment before the user's requested
commit, against the pinned starting revision. Independent Standards and Spec
reviewers read repository policy and the accepted slice. Standards initially
reported one P2; Spec reported two P2s (including the same binary-classification
issue). Both fixes received red/green regressions and separate reviewer follow-up.
Final Standards: **0 unresolved**. Final Spec: **0 unresolved**.

- The binary-marker regression initially failed with `Some(Binary)` instead of
  ordinary text. Classification now recognizes Git metadata before the first
  hunk, rather than phrases inside file content or filename headers.
- The oversized-row regression initially failed with `None` instead of
  `Some(PreviewLimit)` after a hunk header fit. A row that cannot fit by itself is
  now explicitly unsupported, and the final encoded response is rechecked.
- Early expected red tests failed compilation before parser/row types existed.
  Development checks caught a macOS device-ID conversion, Clippy issues, an unused
  generated React import and an extra Code Block keyboard stop; all were fixed.
- React Compiler initially skipped a logical assignment operator in the hook;
  explicit assignment removed the diagnostic. DOM tests needed controlled
  `ResizeObserver`/animation facilities and asynchronous Base UI unmount settlement.
  A pending-poll mock incorrectly returned idle; its scenario was corrected.
- A removed-selection regression exposed structured errors rendering as
  `[object Object]`; the existing repository-error formatter now preserves the
  actionable message.
- The first shadcn add failed because its nested executable selected pnpm 10's
  store rather than project pnpm 12. A temporary external pnpm-to-Corepack wrapper
  corrected runner selection. Installation was retried with the existing Button
  overwrite declined; compact source was preserved. No global tooling config was
  changed.
- Production frontend compilation retains Vite's warning for a **564.99 kB**
  minified JavaScript chunk (**179.70 kB** gzip); CSS is **98.72 kB** (**16.29 kB**
  gzip). No responsiveness claim is inferred from build size.

## 05b handoff and qualification limits

Use the actual Rust request/reply types and TypeScript validators as the contract;
this report is not a second schema. `Changes.refresh=false` and
`ChangePreview.load=false` only retrieve observations; the true variants admit
coalesced work. Pending settlement uses 250 ms observation polling. Two-second edit
refresh, visibility-based refresh policy and complete race/content coverage remain
05b. Preview fetches belong to the active Changes tab.

Each successful refresh creates a fresh comparison UUID against resolved HEAD and
keeps file IDs for raw paths surviving the prior comparison. Stale list cursors and
preview targets fail explicitly. Retained preview content carries its own comparison
UUID and HEAD, which may differ from a failed/pending requested comparison. A file
removed from the inventory remains selected and named until the user chooses
another; it is not silently replaced.

The implementation additionally limits acquired status/regular-file content to
256 KiB and inventories to 4096 files. Exceeding acquisition limits is an explicit
failure/unsupported result, never a clean or valid partial inventory. Git retains
its existing 256 KiB output and two-second command deadline. Serialized list and
preview budgets are 48 KiB, including actual metadata/escaping; lists are at most
32 entries, previews at most 200 rows with no diff pagination. Failure envelopes
re-budget retained preview rows when necessary. Observations are ephemeral and
bounded; restart/cache eviction returns idle and requires fresh inspection.

Production packaging, actual Git/SQLite/process inspection qualification, actual
Tauri desktop/CSP/keyboard/resizing/focus/performance qualification, and native
provider qualification are **not run**. They are not established by mocked units.
Existing completed milestone evidence is preserved. Ticket 05a authorizes no live
provider submissions. Parent 05 and the milestone remain open.
