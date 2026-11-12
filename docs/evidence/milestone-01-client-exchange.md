# Milestone 01 — client exchange follow-up

2026-11-12. The selected architecture follow-up makes the existing client exchange
deterministically testable. Ticket 03 product behavior and full milestone
qualification remain incomplete.

## Source and scope

Baseline `758b87e63d0d34fbc87fb695f032276571448172` plus the uncommitted
`connection.rs` and `connection/tests.rs` changes recorded below. The preceding P3
lock-error fix in `supervisor.rs` and its regression test were preserved; they were
staged during this work. Concurrent research documents were left untouched.

`SupervisorClient::snapshot_with_version` now calls one private exchange path.
Its small internal `ClientEffects` seam supplies private-access, connection and
clock outcomes; `FrameStream` supplies timeout and byte-I/O outcomes. Production
still calls the same private-access helper and uses one owned Unix stream per
exchange. Request construction, frame bounds, deadlines and response parsing stay
in production code under test. Public interfaces, wire format, error mappings,
discovery policy and Tauri presentation are unchanged.

Eight new client unit scenarios cover exact protocol/version/cursor transmission,
chunked reads and partial writes, incompatible-version guidance, access/connect
failures, timeout-configuration failures, partial-write failure, oversized outgoing
requests, malformed/incomplete/oversized incoming replies, read failure and absolute
deadline expiry. They check that failed exchanges do not reconnect or repeat the
request, and that connected streams are dropped on success and failure.

The existing discovery, server/framing, owner-admission and tooling tests remain;
none were removed or replaced. Application-supervisor responses are synthetic,
not native-provider fixtures. Controlled access outcomes establish client ordering;
the existing owner-module tests retain private-access validation coverage. Units
do not establish actual OS restrictions, process survival or native compatibility.

## Verification

Environment: Darwin arm64, Node 26.13.1, pnpm 12.18.0, Rust/Cargo 1.99.1.
Hardware/OS cohort comes from retained bootstrap evidence: MacBook Pro Mac16,8,
Apple M4 Pro, 48 GB RAM, macOS 26.2; hardware was not remeasured. All commands used
the worktree-default `target`, warm dependency/build caches, dev/test profiles and
serial Rust harness threads / Node test concurrency 1. No fresh-target compilation
was performed. Verification/build commands ran sequentially.

- `cargo fmt --all`: exit 0 before verification.
- `/usr/bin/time -p corepack pnpm test:measure`: exit 0, **1.52 s** wall.
  Unit compilation **0.405737 s**, separate from direct execution. All six runs
  passed **33 Rust + 12 tooling tests** each. Rust warm-up **0.441664 s**;
  five warm runs median **0.006633 s**, range **0.005481–0.010453 s**. Node
  warm-up **0.097994 s**; five warm runs median **0.086251 s**, range
  **0.085736–0.100243 s**.
- `/usr/bin/time -p corepack pnpm verify`: exit 0, **1.72 s** wall.
  Prettier, rustfmt, TypeScript and all-target/all-feature Clippy passed.
  **33 Rust + 12 tooling tests** passed, 0 failed. Unit compilation
  **0.095103 s**; direct Rust execution **0.003648 s**, Node execution
  **0.092391 s**. This complete gate ran each retained check once, separately
  from the measurement cohort.
- `/usr/bin/time -p corepack pnpm qualify:process`: exit 0, **1.83 s** wall,
  with sandbox escalation for local subprocess/socket access. Compilation
  **0.39 s**. Discovery **5 passed**, with **1 intentionally ignored helper**
  exercised by its parent desktop-process-kill test; startup **1 passed**.
  Fixtures use disposable private state and Cargo's reported supervisor executable,
  not the staged sidecar. Owner exclusion, private access, version checks,
  desktop-process loss/reconnect and status exchange passed through actual processes.
- `git diff --check`: passed. The evidence document was added after the code gate
  and checked separately with Prettier.

No verification failures occurred. Actual Tauri desktop qualification, packaging,
live providers and fresh-private-target compilation were not run. No provider
submission, authentication change or RAT inspection occurred. Existing desktop
and milestone evidence remains retained; the process harness is not a new desktop
workflow qualification.

## Source identity

SHA-256 of the verified code and compiled Rust unit executable:

```text
9d7db9946ed3c96b174195c3eb3ca71514eb67cd09f6b509c4a3ecb4e8b38b23  crates/core/src/connection.rs
5be6de749a1dcf0f5ca3cbecbb8372ec36e870e82b171490dca03428614377da  crates/core/src/connection/tests.rs
04259ec2d64bdae4fe72ec4ff105ea35c40a51ea5ddbb240c348e0c6502f5e3c  crates/core/src/supervisor.rs
b86cd230f23074ea348c1ebb11506c9e75f5f74acecfe8ff035d249e6c732023  crates/core/src/supervisor/tests.rs
52307734d05f200cac1388c82bda043dbe75d2bb8508ef29f2f20d71e260f758  target/debug/deps/workspace_core-847732502cf0fdc3
```

Direct Rust measurement command: `target/debug/deps/workspace_core-847732502cf0fdc3 --test-threads=1`.
Direct tooling command: `node --test --test-concurrency=1 scripts/cargo.test.mjs scripts/prepare-supervisor.test.mjs`.
