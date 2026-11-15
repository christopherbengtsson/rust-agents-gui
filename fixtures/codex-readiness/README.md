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
these schemas; none are captured account/credential payloads. The fixture child
implements only `--version`, initialize/initialized, account/read and config/read.
It rejects login/thread/turn/approval methods. Synthetic faults establish application
recovery, not native compatibility. Selected `account/read` sends
`refreshToken: false`; config reads use `includeLayers: false`. No auth-status
token export, login, logout, thread or turn operation is implemented.

Official transport source `codex-rs/app-server-transport/src/transport/stdio.rs`
uses one JSON message per line and EOF cleanup. Provider framing is distinct from
the application's supervisor codec. Future approval/chat work must inspect its
exact supported schemas and retained evidence; this fixture authorizes no approval
matching assumptions. No archived RAT files were inspected or changed.
