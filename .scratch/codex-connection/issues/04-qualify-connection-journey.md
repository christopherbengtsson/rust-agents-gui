# 04: Qualify the complete connection journey

**What to build:** Establish cumulative evidence that Providers/readiness
and official browser sign-in form one usable desktop connection journey, with truthful
status, preserved progress and accountable lifecycle cleanup. Fix scoped defects
revealed by this qualification without starting successor work.

**Blocked by:** [02a — Provider runtime compatibility policy](02a-runtime-compatibility-policy.md) (after 02)

**Status:** ready-for-agent

## Start here

Follow the [fresh-session procedure](../spec.md#authority-and-fresh-sessions), then
confirm the checked acceptance and dated focused evidence of 01, 02 and 02a. Read
[verification and qualification](../spec.md#verification-and-qualification),
[scope boundary](../spec.md#scope-boundary),
[Milestone 01 A3](../../../docs/MILESTONE-01.md#observable-acceptance) and
[verification policy](../../../docs/agents/verification.md). Earlier fixture
results and source inspection are not actual authentication or native-agent proof.
Review [architecture and extension constraints](../spec.md#architecture-and-extension-constraints)
and the workspace-provider cleanup handoff alongside that evidence. The
[authentication scope amendment](../spec.md#authentication-scope-amendment--2026-11-15)
defers 03, including both an interactive terminal and a read-only command viewer.
Do not implement its archived plan during qualification.

## Acceptance

- [ ] Record an A3 requirement-to-evidence mapping across 01, 02, 02a and this cumulative cohort, including supported runtime/schema/fixture provenance, source identity, effective configuration, exact commands/results, failed attempts and unperformed limits. Preserve predecessor evidence; identify 03 as deferred rather than passed and cover shared gaps here instead of mechanically repeating unchanged checks.
- [ ] Map architecture acceptance to evidence: one Rust provider-connection owner, concrete adapters supplying evidence without duplicated policy, one provider-observation feature module reusing the foundation, provider-independent workspace records, and exact provider/configuration/attempt identity. Retain deterministic synthetic-target evidence for independent readiness, cross-target result isolation, same-scope readiness/login and duplicate-login exclusion, and scoped cleanup fencing; do not promote it to native multi-provider qualification.
- [ ] Qualify the packaged desktop through the production core/supervisor path with private fixture state: existing authentication first without requiring a login-method choice, unknown billing/account data, missing/incompatible setup and recovery, one browser sign-in action with handoff/cancel/reopen and authoritative recheck, and failures that retain stale/uncertain observations. A CLI selector, embedded terminal and command-output viewer are outside this cohort.
- [ ] Qualify browser admission and lifecycle: duplicate/competing requests, provider/Settings/workspace navigation and progress return, desktop-only crash/reconnect, supervisor loss, late setup/cancel results and explicit Quit. Confirm retained ownership, no automatic sign-in replay or agent launch, physical process cleanup, and visible/fenced cleanup uncertainty.
- [ ] Qualify ordinary/minimum-window layout, keyboard access, accessible names/landmark labels, focus/return across General/Providers/tabs, nested-overlay Escape precedence, disabled-Claude explanations/actions, and workspace reading-context preservation. Record actual accessibility checks and their limits separately from controlled DOM units. Record a hands-on connection trial with Christopher when feasible, without claiming general usability from one trial.
- [ ] Keep deterministic unit, actual fixture-desktop, real-authentication and native-agent results separately labeled. Agree any real-authentication scenarios before running them and record their cohort/results without signing out or replacing credentials silently. If not agreed/performed, record that limit explicitly; it neither proves native login nor waives the milestone's remaining qualification. Submit no agent threads/turns.
- [ ] Resolve scoped defects, run the configured checks relevant to any fixes and affected cumulative paths, and publish dated evidence. Check only verified acceptance. Once required child acceptance passes, link the evidence from parent 06's aggregate acceptance; keep whole-milestone/native-agent acceptance open and hand off to existing 06a without implementing it.

## Handoff boundary

This closes the connection feature's cumulative fixture scope, not the milestone's
native coding workflow. [Parent 06](../../milestone-01/issues/06-connect-codex-with-actionable-setup-diagnostics.md)
aggregates the results. [06a — Storage evaluation](../../milestone-01/issues/06a-evaluate-storage-before-durable-chat.md)
and then [07 — Durable streamed chat](../../milestone-01/issues/07-durable-streamed-chat-and-uncertain-send-recovery.md)
remain separate selected sessions. Retain failures and unperformed required checks
as open acceptance; do not promote fixture success to native authentication or
execution qualification. Stop after this ticket.

## Comments

2026-11-15: Published from the approved four-session split. Implementation and
qualification acceptance remain unchecked.

2026-11-15: Added cumulative architecture/identity evidence mapping and the
workspace-provider cleanup handoff. Actual desktop qualification remains the
Codex/fixture journey; Claude/OpenCode integration is not introduced.

2026-11-15: The user-approved [presentation refinement](../spec.md#presentation-refinement--2026-11-15)
supersedes the original permanent Refresh presentation. Include automatic verification on actual Codex-tab entry/return, no extra activation checks on preload/rerender/reconnect, Last known evidence while checking, and the healthy screen without Refresh. Qualify contextual Check again recovery and visible login controls outside collapsed Setup help alongside the existing combined journey.

2026-11-15: The user deferred 03 and approved one runtime-managed browser sign-in
action alongside automatic detection of existing authentication. Rebased this
ticket on completed 02a, retaining 01/02/02a evidence and all browser ownership,
cleanup, reconnect and accessibility qualification. Terminal and CLI acceptance
is deferred, not waived by a pass. Keep verification bounded: reuse unchanged
evidence, run one focused fixture cohort and relevant gates once, and rerun only
affected checks after a concrete failure or scoped fix. No product changes or
qualification were performed in this scope update.
