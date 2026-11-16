# Ticket 07 — supervisor deletion and package qualification

Baseline: `423c1fd5323a43a52eda6c82bd7f030145a4871c`, completed ticket 06. Only SQLx ticket 07 is implemented. Ticket 08 and the parent/milestone remain open. No real provider, authentication, agent turn, RAT inspection or existing-user-state reset was performed.

## Final design and coverage

The workspace and macOS app have one production native Tauri/core path. The supervisor crate/binary, application discovery/private sockets/peer negotiation/length-prefix exchange, obsolete host exports, peer-incompatible failure, staging script/resource and retired process harnesses are removed. Current native reply/failure/feed contracts remain generated. Direct typed replies and contextual frontend validation preserve exact incarnation/action/target/revision checks and no automatic replay. Provider framing and strict approval matching remain unchanged.

The lease lives in `state_lease`, independent of any application endpoint. It preserves private directory and regular single-link lease validation, never replaces `owner.lock`, and propagates non-contention lock errors. NativeHost retains startup and whole-core Quit. Old host-only startup forwarding and the obsolete socket validation selector are removed.

[Coverage disposition](coverage.md) and [per-case inventory](retired-assertion-inventory.json) map 90 retired Rust cases plus the old Python qualification cases. Useful assertions now belong to native/lifetime/domain/feed/task units and actual native/storage/process qualification. The pure Rust corpus is retained; only peer-incompatible evidence is retired. New direct-feed tests retain acceptance-before-ack, resync changes, heartbeat, absolute deadlines, exact matching, cancellation, sink failure, bounded replacement, abandoned cleanup and panic collection. Native registration restores the old pre-dispatch UTF-8 path rejection.

Default application units explicitly select the core library, injected tooling and mocked-IPC UI tests. They require no staged sidecar and perform no real application filesystem/SQLite/Git/process effects. Actual storage/process/native commands are opt-in and bounded, separate from preparation, static checks, compilation, complete verification and packaging. Cargo target ownership/symlink safeguards remain enforced.

## Development and packaging

[Private development instructions](../../../development-state.md) define exact overrides and `native-schema-4`, independent of retired binary/protocol hashes. Dev owns a compiled debug executable and Vite directly. Frontend HMR remains; Rust changes require completed Quit and explicit restart. Ordinary stop awaits the retained native child before stopping Vite. Pending cleanup reports after three seconds and never escalates. Repeated stop shares one attempt. Only retained unexited child handles authorize signals; native SIGKILL/nonzero termination is recovery/failure.

The launcher accepts requested Vite service shutdown as signal SIGTERM or numeric 143; native termination never receives that exception. Actual dev qualification caught this distinction, which has failing/passing unit evidence. Normal packaging no longer stages any executable. The generated staging directory was moved aside into a retained disposable location before builds; subsequent successful packages had no access to it.

The normal app contains only its Info.plist and native executable; no provider/bridge executable is bundled. The remaining synthetic runtime is copied into each disposable fixture root, SHA-256 recorded, with Codex 0.169.0 / source `d32063f96c2dff1dc96af839b3952abb6acfeda4` provenance. Fixture traffic is synthetic readiness/login and explicitly excludes thread/turn/approval calls. The source-linked SQLx qualifier uses the current implementation, not another application driver.

## Qualification and limits

`native-q4-final.json` is the final fixture-only package cohort: production command registration/first creation, concurrent creation with inspection, provider readiness/browser login across navigation, actual WebView destruction/recreation with unchanged incarnation/attempt, visible progress-return/cancel, competing same-root rejection, same-inode handover, startup/activity/repeated Quit, native crash/reopen with new incarnation and interrupted/no-replay/survivor fencing, and failed cleanup retaining the actual OS lease. Deliberate SIGKILL is recorded as recovery, never completed Quit. All required ordinary exits are 0.

`current-smoke.json` separately uses the normal package and real dev launcher. It verifies package startup/drain without resources, same-root competitor rejection, identical lease inode on handover, incompatible metadata rejection without changing its version/table/value, and deliberate initialization of a new current-schema root while preserving the incompatible root. Failed incompatible-state cleanup retains the owner until a recorded recovery kill. Dev starts its own Vite/native children with an exact override and exits 0 after native drain. Every repository, worktree, branch and provider home used by these probes is disposable; existing locations remain untouched.

`storage-final.json` records actual SQLx/SQLite opening/read-only enforcement, pinned-reader checkpoint pressure, action/continuation/reservation atomicity, failed commit and controlled SQLITE_FULL retirement, bounded pagination, invalid file opening, close and current-build reopen. Engine: SQLx/sqlx-sqlite 0.9.0, libsqlite3-sys 0.37.0, bundled SQLite 3.51.3. `effects-final.json` separately records synthetic OS-process I/O/cancellation/deadline/descendant cleanup and parent-loss recovery. No alternative production SQLite driver is present.

Unperformed: live provider/authentication/chat, physical disk-full/I/O and power-loss faults, physical rollback/close-ack faults, same-user replacement races (excluded contract), fresh-private-target compilation, full accessibility/user trial and actual JavaScript heap bytes. The fixture observes count retention only; no fixed heap or stream-pressure claim is made. Ticket 08 owns cumulative foundation pressure/handoff. Historical supervisor results remain under their original cohorts.

## Failed and superseded attempts

All commands, source/dirty hashes, exits and wall times are retained in `verification.json` and named logs. TDD red cases establish CI staging removal, package target selection, native input rejection and launcher/Vite exit handling. An initial extraction run rejected an outdated lockfile; static checks then caught unused old imports/completion code. Complete gates stopped early for formatting of the changed check script or newly emitted native evidence, and for a redundant closure after selector removal; these were corrected without weakening checks.

An offline lockfile regeneration unintentionally advanced bumpalo/cc/jiff-tzdb/mio. Their original pins were restored; no dependency upgrade remains. Early package/native/storage/effects runs are retained as a separate transient-lock cohort. Final named cohorts rerun the intended graph and final source. The first normal smoke passed package/reset/ownership but failed launcher reporting because Vite returned 143 after successful native drain; its `*-first` reports/logs remain. The corrected and final cohorts pass.

Complete gates select each retained check once. Failed gates stopped before application units; later full repetitions correspond to actual source/test changes. Packaging remains separate, with no hook-triggered duplicate full suite. All builds used the default real, worktree-claimed Cargo target sequentially; no shared/symlinked target or provider binary substitution was used.

## Independent review

Implementation commits: `dbebd8a` and `5f217e2`. [Fresh-context Standards/Spec reports](review.md) finish with **0 findings on each axis**. Both final reviewers accepted explicit `gpt-6.1-sol` / `high` selectors; engine identity is not independently observable beyond those selectors. The initial Spec P2 and its verified correction remain recorded.

## Final measurements

Environment: Mac16,8 / 12 CPUs / 48 GiB; macOS 26.2 arm64, Rust 1.99.1, Node 26.13.1, pnpm 12.18.0. The default claimed target uses warm test/dev/release caches; compilation and execution are separate. See `environment.json`, `verification.json` and `cargo-tree.log`.

| Gate                      | Exit | Wall seconds |
| ------------------------- | ---- | ------------ |
| `complete-final-current`  | 0    | 22.632       |
| `compile-final-current`   | 0    | 1.936        |
| `package-qualified-final` | 0    | 16.559       |
| `native-q4-final`         | 0    | 10.134       |
| `storage-final`           | 0    | 2.160        |
| `effects-final`           | 0    | 16.424       |
| `package-normal-final`    | 0    | 12.378       |
| `normal-current`          | 0    | 7.950        |

The complete gate passes **172 Rust / 47 tooling / 261 mocked-IPC UI tests**. It includes read-only generated drift, formatting, Oxlint, TypeScript, frontend bundling and workspace all-target/all-feature Clippy; each retained gate is selected once. The later explicit compile gate also passes.

Already compiled Rust units, one warm-up/five warm executions, one harness thread: **median 0.359884 s**, range **0.358982–0.364855 s**, all 172 pass each time. No build overlapped these measurements. No throughput/performance winner is inferred.

Normal package content totals: **26,007,895 → 18,266,727 bytes**, delta **-7,741,168 bytes**. `package-delta.json` lists per-file sizes/hashes. Ticket 06 recorded 48.720 s for its normal package; the final ticket 07 normal package takes 12.378 s in this warm cohort. These differing source/cache/build paths do not isolate a build-speed improvement. Normal and qualification builds are separate: the former excludes all injected WebView/frontend instrumentation.

## Review correction: foreground Ctrl-C

Initial implementation `dbebd8a` received Standards **0 findings** and Spec **1 P2 finding**: foreground terminal Ctrl-C could signal Vite/native directly before launcher-mediated native-first drain. The launcher now creates isolated direct-child process groups and ignores their terminal stdin, while continuing to signal only retained child handles. Bounded Vite readiness accumulation also handles split stdout chunks.

A failing/passing launcher unit locks this policy. `launcher-process.json` separately qualifies actual process groups with controlled synthetic children: terminal-style group SIGINT reaches only the launcher, native receives SIGTERM, service remains alive beyond the three-second pending report, no force escalation occurs, repeated Ctrl-C observes one attempt, and service exits only after native completion. This fixture tests the launcher under OS signals; it is not a Tauri core drain substitute. `ctrlc-smoke.json` reruns the normal package/reset cohort and the actual dev launcher with process-group SIGINT; all ordinary exits are 0. Native Q4's actual pending/failed ownership remains separately qualified.

The final complete gate after this correction selects **172 Rust / 48 tooling / 261 UI tests**. Previous successful full gates describe their respective earlier source cohorts; review correction changes only launcher/tooling/evidence, so the final packaged Rust binary, generated contracts and native/storage/process qualification remain source aligned.

Post-review complete gate: exit 0, 17.815 seconds. The initial Spec finding and correction are preserved in [review.md](review.md).

Ticket 07 is complete. Final implementation and actual qualification pass within the limits above. The final follow-up commit records review/completion evidence only. Stop before ticket 08; no parent spec or milestone completion is claimed.
