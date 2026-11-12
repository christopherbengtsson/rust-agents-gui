# Milestone 01 ticket 02 — supervisor discovery and connection

2026-11-12 · Status-only deterministic and bundled desktop checks passed. Milestone 01 remains incomplete. No agent runtimes or native submissions were used.

## Implementation and environment

Source baseline: `03227e95253aaaff322fc7bc2a47eb03c59ed43f` plus the ticket 02 commit containing this record. Same macOS/toolchain cohort as [bootstrap](milestone-01-bootstrap.md); Cargo and pnpm lockfiles retained. Private test state only, under `/tmp`; packaged desktop smoke used `AGENT_WORKSPACE_STATE_DIR` without an executable override, exercising the bundled supervisor resource.

Rust core owns on-demand discovery, exclusive owner admission, and desktop connection state. The supervisor holds an OS lock for its lifetime; contenders cannot replace its endpoint. The persistent lock inode must never be deleted for recovery. Only the admitted owner replaces a stale socket. Private directory/socket ownership and permissions are validated; symlink endpoints/state and unsafe lock files are rejected. The executable sets umask 077 before creating threads or sockets.

Protocol 1 requires the same application version. Requests/responses are bounded to 4096 bytes, frame reads have an absolute two-second deadline, writes have a two-second timeout, and the server admits at most eight concurrent connections. No peer-controlled output is logged. Snapshot responses contain authoritative status and an owner/revision cursor. This status-only implementation always returns a full snapshot; a foreign, old, or future cursor sets `reset`. There are no mutable events yet. This is a catch-up baseline for later behavior, not a claim of durable event/action recovery.

## Commands and results

- `cargo test --locked -p workspace-core --test status`: passed, 2 tests. Status and snapshot cursor/reset behavior through the core interface.
- `cargo test --locked -p workspace-supervisor --test startup`: passed. Separate-process status exchange.
- `cargo test --locked -p workspace-supervisor --test discovery`: passed, 5 parent tests. One ignored helper is launched explicitly as a desktop subprocess by its parent test. Four simultaneous desktop clients find one owner; a competing actual supervisor exits unsuccessfully; dropping clients and killing a desktop subprocess leave that owner alive. Reopening catches up to the same cursor/PID. Separate tests cover private modes, insecure/symlinked state rejection, protocol/application mismatch guidance, oversized-frame rejection and subsequent availability. Losing the owner causes Refresh to fail without starting another owner; explicit reopen can restore status from a new owner after acquiring the released lock.
- `corepack pnpm check`: passed (TypeScript, Cargo check, Clippy with warnings denied).
- `corepack pnpm build`: passed, including frontend compilation and the macOS app bundle with supervisor resource.
- `corepack pnpm format:check`: passed.
- `corepack pnpm test`: full locked workspace suite passed, 8 tests; the subprocess helper is intentionally ignored by the normal runner and exercised by its parent test.
- `git diff --cached --check`: passed.

## Bundled desktop smoke

Launched `target/release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop` with a newly created mode-700 test directory and `AGENT_WORKSPACE_STATE_DIR` set to it. No `AGENT_WORKSPACE_SUPERVISOR` override.

1. Desktop PID **53254** displayed supervisor PID **53259**, owner **53259-1794492996186840000**, revision **1**, and Ready. Refresh retained the same owner. [Connected screenshot](milestone-01-discovery/connected.png).
2. Sent SIGKILL to that desktop process only and relaunched the same bundled executable/state. Desktop PID **53313** displayed the same supervisor **53259**, owner identity, and revision. [Reconnect screenshot](milestone-01-discovery/reconnected.png).
3. Changed the test socket to mode 666 and refreshed. The desktop visibly rejected access and explained the private-state requirement. [Error screenshot](milestone-01-discovery/access-error.png). Restored mode 600, terminated the test-owned desktop and supervisor, and removed disposable state.

## Limits

These checks establish the ticket 02 portion of A10 and status-only desktop-disconnect/reconnect in A8. Desktop-visible version-mismatch wording is exercised through the shared typed process interface; the desktop screenshot covers access failure. Provider execution, durable action results/reconciliation, surviving runtimes, and owned Quit/flush remain later tickets. No fixture adapter or provider protocol fixtures were changed. Native provider qualification was not run or authorized. No full-workflow performance or remaining A1–A10 qualification is claimed.

## Review

The required code-review skill reviewed the staged diff against the source baseline above with independent Standards and Spec agents before commit.

### Standards

No findings. The staged diff follows the documented ownership, thin-Tauri, private-state testing, and retained-evidence requirements. Recovery policy stays in Rust core, and actual supervisor-process tests cover changed ownership and reconnect behavior. No material baseline code smells identified.

### Spec

No findings. No missing or partial ticket 02 requirements, scope creep, or incorrect implemented behavior identified. The status-only snapshot/reset contract supplies catch-up for this slice; durable action recovery and runtime reconciliation remain deferred. Bundled supervisor preparation supports on-demand startup directly.

Review totals: Standards 0; Spec 0. No worst issue on either axis.
