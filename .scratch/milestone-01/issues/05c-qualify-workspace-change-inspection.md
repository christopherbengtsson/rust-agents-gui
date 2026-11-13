# 05c: Qualify workspace change inspection

**What to build:** Demonstrate the implemented change-inspection workflow in the
actual desktop, fix qualification/review findings and retain evidence for parent
05 acceptance.

**Blocked by:** [05b: Refresh workspace changes reliably](05b-refresh-workspace-changes-reliably.md).

**Status:** ready-for-agent

**Parent:** [05: Inspect uncommitted workspace changes](05-inspect-uncommitted-workspace-changes.md).

## Start here

Read the [accepted plan](../change-inspection-plan.md), the completion comments and
fixture instructions in [05a](05a-inspect-workspace-changes-manually.md) and
[05b](05b-refresh-workspace-changes-reliably.md),
[MVP](../../../docs/MVP.md), [Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[component guide](../../../src/components/README.md). Apply repository-local Rust,
React and shadcn guidance to review/fixes. Continue on
`feature/m01-change-inspection`; review its full diff against `8aa37ab`.

This is qualification of an implemented end-to-end feature, not the first time
its UI is assembled. It includes fixes necessary to meet the accepted behavior.
Use disposable repositories and private application state. No live provider
submissions are authorized.

## Acceptance

- [ ] The configured complete verification gate passes and production packaging
      is recorded separately. Inspect current scripts and sidecar provenance;
      preserve failed attempts and distinguish compilation from test execution.
- [ ] Actual Tauri desktop evidence through the production core/supervisor path
      demonstrates modifications, additions, deletions, a complete clean comparison,
      unsupported content, limited preview, further edits and Git/read failure with
      retained last-good results. Identify every fixture or synthetic fault used.
- [ ] Switching tabs/workspaces and disconnecting/reconnecting during inspection
      preserve exact targets and show honest pending/stale states. File edits that
      retain their status category still refresh. Inspection starts no provider work.
- [ ] Keyboard and accessibility evidence covers tab activation, file selection,
      resizing/toggling, diff scrolling, status/error names, Sheet dismissal and focus
      return. Check long paths and default/minimum window sizes in the actual WebView.
- [ ] Measure navigation feedback against 100 ms with the largest supported
      preview, record source/tool versions, hardware and exact measurement method,
      and distinguish this workload from deferred full-MVP concurrency qualification.
- [ ] Review the entire feature against repository standards and the accepted
      plan/parent acceptance. Fix concrete findings and run checks justified by the
      fixes. Any retained tradeoff or unperformed check is explicit.
- [ ] Retained evidence under `docs/evidence/` records exact commands, counts,
      statuses and timing under verification policy, plus screenshots/accessibility
      captures. Keep unit, fixture-desktop and native results separate; native results
      for this ticket are not run.
- [ ] Cross-check every original parent 05 checkbox against evidence. Check only
      verified items and append a dated parent completion comment linking the final
      evidence and qualification limits. Leave Milestone 01 and unrelated tickets open.

## Completion

Commit evidence and fixes on the same feature branch. Record final revision,
review outcome and evidence links in this ticket. If qualification fails or is
unperformed, retain those results and leave corresponding parent/child acceptance
unchecked. A passing unit suite alone does not close parent 05.
