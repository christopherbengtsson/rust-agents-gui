# 02: Reliable supervisor discovery and connection

**What to build:** Open the desktop against one authoritative local supervisor, with visible ownership/version failures and reconnection to the same process after a desktop connection loss.

**Blocked by:** 01: Bootstrap and verified startup

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A10 ownership/transport foundation; desktop-disconnect part of A8, initially with status-only work.

- [x] Start or discover a supervisor on demand for a selected private application state; reject a competing owner even during concurrent startup attempts.
- [x] Use a user-private local connection and verify access restrictions; reject incompatible client/supervisor versions with actionable desktop feedback.
- [x] Killing or disconnecting the desktop alone leaves the supervisor alive. Reopening reconnects to the same owner rather than launching a competitor.
- [x] Expose authoritative status/snapshot observations through the same typed Rust interface used by Tauri, with bounded transport/output and a catch-up mechanism that later behavior can extend.
- [x] A lost connection grants no authority to repeat effects. Keep connection handling/policy in the owning Rust modules and presentation thin.
- [x] Verify owner exclusivity, access restrictions, version mismatch, desktop disconnect, and reconnect across actual supervisor processes using private test state; record commands/results.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

Implemented and verified 2026-11-12. See [retained evidence](../../../docs/evidence/milestone-01-discovery.md) for process checks, bundled desktop kill/reopen screenshots, commands, and limits. No live provider submissions. The default triage vocabulary has no completed label; checked acceptance items and this evidence record completion.
