# Agent Workspace

Local macOS desktop workspace with a Tauri presentation connected to a separate
Rust supervisor. The completed baseline is Milestone 01 tickets 01–02: startup, private
supervisor discovery, status and reconnect. Ticket 02a established the React/shadcn shell; its completed
[desktop cohort](docs/evidence/milestone-01-ui-foundation.md#desktop-qualification-follow-up--2026-11-12)
is preserved. The current [Sidebar refactor and branding fix](docs/evidence/milestone-01-sidebar-refactor.md#actual-tauri-requalification--2026-11-1213)
passed actual Tauri requalification on 2026-11-12–13.
Ticket 03 adds existing-repository registration, durable identity, per-repository
details and restored navigation. Tickets 04–06 (isolated workspaces, change inspection
and Codex setup), chat and the rest of the milestone remain unimplemented.

## Development

Requires Apple Silicon macOS 13.3 or newer, Xcode Command Line Tools, Rust through rustup, Node
and Corepack. Use the versions in `rust-toolchain.toml`, `.node-version` and
`package.json`; retain the lockfiles when reproducing the baseline.

```sh
corepack pnpm install --frozen-lockfile
corepack pnpm dev
```

The supervisor is the execution owner. Refresh retrieves observations; desktop
crash/disconnection leaves the supervisor alive, and reconnect does not replay
actions. Explicit Quit with owned cleanup is future ticket 09. Use private disposable
application state for qualification and never remove an active owner's lock.

## Testing and build

Read [testing and verification](docs/agents/verification.md) before selecting
commands. The checked-out package scripts define available commands. Tickets
01–02 predate that policy; their completed desktop/process evidence is retained.
The current default test gate is unit-only, with controlled filesystem, process,
transport and time outcomes. The verification foundation originally planned for
ticket 03 was brought forward and remains the default gate.

| Command                         | Purpose                                                                                                                                                                                    |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `corepack pnpm test`            | Compile the Rust core library unit harness, then execute Rust, tooling and mocked-IPC presentation unit tests; report compilation and execution separately. No staged sidecar is required. |
| `corepack pnpm test:measure`    | Compile once; measure one warm-up and five warm executions of each unit harness.                                                                                                           |
| `corepack pnpm compile`         | Check workspace compilation independently.                                                                                                                                                 |
| `corepack pnpm check`           | TypeScript and Clippy, including retained qualification targets; no test execution.                                                                                                        |
| `corepack pnpm format:check`    | Prettier and rustfmt checks.                                                                                                                                                               |
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
workflow is implemented. Protocol 2 requires matching desktop/supervisor builds.

As remaining tickets add behavior, `workspaces` extends workspace policy and
`runtimes` owns provider process mechanics.
Keep these responsibilities in Rust; Tauri remains presentation and forwarding.

See the [product contract](docs/MVP.md), [milestone status](docs/MILESTONE-01.md),
[bootstrap evidence](docs/evidence/milestone-01-bootstrap.md) and
[discovery evidence](docs/evidence/milestone-01-discovery.md) and
[UI foundation evidence](docs/evidence/milestone-01-ui-foundation.md) and
[repository registration evidence](docs/evidence/milestone-01-repositories.md).
