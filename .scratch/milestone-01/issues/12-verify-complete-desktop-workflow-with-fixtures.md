# 12: Verify the complete desktop workflow with fixtures

**What to build:** Prove the complete Milestone 1 workflow in the actual desktop using deterministic fixtures and retain an auditable A1–A10 result set before native submissions.

**Blocked by:** 05: Inspect uncommitted workspace changes; 11: Recover from supervisor failure and surviving runtimes

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** Integrated deterministic and desktop verification for all A1–A10; verification stages 1–2.

- [ ] Audit A1–A10 against migrated unit coverage and retained process evidence, fill uncovered application-decision regressions under the testing policy, and pass the deterministic stage before the full desktop verification stage. Mark actual durability, ownership, cleanup and responsiveness unqualified until their separate desktop checks pass.
- [ ] Complete repository registration, selected-commit workspace creation, streamed chat, allow/deny, diff refresh, interrupt, Quit/restart, readable history, and exact fixture resume in the actual desktop.
- [ ] Demonstrate desktop-only crash/reconnect without replay, supervisor failure/recovery, uncertain sends/approvals/resumes, pending approval during Quit, stale answers, incompatible owner versions, and actionable storage/shutdown errors.
- [ ] Use the production core/supervisor interfaces with the deterministic adapter at the supported execution seam; preserve exact matching and fixture provenance.
- [ ] Record screenshots of working and error states and navigation/history-opening measurements on the test Mac against the 100 ms/one-second targets under this single-conversation workload.
- [ ] Run repository build, formatting, static checks, and relevant tests; record exact commands/results, implementation revision, hardware, tool/runtime/schema versions, configuration, and evidence locations.
- [ ] Record A1–A10 deterministic/desktop pass/fail/not-run results and native checks as unperformed. Fix failures before native qualification; do not mark the milestone Passed on fixture success.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
