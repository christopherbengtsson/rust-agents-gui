# 05b: Refresh workspace changes reliably

**What to build:** Keep visible change inspection current after edits while
retaining honest last-good results and preventing late replies from changing a
different workspace or file.

**Blocked by:** [05a: Inspect workspace changes manually](05a-inspect-workspace-changes-manually.md).

**Status:** ready-for-agent

**Parent:** [05: Inspect uncommitted workspace changes](05-inspect-uncommitted-workspace-changes.md).

## Start here

Read the [accepted plan](../change-inspection-plan.md), 05a's completion comment,
[MVP](../../../docs/MVP.md), [Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[component guide](../../../src/components/README.md). Apply repository-local Rust,
React and shadcn skills. Continue on `feature/m01-change-inspection` from the
completed 05a commit. Inspect the implemented interface before extending it.

Extend the existing manual workflow; retain one inspection owner, one selected
preview and the full agreed presentation. This session owns the refresh loop from
edits through Rust observations to the visible desktop, plus complete deterministic
failure/content coverage.

## Acceptance

- [x] Visible summaries refresh every two seconds with one read sequence in
      flight. The selected preview refreshes only while Changes is active. Pending
      observation reads do not continuously requeue inspection work.
- [x] Automatic reads pause when changes surfaces/document are hidden,
      disconnected or failed. Returning refreshes observations; errors retain manual
      Refresh. Visibility changes never create/replay workspace or provider actions.
- [x] Workspace/owner/request generations isolate late and out-of-order replies.
      Changes to HEAD, selected-file evidence or comparison identity invalidate
      obsolete previews/list continuations explicitly.
- [x] Further edits update the selected preview even when Git's status category
      for the file stays unchanged. Tab switching preserves reading context without
      retaining an inactive heavy renderer; selecting a different file resets it.
- [x] Removed selections are explained rather than silently replaced. Git/read
      errors retain their last-good result with visible staleness; a later successful
      refresh clears that failure. Unsupported content remains distinct from I/O
      failure and from a clean comparison.
- [x] Complete supplied-outcome coverage includes CRLF/LF, missing final newline,
      empty files, unusual/non-UTF-8 paths/content, binary files, symlinks/submodules,
      conflicts, metadata-only changes, special files, size/row/frame limits and
      staged/unstaged cancellation. Reads neither follow symlinks nor open special
      files as text. No real Git/filesystem/storage/process dependency enters units.
- [x] Frontend units exercise controlled timers, visibility, target changes,
      unmount/cleanup, stale replies, last-good retention and bounded rendering.
- [x] The agreed resizing, Sheet selection/focus lifecycle, keyboard interaction,
      long paths and narrow-window layout are complete. Keep source highlighting
      optional and use the configured Compiler before adding measured optimizations.
- [x] Relevant checks pass and a dated comment links the implementation commit,
      coverage, results and remaining actual-desktop/native qualification.

## Completion and handoff

Commit the working increment on the same branch. Record executable fixture
scenarios or setup instructions for changed, clean, unsupported, limited,
refreshed and failed states, including how to demonstrate failure through the
production inspection path in disposable state. Synthetic failures must be
identified as such; unit mocks are not desktop evidence.

Stop after this slice and hand qualification to
[05c](05c-qualify-workspace-change-inspection.md). Keep parent 05 open until that
evidence exists. This ticket authorizes no live provider submissions.


## Comments

### 2026-11-13 — refresh reliability implemented, verified and reviewed

Implementation commit: `50685d5cd8325d595a8868ce3b768638e6806f91` on
`feature/m01-change-inspection`.
[Verification and handoff evidence](../../../docs/evidence/milestone-01-change-inspection-refresh.md)
records source hashes, tool/hardware identity, commands, counts, timings,
expected red tests, review correction and qualification limits.

The configured complete gate passed **108 Rust + 25 tooling + 70 UI** checks,
formatting, lint, TypeScript and all-target/all-feature Clippy. Review added one
actual-WorkspacePanel navigation regression and moved the read lane above keyed
views. Final affected verification passes **71 UI checks**, TypeScript, lint,
formatting and production frontend compilation. Unchanged Rust/tooling results
are retained. One warm-up and five warm runs were measured, separately from Rust
compilation; final UI measurements were repeated after the correction. The
existing production bundle-size warning remains. Commit hooks passed format/lint.
Independent Standards and Spec review follow-ups report **0 unresolved findings**.

The existing inspection request/reply interfaces and protocol 5 remain in use.
WorkspacePanel retains one physical read lane across keyed view resets;
WorkspaceInspection's hook owns visible observations. Two-second refresh and
250 ms pending observation polling are distinct; hidden surfaces/documents,
disconnection and failures pause automatic reads. Reconnect retrieves observations.
Rust invalidates old previews/queued jobs, while supplied-outcome units exercise
content/failure/race/boundary cases without real application dependencies. Selected
preview reading context is retained across tab switches and reset for another
file. The component guide records the stable-reader requirement.

[Disposable production-path fixture instructions](../../../docs/evidence/milestone-01-change-inspection-refresh/fixture-instructions.md)
cover changed, clean, unsupported, limited, refreshed and failed states, including
synthetic permission/Git-identity faults and restoration. These instructions are
**prepared, not executed desktop evidence**. Checked acceptance records implemented
behavior and controlled verification; actual WebView accessibility, scrolling,
resizing, focus, minimum-window layout and latency remain qualification work.

Stop here and hand qualification/packaging to
[05c](05c-qualify-workspace-change-inspection.md). Actual desktop/OS/process
qualification and native-provider qualification are **not run** in this slice.
No provider submissions occurred. Parent 05, Milestone 01 and unrelated tickets
remain open; completed earlier evidence is preserved.
