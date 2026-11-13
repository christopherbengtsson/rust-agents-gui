# 06d: Qualify Codex setup and review

**What to build:** Qualify the completed Settings/setup slice in the actual desktop
using deterministic fixtures/private state, fix review defects and cross-check parent 06.

**Blocked by:** [06c: Protocol/process hardening](06c-harden-codex-protocol-and-process-recovery.md).

**Status:** ready-for-agent

**Parent:** [06: Codex setup](06-connect-codex-with-actionable-setup-diagnostics.md).

## Start here

Read the [accepted plan](../codex-setup-plan.md), all child completion evidence,
[MVP](../../../docs/MVP.md), [Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[component guide](../../../src/components/README.md). Apply repository-local Rust,
React and shadcn skills and final Standards/Spec review. Use disposable private
application state; preserve completed milestone evidence and avoid RAT inspection.

Switch to `feature/m01-codex-setup` and continue from 06c's completed commit. Keep
qualification fixes and evidence on that branch; do not create a new branch or
implement on `main`.

## Acceptance

- [ ] Stage/build from identified current source with sidecar provenance, matching
      desktop/supervisor versions and private fixture state. Keep production core,
      supervisor, transport and WebView paths; mocked IPC is not desktop evidence.
- [ ] Demonstrate Settings entry/Back, keyboard/focus, one active main/skip link,
      accessible controls/errors, minimum-window layout, long paths, retained
      workspace tab/file/panel/reading context and hidden portal/effect cleanup.
- [ ] Demonstrate setup pending/success and actionable missing executable,
      incompatible version/protocol, unavailable auth, config/EOF/timeout and
      cleanup-uncertain states supported by the fixture cohort. Check saved-config
      identity, preference persistence and truthful unknown billing/provider states.
- [ ] Demonstrate navigation and desktop reconnect during a check without replay;
      show exact action settlement without context theft and authoritative recovery
      after an unavailable Save reply. Verify optional diagnostics never create a
      repeated-login/Settings prerequisite for the future chat workflow.
- [ ] Verify actual fixture process/worker stop and owned cleanup outcomes within
      the implemented setup slice, including retained uncertainty/fencing. Keep
      wider A7–A10 lifecycle/chat qualification with their owning tickets.
- [ ] Demonstrate copy success/failure and scoped external documentation opening
      without leaving the application in the WebView; confirm CSP behavior.
- [ ] Measure navigation feedback against the MVP's 100 ms target under this
      workload. Record method, hardware, source, cache/profile state and results;
      this is not full-MVP concurrency or history-opening qualification.
- [ ] Run the configured complete gate, required unit measurements and production
      packaging as distinct operations, retaining failures and source/version/hash
      identity. Re-run affected checks after fixes without redundant full suites.
- [ ] Final Standards and Spec review findings are resolved or explicitly recorded.
      Cross-check every original parent 06 acceptance item against evidence before
      checking it; leave unsupported/unperformed requirements open.
- [ ] Unit, fixture-desktop/process and any permitted native metadata results are
      separate. Record zero live submissions; native execution compatibility,
      Claude and whole Milestone 01 remain unqualified.

## Completion and stopping point

Commit fixes/evidence and add dated child/parent comments with reproducible commands,
counts/timings, captures, review results and remaining limits. Check predecessor
items deferred solely to actual desktop evidence only after those checks pass.
Complete parent 06 only if its original acceptance has evidence; do not complete
the parent spec or unrelated tickets.

No live thread/turn, agent submission, automatic scenario retry, login mutation or
broader native-provider cohort is authorized. Metadata-only observations, if
performed under parent 06's allowance, remain distinct from fixture and native
execution qualification. Stop after this child; ticket 07 requires its own session.
