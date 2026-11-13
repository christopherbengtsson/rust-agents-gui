# 06b — Codex setup end to end

2026-11-13. Implemented on `feature/m01-codex-setup` from
`c5130794eabf8273c1650e7cb3f52b958f30fd1a`. This report belongs to the implementation
commit and subsequent review fixes. Parent 06, 06c–06d and Milestone 01 remain open.

## Contracts and ownership

Core's concrete `runtimes` module owns setup admission, sequencing, recovery and
observations independently of `workspaces`. `Runtimes::request` accepts Observe,
Action, Save and Check requests. Mutations bind supervisor owner, runtime Codex,
configuration revision and UUID action identity. Reads target the current owner;
historical action lookup retains its original owner/request identity. A 32-action
session cache is only an optimization: cache misses read durable action identity
before fresh revision/cleanup admission rules or effects. A repeated admitted check
returns its exact lookup status and never respawns. During an occupied lane, an
uncached identity returns unresolved lookup status without admitting another job.
Rejected conflicting payloads are separate from the canonical action. Action reads
carry the original retained mutation as `expected`, letting Rust reconstruct a
conflict rejection after restart without persisting or replaying that payload.

One runtime-owned worker and a capacity-one channel serialize persistence and
process work. One request may be queued or executing; retained cleanup reconciliation
can coexist with that queued identity lookup, while its fence blocks new setup
effects. Shared-state locks only copy/admit/publish observations; storage,
process waits, stdio and cleanup occur outside them. Check begins only after the
action and recovery intent commit. Save starts no Codex process and atomically
updates the executable revision and recorded result, invalidating readiness.

SQLite schema **4** adds runtime_configuration, runtime_actions and runtime_recovery
to the existing private metadata database. The narrow RuntimeStore seam exposes no
repository policy/error type. The historical common metadata opener remains inside
storage and translates errors at that seam; broad storage/exchange refactors were
not required. Transaction control distinguishes confirmed rollback from uncertain
COMMIT/rollback state. Failed connections close before authoritative reopening;
readback never sees its own unresolved transaction and never retries a write.

Wire protocol **6** adds a typed Runtime response to the existing bounded socket
exchange. Mixed domain requests reject. SupervisorClient and SupervisorConnection
forward requests only to the discovered owner; a failed action exchange never
starts another owner or replays. Tauri forwards one typed command off its executor;
TypeScript validates observations, discriminants and exact lookup targets, including
the retained original mutation payload after an owner change. Shared exchange
consolidation remains deferred.

## Protocol, budgets and lifecycle

The [fixture provenance](../../crates/core/src/runtimes/fixtures/README.md) records
Codex 0.167.2, official source revision, retained source/schema hashes and synthetic
inputs. Metadata calls are initialize → initialized → account/read
(`refreshToken:false`) → config/read (`includeLayers:false`, no project CWD).
No login/logout, credential file access, configuration write, model listing, thread
or turn is performed. Request IDs preserve string versus signed integer identity;
unexpected server requests fail closed and grant no approval.

Discovery uses the saved absolute override, otherwise inherited absolute PATH
entries, ~/.local/bin, /opt/homebrew/bin and /usr/local/bin. An invalid override
never falls back. Commands invoke the executable directly with --version or
app-server and a private application-owned CWD. Version must equal 0.167.2.
Native and synthetic stdio adapters share command construction, framing, sequencing,
budgets and high-level cleanup decisions.

| Budget                                                       | Value                                                     |
| ------------------------------------------------------------ | --------------------------------------------------------- |
| Queued or executing request                                  | 1                                                         |
| Admitted setup effects, including owned cleanup              | 1                                                         |
| Cached action records                                        | 32                                                        |
| Executable preference                                        | 4096 UTF-8 bytes                                          |
| Provider frame                                               | 256 KiB                                                   |
| Total received stdout + stderr per check                     | 1 MiB                                                     |
| Queued stdin bytes                                           | 64 KiB                                                    |
| Poll/drain batch                                             | 64 chunks, native chunks at most 4096 bytes               |
| Version                                                      | 5 s                                                       |
| Initialize                                                   | 10 s                                                      |
| Each account/configuration read                              | 5 s                                                       |
| Overall check, including reserved cleanup                    | 30 s                                                      |
| Cleanup                                                      | close stdin; TERM at 1 s; KILL at 2 s; unconfirmed at 3 s |
| Explicit worker shutdown observation                         | 4 s                                                       |
| Existing SQLite busy timeout / socket frame / socket timeout | 100 ms / 64 KiB / 2 s                                     |

Native stdio is nonblocking, independently drains stderr and bounds writes/frames
before allocation. Cleanup policy has its own controlled effect seam. The retained,
unreaped child pins process-group signaling authority; cleanup observes group
absence, drained pipes and release of a private inherited execution lease. Child
exit alone never means descendant cleanup. There is no signal after reaping or
from a saved PID. Failed cleanup retains process handles with the runtime worker
and a durable launch fence; shutdown reports failure. Subsequent owned cleanup
reconciliation can clear the fence only after confirmed cleanup and durable
completion recording, without repeating a check. Core exposes shutdown_runtimes. If its runtime owner is destroyed after a failed
bounded shutdown, it transfers the unfinished JoinHandle to private runtime
recovery ownership. The worker continues bounded cleanup attempts and retains
its process handles until confirmed; it does not detach or block destruction
indefinitely. No successful supervisor Quit/flush workflow is claimed here;
that existing milestone lifecycle work remains separate.

Only preferences, action/recovery markers and boot identity persist. Successful
diagnostic details are session observations. Supervisor restart never repeats a
pending launch. Same-boot unresolved ownership remains fenced even if a lease looks
free; a different known boot establishes previous processes cannot survive. Unknown
boot identity remains fenced. This conservative policy may require restarting the
Mac after an ambiguous supervisor crash; no unsafe same-boot replacement is offered.

## Settings behavior

App owns one useRuntimes hook outside Settings and the retained workbench Activity.
Mount/navigation/reconnect only observe. The hook retains submitted mutation
identity before IPC; unavailable replies use exact status reads, without replay.
Owner changes drain earlier reads, preserve historical recovery identity and ignore
stale responses. Transport failure labels retained evidence stale instead of claiming
Codex failed. Save/check admission and recovery decisions stay in Rust.

CodexSettings owns only its draft/form and recovery composition. As selected during
planning, Back unmounts/discards unsaved edits; returning shows the saved value.
A submitted Save may still commit after leaving. Baseline updates preserve newer
draft edits while the screen remains mounted. Check requires a saved configuration
matching the input. Pending work prevents another Save/reset/check but permits typing.

Results show stage, time, executable/version, selected authentication/provider and
billing evidence or unknown, with separate actionable error states. “Setup check
passed” establishes metadata only, not model/project compatibility. Nothing in
this module gates later normal Send/Resume or requires login each app launch.
Raw config, credentials, provider errors and stderr are excluded from observations
and ordinary diagnostics. Custom provider billing remains unknown.

The application-owned documentation command accepts only install/authentication
destination keys; opener's automatic JS link handling is disabled and no generic
opener capability is granted. Existing Field labels/errors, compact controls,
anchor semantics, Base UI refs and CSP remain. Copy/open failures remain visible.

## Verification and remaining qualification

The agreed TDD seams use substituted storage, filesystem, process/stdio, time and
IPC outcomes, retaining production policy, command construction and interpretation.
No unit test starts a runtime/supervisor, SQLite database, Git process or production
wait. Initial red attempts included missing Rust implementation symbols and the
missing CodexSettings module. Focused Rust/DOM suites passed after implementation.
An intermediate Clippy run found formatting, enum size and argument-count issues;
those were corrected before the complete gate.

The initial configured [complete gate](milestone-01-codex-setup/verify-initial.log)
passed **133 Rust + 25 tooling + 109 UI** tests, format, lint, TypeScript and Clippy.
Its total wall time was not captured. Subsequent storage-read recovery and shutdown
changes require the final recorded gate below. No failed attempt is a native defect
claim. Build, measured-unit and final-review results are appended after completion.

[Private-state fixture instructions](milestone-01-codex-setup/fixture-instructions.md)
cover production-supervisor success and primary synthetic failures. These
instructions have **not** been exercised as actual desktop/process qualification
in 06b. Unit success is not SQLite durability, OS ownership/lease/descendant cleanup,
WebView accessibility/CSP/responsiveness or native compatibility evidence. Those
remain 06d. Native-provider processes/submissions, login changes and runtime
configuration changes: **zero** in this implementation session.

## Final reviewed-source results

Runtime source: `77f84c81291d44299f705103d380debc8f5114b5`. The final evidence
commit changes documentation/logs only. [Source/environment identity](milestone-01-codex-setup/source-identity.json)
records file SHA-256 values, target, toolchain and the dirty documentation state.
Machine: Apple M4 Pro, 48 GiB, macOS 26.2 ARM64; Rust/Cargo 1.99.1, Node 26.13.1,
pnpm 12.18.0. Units use the default worktree target, test profile, warm cache and
one harness worker/thread. No fresh-private-target compilation was performed.

| Command                      | Result                                                                           | Wall time |
| ---------------------------- | -------------------------------------------------------------------------------- | --------- |
| `corepack pnpm verify`       | Format, lint, TypeScript, Clippy; **140 Rust + 25 tooling + 113 UI**, all passed | 10.482 s  |
| `corepack pnpm test:measure` | One warm-up + five warm executions for each harness, all passed                  | 49.277 s  |
| `corepack pnpm build`        | Production app bundle built, 15.31 MiB; no app/provider launched                 | 41.648 s  |

The [command/timing records](milestone-01-codex-setup/checks.json) distinguish
intermediate and reviewed-source cohorts. Final logs:
[verification](milestone-01-codex-setup/verify-reviewed.log),
[measurements](milestone-01-codex-setup/unit-measurements-reviewed.log),
[packaging](milestone-01-codex-setup/build-reviewed.log).
The existing Vite chunk-size advisory remains visible in packaging logs; packaging
completed successfully. An intermediate commit hook rejected generated evidence
JSON formatting; formatting was corrected, with no verification failure concealed.

Warm unit compilation took 0.076989 s separately from execution. Five-run warm
execution median/range: Rust **0.389715 s / 0.388297–0.390849 s**; tooling
**0.837268 s / 0.827543–0.848928 s**; UI **6.958942 s / 6.901620–6.968020 s**.
These are harness measurements, not actual WebView responsiveness claims.
The [staged sidecar provenance](milestone-01-codex-setup/sidecar-provenance.json)
records Cargo's exact executable, source, target/profile and binary hash.

Independent [Standards and Spec reviews](milestone-01-codex-setup/review.md) found
and prompted fixes for lifecycle ownership, definite Save rejection, primary error
preservation, recovered UI outcomes, read-lane feedback and exact durable action
identity/restart recovery. Final re-review: **Standards 0, Spec 0 unresolved**.

06b implementation and controlled-check acceptance is complete. 06c expands
adversarial coverage; 06d owns actual desktop/process, accessibility/responsiveness,
SQLite durability/migration, OS cleanup and separately permitted native metadata
qualification. Their unperformed results remain open. Parent 06 remains open.

## 06b recovery correction — 2026-11-13

This scoped follow-up addresses the three missed 06b review cases against
`0b2e287`; it does not implement the remaining 06c cohort or perform 06d
qualification. Previous implementation, review and verification results above
remain historical evidence.

- Lease acquisition failure before either launch now returns unconfirmed cleanup
  even without a quarantined Session. Private cleanup evidence distinguishes
  confirmed cleanup, retained owned handles, and unowned lease uncertainty. The
  latter persists the recovery fence without granting signaling authority or
  retaining a worker merely to poll an unowned lease.
- A worker-owned PendingCheck retains its original action/request, configuration
  identity, diagnostic, cleanup evidence and completion-record state. Later
  observations, cached/evicted repeated actions and rejected mutations cannot
  redirect cleanup persistence to another action. Background finalization also
  preserves a concurrently queued request's admission lane.
- Confirmed process cleanup survives final-record rollback or uncertain COMMIT.
  The owner retains the diagnostic and a distinct storage fence, rereads exact
  authoritative action status, and either accepts a matching committed result
  or retries only completion recording and that action's recovery-marker removal.
  Failed readback performs no write. No Save, process or metadata request replays.
  Pending finalization participates in existing bounded worker shutdown/recovery;
  a storage-only timeout reports storage uncertainty rather than unconfirmed
  process cleanup. Combined metadata/cleanup failures preserve the primary error.

Public requests/replies, wire protocol 6 and SQLite schema 4 are unchanged.
Same-boot and unknown-boot unresolved ownership remain fenced. Only a different
known boot can exclude earlier survivors without retained process authority.
Success details remain session observations, not durable provider diagnostics.
The pinned Codex 0.167.2 protocol/schema was reinspected; source and retained
schema/success-transcript bytes are unchanged. New synthetic lease and combined
faults are documented beside fixtures and in the private-state instructions.

### Controlled regression mapping

| Missed requirement                                                          | Production-policy regression evidence                                                                                                                                                            |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Truthful cleanup/fencing on early lease failure                             | `lease_contention_before_either_launch_is_not_confirmed_by_empty_quarantine`; `unowned_lease_uncertainty_persists_without_claiming_worker_handles`                                               |
| Recover final persistence while retaining known cleanup and original result | `confirmed_cleanup_retries_only_completion_after_storage_recovers`; `lost_completion_reply_is_read_back_without_another_write`; `failed_check_evidence_survives_completion_persistence_recovery` |
| Cleanup stays bound to the original check                                   | `delayed_cleanup_keeps_its_check_identity_across_other_requests`, including an evicted repeated action and a rejected fresh mutation                                                             |
| Recovery retains lane/lifecycle boundaries                                  | `background_finalization_preserves_a_queued_request_lane`; `pending_finalization_participates_in_bounded_shutdown_without_repeating_cleanup`                                                     |
| Existing boot policy                                                        | `recovery_fence_needs_a_different_known_boot_to_exclude_survivors`                                                                                                                               |

The initial regression compilation intentionally failed because the new recovery
boundary/types did not exist; see [red log](milestone-01-codex-setup/06b-recovery/red.log).
Intermediate focused units and Clippy passed. The final focused runtime suite
passed 34 tests (115 filtered out). All behavior dependencies are substituted:
no SQLite database, supervisor/runtime process, Git command or production wait
runs in these units. No previous coverage was removed.

Independent Standards and Spec reviews found zero unresolved findings after the
shutdown-error classification follow-up. The Spec follow-up identified that bounded
shutdown still reported cleanup uncertainty for persistence-only pending work;
that finding was fixed and covered before final verification. Reviews ran read-only,
with no separate build/test commands.

Final command results, source hashes, versions, target/profile/cache conditions and
packaging provenance are recorded under
[`06b-recovery/`](milestone-01-codex-setup/06b-recovery/).
Desktop/fixture-process, SQLite durability/migration, OS lease/descendant cleanup,
WebView accessibility/responsiveness and native compatibility remain **unperformed**
for this correction. Native provider processes/submissions, login changes and
runtime configuration changes: **zero**. Parent 06, 06c–06d and Milestone 01 stay open.

### Final correction verification

All final commands passed against the file hashes in
[source identity](milestone-01-codex-setup/06b-recovery/source-identity.json).
The source was dirty relative to `0b2e287` during these runs; this correction's
commit contains that exact runtime source. Machine/toolchain and cache conditions
are captured there. Units used the default worktree target, test profile, one
harness worker/thread and warm cache; no fresh-private-target compilation ran.

| Command                      | Result                                                                           | Wall time   |
| ---------------------------- | -------------------------------------------------------------------------------- | ----------- |
| `corepack pnpm verify`       | 149 Rust + 25 tooling + 113 UI tests; format, lint, TypeScript and Clippy passed | 9.974416 s  |
| `corepack pnpm test:measure` | One warm-up plus five warm executions per harness, all passed                    | 47.861941 s |
| `corepack pnpm build`        | Production app bundle built, 15.32 MiB; no application/provider launched         | 41.666775 s |

Warm unit compilation: **0.075321 s**, measured separately from executable runs.
Warm median/range: Rust **0.385233 s / 0.383898–0.387985 s**; tooling
**0.795671 s / 0.784829–0.803927 s**; UI
**6.815792 s / 6.658353–6.885767 s**. These measure harnesses, not WebView latency.

Retained logs: [verification](milestone-01-codex-setup/06b-recovery/verify.log),
[measurements](milestone-01-codex-setup/06b-recovery/measure.log),
[packaging](milestone-01-codex-setup/06b-recovery/build.log).
[Command records](milestone-01-codex-setup/06b-recovery/checks.json),
[staged sidecar provenance](milestone-01-codex-setup/06b-recovery/sidecar-provenance.json)
and [review results](milestone-01-codex-setup/06b-recovery/review.md) retain the
separate cohorts and source/build identity. The Vite chunk-size advisory remains;
packaging succeeded.

Intermediate commands were `node scripts/cargo.mjs test --locked -p workspace-core
--lib runtimes:: -- --test-threads=1` (initial missing-interface compilation
exit 101, later 30 and final 34 runtime units passed), `node scripts/cargo.mjs test
--locked -p workspace-core --lib -- --test-threads=1` (148 units passed before the
last regression), and `node scripts/cargo.mjs clippy --locked -p workspace-core
--all-targets -- -D warnings` (passed). Their logs are retained alongside the
final cohort; intermediate wall times were not captured. Final complete checks
include all nine added Rust units, bringing the total to **287 tests**.

Captured correction logs normalize trailing whitespace and final blank lines; substantive output and outcomes are retained.
