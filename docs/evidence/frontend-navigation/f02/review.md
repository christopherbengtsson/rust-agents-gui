# F02 review — 2026-11-14

Fixed point: `002bd850c003ba4d6c5fd1d97868ba8bcabfaa73`. Reviewed uncommitted ticket
changes and new corpus/decoder/unit files; excluded unrelated `docs/IDEAS.md`.
Two fresh-context reviewers applied repository authority and verification policy.
Neither edited files or ran checks concurrently with the verification owner.

## Standards

No additional concrete Standards findings.

The confirmed inspection guard defect in `src/lib/changes.ts` rejected legitimate
queued/running observations retaining an earlier error. It diverged from Rust's
actual observation contract and the repository's typed IPC guidance. Owner checks
occur before domain dispatch, context stays outside durable payloads, uncertainty
survives later rejected attempts, and new tests use controlled outcomes. Corpus
provenance distinguishes application traffic from native-provider evidence.

## Spec

No additional spec findings beyond the inspection retry-state guard.

Reviewed owner rejection before dispatch, serving-owner/kind/target checks, retained
action identity and uncertainty across retry/reload, bounded snapshot decoding,
protocol/application compatibility and the shared pure corpus. Existing Vitest
selection includes both new files; native tests substitute streams/dispatch and
corpus checks use pure serialization. Scheduling, push and Query stay outside F02.

## Disposition

Corrected the inspection guard against the actual Rust transition behavior and
added queued/running summary and preview regressions. Both parameterized tests
failed before the fix; the corrected contract/inspection cohort passed 31 tests,
and TypeScript/lint passed. Added a final controlled late-registration-failure
regression; its nine-test retained-delivery cohort passes. Final complete-gate and
measurement logs accompany the evidence. No unresolved review finding remains.

Standards: one resolved finding, zero unresolved. Spec: the same resolved finding,
zero unresolved. Actual desktop/process/native qualification remains separate.

A read-only Spec follow-up reviewed the corrected guard and late-settlement test
and confirmed no unresolved findings. The final gate and measurement cohort pass
115 Rust + 25 tooling + 89 frontend checks; the production bundle also succeeds.
