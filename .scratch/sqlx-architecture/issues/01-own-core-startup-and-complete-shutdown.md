# 01: Own core startup and complete shutdown

Status: ready-for-agent

Blocked by: None (can start immediately).

## What to build

Make startup, pending Quit and completed Quit truthful for all existing core work,
initially on the incumbent host. Losing a request cannot abandon an admitted action
or release the state lease before workspace/provider work physically completes.

## Context and ownership

Read the [spec](../spec.md#core-lifetime-and-task-ownership),
[amendments A1–A3/A8](../amendments.md), [MVP](../../../docs/MVP.md),
[milestone](../../../docs/MILESTONE-01.md), [verification](../../../docs/agents/verification.md)
and [observations](../../../docs/agents/observations.md). Use rust-engineering and
installed Intent discovery before substantial edits. Inspect the actual source/
working tree; preserve others' edits and historical evidence.

Own core lifetime/lease composition and concrete workspace/provider completion
interfaces. Keep domain settlement/retry decisions with their owners. Retain the
incumbent supervisor solely to exercise this seam; do not add a generic host adapter.

## Acceptance

- [ ] One UI-independent lifetime owner holds state lease, phase, admitted-start guards and domain completion handles on the existing driven runtime; no runtime per request.
- [ ] Implement root fence/guard transfer and domain fence ordering from the spec. Cover a start paused at every transfer point while Quit begins; it is either drained or rejected without escaped effects.
- [ ] Domain-owned operation sets continuously consume success/failure/panic with exact identities and settle outcomes independently of reply delivery. Remove failed-reply result loss; retain bounded compact faults when persistence fails.
- [ ] Whole-core Quit replaces provider-only success and workspace Drop/join as the normal path. Remove whole-app provider shutdown actions as lifetime assumes their responsibility; retain provider-local stop/join. Collect all errors, including task panic, rather than skipping later cleanup after the first failure.
- [ ] Partial startup visits every acquired resource; repeated Quit/deadline observes one retained attempt. Root shutdown coordination is outside drained sets. Lease release follows actual workspace/provider/blocking completion and final persistence; a failed/pending cleanup retains ownership.
- [ ] Publish an old-to-new coverage map for intent-before-effect, duplicate actions, owner exclusion, cleanup, reservations and observations, identifying future replacements and retired-format-only checks. Do not delete useful assertions without passing replacements.
- [ ] Preserve current desktop behavior on the incumbent host with focused fixture startup/status/Quit evidence and separate actual lease/process exclusion evidence. This is not qualification of the future native host.

## Verification and limits

Q1: substituted units for cancellation, panic, partial startup, admission/fence,
deadline and lease-last ordering with controlled events and owned completion.
Real lease/process fixtures belong to separate bounded qualification. Inspect the
current scripts and run relevant compilation/unit/static gates once. No provider
authentication or agent submissions; do not reset user state. Record source,
commands/results, failures and limits in dated evidence.

## Comments

2026-11-16: Published from the reconciled spec; all implementation acceptance is unchecked.
