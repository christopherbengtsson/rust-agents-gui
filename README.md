# Agent Workspace

Local macOS desktop workspace with one UI-independent Rust core hosted inside native Tauri. The core owns SQLx storage, workspace actions, provider setup and whole-app Quit. WebView replacement observes the same core; native failure requires durable interrupted/uncertain recovery. No application supervisor, discovery socket or peer protocol is shipped.

Repository/workspace navigation, change inspection and fixture-qualified Codex setup are implemented. Durable streamed chat and cumulative foundation qualification remain later tickets. Historical milestone evidence remains under `docs/evidence`; the current native cutover starts with [ticket 06](docs/evidence/sqlx-architecture/06/README.md).

## Development

Requires Apple Silicon macOS 13.3 or newer, Xcode Command Line Tools, Rust through rustup, Node
and Corepack. Use the versions in `rust-toolchain.toml`, `.node-version` and
`package.json`; retain the lockfiles when reproducing the baseline.

```sh
corepack pnpm install --frozen-lockfile
corepack pnpm dev
```

Inspect the directory returned by `git rev-parse --git-path hooks` before changing
hook configuration. Preserve existing custom hooks by adding formatting/lint checks
to that setup. Otherwise, enable the repository hook with
`git config --local core.hooksPath .githooks`; an existing `.githooks` setting needs
no change.

Development prints its exact private state root, defaults to `~/.agent-workspace/dev/<canonical-worktree hash>/native-schema-1`, and honors `AGENT_WORKSPACE_STATE_DIR` exactly. Vite provides frontend HMR. Rust edits require completed Quit and an explicit `pnpm dev` restart; the launcher owns the native executable directly so a rebuild watcher cannot terminate the execution owner. Stop waits for native drain, reports pending cleanup and never escalates automatically.

See [private development state and reset](docs/development-state.md) before changing state. Old metadata/preferences need no migration; preserve every worktree/branch, provider home/native history and evidence set. Never remove `owner.lock` to bypass ownership.

## Testing and build

Read [testing and verification](docs/agents/verification.md) before selecting
commands. The checked-out package scripts define available commands. Tickets
01–02 predate that policy; their completed desktop/process evidence is retained.
The current default test gate is unit-only, with controlled filesystem, process,
transport and time outcomes. The tooling gate also runs bounded native Oxlint CLI configuration checks
against disposable source fixtures.

| Command                                                             | Purpose                                                                                                                                                                                    |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `corepack pnpm test`                                                | Compile the Rust core library unit harness, then execute Rust, tooling and mocked-IPC presentation unit tests; report compilation and execution separately. No staged sidecar is required. |
| `corepack pnpm test:measure`                                        | Compile once; measure one warm-up and five warm executions of each unit harness.                                                                                                           |
| `corepack pnpm compile`                                             | Check workspace compilation independently.                                                                                                                                                 |
| `corepack pnpm check`                                               | TypeScript and Clippy, including retained qualification targets; no test execution.                                                                                                        |
| `corepack pnpm format:check`                                        | Oxfmt and rustfmt checks.                                                                                                                                                                  |
| `corepack pnpm lint`                                                | Frontend Hook/compiler and JSX accessibility lint; toolkit Tauri dependency restrictions.                                                                                                  |
| `corepack pnpm verify`                                              | Formatting, frontend lint, static checks and unit tests, each once; excludes packaging and process qualification.                                                                          |
| `corepack pnpm qualify:storage -- <new-report.json>`                | Bounded actual SQLx/storage qualification in new disposable state.                                                                                                                         |
| `corepack pnpm qualify:effects -- <new-report.json>`                | Bounded synthetic process/descendant qualification.                                                                                                                                        |
| `corepack pnpm qualify:native -- <qualified.app> <new-report.json>` | Actual native/WebView/login-fixture lifecycle qualification, with a 180-second bound.                                                                                                      |
| `corepack pnpm build`                                               | Package the normal native macOS app separately; no sidecar preparation.                                                                                                                    |

Build, static-check and unit scripts reject symlinked Cargo targets and targets
claimed by another worktree. Each target retains `.agent-workspace-owner.json`;
choose a fresh private target when moving between worktrees. Run one verification
or build command at a time per worktree; share download caches rather than targets.

The Vite server stays on `127.0.0.1:1420` with a strict port and excludes Rust
sources and Cargo output from its watcher. The frontend targets
Safari 16.4 to match Tailwind 4; the macOS bundle minimum is 13.3. These are
compatibility settings, not qualification of every supported OS.

## Continuous integration

[GitHub Actions CI](.github/workflows/ci.yml) runs on pull requests targeting
`main`, pushes to `main`, and manual dispatch. One `macos-15` Apple Silicon job
uses the repository's Node, pnpm and Rust versions. It installs frozen frontend
dependencies and runs `corepack pnpm verify:ci`: generated drift, formatting, lint,
TypeScript, frontend bundling, Clippy and isolated units, each selected once.

CI does not package/sign the app, launch desktop processes, run
provider sessions, or qualify native behavior. Those checks retain their separate
milestone gates. Actions are pinned to release commits. Rust dependency downloads and selected
compiled artifacts are cached; the target ownership marker and
frontend outputs are excluded. Every cache hit still runs the full checks. pnpm
caching is deferred. See [cache research](docs/research/github-actions-caching.md)
for cache boundaries and validation. See the [CI research](docs/research/github-actions-ci.md)
for versions and sources. Hosted execution remains unverified until this branch
is pushed and a matching GitHub event runs.

## Ownership

- `crates/core`: UI-independent actions, observations, native lifetime and SQLx storage.
- `src-tauri`: forwards targeted requests and presents observations.
- `src`: React/TypeScript presentation with shadcn Base UI components and Tailwind.

`workspaces` owns repository identity, registration, availability and reconciliation;
`storage` owns SQLite metadata under private application state. Main and linked
worktrees share one repository identity; separate clones remain distinct. Reopening
retrieves saved registrations and action status. Missing locations stay visible,
and registration performs read-only Git inspection without starting agents.

Desktop preferences remember the last selection and an uncertain registration's
action ID. Refresh retrieves authoritative status; edited paths are separate explicit
requests after the pending action is resolved. No repository relocation or removal
workflow is implemented. Current metadata uses schema 4. Incompatible metadata is rejected;
select fresh private development state deliberately rather than migrating retired builds.

Adding a new repository records its first-workspace continuation atomically with
registration. The repository's + creates another immediately. Generated footballer
names, branches and managed folders remain stable across recovery. Creation uses
locally available origin/HEAD, otherwise exactly one usable local main/master;
missing or ambiguous defaults remain actionable. No creation dialog, base picker,
fetch, setup script or Codex launch occurs. Workspace details show the pinned base,
branch, folder, progress and partial artifacts. Explicit reconciliation retrieves
and safely continues the same workspace; refresh never repeats uncertain effects.

As remaining tickets add behavior, `workspaces` extends workspace policy and
`runtimes` owns provider process mechanics.
Keep these responsibilities in Rust; Tauri remains presentation and forwarding.

See the [product contract](docs/MVP.md), [milestone status](docs/MILESTONE-01.md),
[bootstrap evidence](docs/evidence/milestone-01-bootstrap.md) and
[discovery evidence](docs/evidence/milestone-01-discovery.md) and
[UI foundation evidence](docs/evidence/milestone-01-ui-foundation.md) and
[repository registration evidence](docs/evidence/milestone-01-repositories.md) and
[workspace evidence](docs/evidence/milestone-01-workspaces.md) and
[readable naming/dev follow-up](docs/evidence/readable-workspaces.md).

## Frontend tooling

Oxfmt owns frontend/document formatting at width 80, with import, Tailwind-class
and package-field sorting disabled. `.prettierignore` remains the shared ignore
file; commands also pass `.gitignore`. Rust formatting stays with rustfmt.

Oxlint runs native React Hooks/compiler and JSX accessibility checks plus direct
Tauri dependency restrictions in toolkit layers. Warnings and unused suppressions
fail lint. Explicit rule configuration avoids broadening scope with tool defaults.
The table-cell role exception preserves the accepted `td role="gridcell"` case.

Vite uses the experimental native React Compiler with recoverable diagnostics
visible. `oxc-transform-react` is pinned to 0.156.0 for plugin-react 6.3.0's peer
range. Unsupported constructs may remain unoptimized: the observation Hook keeps
its `finally` cleanup. TypeScript checking remains separate. Compiler output and
mocked IPC checks do not establish actual desktop behavior; see [migration evidence](docs/evidence/native-oxc-toolchain.md).

Frontend navigation uses file routes in `src/routes`; the URL owns repository,
workspace and Workspace/Changes view selection. Startup at `/` restores the last
valid main destination once. Explicit URLs and history take precedence; invalid
explicit targets stay visible with recovery guidance. Pending action IDs remain
separate from presentation preferences. Settings has an F06 route placeholder;
the footer entry and full Settings interaction remain F07.

`corepack pnpm generated:write` explicitly regenerates contracts and the committed
`src/routeTree.gen.ts`. `generated:check` renders expected route output in an
isolated mirror and rejects drift without repairing source. Dev, checking,
frontend build, Tauri and CI share this preparation owner; no prior Vite run is
needed for standalone TypeScript. CI checks TypeScript before bundling. The Router
plugin runs before React Compiler and splits optional route components. Generated
output has no manual formatter owner; route source files remain authoritative.
