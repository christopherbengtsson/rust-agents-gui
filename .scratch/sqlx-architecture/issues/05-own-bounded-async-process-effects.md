# 05: Own bounded async process effects

Status: ready-for-agent

Blocked by: [04 — Shared provider persistence and driver cutover](04-persist-provider-actions-through-shared-storage.md).

## What to build

Keep Git/provider work and cleanup responsive on the driven runtime while replacing
manual pipe polling and workspace wake threads with maintained async mechanisms.

## Context and ownership

Read [process decisions](../spec.md#process-effects-and-provider-failure-domains),
[lifetime/bounds](../spec.md#core-lifetime-and-task-ownership),
[process source assessment](../review-research.md#7-maintained-process-libraries-useful-mechanisms-with-narrower-guarantees),
[MVP](../../../docs/MVP.md), [verification](../../../docs/agents/verification.md) and
[observations](../../../docs/agents/observations.md). Use rust-engineering's async,
synchronous-resource and applicable framed-I/O guidance. Inspect actual provider
schema/provenance before touching adapters/fixtures; preserve others' edits.

Own shared process mechanisms and finite Git dispatch; provider interpretation
and workspace reconciliation remain local to their domains. Add no universal run
interface, process-wrap dependency or speculative portability layer.

## Acceptance

- [ ] Use bounded Tokio pipes/waits/channels and admission-before-spawn/dispatch, preserving parser and command construction. Eliminate replaced manual nonblocking read/sleep and wake-thread mechanics without unbounded wait_with_output collection.
- [ ] Retain finite Git versus long-lived protocol interfaces, exact action/process identities, absolute deadlines, partial transmission uncertainty, output/frame limits and explicit EOF behavior.
- [ ] Keep physical permits through child/pipe/blocking completion independently of async waiters. Continuously reap outcomes and preserve cleanup failures without early-return skipping other operations.
- [ ] Preserve owned unreaped-leader/process-group signaling and quiescence evidence, escalation and explicit unknown cleanup. Leader exit, ECHILD, token cancellation or kill-on-drop alone does not become descendant completion evidence.
- [ ] Ordinary pressure leaves reserved cleanup/status paths usable; no locks/gates span long process effects or joins. Native parent loss remains a recovery case and saved PIDs grant no signaling authority.
- [ ] Substituted units replace useful polling/thread assertions with deterministic streams/events/time, including partial output, cancellation, deadline, panic and cleanup failures. Update old-to-new map.
- [ ] Separately qualify actual Git/provider-process fixtures, pipe saturation, group cleanup, abandoned waiter and parent-loss observations with disposable state. Record escaped-descendant limits honestly; no claims of universal containment.

## Verification and limits

Q3 uses unit substitution first, actual processes separately, then focused incumbent
desktop creation/inspection/setup/owned-Quit smoke and relevant compile/static gates.
No live authentication/agent work. This prepares native hosting, not a concurrency
or pressure pass for unimplemented chat. Record exact commands, source and limits.

## Comments

2026-11-16: Published with the process-wrap 10.0.1 assessment; acceptance is unchecked.
