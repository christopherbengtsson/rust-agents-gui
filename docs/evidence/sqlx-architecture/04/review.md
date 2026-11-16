# Ticket 04 two-axis review

Fixed point `f3fd165d7c9c0fc4c327a0ce36555b25e01ebb24`; diff `git diff f3fd165d7c9c0fc4c327a0ce36555b25e01ebb24...HEAD`. Initial reviewed commit: `24cb5f8 Persist provider actions through shared SQLx storage (ticket 04)`.

Both reviewers were launched separately with `fork_turns: none`, explicit `model: gpt-6.1-sol` and `reasoning_effort: high`. Both tool calls accepted these settings without a selector error. The reviewers cannot independently inspect the execution engine through their tool metadata; this records the selected request, not an independently measured model identity. No alternate model/effort was selected.

## Standards

Initial report: **one documented-standard violation**. Provider startup called `Launch::capture()` directly inside async `open`, although capture performs synchronous filesystem canonicalization of the configured Codex home. This conflicted with the Rust async-lifecycle instruction to dispatch synchronous filesystem work away from executor workers. A slow mounted home could block an executor worker. The reviewer requested retained blocking dispatch before shared async storage restoration. No additional actionable Fowler-baseline smells were found.

Correction: provider startup now awaits `spawn_blocking(codex::Launch::capture)` from the retained startup task. Shared SQLx access remains asynchronous; no database-only blocking dispatch was reintroduced. Final correction review of `4c13a7a` reports **0 unresolved**.

## Spec

Initial report: **zero actionable findings**. The reviewer found shared SQLx composition, intent-before-effect, exact pending identities/scopes, retained persistence failures and requester-independent settlement aligned with ticket 04. No later-ticket scope creep or relabeling of fixture evidence as native/physical qualification was found. A historical action-lookup concern was examined and excluded as inherited behavior without a concrete ticket-04 defect. Final correction review of `4c13a7a` reports **0 unresolved**.

## Additional implementation audit

After initial review, the implementer extended the late-cleanup regression to assert both the phase and its required problem/diagnostic. It failed: a late readiness result retained the cleanup phase but replaced its cleanup evidence, violating the frontend's phase/problem invariant. Completion now retains the original cleanup problem, diagnostic and last-good evidence once fenced. The red/green result, fresh generated preparation, combined verification and replacement package are retained separately; original cohorts are not relabeled.

Reviewers performed read-only review, with no verification, provider/authentication, RAT inspection or edits.

## Final review reports

Reviewed runtime commits `24cb5f8` and `4c13a7a` against the pinned baseline. Subsequent closing changes are evidence/ticket documentation only.

### Standards

Initial findings **1**; resolved **1**; final unresolved **0**. The reviewer confirms retained/awaited blocking launch capture before async SQLx restoration satisfies the executor-work rule. The late-cleanup correction preserves original failure evidence and last-good data. No remaining concrete documented-standard violations or actionable Fowler-baseline smells.

### Spec

Final unresolved **0**. The reviewer confirms both corrections preserve startup/cleanup ownership and remain within ticket 04; replacement binary evidence stays separate from earlier cohorts and explicit qualification limits remain. No ticket 05/06 implementation or scope creep.

Review totals: **Standards 0 unresolved (1 corrected); Spec 0 findings**. Neither axis has an unresolved worst issue. Both requested model/effort settings were accepted as recorded above; no selector failure occurred.
