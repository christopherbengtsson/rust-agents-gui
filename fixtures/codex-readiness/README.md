# Codex readiness fixture provenance

Current native-host qualification uses `corepack pnpm qualify:native`; see
[development-state instructions](../../docs/development-state.md#separate-qualification-commands).
`runtime.py` and the official schemas remain current provider fixtures. The
unselected `qualify.py` and `qualify-discovery.py` supervisor drivers, including
their original usage instructions, are retained historical checkpoint material.
Neither is a supported current qualification path.

Selected 2026-11-15, before adapter implementation: installed `codex-cli 0.169.0`,
Apple Silicon standalone executable SHA-256
`739cda86e15d30a6e2b40cf8dcbbfb988aaf48b305b27065573bb8e9fd41d002`.
Official `openai/codex` tag `rust-v0.169.0`, source commit
`d32063f96c2dff1dc96af839b3952abb6acfeda4`.

`schema/` contains unmodified official schemas from that commit under
`codex-rs/app-server-protocol/schema/json/{v1,v2}/`. Source:
https://github.com/openai/codex/tree/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/app-server-protocol

Protocol inputs in Rust units and `runtime.py` are **synthetic**, derived from
these schemas; none are captured account/credential payloads. The readiness
operations are `--version`, initialize/initialized, account/read and config/read.
It rejects thread/turn/approval methods. Synthetic faults establish application
recovery, not native compatibility. Selected `account/read` sends
`refreshToken: false`; config reads use `includeLayers: false`. No auth-status
token export, logout, thread or turn operation is implemented.

## Browser authentication extension — 2026-11-15

Ticket 02 adds synthetic `account/login/start` (ChatGPT only),
`account/login/cancel`, and exact `account/login/completed` notifications at the
same process/stdio seam. The five authentication schemas in `schema/` are unmodified
official files from the same pinned source revision above. Messages and faults
remain synthetic; no account, token or credential traffic was captured.

The pinned [account processor](https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/app-server/src/request_processors/account_processor.rs)
sets `open_browser:false`, returns `authUrl`/`loginId`, limits browser login to ten
minutes, correlates cancellation by native login UUID, and returns `canceled` or
`notFound`. The adapter retains newline framing and correlates both RPC responses
and completion notifications. It never requests token-bearing auth-status APIs.

Run `qualify.py <supervisor> <new-report.json> --browser` for the explicit process
cohort, or insert the packaged desktop executable before `--browser` for manual
desktop qualification. The harness hosts a disposable loopback landing page;
its Complete/Fail links change synthetic evidence only. `scenario <name>`,
`snapshot`, `kill-desktop`, `restart-desktop`, `processes`, and `stop` are harness
commands. They do not run real authentication or agent work. HTTPS URLs are
accepted in production; HTTP loopback URLs are permitted only in captured fixture
launches. Reopening uses the original Rust-held URL.
Add `--headless` to process qualification to use an intentionally invalid browser
URL and exercise process/reconciliation/cleanup without opening or focusing the
user's browser. This mode does not qualify actual browser handoff.

Official transport source `codex-rs/app-server-transport/src/transport/stdio.rs`
uses one JSON message per line and EOF cleanup. Provider framing is distinct from
the application's supervisor codec. Future approval/chat work must inspect its
exact supported schemas and retained evidence; this fixture authorizes no approval
matching assumptions. No archived RAT files were inspected or changed.

## Runtime compatibility prerequisite — 2026-11-15

The minimum stable release is 0.169.0, with newer stable versions admitted only
when the required protocol checks succeed. `GatewayOAuthReadResponse.json` is an
unmodified official schema from the same pinned revision above. The request has
no params; its definition is in `src/protocol/common.rs` (`GatewayOAuthRead`).
The pinned app-server README's Gateway OAuth sign-in section requires successful
`account/gatewayOAuth/read` on each connection after initialize/initialized when
using `explicitGatewayOauth`. Initialization alone can silently ignore that flag.
A successful probe confirms support even with `required:false` or `notReady`;
it does not prove authentication. No gateway login/export flow was added.

Version strings (including synthetic 0.170.0), additional fields/notifications and
unsupported/malformed/failed/delayed probe scenarios remain **synthetic**. They
exercise admission and recovery, not native compatibility with those releases.
`runtime_version` in private fixture control selects the reported stable version.
Use `qualify.py <supervisor> <new-report.json> --browser --headless --compatibility`
for the focused compatibility process cohort. The desktop mode also accepts
`version <semver>` to select synthetic version evidence without using user credentials.

## Cumulative connection qualification — 2026-11-15

Desktop mode with `--browser --journey` copies the fixture executable and packaged
app into fresh private state, isolates desktop preferences, and prepares a
disposable repository. `runtime-missing` / `runtime-restore` remove/restore only
that private executable at the captured path; `dirty-workspace` edits only the
one workspace created in that disposable repository. `capture <label>` saves full
observations and process events alongside the report. `compete` exercises exact
duplicate admission, competing-attempt rejection and readiness exclusion.
`kill-supervisor` / `restart-supervisor` retain loss/recovery evidence; restarting
does not clear the product's durable cleanup fence. Harness termination after
that scenario is cleanup, not a successful product Quit claim.
`--resume-from <completed-report.json>` reuses only that cohort's retained
`/private/tmp/aw-ready-*` state/repository for an affected-check rerun after its
owner has exited. It creates a new copied bundle and requires a new output report;
it neither resets data nor clears cleanup fences. Raw events remain cumulative.

`unknown_billing` supplies schema-valid `account:null` and
`requiresOpenaiAuth:false` using the retained official **0.169.0**
`GetAccountResponse.json`, source **d32063f96c2dff1dc96af839b3952abb6acfeda4**.
It establishes unknown billing/account presentation, not a captured account or
native provider behavior. All journey faults and runtime messages remain synthetic.

## Discovery refinement — 2026-11-15

Connection 05 retains official Codex 0.169.0 source
`d32063f96c2dff1dc96af839b3952abb6acfeda4` and the schemas above. The existing
initialize/gateway capability sequence now exposes a local validation checkpoint
before account/config reads, without adding a provider method. Synthetic protocol
and discovery outcomes remain synthetic; no credentials or native messages are
captured. Version/capability validation and authentication are distinct, and a
validated executable can be remembered even when authentication is unavailable.
