# Ticket 07 independent reviews

Baseline: `423c1fd5323a43a52eda6c82bd7f030145a4871c`.
Initial reviewed commit: `dbebd8a59a70111d4f1147284063b19711a19971`.
Both initial reviewers were launched in fresh contexts (`fork_turns: none`) with explicit `model: gpt-6.1-sol` and `reasoning_effort: high`. Both selectors were accepted without rejection. Engine identity is not independently observable beyond the accepted selectors. Review was read-only; no reviewer ran qualification, provider or authentication actions.

## Initial Standards

**0 concrete violations; 0 actionable baseline smells.** Reviewed native lease extraction, host/task ownership, removed transport paths, direct-feed coverage replacements, launcher shutdown, packaging/qualification tooling, generated-contract removal and evidence against repository policies and engineering skills. Ownership/physical completion and substituted owner-local units are retained; actual OS/storage/native/package qualification is separate. Historical supervisor cohorts remain distinguishable.

## Initial Spec

**1 finding (P2).** Removing process-group isolation allowed terminal Ctrl-C to reach Vite and the native child directly. Vite could terminate while native cleanup remained pending; its SIGINT was classified as failure because only requested SIGTERM/143 was accepted. This violated ticket 07's requirement: “ordinary stop waits for retained shutdown; deadline reports pending/failure, with force-kill recorded as recovery rather than success.” The smoke test signalled the launcher PID and missed the foreground group case. No other missing/partial requirements, scope creep or incorrectly implemented requirements were identified.

## Correction and verification

Retained children now use isolated process groups and ignored terminal stdin. Launcher shutdown continues to use live child handles, waits for native completion before stopping Vite, and retains one attempt on repeated Ctrl-C. Startup readiness accumulation is bounded and handles split stdout chunks.

`tdd-ctrlc-red.log` / `tdd-ctrlc-green.log` record the regression unit. `launcher-process.json` qualifies real process-group Ctrl-C with controlled synthetic native/service children and a pending drain longer than three seconds: no direct interrupt reaches either child, the service remains alive until native release, repeated Ctrl-C does not escalate, and completion exits 0. `ctrlc-smoke.json` separately qualifies the actual normal package/reset cohort and real dev launcher using foreground group SIGINT; all ordinary exits are 0. These fixtures remain distinct from actual native Q4 and its failure/recovery cases.

`complete-after-review.log` passes generated drift, formatting, lint, TypeScript, frontend bundling, all-target/all-feature workspace Clippy and **172 Rust / 48 tooling / 261 UI tests**. Complete gate wall time is recorded in `verification.json`; no new package compilation is needed for launcher-only changes. A fresh final Standards/Spec review follows this correction.

## Final review setup

Final reviewed implementation: `5f217e2` (with `dbebd8a`). Both final reviewers were newly spawned with fresh contexts (`fork_turns: none`), explicit `model: gpt-6.1-sol` and explicit `reasoning_effort: high`; both selectors were accepted without rejection. They reviewed the complete nonempty diff using `git diff 423c1fd5323a43a52eda6c82bd7f030145a4871c...HEAD`. The commit list was supplied to each reviewer. Standards included the repository sources and the full smell baseline; Spec used ticket 07 and governing spec/amendments. Neither reviewer ran competing builds/verification or any provider/authentication action.

## Standards

**0 concrete violations; 0 actionable baseline smells.**

Independently reviewed the final `423c1fd…HEAD` changes through `5f217e2`, including lease extraction, native input admission, task completion, transport deletion, direct-feed replacement coverage, launcher shutdown and Ctrl-C isolation, qualification tooling, packaging, dependency removal, generated contracts and evidence disposition.

The changes preserve the applicable repository requirements: Rust domain ownership, retained physical work and cleanup, lease exclusion, current-contract validation, substituted application units, separate actual qualification and historical evidence. The TypeScript map removal introduces no unchecked assertions or weakened typing.

Applied all supplied smell categories as judgment calls. The private injected resource seams support required controlled outcomes; the launcher’s retained child records express actual lifecycle ownership. Neither warrants speculative abstraction removal or consolidation.

Read-only review only. No edits, builds, tests, qualification, provider or authentication actions were performed. Recorded passing checks were inspected as evidence, not independently rerun. Ticket 08 and broader qualification remain outside this review.

The requested `gpt-6.1-sol` / `high` selectors were supplied for this review invocation; engine identity is not independently observable from within the agent.

## Spec

**0 findings** against `423c1fd...HEAD` at `5f217e2`.

- Missing or partial requirements: **0** within ticket 07.
- Unrequested scope creep: **0**.
- Implemented requirements that appear wrong: **0**.

The final launcher isolates native/Vite children from foreground Ctrl-C, retains one shutdown attempt, waits for native completion before stopping Vite, reports pending cleanup, and treats native force termination as recovery/failure. The retained OS process-group evidence and actual dev smoke cover the reported correction.

Supervisor/transport/staging removal, lease ownership, assertion replacements, generated contracts, private-state instructions and separate qualification/package gates align with the accepted specification and amendments. Evidence preserves failures and clearly leaves live-provider, physical-fault, heap-byte, fresh-target compilation and ticket 08 qualification limits open.

Read-only review only; no files changed or verification processes executed.

Final totals: **Standards 0 / Spec 0**; no outstanding worst issue on either axis. Initial Spec P2 is fixed and its failing/passing evidence is retained above.
