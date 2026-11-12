# Agent Workspace

Local macOS desktop workspace, using a Tauri presentation connected to a separate
Rust supervisor. Milestone 01 currently provides status, private supervisor discovery, and reconnect.

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

Rust starts or discovers the supervisor on demand. The default private application
state is `~/Library/Application Support/AgentWorkspace`; select test state with
`AGENT_WORKSPACE_STATE_DIR=/tmp/my-private-state corepack pnpm dev`. Existing state
must belong to this user with mode 700. State paths must fit the Unix socket limit.
No provider executable is launched.

Refresh retrieves a versioned full snapshot from the connected owner. Desktop exit,
crash, or launcher Ctrl-C leaves that owner alive; reopening reconnects to it.
If the supervisor exits, Refresh reports the loss; reopen the desktop to discover
an owner again. Connection loss never repeats an action. Graceful explicit Quit
with owned-work cleanup is ticket 09; this status-only slice has no work to stop.
For disposable test state, terminate only the supervisor you launched before removing
that state. Never remove `owner.lock` to bypass an active owner.
A second development launch must wait for the first to exit (Vite uses port 1420).

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
The app includes the supervisor as an executable resource and can also be opened
directly. `scripts/prepare-supervisor.mjs` builds and stages that resource before
development or packaging. `AGENT_WORKSPACE_SUPERVISOR` can select an explicit
executable for development; ordinary packaged startup uses the bundled executable.

Run `corepack pnpm format` to format source. Cargo and pnpm lockfiles are committed;
use `--locked`/`--frozen-lockfile` when reproducing the baseline.

## Ownership

- `crates/core`: Tauri-independent typed status and local connection interface.
- `crates/supervisor`: process hosting the core, bound to a private Unix socket.
- `src-tauri`: forwards status requests off the rendering thread.
- `src`: TypeScript presentation of observations and errors.
- `scripts/start.mjs`: development launcher; it does not own the supervisor.

The versioned connection uses newline-delimited JSON, a 4 KiB frame limit, a
two-second frame deadline, and at most eight simultaneous connections. Unix directory
and socket permissions restrict access to this user. An OS-held state lock fences
concurrent owners, including startup races; only its holder may replace a stale
socket. Incompatible application/protocol versions return setup guidance.
Status snapshots carry an owner/revision cursor: reconnect returns the authoritative
snapshot, and an unknown or stale cursor requires replacing prior observations.
There is no event history in this status-only slice; future changes extend this
snapshot contract rather than requiring clients to replay effects.

See [discovery evidence](docs/evidence/milestone-01-discovery.md).

See [bootstrap evidence](docs/evidence/milestone-01-bootstrap.md),
[the product contract](docs/MVP.md), and [Milestone 01](docs/MILESTONE-01.md).
