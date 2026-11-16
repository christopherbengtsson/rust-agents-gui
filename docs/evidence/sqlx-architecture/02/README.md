# Ticket 02 — SQLx storage and settlement

2026-11-16. Scope: ticket 02 only. Baseline `f6ac1ddd7c8aba50ad643cb4d45b93777b512887`; integration branch `sqlx-storage-cutover`. Ticket 01 evidence was inspected and remains unchanged. Tickets 03/04, combined Q2 completion, merge, native host, authentication and live-provider actions remain excluded.

## Implementation and handoff

The source-linked `storage-checkpoint` package compiles the actual private production storage module and mechanically extracted domain records. It contains no alternate driver, storage implementation or public product storage API. SQLx 0.9.0 replaces rusqlite in the workspace graph. Existing workspace/provider consumers intentionally do not compile until tickets 03/04 convert them; no bridge, runtime-per-call or dual implementation was added.

Core startup must retain `Storage::start`'s opening receipt while holding the state lease. Cancelling its wait does not cancel startup/partial cleanup. Share the resulting handle across domains. `write`/`read` perform immediate bounded admission and return retained completion receipts. `Delivery::revision` reports the committed point; `into_records` transfers a decoded page and its capacity permit together. Dropping a receiver does not abandon admitted work. Close fences admission, drains all admitted jobs, observes checkpoint results and both pool barriers, and retains a repeatable report. Any unresolved cleanup/fault remains evidence for core/domain fencing and lease retention; pool closure cannot erase it.

Writes are typed for current registration, initial creation, workspace/reservation/cursor and existing provider records. Domain owners retain retry, credential-scope and external-effect decisions. Provider qualification below only writes synthetic non-secret records; it invokes no provider, authentication or configuration effect.

Failed statement/commit/rollback scope cleanup precedes explicit acquired-connection retirement. A retirement guard blocks establishment/reuse until acknowledgment and fences unknown/panicked retirement. Return hooks order queued cleanup with ping before checking tracked transaction depth, retain hook failures and reject unhealthy connections. Read admission captures a finite settlement frontier (exact action for action lookups); later writes cannot extend it. Known retirement permits fresh reconciliation snapshots; a storage health fault conservatively fences new writes for that owner. Neither absence nor error authorizes replay.

Fresh metadata is `metadata.sqlite3`, application ID `0x41575332`, schema version 1. Retired `workspace.sqlite3` or incompatible metadata produces fresh-state guidance. No reset occurs. To reset deliberately: stop and drain the owning build, retain owner exclusion, select a new canonical private state root and fresh managed locations, and preserve repositories/worktrees, runtime credentials/native history and evidence. Treat stopped database/WAL/SHM as one set. No migration/import/old-worktree takeover is supplied.

Opening validates canonical root, owner/mode and regular single-link database/journal/WAL/SHM files initially and through establishment/reuse hooks. Application-created metadata uses private mode and native NOFOLLOW. SQLx opening is not atomic NOFOLLOW: same-user path replacement between validation and native open is explicitly outside the accepted contract. No umask changes, VFS, raw-handle adoption or driver fork is used.

## Settings, bounds and engine

Required: WAL, FULL synchronization, macOS fullfsync and foreign keys. Provisional: one writer, two read-only/no-create readers, one-second pool acquisition, 100 ms busy timeout, 1,000-page autocheckpoint, no timed churn. Reader connections do not request a journal-mode transition or immutable mode.

Storage admits at most 64 writes and eight read interests before task creation. Serialized write envelopes are capped at 64 KiB (4 MiB across admitted writes); reads accept at most 4 KiB of text. Short snapshots cap materialized records at 32 rows/48 KiB, return committed revision/keyset continuation and explicitly reject oversized data. One buffered driver row, eight worker commands and 16 cached statements per connection bound additional driver copies. SQL projects oversized record bodies to NULL before driver materialization. Decoding/SQL bind copies are additional bounded allocations, not included in the 4 MiB serialized-envelope count; this is not a hard process/RSS claim. Completed undelivered and decoded results retain their admission permits. Health faults fence writes; repeated read failures are bounded by the retained fault admission threshold.

Resolved engine: `libsqlite3-sys 0.37.0`, bundled SQLite **3.51.3**, source ID `2026-03-13 10:38:09 737ae4a34738ffa0c3ff7f9bb18df914dd1cad163f28fd6b6e114a344fe6d618`. The runtime probe matches the bundled header. [SQLite 3.51.3 release notes](https://www.sqlite.org/releaselog/3_51_3.html) identify the WAL-reset fix and the same source ID. SQLx published source revision is `003b698e99e024f3621b8043a2426fde5b741171`; local published source was inspected for borrowed transactions, acquired close, return hooks, worker ping and options. Only `sqlite-bundled` and `runtime-tokio` are selected with SQLx defaults disabled. No optional SQL macros/offline metadata, load-extension, deserialize or unlock-notify feature is requested.

## Coverage and verification

Installed Intent discovery succeeded; listed TanStack skills do not match this Rust concern. Used implement, rust-engineering and the agreed TDD seams under repository substitution policy. Unit checks perform no real SQLite/filesystem/process work. Actual SQLx/disk probes use new disposable private roots and a held qualification lease.

| Former assertion                                                               | Replacement / retained boundary                                                                                                                           |
| ------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Commit required; failed transaction cannot silently reuse an active connection | Retirement/uncertainty, return-ping ordering and replacement-fence units; separate real deferred-FK COMMIT fault                                          |
| Commit/rollback error evidence survives cleanup                                | Both original and retirement errors retained; requester loss/panic/known-versus-unknown settlement units                                                  |
| Current records preserve recovery facts                                        | Source-linked current record round-trip, corrupt-record rejection and actual same-build reopen                                                            |
| Old schema/label migration                                                     | Deleted migration ladder, legacy label helpers and migration-only test portion; no compatibility replacement required                                     |
| Numeric catalogue variants                                                     | Existing catalogue-exhaustion assertion retained independently of removed migration test                                                                  |
| Workspace/provider intent and effect policies                                  | Original domain units retained in source; temporarily unbuildable until 03/04 adapt consumers                                                             |
| Admission, reads and close                                                     | Result-capacity, bounded encoding, file-validation and collect-all close units; actual acquisition, keyset pagination and pinned-reader checkpoint probes |

Intermediate records: `policy-red.log` reproduces unsafe reuse; `policy-green.log` passes its replacement. `units-first.log`, `units-return-guard.log`, `units-retirement.log`, `units-guard.log`, `units-close.log` retain successive focused results. Initial compile attempts exposed source-link module paths, SQLx 0.9 static SQL safety, a private qualification helper and lint findings; corrected checks are retained separately. `workspace-check.log` additionally caught missing macro scope during extraction; `workspace-check-second.log` records the corrected consumer gap.

Actual disk attempts: `qualification-first.log` did not compile because the settings helper was private; `qualification-second.log` rejected its incorrectly targeted commit-fault fixture. The deferred-FK fixture fixed that harness error. `qualification-third.log` passed the original cohort; `qualification-fourth.log` also passed atomic continuation/reservations, provider record deduplication and byte/keyset pagination. These are checkpoint results; final source verification/review is recorded below.

No physical disk-full/I/O fault, physical rollback/close acknowledgment fault, kill-point/power-loss, native host, desktop or live-provider qualification is claimed. Unperformed physical faults leave their claimed qualification open, not silently passed. Q2 remains open through ticket 04.

## Final review and verification

Both initial reviewers ran in fresh contexts with explicit `model: gpt-6.1-sol` and `reasoning_effort: high`; both settings were accepted. Fixed diff: `git diff f6ac1ddd7c8aba50ad643cb4d45b93777b512887...HEAD`, starting implementation commit `c85add8`.

### Standards

Initial report: one P2 violation — status reads waited for the live global write set to empty and could starve under continuous writes. The fix captures bounded per-operation settlement sequences at read admission, so later writes (including duplicate action IDs) cannot extend the barrier. `frontier-red.log` reproduces the defect; `review-regressions-green.log` passes the replacement. One discretionary numeric validation-tag smell was addressed with a named, exhaustive record-shape enum. No tooling findings were counted as review findings.

### Spec

Initial report: one P2 finding — opening-hook validation/unknown-retirement evidence could be discarded behind an acquisition timeout after partial-open cleanup. Startup now returns the acquisition error and retained structured hook faults/uncertainty together. `startup-fault-red.log` reproduces the loss; `review-regressions-green.log` passes the replacement. No scope creep was found. A harness failure/deadline is a failed probe; it does not claim a successful clean drain.

Final-source checks and follow-up review are recorded after the corrective checkpoint. Intermediate workspace compilation and combined cutover remain explicitly open.
