# 12: Verify the complete desktop workflow with fixtures

## Native core amendment — 2026-11-16

Apply the [accepted native-core/SQLx spec](../../sqlx-architecture/spec.md) and
[amendments A2/A3/A8](../../sqlx-architecture/amendments.md) before this ticket.
They replace separate-supervisor/native-crash continuation and single-active
instructions with native ownership, WebView continuity, durable native-failure
recovery and the concurrent three-conversation scenario. Preserve exact controls,
remaining milestone scope and historical evidence. Acceptance remains unchecked;
this publication authorizes no live-provider or authentication action.

**What to build:** Prove the complete Milestone 1 workflow in the actual desktop using deterministic fixtures and retain an auditable A1–A10 result set before native submissions.

**Blocked by:** [05 — Workspace inspection](05-inspect-uncommitted-workspace-changes.md); [11 — Native failure and surviving runtimes](11-recover-from-supervisor-failure-and-surviving-runtimes.md).

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** Integrated deterministic and desktop verification for all A1–A10; verification stages 1–2.

- [ ] Qualify the accumulated React/shadcn interface against the shared compact dark baseline and retained reference views. Check navigation, workspace context, transcript/composer, changes and approval/recovery states together, including keyboard focus, readable long content and ordinary/minimum window sizing under the actual Tauri CSP; record screenshots and observed failures separately from unit results.
- [ ] Audit A1–A10 against migrated unit coverage and retained process evidence, fill uncovered application-decision regressions under the testing policy, and pass the deterministic stage before the full desktop verification stage. Mark actual durability, ownership, cleanup and responsiveness unqualified until their separate desktop checks pass.
- [ ] Complete repository registration, selected-commit workspace creation, streamed chat, allow/deny, diff refresh, interrupt, Quit/restart, readable history, and exact fixture resume in the actual desktop.
- [ ] Demonstrate WebView replacement/catch-up without replay, native failure/reopen with durable-prefix recovery, uncertain sends/approvals/resumes, pending approvals during Quit, stale answers, wrong core incarnation, competing state lease and actionable storage/shutdown errors. Do not recreate peer protocol/version negotiation.
- [ ] Use the production native-hosted core/SQLx/Tauri path with grounded deterministic execution fixtures; preserve exact matching and provenance. Setup-only fixtures are insufficient for chat/approvals.
- [ ] Close Q6 fixture pressure with the predeclared three-conversation trace: hot and quiet conversations in workspace A, a quiet conversation in B, slow writer, same-topology approvals/interrupt, navigation/history and final Quit. Require progress for both quiet conversations, bounded app memory, exact effects/control targeting and complete accounting of offered/admitted/committed/failed/outstanding events/bytes. Report sustained success separately from explicit connection overload/prefix/gap/cleanup uncertainty; no silent loss or apparent complete history.
- [ ] Record screenshots and navigation/history measurements on the test Mac against 100 ms/one-second targets under that concurrent workload, plus declared control/settlement deadlines, actual receipt/completion, app heap/buffers versus child RSS, queue/acquisition/commit/checkpoint/WAL behavior and all drain outcomes. Use repository timing methodology; the conversation counts are not product caps.
- [ ] Run repository build, formatting, static checks, and relevant tests; record exact commands/results, implementation revision, hardware, tool/runtime/schema versions, configuration, and evidence locations.
- [ ] Record A1–A10 deterministic/desktop pass/fail/not-run results and native checks as unperformed. Fix failures before native qualification; do not mark the milestone Passed on fixture success.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

2026-11-16: Reconciled with the native-core/SQLx spec and amended A8/A10/concurrency
acceptance. Existing dependency edges remain; milestone 07 now inherits SQLx 08.
Qualification stays open and no production implementation is claimed.
