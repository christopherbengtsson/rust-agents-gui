# Agent Workspace

Local macOS desktop workspace, using a Tauri presentation connected to a separate
Rust supervisor. This is the status-only bootstrap for Milestone 01, ticket 01.

## Development

Requires Apple Silicon macOS, Xcode Command Line Tools (`xcode-select --install`),
Rust through rustup, Node **26.13.1**, and Corepack. Rust **1.99.1**, rustfmt, and
Clippy are selected by `rust-toolchain.toml`. With fnm installed:

```sh
fnm install
fnm use
corepack pnpm install --frozen-lockfile
corepack pnpm dev
```

`packageManager` pins pnpm **12.18.0**. Corepack avoids depending on the version
of a globally installed pnpm shim. The frontend is plain TypeScript with Vite;
it owns presentation only.

The launcher builds and starts `workspace-supervisor` in a separate process with
a fresh private directory under `/tmp`. It waits for the socket to be bound, then
starts Tauri and Vite. The desktop receives the endpoint through
`AGENT_WORKSPACE_SOCKET`, requests typed status through Rust, and displays the
supervisor and desktop process IDs. Refresh status performs another observation.
No provider executable is launched.

Quit the desktop or press Ctrl-C in the launcher terminal to end the bootstrap
and remove its temporary directory. This development harness manages only its
own processes; it has no admitted work or persistent state. Product discovery,
reconnect, and graceful owned-work shutdown belong to later tickets. A second
development launch must wait for the first to exit (Vite uses port 1420).

## Verification and build

```sh
corepack pnpm frontend:build
corepack pnpm check
corepack pnpm format:check
corepack pnpm test
corepack pnpm build
corepack pnpm start
```

`check` runs TypeScript, Cargo check, and Clippy with warnings denied. `test` runs
the workspace suite, including a real supervisor-process exchange in private
temporary state. For a focused check:

```sh
cargo test --locked -p workspace-core --test status
cargo test --locked -p workspace-supervisor --test startup
```

`build` creates the release supervisor and
`target/release/bundle/macos/Agent Workspace.app`. `start` launches the bundled
desktop executable against a separately started release supervisor, without Vite.
For this bootstrap, use the launcher: opening the app directly without an endpoint
shows setup guidance. On-demand discovery and packaged supervisor startup are
ticket 02 work.

Run `corepack pnpm format` to format source. Cargo and pnpm lockfiles are committed;
use `--locked`/`--frozen-lockfile` when reproducing the baseline.

## Ownership

- `crates/core`: Tauri-independent typed status and local connection interface.
- `crates/supervisor`: process hosting the core, bound to a private Unix socket.
- `src-tauri`: forwards status requests off the rendering thread.
- `src`: TypeScript presentation of observations and errors.
- `scripts/start.mjs`: bootstrap process launcher, extended as product lifecycle
  is implemented.

The connection uses newline-delimited JSON, a 4 KiB frame limit, and two-second
socket I/O timeouts. The supervisor refuses a non-private socket directory or an
existing socket path. Protocol compatibility, exclusive application-state
ownership, discovery, catch-up, and reconnect remain ticket 02 acceptance work.

See [bootstrap evidence](docs/evidence/milestone-01-bootstrap.md),
[the product contract](docs/MVP.md), and [Milestone 01](docs/MILESTONE-01.md).
