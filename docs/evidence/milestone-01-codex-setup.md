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
before effects. A repeated admitted check returns status and never respawns.

One runtime-owned worker and a capacity-one channel serialize persistence and
process work. The admission limit includes in-flight work; competing mutations
reject Busy. Shared-state locks only copy/admit/publish observations; storage,
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
| Outstanding runtime work, including in flight                | 1                                                         |
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
and a durable launch fence; shutdown reports failure. Destructor cleanup remains
a last-resort wait, not a successful bounded-shutdown claim.

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
