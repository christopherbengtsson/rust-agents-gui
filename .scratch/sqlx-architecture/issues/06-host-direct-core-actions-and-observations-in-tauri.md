# 06: Host direct core actions and observations in Tauri

Status: ready-for-agent

Blocked by: [05 — Bounded async process effects](05-own-bounded-async-process-effects.md).

## What to build

Run existing workspace/provider workflows inside the native Tauri process with
direct targeted actions, bounded push observations and one truthful Quit owner.
WebView replacement preserves work; native restart recovers instead of replaying.

## Context and ownership

Read [spec](../spec.md), [amendments A1–A5/A8](../amendments.md),
[ADR 0008](../../../docs/adr/0008-native-core-and-direct-sqlx.md),
[frontend retained policy](../../frontend-navigation/spec.md),
[MVP](../../../docs/MVP.md), [milestone](../../../docs/MILESTONE-01.md),
[verification](../../../docs/agents/verification.md),
[observations](../../../docs/agents/observations.md) and
[component guide](../../../src/components/README.md). Use Rust/React/UX/XState
skills where touched and installed Intent guidance. Preserve others' edits.

Own native composition, commands, current generated contracts, channel adapter
and affected feature observations/diagnostics. Keep domain policy in Rust and
feature cache/freshness behind existing interfaces. No second host path is selected
at runtime; incumbent source may remain unreachable until 07 deletes it.

## Acceptance

- [ ] Native startup constructs one core/lease/storage lifetime on Tauri's singleton runtime. No sidecar launch/discovery is used by the new path. Startup/recovering/fenced/stopping states are actionable and owner exclusion precedes storage/effects.
- [ ] Commands call concrete domain handles with expected core incarnation. Update every feature/progress/pending-interest owner check and regenerate Rust declarations/validators together; remove peer-version compatibility from this path without weakening target/revision/membership checks.
- [ ] One app-scoped channel adapter retains bounded invalidation/subscription/gap-resync and accept-before-ack. Await old setup/delivery cleanup before replacement; no view unmount stops domain work or releases physical request capacity early.
- [ ] Preserve Router/Query/XState/Zustand roles, disabled automatic action retries, last-good stale observations and pending action identities. Implement a small Query-owned inactive-entry count policy; defer serialized-byte LRU/second body index and measure actual retention.
- [ ] Provider readiness/browser-login fixtures survive navigation and WebView replacement with exact scope, progress return/cancel and no replay. Native failure creates a new incarnation and restores interrupted/unknown evidence with surviving-work fences.
- [ ] Intercept ordinary ExitRequested synchronously, retain one async whole-core shutdown and allow exit only after completion. Quit during startup, repeated Quit and deadlines retain handles/errors; future restart/update paths cannot bypass the documented completed-drain rule.
- [ ] Present core startup/storage/runtime diagnostics separately from provider readiness, preserving accessible recovery controls. Add scoped/redacted tracing for action/incarnation/task/queue/commit/shutdown evidence, excluding secrets/auth URLs/transcript bodies.
- [ ] Run generated write/check preparation and substituted owner/cancellation/freshness units, then actual native-host fixture desktop: registration/first creation, concurrent creation/inspection, setup/login/cancel, WebView replacement, native crash/reopen, competing owner and Quit races. Record this as new Q4 evidence.

## Verification and limits

Inspect current scripts; native dev/package staging may need the minimal change
to enable this slice, with final deletion/package audit owned by 07. Actual Tauri
and SQLx/process results remain separate from DOM/application units. No live login
or agent turns, no user-state reset, no unimplemented chat continuity claim.

## Comments

2026-11-16: Published for the exclusive native-host cutover; acceptance is unchecked.
