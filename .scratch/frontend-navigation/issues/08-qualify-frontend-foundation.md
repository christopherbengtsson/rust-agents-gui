# 08: Qualify the foundation

Status: ready-for-agent

Blocked by: [F07](07-routed-settings.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), then read
the [owning plan section](../spec.md#verification-and-performance) and the predecessor’s dated evidence.
This ticket is the session scope; successors remain separate.

## Context and implementation

Qualify the complete foundation against every agreed requirement, preserving earlier
milestone evidence as historical. Unit checks establish application decisions;
actual Tauri/supervisor behavior and responsiveness require separate evidence.
F01 captured the baseline assets and toolchain but did not qualify the new features.

Create an acceptance-to-evidence matrix and fix findings within the implemented
foundation. Keep each failed attempt and unperformed check visible.

- Run configured formatting/static/unit gates once per completed change set and packaging separately; use the required warm-up/five-run unit measurements and record exact source/profile/cache/commands.
- Use disposable repositories/private state through the actual supervisor/Tauri seam. Demonstrate availability and workspace progress without covered 250 ms polls, failures/last-good state and exact action targets.
- Qualify desktop disconnect/reconnect, owner replacement, overflow/resync, frontend acknowledgement stall, socket/thread limits, subscription cleanup and no duplicate actions.
- Qualify nested-route reload with assets and IPC, direct missing/off-page destinations, Settings keyboard/focus/return and retained context at minimum and ordinary sizes. Preserve existing narrow Sheet/resizable inspection behavior.
- Compare initial/lazy/total raw/gzip/Brotli JS, packaged size, startup/navigation, IPC counts/outstanding work/hidden activity and retention after navigation/expiry against equivalent baseline conditions.
- Cross-check F01–F07 acceptance and whole-plan requirements. Keep parent acceptance open where actual qualification fails or remains unperformed.

Baseline source footholds:

- [docs/evidence/frontend-navigation/f01.md](../../../docs/evidence/frontend-navigation/f01.md) — baseline, exact versions and previous gate limits.
- [docs/evidence/frontend-navigation/baseline.json](../../../docs/evidence/frontend-navigation/baseline.json) — pre-implementation emitted sizes.
- [docs/evidence/milestone-01-change-inspection-qualification.md](../../../docs/evidence/milestone-01-change-inspection-qualification.md) — retained inspection workflow and measurement method.
- [docs/agents/verification.md](../../../docs/agents/verification.md) — test substitution, timing and qualification authority.

## Required scenarios

- Existing deterministic regressions and new contract/scheduling/push/route scenarios pass without real application dependencies in units.
- Actual socket/channel/cursor recovery and stalled-consumer behavior agree with the controlled cases.
- Navigation meets the 100 ms target under the current representative workload; history/chat targets remain deferred until implemented.
- Package provenance matches measured source; retain failed or stale-build attempts and distinguish contamination from product defects.

Use [repository verification policy](../../../docs/agents/verification.md) and
the existing gate. Preserve prior useful assertions; actual desktop/process evidence
remains separate from controlled units.

## Acceptance

- [ ] Every foundation requirement has a linked pass/fail/not-run result and supporting source/command/evidence.
- [ ] Production packaging and actual fixture-desktop/accessibility/push/recovery qualification pass.
- [ ] Size and performance costs/benefits are measured and material regressions resolved or explicitly returned for decision.
- [ ] Only verified ticket/parent acceptance is checked; original milestone 06 can start after foundation closure.

## Handoff boundary

No provider submissions, storage migration, full owned-Quit qualification or chat/history implementation. Those remain separately scoped milestone work. External native links remain F09 follow-up.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.
