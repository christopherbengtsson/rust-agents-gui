# F04f independent review

Reviewed the complete staged product diff against
`1fc3d936bd1670f9451d21387975bc116204bb65`, the completed F04e tip. HEAD still
identified that baseline during review; there were no intervening commits.
The code-review skill's Standards and Spec axes ran in separate fresh-context
agents with repository policy and the originating ticket/spec supplied. Reviews
were read-only; reviewers ran no verification commands.

## Standards

Initial result: **one P2**, pending feed failure visibility.

`configure` cleared the error while native stop/join was still unresolved.
Observation guidance and the component guide require that failure and cleanup
uncertainty remain visible. Configuration now changes the desired owner only;
replacement clears the message in `begin` after successful cleanup.

The regression “owner changes preserve the delivery failure while exact native
cleanup is pending” failed before the fix and passed afterward. See
[red](review-feed-red.log) and [green](review-feed-green.log).

Follow-up: **0 outstanding findings**. The reviewer confirmed pending visibility,
message clearing on replacement, and the regression's coverage.

## Spec

Initial result: **two P2 findings**.

1. Unrelated accepted batches restarted the inspection wait timer, allowing busy
   feed traffic to postpone external-Git refresh indefinitely. Waiting now exits
   for unavailable baseline or dirty inspection only; clean unrelated hints keep
   the existing two-second delay. See [red](review-cadence-red.log) and
   [green](review-cadence-green.log).
2. First/Next could overwrite a queued explicit refresh while physical work
   drained. The page action now preserves existing refresh priority. Existing UI
   disables paging during loading, limiting exposure, but the feature callback
   contract must retain priority too. See [red](review-priority-red.log) and
   [green](review-priority-green.log).

The reviewer also observed the pending-failure visibility defect in the frozen
initial diff and confirmed its correction in the current source. No scope creep
was identified.

Follow-up: **0 outstanding findings**. The reviewer confirmed both fixes and the
retained passing regressions. Standards and Spec remain separate conclusions.
