# 02: Strengthen observation contracts

Status: ready-for-agent

Blocked by: [F01](01-publish-architecture-and-tooling.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#ipc-contracts-and-supervisor-ownership) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

The baseline native envelope has no caller-owner binding. Feature calls connect
whatever supervisor owns the current socket; frontend Query keys alone cannot
establish which owner served a reply. Connection snapshots also bypass the runtime
validation already used by repository/workspace replies.

Use the caller's observed owner as request context, separate from durable action
payloads. Reject mismatch before domain dispatch; verify serving owner before any
cache insertion. Separate admission/delivery evidence for an attempt from the durable
action outcome. Busy or OwnerChanged on a later retry must preserve earlier
uncertainty and the original action ID/payload.

- Update targeted native/Tauri request and response contracts together; retain status discovery as the authoritative way to learn an owner.
- Add bounded runtime snapshot/desktop-identity decoding and consistent owner/target checks. Keep structural validity separate from current authority.
- Bump protocol compatibility and retain application-version checks; update old/new-version rejection coverage.
- Add a small application-owned serialized corpus, checked by pure Rust serialization and consumed by TypeScript decoders. Keep semantic guards and prove the frontend gate selects the new tests.

Baseline source footholds:

- [crates/core/src/connection.rs](../../../crates/core/src/connection.rs) — native envelope, framing and client/dispatch seam.
- [src-tauri/src/main.rs](../../../src-tauri/src/main.rs) — Tauri arguments and error forwarding.
- [src/hooks/use-supervisor-connection.ts](../../../src/hooks/use-supervisor-connection.ts) — currently unvalidated snapshot/desktop identity.
- [src/lib/repositories.ts](../../../src/lib/repositories.ts) — repository decoding and retained registration.
- [src/lib/workspaces.ts](../../../src/lib/workspaces.ts) — workspace decoding and exact-target checks.

## Required scenarios

- Wrong owner is rejected before core routing, durable intent, probes or inspection jobs.
- Replacement-owner and wrong-kind/target replies cannot populate the earlier owner scope.
- A lost reply followed by locally rejected retry retains prior uncertainty until authoritative matching status settles it.
- Malformed snapshots, impossible variants and existing frame/page/row/byte bounds remain explicit failures.

Use [repository verification policy](../../../docs/agents/verification.md) and
the existing gate. Preserve prior useful assertions; actual desktop/process evidence
remains separate from controlled units.

## Acceptance

- [x] Targeted requests/replies are owner-bound across Rust, Tauri and TypeScript.
- [x] Snapshot and feature decoders reject malformed or mismatched observations.
- [x] Transport outcomes preserve retained action identity and prior uncertainty.
- [x] Protocol compatibility and cross-language corpus checks pass without real processes/storage.

## Handoff boundary

F03 owns request scheduling; F04 owns push; F05 owns Query adoption. Keep existing UI behavior and default unit isolation working. No storage migration or provider adapter changes.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.

### 2026-11-14 — F02 implemented and verified

Confirmed F01's checked acceptance/detailed evidence and continued the existing
feature branch without resetting it. Protocol 6 binds targeted native/Tauri replies
to caller-observed owner, validates snapshot/desktop/feature observations, and keeps
attempt delivery evidence separate from retained durable action identity. Pure
Rust/TypeScript corpus and controlled recovery regressions pass; review findings
were corrected. See [F02 evidence](../../../docs/evidence/frontend-navigation/f02.md)
for exact source, commands, measurements, failed attempts, independent Standards/Spec
review and qualification limits. Final verification passed 115 Rust + 25 tooling +
89 frontend checks, measured units, and frontend bundling. The concurrent `docs/IDEAS.md` edit is preserved.
No actual desktop/process/packaging/native-provider qualification is claimed.
F03 remains separate and unimplemented; stop after this ticket.
