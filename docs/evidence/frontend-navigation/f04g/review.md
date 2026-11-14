# F04g independent review

Fixed point: `5140ecee8a938c996cd4759917c8355eebc3ac2e` (completed F04f).
Reviewed work-in-progress ticket diff and qualification helpers; no predecessor
changes or successor implementation were included. Reviewers received the parent
spec, MVP/milestone, verification/observation policies and engineering guidance.

## Standards

Initial independent review reported:

- **P1 — Retained summary errors still trigger the inspection loop.** Query keeps
  the old ready body after a rejected summary request. Preview eligibility must
  include the current query error, or a dirty selected preview repeatedly queues
  summary reads. Add retained-preview/rejected-summary coverage.
- **P2 — Startup timeout stops protecting the qualifier after its first byte.**
  Readability followed by blocking `readline()` can wait forever on a partial line.
  Keep readiness under one absolute deadline and ensure cleanup observes exit.
- **P2 — Five-second heartbeat qualification accepts immediate heartbeats.**
  Recording elapsed time alone does not establish cadence; assert a bounded
  interval around the specified five seconds.

A follow-up review of the added lost-reply helper reported:

- **P2 — Endpoint restoration must cover proxy setup failures.** Establish signal
  handlers/cleanup ownership before endpoint rename; track rename/bind steps and
  restore the real endpoint on every exit. Join serving workers before capturing
  and joining late resume workers.

All four findings were resolved. A further evidence constraint required successful
upstream admission, matching owner and exact action identity before calling a lost
reply admitted. The helper now asserts those facts. Actual logs contain one
admitted dropped creation, exact action lookup and same-owner recovery.

Final follow-up: **0 actionable Standards findings**. Reviewers explicitly left
unfinished final qualification/evidence gates to the implementer, without claiming
those gates complete on the basis of code review.

## Spec

Initial independent review reported:

- **P2 — Pause preview-driven retries when the summary request fails.** Checking
  only the retained ready body violates “Preserve error pause, last-good data”.
  Assert one read, visible error and retained preview, no retry without a newer hint
  or explicit refresh, and successful matching recovery.

The parameterized pending/rejected-summary regression failed before the error
condition was added and passes afterward. No confirmed additional Spec defect or
scope creep was found. Follow-up: **0 outstanding Spec findings**. Actual admitted
reply loss, matching action lookup and lack of duplicate mutation were subsequently
recorded in the final desktop cohort; review itself is not its qualification.

## Resolution evidence

- `pending-preview-red-bounded.log` → `pending-preview-green.log`: original pending
  summary loop, then all eleven actor cases passing.
- `review-retained-error-red.log` → `review-inspection-green.log`: error-path
  variant, then all twelve actor cases passing.
- `partial-readiness-negative.json` / `.log`: deliberately partial readiness exits
  at the eight-second limit and records joined child cleanup. Its nonzero exit is
  expected negative-qualifier evidence, not a failed product gate.
- `reviewed-actual-release-protocol.json`: nineteen actual socket scenarios pass,
  including an asserted heartbeat interval and physical blocking-work admission.
- `lost-reply-proxy.jsonl`, `lost-reply-recovery.json`, `proxy-cleanup.json`: exact
  admitted action, no replay, joined helper cleanup and restored endpoint.

Summary: **Standards 0 / Spec 0 outstanding findings**.
