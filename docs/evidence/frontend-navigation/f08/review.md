# F08 independent review

Fixed point: F07 `024bd636c184bcddaac8645d83cf20423fcf2b17`.
Review scope: working-tree changes from that point, including the Rust/Tauri
replacement boundary, sidecar preparation and qualifier, then the final React
visibility change and its routed regressions. No successor scope.

The code-review skill was applied with separate fresh-context Standards and Spec
agents. Repository precedence, MVP, verification/observation policy, component guide
and applicable local React/Rust guidance were supplied to both. Reviewers made no
edits and did not rerun gates during the primary agent's gate execution.

## Standards

Initial Rust/tooling review: no actionable findings. Old native handles/permits
remain owned through join, replacement policy stays in Rust, Tauri delegates, and
staging uses a fresh inode with substituted failure cases.

Final React review: no actionable findings. Visibility releases collection/detail
interests with complete dependencies while app owners and retained-action interests
stay independent. Settlement refresh only pumps current interests. Tests cover
hidden hints, dirty return and action lookup without replay. Documentation matches.

## Spec

Initial Rust/tooling review: no concrete product finding. Production feed sessions
use fresh UUIDs; no speculative ID tombstone restriction is required.

One **P2 finding**: app-lifetime repository/workspace hooks retained hidden
collection interests in Settings, causing hidden reads and preventing expiry.
Fixed in this changeset. The reviewer rechecked the visibility gating and final
three routing cases: **no remaining concrete spec findings**. App owners and
independent action interests survive, and return attaches against dirty generations.
Actual 310-second zero-body/zero-feature-read expiry and native held-reply action
settlement supplement the controlled cases.

Final evidence/report review is recorded in the completion addendum below.

Summary before evidence addendum: Standards 0 actionable findings; Spec 1 P2 fixed,
0 unresolved product findings. Material performance costs and explicit unperformed
checks remain qualification disclosures, not silently accepted improvements.

## Final evidence addendum

Both independent reviewers checked the final report, 313-row matrix, failure ledger,
source hashes/fresh-checkout patch provenance, and ticket/spec/Milestone updates.
Both report **0 actionable final findings**. Evidence links resolve and logs
support counts/timings. Production, inspector, fresh-checkout and unchanged-native
cohorts remain distinct. Cold native startup/total heap and other unperformed checks
remain disclosed, with material costs explicitly returned and milestone/provider
acceptance open. The report's inventory count was corrected to 313. Reviewers ran
no builds/tests and made no edits during this read-only final review.

Final totals by axis: **Standards 0 unresolved; Spec 1 P2 fixed, 0 unresolved**.
