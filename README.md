# Agent Workspace

Local macOS desktop workspace with a Tauri presentation connected to a separate
Rust supervisor. The completed baseline is Milestone 01 tickets 01–02: startup,
private supervisor discovery, status and reconnect. Tickets 03–06 are not implemented: repository registration, isolated workspaces, change
inspection and Codex setup. Chat and the rest of the milestone remain unimplemented.

## Development

Requires Apple Silicon macOS, Xcode Command Line Tools, Rust through rustup, Node
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
03–06 must use the current unit-test policy from the start; an isolated unit gate
is established in ticket 03. Inspect command behavior before assuming a script selects only unit tests.
Keep compilation, unit execution, static checks, complete verification and release
packaging separately identifiable.

## Ownership

- `crates/core`: Tauri-independent actions, observations and connection decisions.
- `crates/supervisor`: process hosting the core and owning execution/storage.
- `src-tauri`: forwards targeted requests and presents observations.
- `src`: TypeScript presentation.

As the remaining tickets add behavior, `workspaces` owns repository/workspace policy,
`storage` owns authoritative writes and `runtimes` owns provider process mechanics.
Keep these responsibilities in Rust; Tauri remains presentation and forwarding.

See the [product contract](docs/MVP.md), [milestone status](docs/MILESTONE-01.md),
[bootstrap evidence](docs/evidence/milestone-01-bootstrap.md) and
[discovery evidence](docs/evidence/milestone-01-discovery.md).
