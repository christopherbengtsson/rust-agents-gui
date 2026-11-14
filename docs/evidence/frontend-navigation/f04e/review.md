# F04e independent review

Baseline: `851ea98a1079e99d05c292f8b755badecc9a2822`.
Reviewed the staged pre-commit implementation with
`git diff --cached 851ea98a1079e99d05c292f8b755badecc9a2822 --`.
New product files were staged before review. Evidence and ticket completion are
added after final gate results; source hashes identify the reviewed product bytes.

Used the implement workflow's code-review skill with two fresh-context, read-only
agents. Both received repository precedence, MVP/milestone, verification,
observation/TanStack/component guidance and F04e/spec pointers. Neither ran tests,
builds or edited files. Parent ran configured verification serially.

## Standards

Initial finding: **P2 — failed-read freshness disappeared when interest was
released.** The attempted generation lived only in the disposable interest and
was captured before queueing; remount or a hint during queueing could retry an
error without a subsequent hint or explicit refresh.

Resolution: Query retains the original typed failure plus physical-start generation
for the Query entry's lifetime. New interests consult that generation. Logical
admission failures retain their rejection generation. Workspace seeding also
preserves existing detail/error observations. Passing regressions cover failure
hide/remount, queued hint before failed physical start, new hints and explicit retry.

Follow-up review: **0 remaining Standards findings**, including no additional
meaningful heuristic findings.

## Spec

Initial finding: **P2 — integrated Refresh status could lose explicit reprobe.**
A delayed status check temporarily detaches owner-bound interests. The old refresh
callback ran before those interests were recreated and discarded its reprobe flag.

Resolution: Repository refresh supplies at most three exact owner-bound domain
targets. Missing-interest refresh flags survive the attachment wave, are consumed
by matching interests, and expire for hidden/obsolete targets after baseline
attachment. Replacement owners discard old requests. The controlled App regression
holds the status reply through rendered Checking, then verifies a physical
refresh=true request after reconnection.

Follow-up review: **0 remaining Spec findings**; the reviewer also inspected the
failure-generation and workspace-seeding changes.

Final result: **Standards 0 / Spec 0 outstanding findings**.
