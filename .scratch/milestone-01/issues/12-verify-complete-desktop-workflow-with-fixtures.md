# 12: Verify the complete desktop workflow with fixtures

**What to build:** Prove the complete Milestone 1 workflow in the actual desktop using deterministic fixtures and retain an auditable A1–A10 result set before native submissions.

**Blocked by:** 05: Inspect uncommitted workspace changes; 11: Recover from supervisor failure and surviving runtimes

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** Integrated deterministic and desktop verification for all A1–A10; verification stages 1–2.

- [ ] Qualify the accumulated React/shadcn interface against the shared compact dark baseline and retained reference views. Check navigation, workspace context, transcript/composer, changes and approval/recovery states together, including keyboard focus, readable long content and ordinary/minimum window sizing under the actual Tauri CSP; record screenshots and observed failures separately from unit results.
- [ ] Audit A1–A10 against migrated unit coverage and retained process evidence, fill uncovered application-decision regressions under the testing policy, and pass the deterministic stage before the full desktop verification stage. Mark actual durability, ownership, cleanup and responsiveness unqualified until their separate desktop checks pass.
- [ ] Complete repository registration, selected-commit workspace creation, streamed chat, allow/deny, diff refresh, interrupt, Quit/restart, readable history, and exact fixture resume in the actual desktop.
- [ ] Demonstrate desktop-only crash/reconnect without replay, supervisor failure/recovery, uncertain sends/approvals/resumes, pending approval during Quit, stale answers, incompatible owner versions, and actionable storage/shutdown errors.
- [ ] Cross-check 06e communication and 07 streamed observations under sustained
      fixture output: slow observation/catch-up must leave targeted interrupt and
      navigation usable. Verify 09's admission fence and failed stop/flush states,
      then 11's recovery without competing ownership or duplicate actions. Include
      pending approval during Quit once 08 and 09 are both complete; retain exact
      approval semantics and evidence of the outcome.
- [ ] Use the production core/supervisor interfaces with the deterministic adapter at the supported execution seam; preserve exact matching and fixture provenance.
- [ ] Record screenshots of working and error states and navigation/history-opening measurements on the test Mac against the 100 ms/one-second targets under this single-conversation workload.
- [ ] Run repository build, formatting, static checks, and relevant tests; record exact commands/results, implementation revision, hardware, tool/runtime/schema versions, configuration, and evidence locations.
- [ ] Record A1–A10 deterministic/desktop pass/fail/not-run results and native checks as unperformed. Fix failures before native qualification; do not mark the milestone Passed on fixture success.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
