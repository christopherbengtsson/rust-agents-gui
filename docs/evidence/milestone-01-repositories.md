# Ticket 03 — repository registration and restored navigation

2026-11-13. Implementation and actual desktop qualification on
`feat/milestone-01-repository-registration`, starting at
`6ded0f24e7cd26f8b8dc877ccb51e69c707dce1d`. Whole Milestone 01 remains incomplete.
This ticket performs no provider submissions.

## Implementation and source

`workspaces` owns durable action admission, canonical shared-Git-directory identity,
availability, bounded navigation and read-only recovery. `storage` owns SQLite
transactions outside project files. One bounded worker releases observation locks
while validating through the shared read-only Git helper. Tauri forwards typed
protocol-2 requests. The React observation owner retains action IDs and retrieves
status after uncertainty; reconnect never resubmits registration.

Registered locations are the first selected canonical checkout. Linked worktrees
share identity; independent clones stay distinct. Unavailable records remain saved
and selectable. Protocol frames are limited to 64 KiB; pages have at most 32 rows
and a 48 KiB byte budget. Admission allows eight pending registrations; storage
has no repository-count restriction. Availability caches/probe queues are bounded
to 64 entries.

SQLite uses schema version 1, private database files, rollback journaling,
`synchronous=FULL` and macOS `fullfsync=ON`. A committed metadata transaction is
the registration success boundary. Commit failure is reconciled against committed
action status; failed transaction cleanup fences uncommitted reads. This cohort
does not establish power-loss durability.

Official base-nova Field, Input Group, Spinner and Collapsible were added through
shadcn CLI 4.23.0, preserving existing Button/Input/Separator. Repository selection
and disclosure are independent. Expanded details show path and availability;
selection survives collapse and reopening. The native picker only fills the path;
Register is explicit. Add repository sits above supervisor status.

Research: [identity](../research/conductor-repository-identity.md),
[components](../research/ticket-03-shadcn-components.md),
[sidebar reference](../research/ticket-03-conductor-sidebar.md).

## Environment and verification

macOS 26.2 (25C56), arm64 Mac16,8 / Apple M4 Pro, 48 GiB. Rust 1.99.1,
Node 26.13.1, pnpm 12.18.0, Git 2.50.0. Existing worktree-default `target`,
warm dependency/build caches, serial Rust/tooling harnesses and one Vitest worker.
The initial focused frontend checks used the shell's Node 24.21.0; final checks
and packaging explicitly use the pinned Node through `fnm exec --using 26.13.1`.
[Environment](milestone-01-repositories/environment.txt)

Focused red/green checks exercised workspace decisions and the mocked-IPC UI.
The initial Clippy run failed on single-character string append and test-module
placement; both were corrected. The regression for preserving a causal storage
failure first failed, then passed after the fix. No default unit uses real Git,
SQLite, supervisor processes or production waits.

| Gate                                      | Result                                                                  |
| ----------------------------------------- | ----------------------------------------------------------------------- |
| Complete formatting/lint/static/unit gate | Pass: 51 Rust, 16 tooling, 22 UI units; exit 0; 5.15 s wall             |
| Warm execution measurement                | Pass: one warm-up and five warm executions per harness; 13.92 s wall    |
| Retained process qualification            | Pass: six scenarios, one ignored subprocess helper; exit 0; 2.53 s wall |
| Production packaging                      | Pass: default production configuration; exit 0; 35.76 s wall            |
| Live provider / native agent cohort       | Not run; not authorized or needed for ticket 03                         |

The existing `core.hooksPath=.githooks` configuration was preserved; that directory
is absent in this checkout. No new hook or duplicate hook verification was added.

Exact commands (each prefixed with `/usr/bin/time -p`):

```sh
fnm exec --using 26.13.1 corepack pnpm verify
fnm exec --using 26.13.1 corepack pnpm test:measure
fnm exec --using 26.13.1 corepack pnpm qualify:process
fnm exec --using 26.13.1 corepack pnpm build
```

The final complete gate compiled/located the warm core unit harness in 0.110651 s.
Execution was measured separately: Rust 0.017896 s, tooling 0.471550 s, UI 2.064037 s.
Measurement-only compilation was 0.078220 s. Warm execution medians/ranges:

| Harness           | Median     | Range               |
| ----------------- | ---------- | ------------------- |
| Rust, 51 units    | 0.017454 s | 0.017247–0.017566 s |
| Tooling, 16 units | 0.465731 s | 0.464542–0.478508 s |
| UI, 22 units      | 1.797247 s | 1.791572–1.815286 s |

Fresh-private-target compilation was not measured. These are harness timings,
not desktop navigation or provider latency measurements.
[Complete gate](milestone-01-repositories/verify.log),
[measurement](milestone-01-repositories/unit-measurements.log),
[process qualification](milestone-01-repositories/process-qualification.log),
[production packaging](milestone-01-repositories/production-packaging.log)

The first complete-gate attempt failed after 50/51 Rust units: an unused optional
repository field was serialized as null in status requests. The existing exact-wire
test caught this; the optional field now stays omitted when unused. Its focused
check and the complete gate then passed. Tooling/UI execution was not reached in
that failed attempt. [Failed gate](milestone-01-repositories/verify-initial-failed.log),
[earlier static failure](milestone-01-repositories/static-initial.log)

## Actual desktop cohort

Used the actual packaged Tauri/WebKit app, production core/supervisor path,
disposable checkouts and mode-700 `/private/tmp/rag-03-m7m5hrys/state`.
No mocked IPC was used for these desktop observations.

An isolated build configuration used a separate bundle identifier. Adding its
optional `dataStoreIdentifier` first failed inside Tauri's context code generation
(`Vec<u8>` versus `[u8; 16]`). Removing that optional field allowed packaging.
Both results are retained. Native CUA initially could not resolve the changed
identifier at the existing bundle path. A private copy named Registration
Qualification with identifier `dev.rag.registration`, changed display metadata,
and ad-hoc signatures made discovery unambiguous. Application code and assets
came from the packaged build; full original/copied binary hashes are retained,
including possible signature differences. WebKit used a separate bundle-specific
data directory. [Configuration](milestone-01-repositories/qualification-config.json),
[failed packaging](milestone-01-repositories/packaging-failed-datastore.log),
[successful packaging](milestone-01-repositories/packaging.log),
[copy context](milestone-01-repositories/desktop-copy-launch.json)

| Scenario                    | Observed result and evidence                                                                                                                                                                                                                                                                                                                                         |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Initial empty navigation    | Connected, no repositories, working Add repository. [01](milestone-01-repositories/01-empty.png)                                                                                                                                                                                                                                                                     |
| Dirty checkout registration | Alpha registered; original checkout retained dirty tracked/untracked/deleted files. [02](milestone-01-repositories/02-alpha-registered.png)                                                                                                                                                                                                                          |
| Independent disclosure      | Closing Alpha details retained Alpha main-panel selection. [03](milestone-01-repositories/03-collapsed-selection-retained.png)                                                                                                                                                                                                                                       |
| Two repositories            | Beta, an unborn Git repository, registered separately. [04](milestone-01-repositories/04-two-repositories.png)                                                                                                                                                                                                                                                       |
| Linked worktree             | Registering linked-alpha selected the existing Alpha identity and first location; still two entries. [05](milestone-01-repositories/05-linked-worktree-deduplicated.png)                                                                                                                                                                                             |
| Invalid folder              | Plain folder produced an actionable Git error and no repository entry. [06](milestone-01-repositories/06-invalid-folder.png)                                                                                                                                                                                                                                         |
| Desktop crash/reopen        | Desktop SIGKILL/reopen restored Beta selection and two entries; supervisor 19828 remained the owner. [07](milestone-01-repositories/07-desktop-crash-restored-selection.png)                                                                                                                                                                                         |
| Unavailable location        | Renaming Beta externally, then Refresh, retained its identity/selection and showed recovery guidance. Restoring the directory and Refresh recovered availability. [08](milestone-01-repositories/08-unavailable-retained.png), [09](milestone-01-repositories/09-availability-recovered.png)                                                                         |
| Storage failure             | State-directory mode 500 blocked intent with a visible SQLite readonly error and retained action ID. Restoring mode 700 and explicit Retry registered the alias without another repository identity. [10](milestone-01-repositories/10-storage-failure.png), [11](milestone-01-repositories/11-storage-explicit-recovery.png)                                        |
| Minimum layout and keyboard | Resized to 600×480 logical pixels. Controls and footer remained usable. Tab/Enter opened diagnostics; Escape returned focus to its trigger. [12](milestone-01-repositories/12-minimum-window.png), [13](milestone-01-repositories/13-diagnostics-keyboard.png)                                                                                                       |
| Native picker               | Opened the native directory chooser. Cancellation retained the draft; choosing fixture Alpha filled the absolute path without submitting. Explicit Register then reused its identity. Initial Go to Folder automation attempts were unsuccessful; a later separate-step attempt completed. [14](milestone-01-repositories/14-native-picker-selected.png)             |
| Supervisor restart          | Terminated only owned qualification desktop/supervisor, then reopened. New owner 23194 restored both durable IDs and Alpha selection; six action records remained, including the failed validation, with no automatic resubmission. [15](milestone-01-repositories/15-supervisor-restart-restored.png), [16](milestone-01-repositories/16-new-owner-diagnostics.png) |

All 59 fixture files, including Git metadata, have identical hashes and modes
before and after the cohort. External location/permission faults were restored.
[Comparison](milestone-01-repositories/project-comparison.json),
[fixture setup](milestone-01-repositories/fixture-setup.json),
[restored records](milestone-01-repositories/restored-records.json)

Owned qualification processes were stopped after observation; disposable state was
retained. This is test-process cleanup, not qualification of ticket 09's product
Quit workflow. [Cleanup](milestone-01-repositories/cleanup.json)

## Final production-build follow-up

After the full cohort, status serialization was corrected as described above,
internal development-stage wording was removed from the empty-state copy, and the
registration dialog received the established viewport-height/scroll guard. The
default production package was rebuilt with the final source. A metadata-isolated,
ad-hoc signed copy was launched against the retained private state. It restored
both original repository IDs and Alpha selection. At 600×480, the registration
dialog remained usable; explicit Beta registration selected its original identity.
There were seven action records afterwards: the original six plus this explicit
smoke action, with no automatically replayed action.
[Restored navigation](milestone-01-repositories/17-final-production-restored.png),
[dialog](milestone-01-repositories/18-final-minimum-dialog.png),
[registration](milestone-01-repositories/19-final-registration.png),
[smoke results](milestone-01-repositories/final-smoke.json)

The 59 fixture files still matched exactly. Final source hashes, production and
qualification-copy hashes, and Cargo sidecar provenance identify the cohort.
Only bundle metadata/signatures differ between the production package and its
private UI-discovery copy. Owned processes were stopped; the private copy was
unregistered and the production bundle's LaunchServices registration restored.
[Source](milestone-01-repositories/final-source-manifest.json),
[bundle hashes](milestone-01-repositories/final-bundle-hashes.json),
[provenance](milestone-01-repositories/final-sidecar-provenance.json),
[final cleanup](milestone-01-repositories/final-cleanup.json)

## Review and qualification limits

The requested code-review workflow independently reviewed Standards and Spec
against the starting revision, including a focused follow-up for causal-error
retention and protocol-fixture changes. Both axes reported zero findings. Reviews
were read-only and ran no checks or provider commands.

Unit outcomes establish application decisions; the separately observed desktop
cohort establishes this ticket's workflow on this Mac. Independent clones, capacity
boundaries and pending-intent recovery have deterministic unit coverage; bulk
real-repository creation was not used for capacity tests. No claim is made for
oldest-supported macOS, power-loss survival, VoiceOver output, navigation paint
latency below 100 ms, history-opening latency, provider compatibility or remaining
milestone capabilities. Earlier milestone evidence remains historical.

## Independent-review follow-up — 2026-11-13

Three fresh independent React, shadcn and Rust reviews at `0e45b9b` found two
P2 defects: an oversized registration could remain permanently pending in the UI,
and an unbroken repository name could push Refresh outside the window. Rust had
no additional findings. Both fixes are scoped to those findings.

`workspaces` now shares its existing 4096-UTF-8-byte registration-path check with
the connection client. The client rejects oversized requests as typed `capacity`
before state access, connection or send. Authoritative admission retains the same
check. Existing UI rejection handling clears the pending preference and restores
editing; an already retained oversized request recovers through explicit Retry.
Actual transport errors remain uncertain and retain their action IDs. Header and
footer names now truncate while preserving full accessible text.

Controlled Rust coverage checks the exact byte boundary, multibyte paths, a path
larger than the 64 KiB frame limit, absence of transport on rejection, and preserved
uncertainty for transport failures. Two mocked-IPC UI cases cover fresh/restored
oversized requests, explicit retry identity and corrected submission with a new
action ID. Existing lost-reply recovery coverage remains passing.

Same hardware/toolchain as the original cohort: macOS 26.2, Apple M4 Pro, Rust
1.99.1, pinned Node 26.13.1 and pnpm 12.18.0; warm worktree-default Cargo target,
test profile, serial Rust/tooling harnesses and one Vitest worker. Source identity
is the dirty follow-up to `0e45b9b`; [source hashes](milestone-01-repositories/review-fixes-source-manifest.json)
identify all six changed product/test files.

| Check                           | Result                                                                                                                                                                    |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Focused Rust registration units | 4 passed; exit 0; 1.39 s wall                                                                                                                                             |
| Focused repository UI units     | 15 passed; exit 0; 1.91 s wall                                                                                                                                            |
| Complete `pnpm verify`          | 54 Rust + 16 tooling + 24 UI = 94 passed; exit 0; 6.95 s wall                                                                                                             |
| Warm execution measurement      | One warm-up and five warm executions per harness; exit 0; 15.85 s wall                                                                                                    |
| Browser layout/keyboard fixture | 600×480, 200-character unbroken name: heading/footer truncate, full heading remains accessible, no horizontal overflow, Refresh visible and Tab/Enter activation observed |

Exact commands, each timed with `/usr/bin/time -p`:

```sh
fnm exec --using 26.13.1 node scripts/cargo.mjs test --locked -p workspace-core --lib registration -- --test-threads=1
fnm exec --using 26.13.1 corepack pnpm test:ui src/repositories.test.tsx
fnm exec --using 26.13.1 corepack pnpm verify
fnm exec --using 26.13.1 corepack pnpm test:measure
```

Complete-gate unit compilation/location took 0.121350 s; execution took Rust
0.020873 s, tooling 0.517919 s and UI 2.133355 s. Measurement-only compilation
took 0.083486 s. Warm medians/ranges: Rust 0.020422 s / 0.020152–0.020614 s;
tooling 0.507602 s / 0.505733–0.513162 s; UI 2.023842 s / 1.997409–2.393328 s.
Fresh-private-target compilation was not measured.

The initial focused Rust compile failed because the new test compared a request
enum lacking `PartialEq`; the assertion now compares its targeted registration
payload. That failure is retained alongside passing [Rust](milestone-01-repositories/review-fixes-rust.log),
[UI](milestone-01-repositories/review-fixes-ui.log),
[complete verification](milestone-01-repositories/review-fixes-verify.log),
[measurement](milestone-01-repositories/review-fixes-measure.log) and
[initial compile failure](milestone-01-repositories/review-fixes-rust-initial-failed.log).

The [browser fixture](milestone-01-repositories/review-fixes-layout-harness.txt)
served actual current React/CSS through Vite with mocked IPC and a fixed-size
iframe; native Dia UI supplied Tab/Enter. [Screenshot](milestone-01-repositories/20-review-fixes-long-name.png)
and [observations](milestone-01-repositories/review-fixes-layout.json) retain the
result. Browser-host discovery was unavailable, so the native app surface was
used; its bundle ID was ambiguous until resolved to `/Applications/Dia.app`.
The disposable server was stopped afterwards.

This follow-up did not rerun production packaging, actual Tauri/process
qualification, VoiceOver, or live-provider checks. The browser fixture establishes
layout and keyboard behavior in Dia, not a new native desktop qualification.
Earlier desktop/process evidence remains historical. No provider was started and
RAT was not inspected.
