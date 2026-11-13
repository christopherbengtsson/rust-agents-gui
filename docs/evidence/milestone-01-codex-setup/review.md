# 06b independent code review

Review base: `c5130794eabf8273c1650e7cb3f52b958f30fd1a`.
Implementation checkpoint: `fd52535`. Review fixes: `6ebf470`, `0a27180`, `35c5121` and `77f84c8`.
Read-only reviewers used fresh contexts, repository policies, the accepted plan
and 06b ticket. They ran no tests/builds and changed no files. Each axis remains
separate below.

## Standards

Initial hard violations:

- **P2 — Recovered Save uncertainty disappears without being presented.**
  `useRuntimes` forgot terminal lookup actions, including uncertain results.
  After supervisor restart, the lookup did not populate active; Settings only
  displayed active/diagnostic errors. Preserve/display the recovered action
  outcome before clearing pending identity. Sources: React observation-status
  and explicit-targeted-intents rules.
- **P2 — Enabled actions silently disappear during observation reads.**
  Submit returned while its read lane was occupied, while canAct excluded lane
  state. Polling/Refresh could leave actions enabled while clicks did nothing;
  Reset also cleared its draft. Expose read-lane occupancy to controls. Sources:
  React refs-and-resources and observation-status rules.

Judgment calls: none.

Fixes retain the recovered action separately from pending identity, render its
error, and include rendered read-lane state in action availability. DOM regressions
cover both cases. Re-review of `6ebf470`: **no hard violations or judgment calls
remaining**, including the additional Rust lifecycle/storage changes.

A subsequent **P2** finding identified that a transient conflicting-ID rejection
could be lost on supervisor restart, leaving a retained conflicting payload
unresolved. Status reads now send that original payload as `expected`. Rust
reconstructs rejection against canonical durable identity through the shared
cached/worker lookup policy without replay. A public fresh-owner recovery test
covers it. Re-review of `77f84c8`: **no remaining concrete Standards findings**.

## Spec

Initial findings:

- **P1 — Shutdown abandons unconfirmed process handles.** Channel closure let the
  worker drop quarantined handles; shutdown could report failure after discarding
  the ownership it promised to retain. 06b requires owned handles and bounded
  shutdown without abandonment.
- **P2 — A definitely rejected Save becomes a false crash/uncertainty result.**
  Storage converted every Pending action into Uncertain, overwriting confirmed
  Save rollback with a false stopped-supervisor diagnosis. 06b requires definite
  rejection and uncertain persistence to remain distinct.
- **P2 — Cleanup exit status masks actionable diagnostic errors.** Nonzero exit
  after owned TERM/KILL could replace Timeout/auth/protocol/configuration errors
  with generic Process. Preserve the primary error and separate exit evidence.

Fixes keep the worker reconciling quarantined handles after channel closure;
bounded shutdown retains its JoinHandle, and Core destruction transfers an
unfinished worker to private runtime recovery ownership. The Core exposes its
runtime shutdown seam. Controlled completion tests prove no join/ownership loss
on timeout, and a fixture test proves failed reconciliation retains handles.

Storage now returns raw action evidence. The runtime owner interprets recovered
unfinished actions, while confirmed Save rollback records Failed and preserves the
previous preference. A focused regression exercises the real recovery policy.

Original metadata/version errors now precede nonzero cleanup exit interpretation;
`exit_successful` is separate diagnostic evidence. Synthetic faults also simulate
nonzero exits so the primary-error assertions exercise this ordering.

Spec re-review then found two **P2** identity cases: fresh configuration/cleanup
rules could reject a repeated action after cache eviction instead of returning
its recorded result, and rejecting a conflicting payload could poison the cached
canonical identity. Durable identity now resolves before fresh admission; every
mutation/status reply retains its exact action lookup. Conflicting payloads receive
separate typed rejection while the original action stays canonical. Public
request/response regressions cover eviction, changed configuration, cleanup fences,
conflicting payloads, original repeats and fresh-owner recovery. TypeScript validates
both canonical identity and the exact rejected payload.

Re-review through `77f84c8`: **no unresolved specification defects**.

Initial and follow-up findings: **Standards 3 (worst P2), Spec 5 (worst P1)**.
Final unresolved findings: **Standards 0, Spec 0**. Desktop/native qualification
remains with 06d; reviewers performed no qualification.
