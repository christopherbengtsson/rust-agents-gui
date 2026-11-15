# Codex readiness fixture provenance

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
