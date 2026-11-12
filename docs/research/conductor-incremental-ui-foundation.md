# Conductor reference: incremental desktop UI foundation

Research note and recommendations only. This note does not amend the product contract, spec, tickets, or acceptance gates.

## Findings from primary sources

Conductor's first-workspace guide describes a repository → isolated workspace → agent work → review loop. Its published workspace screenshot visibly places repository/workspace navigation on the left, workspace/chat content and a bottom composer in the center, and changes with terminal/run content on the right. This is evidence for that documented screen, not a complete specification of every current app state or responsive behavior. Sources: [first-workspace guide](https://www.conductor.build/docs/first-workspace), [official workspace screenshot](https://www.conductor.build/docs-assets/images/new-workspace.png).

The documented diff workflow includes a changed-file list, file navigation, unified diff, and commit filtering. Comments and PR actions extend that workflow. These are reference capabilities; the local milestone implements only its accepted uncommitted-change inspection. Source: [Diff viewer](https://www.conductor.build/docs/reference/diff-viewer).

Conductor distinguishes agents sharing one workspace/code state from independent work in separate workspaces. That supports maintaining distinct workspace and conversation navigation concepts as our UI grows. Source: [Parallel agents](https://www.conductor.build/docs/concepts/parallel-agents).

Conductor's current homepage emphasizes cloud execution and multiplayer. Those additions do not change this repository's explicitly local scope. Sources: [homepage](https://www.conductor.build/), [our product contract](../MVP.md).

## Current repository gap

The current frontend renders supervisor connection status, process/version/owner details, and a refresh action; it has no product navigation or workspace/chat composition. Its stylesheet uses a centered diagnostic page. Sources: [frontend](../../src/main.ts), [stylesheet](../../src/style.css).

The milestone already requires a usable desktop composition and lasting modules. Ticket 03 requires repository navigation, but does not explicitly settle the shared layout or visual conventions. Thus incremental UI is compatible with existing scope; making its foundation explicit would reduce divergent feature screens. Sources: [milestone](../MILESTONE-01.md), [ticket 03](../../.scratch/milestone-01/issues/03-register-repositories-and-restore-navigation.md).

## Recommendation: establish the shell, extend it per capability

This is our proposed design direction, not a claim about Conductor's internal implementation:

| Stage | Lasting UI increment |
| --- | --- |
| Before or at the start of 03 | Application shell, repository navigation region, main content region, compact connection status, consistent type/spacing/color/focus conventions, and an honest initial empty state. |
| 03 | Repository registration, selection, restored navigation, validation and unavailable-repository states inside that shell. |
| 04 | Workspace entries and selected-workspace identity: branch, path and provider. |
| 05 | Change inspection region and text diff; working open/close or sizing interactions as needed for usable reading. |
| 06 | Runtime setup/status in the existing layout, with actionable setup errors. |
| 07 | Transcript, tool activity and composer with distinct saved/received/uncertain observations. |
| 08 | Exact-operation approval presentation using the established conversation layout. |
| 09–11 | Interruption, quit, resume and recovery states added to the same navigation/status/action patterns. |
| 12 | Qualify the accumulated desktop workflow and its error states, rather than first composing the application here. |

Choose a small vocabulary of actual layout and interaction patterns, then extract reusable presentation code where those patterns repeat. Preserve the Rust ownership rules: UI selection/layout/focus are presentation concerns; execution, validity, retry and reconciliation policy remain in the core. Source for that constraint: [product contract](../MVP.md).

Establish screenshots of the intended empty application, selected workspace and conversation/change-inspection arrangement as implementation references. Settle exact dimensions and interaction details by inspecting those views, rather than inferring them from documentation alone. A region for future content does not require fabricated messages, inactive feature controls, or terminal/PR capabilities before their milestones.

Keep the single implementation spec. A small UI-foundation checkpoint before ticket 03, or explicitly within its first slice, can establish the shell; subsequent tickets should include their real visual states. Do not introduce a general docking system, full design-system project, or deferred feature implementation for this purpose. The goal is a coherent first product surface that grows alongside the Rust foundation.

## Limits

Sources were official public documentation and one published desktop screenshot, plus read-only inspection of current project files. No installed Conductor app was exercised; exact keyboard, accessibility, resizing and current-version visual behavior were not qualified. No application tests or builds were run. The spec, tickets and code were not modified by this research.
