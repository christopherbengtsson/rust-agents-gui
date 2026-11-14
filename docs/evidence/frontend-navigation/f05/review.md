# F05 independent review

Fixed point: `c49555d807fb375dd4b9333b619894cf0750c446`.
Reviewed `git diff c49555d -- src`, including staged new tests; no intervening
commits. Reviewers worked in fresh contexts, read the ticket and repository
standards, and ran no competing test/build commands. The implement workflow's
code-review skill requires separate Standards and Spec reviewers.

## Standards

Initial report: **2 findings**.

- **P2 — Restore the current layout when the desktop panel remounts.**
  `defaults` captured preferences once, while the panel unmounted below 880 px.
  Edited width/collapse reverted on remount. Basis: React state ownership and
  view resource lifetimes. Resolved using a narrow workspace-context subscription
  and current remembered panel defaults. The corrected regression failed with
  expected 37 / actual 32 before the fix and passes after it.
- **P3 — Update the documented primitive contract.** The component guide still
  described a reading-position ref and deferred F05 persistence. Resolved by
  documenting `initialPosition`, comparison/file keys, `onPosition`, 150 ms
  coalescing, cleanup flush and identity-bound late-save rejection.

Final report: “Both Standards findings are resolved. The remounted panel now uses
current retained layout, and the component guide documents the updated persistence
and scroll callback contracts. The new regression covers width retention across
responsive remounts. 0 remaining findings.”

## Spec

Initial report: **0 findings**. Reviewed allowlisted persistence/migration,
visible storage failures, pending identity separation, comparison/reading
restoration, live refresh, released/draining callbacks, hidden reads, ownership
and narrow subscriptions. Same-comparison retained last-good display remains
readable while a new observation is pending; changed restored comparisons clear
saved context and reject late scroll callbacks.

Final delta report: **0 additional findings**. Responsive retention remains F05
scope; current tab selection stays local until F06, using remembered view only
at initialization. The new regression exercises actual keyboard resize and
narrow/desktop remounting, and documentation matches the implementation.

Evidence/handoff follow-up found no unsupported completion claims or scope defects.
A count-rendering typo was corrected; logs and ticket consistently record 174
frontend units.

Final signoff: **Standards 0 / Spec 0 outstanding**.
