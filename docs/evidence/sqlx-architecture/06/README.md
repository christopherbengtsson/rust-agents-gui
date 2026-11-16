# Ticket 06 — native Tauri core host

Implementation starts at `97265159cbfcc0998df66b861eb119cae1dd910a` on the current branch. Only ticket 06 is in scope. No live provider, real authentication, agent turn, RAT inspection or user-state reset is authorized or performed.

## Ownership and current interfaces

Tauri constructs one app-scoped NativeHost on its singleton Tokio runtime. The host retains startup, incarnation, request completion, channel delivery and one whole-core Quit. Core domains retain execution, reconciliation and SQLx ownership. Lease acquisition uses the existing owner.lock inode without binding an application socket. Explicit state overrides remain exact. Defaults use native-schema-4, independent of a supervisor binary/protocol.

Current generated contracts use CoreSnapshot/CoreStatus/CoreState, native_pid and incarnation on snapshots/replies/feed cursors. Commands target expectedIncarnation. No selected command performs peer negotiation/discovery. Old supervisor source and packaging resources remain unreachable pending ticket 07.

Native ordinary admission: 64 nonwaiting slots and 64 KiB request/result checks. Eight reserved control slots serve action status/cancel/reconciliation. Reply guards retain capacity through native serialization/drop, independently of requester lifetime. This does not bound Tauri's received argument/decode copies or WebView heap. Existing storage reads retain their tighter eight-interest bound.

The direct channel reuses the bounded journal and acknowledgement control: 256 entries, 32 tags/4 KiB batch, one unacknowledged batch, five-second heartbeat and fifteen-second acknowledgement deadline. Replacement waits for setup/delivery cleanup, and a panicked completion remains fenced. Frontend acceptance stays synchronous before acknowledgement.

Query alone owns observation bodies, with five-minute GC and at most 128 inactive settled entries. Disabled feature observers still protect active/pending interests; eviction retains separate identity/freshness/physical-work owners. No serialized body index or fixed JavaScript heap guarantee is introduced.

## Verification and coverage mapping

Application units substitute storage/filesystem/Git/process outcomes. Actual native fixtures use the production registered commands and real WebView, core, SQLx and process owners. The `qualification` desktop feature injects a disposable local driver and records controlled observations; normal builds exclude its commands. Copied readiness runtime and schemas retain Codex 0.169.0 / d32063f96c2dff1dc96af839b3952abb6acfeda4 provenance; all traffic is synthetic.

| Useful prior assertion                                        | Current owner/evidence                                                                                                          |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| Exact serving owner/target and stale reply rejection          | Current incarnation contracts and existing updated feature/validator units; native stale-incarnation scenario                   |
| Startup/admission versus Quit, lease-last and pending cleanup | Retained core lifetime units plus NativeHost startup, failed exclusion and request-loss units; actual startup/owner/Quit cohort |
| Socket subscription ordering, cancellation and replacement    | Retained controlled mechanism units plus direct NativeHost exact-ack/failure-fence units; actual WebView replacement            |
| Physical request lifetime and retained result capacity        | NativeHost abandoned requester/reserved control and serialized-result guard units; existing frontend unsettled registry units   |
| Observation freshness, last-good data and pending identities  | Existing feature/actor/retained-delivery tests updated together; Query inactive-eviction unit                                   |
| Peer-only protocol checks                                     | Incumbent-only source retained for ticket 07 deletion; current native corpus has no peer version                                |
| Provider scope/login/process cleanup and workspace recovery   | Existing substituted policy units retained; separate new Q4 native fixture cohort                                               |

Exact attempts, commands, exits, durations and dirty source hashes are recorded by run.py in verification.json. Failed preparation, compilation and assertions remain recorded. Results, review and remaining limits are appended after completion.

## Q4 fixture results

[native-q4-current.json](native-q4-current.json) passes the actual native-host cohort: registration/first creation; two concurrent creations alongside inspection; synthetic readiness/browser sign-in through navigation; real destruction/recreation of the WebView with unchanged incarnation and attempt; visible progress-return link and Cancel sign-in control; native crash/reopen with a new incarnation and exact interrupted evidence, no repeated login start and a surviving-work fence; competing same-root exclusion; repeated successful Quit during active synthetic login; failed cleanup retaining the actual OS lease; and Quit during startup. Successful native/competitor/startup exits are 0. The deliberate crash and recovery termination after failed drain are SIGKILL and are not successful Quit. Copied native logs preserve checkpoint, action/queue/task and shutdown evidence.

Actual WebKit retention after inserting 160 small fixture entries: **129 entries = 128 inactive + 1 observed; 0 fetching**. WebKit exposed no `performance.memory` heap API. This count result is not an actual JavaScript byte/heap guarantee; heap bytes remain unmeasured. No second observation-body index is added. The frontend measurement listener is included only by the explicit VITE_NATIVE_QUALIFICATION build flag and absent from normal builds.

CUA separately observed the actual first-cohort native diagnostics: native PID and desktop PID both 91141, named core incarnation, accessible diagnostic dialog and Escape returning focus to Connection details. That observation qualifies the changed diagnostic presentation, not a full accessibility/user trial.

Failed/unqualified attempts are retained: first driver incorrectly compared the structured workspace status to a string; second exposed the synchronous Quit runtime bug (replaced with a failing-then-passing unit); a package attempt used the unavailable stable WindowBuilder export; third qualification accidentally consumed the prior binary after that failed build and is excluded as stale; fourth exposed fixture-only asynchronous label-reuse ordering. Fifth and final cohorts pass after correction. No failed result is counted as native success.

Ticket 07 still owns supervisor-source/resource deletion, clean dev/package audit and final launcher/packaging qualification. This slice removes automatic three-second SIGKILL from the launcher; a pending drain is reported and awaited. Native signal-driven Quit is exercised here. Dev still stages the unreachable declared resource until ticket 07 removes it. No current or historical user state, worktrees or credentials are reset.

Final native ordering also keeps the lease after core/request/feed completion until the Tauri signal listener has been aborted and joined. The native adapter then explicitly releases the lease and permits exit. A controlled marker unit covers this final barrier. Snapshot/failure-event presentation is bounded while the host retains the full failure report.

The current storage graph is unchanged: SQLx/sqlx-sqlite 0.9.0 and libsqlite3-sys 0.37.0, bundled SQLite 3.51.3 with the previously qualified WAL fix (ticket 04 evidence). This ticket changes storage tracing only; new native Q4 exercises that production graph. It claims neither new SQLx fault qualification nor a storage performance improvement.

## Final verification before independent review

- `complete-verify-final.log`: generated drift, formatting, Oxlint, TypeScript, all-target/all-feature workspace Clippy and isolated units pass: **235 Rust / 51 tooling / 261 mocked-IPC UI tests**. Complete gate wall time: 23.759 s, including compilation. The earlier complete gate failed on the renamed Hook's stale tooling-test path; that path is corrected and its compiler assertion retained.
- `package-normal.log`: normal release package passes in 48.720 s, including retained unreachable-resource staging. `normal-smoke.json` separately confirms ordinary package startup and signal-driven completed drain; normal artifacts exclude both fixture replacement commands and the frontend retention driver. Ticket 07's deletion/clean-package audit remains open.
- `unit-measurements.json`: already compiled codegen-enabled Rust unit executable, one harness thread, one warm-up/five warm executions; all 235 pass. Median 0.352393 s, range 0.348649–0.362576 s. Compilation is separately recorded in the complete gate. The first timing set accidentally overlapped packaging; it is retained as `unit-measurements-overlapped-excluded.json` and excluded from timing claims. Fresh-private-target compilation was not repeated.
- Environment: Mac16,8 / 12 CPUs / 48 GiB, macOS 26.2 arm64, Rust 1.99.1, Node 26.13.1, pnpm 12.18.0. Default claimed Cargo target, release packaging and warm test profile; no shared/symlinked target override.

React/shadcn review preparation includes the repository skills, installed Base UI 1.9.0 components, CLI 4.23.0 project context (Base UI/base-nova), and official Base Button/Dialog/Alert documentation. No toolkit components, dependency versions, tokens or primitive APIs were changed. The current frontend/native source adds no TypeScript `any`, double assertion, suppression, declaration shim or weakened checking coverage; existing assertions retain their prior scope.

Remaining limits: no live provider/authentication/agent work; no real provider compatibility claim; no JavaScript heap byte measurement; no fresh-target compilation or full accessibility/user trial; no unimplemented chat continuity/stream-pressure claim. Actual fixture, units, normal package and historical evidence remain distinct. Future restart/update must share the native completion path: fence, await core/request/storage/feed completion, join native adapter resources, release the lease last, and only then invoke framework restart. No restart/update implementation is added here.

## Completion and review

Implementation commit: `1cb8f5a`. [Independent reviews](review.md) report **Standards 0 / Spec 0**, with both fresh agents accepting explicit GPT-6.1 (`gpt-6.1-sol`) / High selectors. React and shadcn were explicitly applied. [Source alignment](source-alignment.json) confirms that current Q4, complete verification and normal-package checks used the reviewed production/tooling source hashes. Subsequent changes record evidence only.

The qualification processes and local HTTP servers have exited. CUA closed only remaining browser fixture tabs whose loopback ports matched this ticket's retained controls; unrelated tabs were left alone. Private fixture roots/worktrees and all prior evidence remain available. No ticket 07 implementation or parent/milestone closure is claimed.
