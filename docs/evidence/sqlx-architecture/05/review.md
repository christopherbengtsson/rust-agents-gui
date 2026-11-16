# Ticket 05 — two-axis review

Fixed point: `6bdb1f2d15aac0e847c2fd6e6ca9369c3ba0cd69`. Review command: `git diff 6bdb1f2d15aac0e847c2fd6e6ca9369c3ba0cd69...HEAD`. Initial commit: `e1ef314 Own bounded async Git and provider process effects (ticket 05)`. Correction: `db54303 Preserve absolute process deadlines and cleanup reserve progress`.

Both reviewers launched separately with `fork_turns: none`, explicit `model: gpt-6.1-sol` and `reasoning_effort: high`. Both selectors were accepted without error. Reviewers could not independently inspect or select engine settings inside their delegated contexts. This records accepted launch requests, not an independently measured engine identity. No substitute model or effort was selected. Follow-up review used those same independent reviewers.

## Standards

Initial findings: **2 documented-standard breaches**.

1. The finite Git pipe task recreated the two-second deadline after dispatch, renewing the original budget. This violated ticket 05's absolute-deadline requirement and the evidence README's single-deadline statement.
2. Initial natural-exit timeout was discarded during escalation. A leader that closed pipes, lingered past the deadline and exited zero on TERM could produce successful Git output. This violated the absolute-deadline requirement and Rust error guidance to preserve distinct timeout/cleanup outcomes.

Final report: **No remaining Standards findings in `db54303`.** Both earlier findings are resolved: Git retains its original deadline, and natural-exit expiry remains a timeout even when TERM cleanup exits zero. Cleanup reserve now releases before indefinite unknown-group observation while physical ownership remains retained. The added controlled regressions and `linger-zero` fixture address the reported cases. No additional actionable baseline smells found. The reviewer inspected implementation without executing verification.

## Spec

Initial findings: **2**.

1. **P1:** Two unknown groups could retain both cleanup-reserve permits indefinitely, preventing other owned children from reaching escalation. Ticket 05 requires reserved cleanup/status paths to remain usable.
2. **P2:** Finite Git recreated its deadline after dispatch, contrary to retaining absolute deadlines.

Final report: **No remaining spec findings in `6bdb1f2...db54303`.** Cleanup reserve releases before indefinite unknown-group observation, retaining child identity and ordinary/pipe permits. Finite Git retains its original deadline, rejects expired dispatch, and preserves timeout failure after successful TERM cleanup. Controlled regressions cover third-cleanup progress, delayed dispatch and zero-exit cleanup following natural-exit timeout. No additional source-level gap or scope creep identified. The reviewer correctly left actual qualification/evidence publication pending for the implementing agent.

Both axes were read-only: no edits, builds, tests, providers, authentication, RAT inspection or commits. Separate red logs preserve the original regression failures; replacement verification and actual qualifiers are recorded separately.

Review totals: **Standards 0 unresolved (2 corrected); Spec 0 unresolved (2 corrected)**. Neither axis has an unresolved worst issue.
