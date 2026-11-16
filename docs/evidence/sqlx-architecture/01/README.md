# Ticket 01 — core startup and complete shutdown

2026-11-16. Scope: `.scratch/sqlx-architecture/issues/01-own-core-startup-and-complete-shutdown.md` only. Incumbent supervisor host; no SQLx adoption, native rehost or ticket 02 work. No live provider, authentication action, submission or user-state reset.

## Source and environment

Baseline `c73fafd308c6990dc0e8e19b82327b536402ac3c`. Implementation commits `ff54eff`, `b8e729e`, `b995893`, `28d4546` plus `90e57ba` and `885c05a` final corrections. Final qualification source/provenance is recorded beside the results below. Existing owner processes/private user state were left untouched.

Mac16,8 Apple Silicon, 12 logical CPUs, 48 GiB RAM; macOS 26.2 (25C56), Darwin 25.2.0 arm64. Rust 1.99.1, Node 26.13.1, pnpm 12.18.0, Tokio 1.54.1. Default worktree-private `target/`, warm caches; no target sharing. Rust unit execution uses one harness thread; tooling one process at a time; Vitest one worker. Compilation, application units, actual process fixtures, desktop fixtures and packaging are separate cohorts.

Installed Intent discovery succeeded; only listed TanStack skills were available, none matched the Rust lifecycle concern. Used implement, rust-engineering and TDD at the agreed core lifecycle/workspace completion/provider stop-join seams. Repository dependency-substitution policy overrides upstream TDD's public-interface-only advice.

## Lifetime and interface changes

The host transfers its socket-free state lease into core startup on its existing driven runtime. The root keeps admission guards, phase, concrete domain completion handles and one retained Quit attempt. A self-retained core keeps a failed or unclaimed startup/shutdown owner alive; successful explicit host completion releases that retention and the lease. Root admission and domain final gates fence effects, including implicit repository cache-miss probes. Existing domains have no workspace-to-provider conversation admission yet; their future combined admission remains ticket 04/chat work.

Workspace effects run through a domain-owned JoinSet and retained worker, with explicit drain and fallible SQLite close. Storage closure is permanent for that owner; late status reads cannot reopen it. Provider tasks continuously reap and settle exact target/attempt/generation outcomes, including panic. Compact terminal faults survive failed persistence independently of reply delivery. Failed intent persistence keeps the credential scope fenced. Root/host collection preserves domain failures alongside transport task failures. Whole-app Quit is now a core transport action, removed from provider requests/replies; protocol 15 and the current shared corpus replace protocol 14 directly.

## Coverage map

| Existing guarantee / assertions                                                           | Current replacement or preservation                                                                                                                                                                      | Future replacement / limit                                                                             |
| ----------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ |
| Registration intent before inspection, immutable action and initial-creation continuation | Existing `registration_records_intent_and_replays_its_status_without_another_effect`, `initial_creation_is_linked_once_and_duplicate_registration_has_no_new_workspace`, uncertain-commit tests retained | SQLx typed atomic writes in 02–04                                                                      |
| Creation intent/stage before external mutation                                            | Existing creation policy tests retained, including `pinned_intent_and_each_stage_precede_their_effect_and_completion_is_durable`                                                                         | Async storage/process conversion in 02–05                                                              |
| Same-ID replay and changed-payload rejection                                              | Existing registration/workspace/selection/login identity assertions retained                                                                                                                             | Current-build identities remain binding through later storage/host cutover                             |
| Owner exclusion and cleanup before release                                                | New `lease_is_last_and_failed_cleanup_retains_it`, root deadline test and separate actual same-root process fixture                                                                                      | Native lease/crash qualification in 06–08; incumbent process results are not native-host qualification |
| Workspace Drop/join and provider-only whole-app Quit                                      | Explicit owned completion; `explicit_finish_awaits_physical_completion_and_retains_worker_failure`, core deadline/repeated-Quit, partial-startup and combined-host/domain fault tests                    | Old Drop-as-success/provider shutdown actions removed; provider-local stop/join retained               |
| Worker and reply loss                                                                     | New exact provider panic, creation/registration/inspection panic, panic-plus-failed-persistence/no-replay tests; existing blocking-permit/request-drop tests retained                                    | Driver settlement receipts in 02; streaming/conversation outcomes later                                |
| Final fence and status                                                                    | Root guard test, `fenced_status_cannot_queue_a_probe_or_reopen_closed_storage`; actual lost-request/active-Git Quit fixture                                                                              | Combined conversation admission is not implemented in this slice                                       |
| Permanent reservations and cleanup uncertainty                                            | Existing name reservation, conflict, lost-effect-reply and failed-store tests retained                                                                                                                   | Bounded concurrent allocator and activity records in 04; no current assertions removed                 |
| Observations, exact owner/target/cursor and physical request lifetime                     | Existing Rust/TypeScript observation units and current shared corpus retained; protocol updated to 15                                                                                                    | Supervisor socket/peer tests remain until 06/07 remove their production path                           |
| Retired application-format-only checks                                                    | No migration/legacy-reader tests removed in 01; protocol-14 corpus replaced by current protocol-15 corpus                                                                                                | 02–04 retire superseded storage-format paths; old-format-only checks need no compatibility replacement |

No useful assertion was deleted. Existing historical evidence was preserved. The new process qualification script is explicitly outside unit discovery.

## Verification record

Focused checks included root fencing/transfer, physical workspace completion, creation panic, inspection panic, registration panic with failed persistence, provider panic, and combined host/domain failure. Workspace cohort: 66 passing before added lifecycle regressions; provider cohort: 13 passing before added panic regression. `node scripts/cargo.mjs check --locked --workspace` and focused Clippy passed during implementation. TypeScript checked after contract updates.

Retained failed attempts:

- `build-drift-failure.log`: packaging preparation correctly rejected stale generated manifest after source edits; regeneration resolved it.
- `verify.log`: Rust current-corpus check found protocol 14 after interface moved to 15 (215 passed, one failed).
- `verify-final.log`: corpus rename exposed four TSX imports still naming the retired corpus.
- `verify-pass.log`: Rust 217/217 and tooling 51/51 passed; UI 258 passed, two stale literal protocol assertions failed. `corpus-recheck.log` records all 17 tests in those two files passing after correction.
- One early deadline test used a continuously-ready yield loop with paused Tokio time. It was explicitly terminated (SIGTERM), changed to controlled paused-time waiting, and its focused rerun passed. An initial command used unavailable `python`; no edit ran until `python3` was used.

Final full-gate, packaging and fixture outcomes are recorded below. Durations are execution observations, not comparative performance claims; no fresh-target compilation was performed. Warm-up/five-run measurements are recorded separately below. No power-loss, escaped-descendant, future native-host or native-provider qualification is claimed.

## Code review

Both requested reviewers were launched in fresh contexts with explicit `model: gpt-6.1-sol` and `reasoning_effort: high`; these settings were accepted. Both reviewed the same fixed baseline and final implementation changes. Re-review used the same agents and settings.

### Standards

Initial findings: closed storage could reopen after drain; implicit cache-miss probes bypassed the fence. Follow-up findings: simultaneous host/domain errors were not aggregated; failed persistence could overwrite a registration panic. All were fixed with regression coverage. Final report: **no unresolved Standards findings**.

### Spec

Initial findings: closed-store reopen; worker panic skipped close; inspection panic was absent from Quit aggregation. Follow-up findings: combined host/domain error retention and panic-plus-failed-persistence retention. All were fixed with regression coverage. Final report: **no unresolved Spec findings**. No scope expansion was found.

Final totals: Standards 0 unresolved; Spec 0 unresolved. Review is source evidence, separate from execution and qualification.

## Final verification and qualification

Final runtime source: `c27122d` (full revision in final sidecar provenance). Only evidence and the qualification script's flattened-registration assertion changed afterward. The build recorded dirty=true because untracked evidence was present; no uncommitted Rust/TypeScript runtime change was used. [Sidecar provenance](sidecar-provenance.json) records release profile, target, revision and SHA-256. Supervisor SHA-256: `ab27781b4055c8f297ebed55772ae300ab61c1199f05a08aab4529806d4be46b`.

| Cohort                                   | Command / evidence                                                                                                                                                                              | Result                                                                                                                                                                                                       |
| ---------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Complete static/application verification | `corepack pnpm verify`; [verify-owner-panic.log](verify-owner-panic.log)                                                                                                                        | Exit 0: generated drift, formatting, Oxlint, TypeScript, workspace Clippy all targets/features, **219 Rust units, 51 tooling tests, 260 UI tests**                                                           |
| Packaging                                | `corepack pnpm build`; [build-owner-panic.log](build-owner-panic.log)                                                                                                                           | Exit 0; release macOS app 18.65 MiB. Existing ts-rs attribute notices and Vite large-chunk warning retained; no performance improvement claimed                                                              |
| Actual lease/process exclusion           | `python3 scripts/qualify-core-lifecycle.py target/release/workspace-supervisor docs/evidence/sqlx-architecture/01/process-qualification-final.json`; [result](process-qualification-final.json) | Exit 0; private fresh state, actual second-owner rejection, lost request while Git paused, repeated pending Quit, rejected admission, physical drain, process exit and current-build restart/action recovery |
| Fixture desktop                          | Private cloned/re-signed current release bundle with private state/CODEX_HOME and synthetic runtime override; [context](desktop-context.json), [result](desktop-qualification.json)             | Connected; details matched desktop 23421/core 23426 and exact owner/revision; native Cmd+Q completed owned shutdown; both exact processes exited                                                             |
| Native providers/authentication          | Not run                                                                                                                                                                                         | Not authorized; no submission, authentication or provider action was performed                                                                                                                               |

Additional retained harness failures: [process-qualification.json](process-qualification.json) completed exclusion/pending/drain checks but failed its recovery assertion because the harness expected a nested state instead of Serde's flattened `Registration.state`. Only that assertion was fixed, then a fresh private-state run passed. The first desktop harness parent exited before app binding, and accessibility launched another fixture instance without the private environment; it displayed a frame deadline error and was explicitly excluded. Only that exact fixture desktop process was terminated; existing user/other fixture supervisors were not touched. The retained-parent launch then passed, with process ownership verified before UI actions.

The final review additionally caught a dispatch call discovering a completed panic after checking admission. A red regression reproduced the escaped dispatch; moving reaping before the final fence check made both async and blocking cases pass. The final two reviewers confirmed no remaining findings through `c27122d`.

Limits: the desktop slice covers startup/status/normal Quit, while controlled active-Git/pending/repeated-Quit behavior was qualified at the actual incumbent process seam. It does not establish future Tauri-host lifetime, SQLx durability, native crash/escaped descendants, streaming or provider authentication. No full milestone completion is claimed; ticket 02 remains untouched. Fresh qualification state was created under `/private/tmp/aw-core01-*`; user state, repositories/worktrees, credentials, provider history and earlier evidence were preserved.

Final audit also covered poisoning while a domain decision owns its mutex: `poisoned_decision_owner_retains_active_identity_and_fences_reuse` retains the active registration/workspace identity and blocks reuse. Both reviewers reported no findings on that final narrow change. The final complete gate passed 219 Rust, 51 tooling and 260 UI tests; earlier successful runs are retained as checkpoints.

### Warm execution measurements

`corepack pnpm test:measure` exited 0 on revision `885c05a`: one warm-up plus five warm runs of each cohort; all repeated suites passed. [Raw log](unit-measurements.log). Warm medians/ranges: Rust 0.431822 s (0.429087–0.439081), tooling 0.911940 s (0.900162–0.918626), UI 13.480509 s (13.396175–13.549836). Cached compilation was reported separately at 0.094254 s. These are same-source execution measurements, not before/after comparisons. The added final backend-only poisoning regression is separately remeasured below; tooling/UI sources are unchanged.

Final `c27122d` backend measurement: one warm-up plus five warm runs, 219/219 passing each time. Median 0.440059 s; range 0.438508–0.440907 s. [Commands/results](final-core-measurements.json), [raw executions](final-core-measurements.log). Final release process and desktop fixtures were repeated on that exact runtime revision and passed; the earlier `885c05a` provenance/context/result files remain checkpoints.

The evidence commit hook initially rejected formatting of `final-core-measurements.json`; formatting was corrected before committing. No application check failed at this step.
