# Ticket 02 independent review

Baseline: `e8f2ab4626ec1a65a26bc7fb9b9857d573dba8a2`.
Initial implementation: `8a9e9a5`.
Two fresh-context agents reviewed Standards and Spec independently under the
requested code-review skill. Reviewers performed reads only; root owned all gates
and corrections. Unrelated `docs/IDEAS.md` and ticket 03 edits were excluded.

## Standards

Initial findings:

- P1: Quit required a successfully observed owner, preventing normal exit after
  failed startup even when no supervisor existed.
- P2: A login started after initial desktop connection could lose its target
  descriptor after five-minute cache expiry and subsequent navigation.

Corrections reconcile admitted startup, observe an existing owner or prove absence
through the Discovery seam, and retain both descriptor and attempt observation.
Re-review tightened ordering: status admission precedes the Quit-fence read, and
descriptor retention uses the retained owner through disconnection. Native
connection installation is fenced under the same mutex used by Quit. Controlled
units cover missing/existing/uncertain owners, outstanding startup, navigation and
disconnection beyond cache expiry. Final Standards result: **zero unresolved**.

## Spec

The initial P2 finding was readiness cleanup fencing applied only to one target,
allowing another target sharing credentials to admit a check. Admission now checks
the entire shared scope; a controlled unit proves blocked same-scope effects and
unrelated-target progress. Re-review independently identified the Quit admission
ordering race above. Final Spec result: **zero unresolved**.

Both axes found the corrected architecture consistent with the selected browser
slice and fixture-only allowance. CLI/PTY and real authentication remain deferred.
