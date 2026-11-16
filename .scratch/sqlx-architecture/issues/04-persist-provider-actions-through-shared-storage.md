# 04: Persist provider actions through shared storage

Status: ready-for-agent

Blocked by: [03 — Concurrent workspace admission and reservations](03-admit-concurrent-workspaces-and-atomic-reservations.md).

## What to build

Retain existing provider selection/readiness/login action outcomes through shared
async SQLx persistence, without changing credential authority or the connection journey.
Close the coherent driver cutover across all existing consumers.

## Context and ownership

Read the [spec](../spec.md), [amendments A5/A6/A8](../amendments.md),
[connection ownership](../../codex-connection/spec.md#ownership-and-protocol),
[connection qualification](../../codex-connection/issues/04-qualify-connection-journey.md),
[MVP](../../../docs/MVP.md), [verification](../../../docs/agents/verification.md) and
[observations](../../../docs/agents/observations.md). Use rust-engineering and inspect
supported schemas/provenance before changing any adapter/fixture.

Own provider selection/login persistence, shared composition and adapted incumbent
dispatch. Continue the same 02–04 integration branch. Preserve other edits; domain
retry/cleanup decisions remain provider-owned, never hidden inside storage.

## Acceptance

- [x] All existing workspace/provider consumers await the same storage owner. Delete connection-per-operation/reopen escapes and replaced database-only blocking dispatch; no per-call runtime or custom SQLite worker.
- [x] Provider intent commits before authentication/configuration effects; exact provider/configuration/attempt and credential-scope exclusion survive awaits. Revalidate stale/late results, retain cleanup fences and original persistence errors, and never replay an uncertain login on reconnect/mount.
- [x] Domain outcome ownership survives requester loss/panic. Stop/status and independent target observations remain usable under storage acquisition pressure without a second result ledger in root/Tauri.
- [x] Current core, incumbent host and desktop compile with one SQLite driver and current generated contracts. No rusqlite bridge, retired migrations or old reader survives as a fallback; no second application storage owner is introduced.
- [x] Close Q2/Q3 combined coverage: atomic continuation/reservations, duplicate/payload mismatch, retired connection ordering, settled reads, replacement validation, bounded pages/acquisition, provider scope and all-stage close. Update coverage map before removing useful tests.
- [x] Record separate actual storage/process and focused fixture-desktop results for existing registration/creation/provider selection/login/cancel/Quit. Preserve the completed connection cohort as history; new results describe the incumbent host plus new storage, not the future native host.
- [x] Pass relevant combined generated preparation/static/compilation/isolated-unit gates and package smoke on this branch. Record exact graph/engine/settings and failed/unperformed fault qualification; merge only a coherent tested cutover.

## Verification and limits

SQLx/disk/process qualification remains outside substituted application units.
Use private disposable state and synthetic connection evidence, not real browser
authentication or agent turns. This ticket does not implement conversations or
claim streamed fairness. Publish a dated actual interface/limits handoff for 05/06.

## Comments

2026-11-16: Published as the combined completion gate for 02–04; acceptance is unchecked.

2026-11-16: Implemented the shared provider/storage cutover on `sqlx-storage-cutover`. [Evidence, coverage map, separate cohorts and review](../../../docs/evidence/sqlx-architecture/04/README.md) retain 218 substituted Rust / 51 tooling / 260 mocked-IPC UI tests, 15 actual SQLx cases including controlled engine SQLITE_FULL, eight real disposable workspace/Git cases, incumbent process/recovery/lifecycle and packaged fixture-desktop registration/creation/selection/synthetic-login/cancel/owned-Quit. Combined generated/static/compilation/unit verification and production packaging passed. Checked acceptance is ticket 04's combined incumbent cutover gate, not native hosting or closure of unperformed physical faults; those limits and the dated 05/06 handoff remain explicit. Prior failed probes and connection cohorts remain history. Only ticket 04 is updated; no ticket 05 implementation, live provider, real authentication, user-state reset or merge.
