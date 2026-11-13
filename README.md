# Agent Workspace

Local macOS desktop workspace with a Tauri presentation connected to a separate
Rust supervisor. The completed baseline is Milestone 01 tickets 01–02: startup, private
supervisor discovery, status and reconnect. Ticket 02a established the React/shadcn shell; its completed
[desktop cohort](docs/evidence/milestone-01-ui-foundation.md#desktop-qualification-follow-up--2026-11-12)
is preserved. The current [Sidebar refactor and branding fix](docs/evidence/milestone-01-sidebar-refactor.md#actual-tauri-requalification--2026-11-1213)
passed actual Tauri requalification on 2026-11-12–13.
Ticket 03 adds existing-repository registration, durable identity, per-repository
details and restored navigation. Ticket 04 adds immediate isolated-workspace creation and explicit recovery.
Change inspection, Codex setup, chat and the remaining milestone tickets remain
unimplemented.

## Development

Requires Apple Silicon macOS 13.3 or newer, Xcode Command Line Tools, Rust through rustup, Node
and Corepack. Use the versions in `rust-toolchain.toml`, `.node-version` and
`package.json`; retain the lockfiles when reproducing the baseline.

```sh
corepack pnpm install --frozen-lockfile
corepack pnpm dev
```

Development logs its freshly built supervisor executable and persistent state:
`~/.agent-workspace/dev/<canonical-worktree hash>/p<protocol>-<app-version hash>`.
Compatible restarts reuse it; other worktrees/protocols/versions are isolated.
An explicit `AGENT_WORKSPACE_STATE_DIR` is honored exactly, including its existing
owner/version checks. Release state is unchanged. Rebuilding does not stop an old
supervisor; default dev no longer connects to an incompatible release owner.

Workspace labels come from a [curated catalog of 500 footballers](docs/research/footballer-catalog.md).
Unused plain names are preferred per repository, then numbered variants. Saved
names stay reserved with their history. New branches and folder components are
readable; internal UUID identities remain stable. Existing branches and folders
are preserved, with clean legacy labels and explicit location details.

The supervisor is the execution owner. Refresh retrieves observations; desktop
crash/disconnection leaves the supervisor alive, and reconnect does not replay
actions. Explicit Quit with owned cleanup is future ticket 09. Use private disposable
application state for qualification and never remove an active owner's lock.

## Testing and build

Read [testing and verification](docs/agents/verification.md) before selecting
commands. The checked-out package scripts define available commands. Tickets
01–02 predate that policy; their completed desktop/process evidence is retained.
The current default test gate is unit-only, with controlled filesystem, process,
transport and time outcomes. The tooling gate also runs bounded native Oxlint CLI configuration checks
against disposable source fixtures.

| Command                         | Purpose                                                                                                                                                                                    |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `corepack pnpm test`            | Compile the Rust core library unit harness, then execute Rust, tooling and mocked-IPC presentation unit tests; report compilation and execution separately. No staged sidecar is required. |
| `corepack pnpm test:measure`    | Compile once; measure one warm-up and five warm executions of each unit harness.                                                                                                           |
| `corepack pnpm compile`         | Check workspace compilation independently.                                                                                                                                                 |
| `corepack pnpm check`           | TypeScript and Clippy, including retained qualification targets; no test execution.                                                                                                        |
| `corepack pnpm format:check`    | Oxfmt and rustfmt checks.                                                                                                                                                                  |
| `corepack pnpm lint`            | Frontend Hook/compiler and JSX accessibility lint; toolkit Tauri dependency restrictions.                                                                                                  |
| `corepack pnpm verify`          | Formatting, frontend lint, static checks and unit tests, each once; excludes packaging and process qualification.                                                                          |
| `corepack pnpm qualify:process` | Opt in to inherited supervisor-process qualification using disposable private state.                                                                                                       |
| `corepack pnpm build`           | Prepare the release sidecar and package the macOS app separately.                                                                                                                          |

Sidecar preparation stages Cargo's reported supervisor executable under
`src-tauri/gen/sidecar`, alongside `provenance.json` with source revision, dirty
state, target directory, profile and SHA-256. Private `CARGO_TARGET_DIR` values
are supported. Run the isolated staging regression tests with
`node --test --test-concurrency=1 scripts/prepare-supervisor.test.mjs`.
Build, static-check and unit scripts reject symlinked Cargo targets and targets
claimed by another worktree. Each target retains `.agent-workspace-owner.json`;
choose a fresh private target when moving between worktrees. Run one verification
or build command at a time per worktree; share download caches rather than targets.

The Vite server stays on `127.0.0.1:1420` with a strict port and excludes Rust
sources, sidecar staging and Cargo output from its watcher. The frontend targets
Safari 16.4 to match Tailwind 4; the macOS bundle minimum is 13.3. These are
compatibility settings, not qualification of every supported OS.

## Continuous integration

[GitHub Actions CI](.github/workflows/ci.yml) runs on pull requests targeting
`main`, pushes to `main`, and manual dispatch. One `macos-15` Apple Silicon job
uses the repository's Node, pnpm and Rust versions. It installs frozen frontend
dependencies, prepares the debug supervisor resource for Tauri's static checks,
runs `corepack pnpm verify`, then builds frontend assets without repeating the
TypeScript check. Resource preparation builds the supervisor without launching it.

CI does not package/sign the app, launch desktop or supervisor processes, run
provider sessions, or qualify native behavior. Those checks retain their separate
milestone gates. Actions are pinned to release commits. Rust dependency downloads and selected
compiled artifacts are cached; the target ownership marker, staged sidecars and
frontend outputs are excluded. Every cache hit still runs the full checks. pnpm
caching is deferred. See [cache research](docs/research/github-actions-caching.md)
for cache boundaries and validation. See the [CI research](docs/research/github-actions-ci.md)
for versions and sources. Hosted execution remains unverified until this branch
is pushed and a matching GitHub event runs.

## Ownership

- `crates/core`: Tauri-independent actions, observations and connection decisions.
- `crates/supervisor`: process hosting the core and owning execution/storage.
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
workflow is implemented. Protocol 4 requires matching desktop/supervisor builds. SQLite schema 3 migrates
existing registrations/workspaces without changing internal identities or physical locations.

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
