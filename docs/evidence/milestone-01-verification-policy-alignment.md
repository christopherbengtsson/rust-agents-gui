# Milestone 01 — current verification policy alignment

2026-11-12 · Tickets 01–02 were implemented before `docs/agents/verification.md`
was introduced in `7fbaaa9`. Their implementation and completion evidence are
retained under the requirements that applied then. This record aligns the current
repository with the current policy; it does not retroactively fail those tickets.

Source baseline: `7fbaaa9ae8925df2aa7f1489c319d2b45ffb2a0b`, plus the uncommitted
sidecar fix and policy-alignment changes. Concurrent user edits to AGENTS.md,
the Rust-engineering skill and its research were preserved. Applied the new
Rust-engineering guidance when it appeared during this work. The product
milestone remains incomplete.

Also consulted the subsequently added [design-principles reference](https://rust-unofficial.github.io/patterns/additional_resources/design-principles.html)
as tradeoffs. Dependency seams are private and specific to discovery, owner
admission and framed I/O; decisions remain in those concrete core modules.

## Current commands and boundaries

- `corepack pnpm test` explicitly compiles only `workspace-core --lib` units,
  selects Cargo's reported test executable, then runs that executable and the
  tooling units. Compilation and execution are separately timed. Rust harness
  threads and Node test concurrency are both 1. No staged sidecar is required.
- `corepack pnpm test:measure` compiles once and runs each unit harness once for
  warm-up and five times for warm measurements, reporting median and range.
- `corepack pnpm check` runs TypeScript and Clippy, including all features/targets
  so the retained qualification code is checked. The previous duplicate Cargo
  check pass was removed; standalone compilation remains `corepack pnpm compile`.
- `corepack pnpm verify` runs formatting, static checks and units once each.
  Packaging (`corepack pnpm build`) and actual process qualification
  (`corepack pnpm qualify:process`) remain separately selected commands.
- Inherited supervisor fixtures require the `process-qualification` feature.
  They remain in place with their existing assertions; the default unit gate
  cannot discover or execute them. No new integration tests were added.
- Small private Rust dependency seams supply metadata, lock, process, transport
  and clock outcomes. Real discovery/admission/reconnect decisions, command
  construction, framing, serialization and version matching stay under test in
  their owning core modules. Tauri still forwards observations.
- Build, unit and static Cargo entry points reject symlinked target paths and
  targets claimed by another canonical worktree. An exclusive ownership-marker
  write establishes each target's claim. Relative target selections are forwarded
  to build/desktop tools as the absolute Cargo-reported target directory. No
  shared mutable target was introduced.
- Sidecar staging retains Cargo-reported executable selection and provenance at
  `src-tauri/gen/sidecar`. The earlier [sidecar-fix results](milestone-01-sidecar-review-fix.md)
  describe that earlier verification snapshot; the results here describe the
  subsequent policy-alignment snapshot.
- No existing tracked hook or CI scripts were found. The pre-existing local
  `core.hooksPath=.githooks` setting was preserved; it currently names a missing
  directory. No replacement hooks configuration was installed. Application Git
  execution and workspace recovery do not exist yet and remain later-ticket work.

## Old-to-new coverage and qualification

Unit replacements passed before the inherited process fixtures were excluded from
the default command. The two former `crates/core/tests/status.rs` checks were
moved into private library units with supplied owner identity/PID. The six actual
process checks remain available separately; no OS qualification assertion was
removed.

| Retained check                                          | Current deterministic coverage                                                                                                                                                                                           | Separate qualification retained                                                         |
| ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------- |
| Initial status and snapshot cursor/reset                | `tests::initial_status_reports_the_execution_owner_without_starting_work`; `tests::snapshot_catches_up_or_resets_observations_from_another_owner`                                                                        | Status reported by the actual supervisor process                                        |
| Startup/status exchange                                 | `connection::tests::status_exchange_and_version_rejection_use_real_protocol_parsers`; `spawn_command_preserves_executable_and_single_state_argument`                                                                     | `startup::desktop_client_receives_status_from_a_separate_supervisor_process`            |
| Concurrent desktops, competing owner and reconnect      | `existing_owner_is_discovered_without_lock_probe_or_start`; `absent_owner_starts_once_and_waits_for_readiness`; `concurrent_owner_startup_is_waited_for_without_starting_a_competitor`; supervisor admission/probe tests | `discovery::concurrent_desktops_discover_one_owner_and_reconnect_to_its_snapshot`       |
| Private endpoint, version mismatch and oversized frames | Privacy/lock evidence validation; real frame/parser/version tests; absolute frame deadline with controlled time                                                                                                          | `discovery::private_endpoint_rejects_incompatible_versions_and_unbounded_frames`        |
| Public/symlinked state, no owner start                  | `unsafe_state_or_incompatible_endpoint_never_starts_an_owner`; privacy and state-creation tests                                                                                                                          | `discovery::public_or_symlinked_state_is_rejected_without_starting_an_owner`            |
| Killed desktop, same owner on reopen                    | `dropping_desktop_and_reopening_observes_the_same_owner_without_starting`; `refresh_after_disconnect_retrieves_cursor_without_restart_or_replay`                                                                         | `discovery::killed_desktop_process_leaves_owner_and_reopening_catches_up`               |
| Lost owner, refresh versus explicit reopen              | `lost_owner_never_restarts_on_refresh_but_reopen_can_admit_a_new_owner`; controlled timeout/failure tests                                                                                                                | `discovery::lost_owner_does_not_restart_on_refresh_but_explicit_reopen_recovers_status` |

The 24 Rust units also cover permission/lock/spawn failures, every admission effect
failure, stale socket handling and malformed/incomplete/output-too-large frames.
The 12 tooling units cover sidecar artifact selection/provenance and private-target
ownership/symlink rejection, with command/filesystem outcomes substituted.

Unit success does not establish actual OS access restrictions, exclusion,
process survival, disk durability, desktop responsiveness or provider compatibility.
Those remain qualification claims. No fixture/native claim was promoted to a unit
claim, and no native allowance was inherited.

## Unit measurements

Same hardware cohort as the [bootstrap evidence](milestone-01-bootstrap.md):
MacBook Pro Mac16,8, Apple M4 Pro, 48 GB RAM; macOS 26.2 (25C56), arm64;
Node 26.13.1, pnpm 12.18.0, Cargo/Rust 1.99.1. Dependency lockfiles unchanged.
The sidecar-fix run retained the sandbox-denied hardware `sysctl` query; hardware
here comes from retained evidence, not a new measurement.

A harness temporarily renamed `src-tauri/gen/sidecar`, ran
`env CARGO_TARGET_DIR=/private/tmp/rag-policy-unit-7fbaaa9-target corepack pnpm test:measure`
and restored the directory in `finally`. This private target was fresh, with warm
download caches, test/dev profile and harness threads/concurrency 1. No supervisor,
runtime, real Git operation, application filesystem or production-length wait was
used by the unit tests. Build/harness filesystem operations remain outside tests.

- Whole measurement command: exit **0**, **4.679526 s** wall.
- Fresh-private-target unit compilation: **3.697502 s**, reported separately from
  execution. No sidecar staging or Tauri compilation occurred.
- Rust: **24 passed**, 0 failed, on all six executions. Warm-up **0.306635 s**;
  warm runs **0.004587, 0.003968, 0.003532, 0.003367, 0.003550 s**. Median
  **0.003550 s**; range **0.003367–0.004587 s**. Executable:
  `/private/tmp/rag-policy-unit-7fbaaa9-target/debug/deps/workspace_core-847732502cf0fdc3 --test-threads=1`.
- Node: **12 passed**, 0 failed, on all six executions. Warm-up **0.101574 s**;
  warm runs **0.089485, 0.091010, 0.089669, 0.090387, 0.090547 s**. Median
  **0.090387 s**; range **0.089485–0.091010 s**. Command:
  `node --test --test-concurrency=1 scripts/cargo.test.mjs scripts/prepare-supervisor.test.mjs`.

Measured Rust executable SHA-256:
`c00bfa6be538e3fbc0ac8834d78bea1e13a4feb74f229d610d9af716362bbc62`.
These measurements preceded the final test-module ordering cleanup and the Cargo
CLI wrapper's output/exit forwarding cleanup. Test bodies and production decisions
were unchanged by those cleanups; the final source passed the gate below.

## Actual-process qualification

Command: `/usr/bin/time -p corepack pnpm qualify:process`, using the existing
worktree-default target/test profile and serial harness threads. Cargo selects the
current test executables; fixtures use their compiled `CARGO_BIN_EXE` supervisor
path and disposable private state, rather than the staged sidecar.

- Sandboxed attempt: exit **1** (underlying Cargo **101**), **21.20 s** wall;
  discovery **1 passed, 4 failed, 1 helper ignored**, with supervisor-startup
  timeouts; startup harness was not reached. Retained as a failed environment
  check, not discarded or reported as a proven product defect.
- Rerun with sandbox escalation: exit **0**, **1.01 s** wall; Cargo compilation
  **0.03 s**, discovery **5 passed, 1 helper ignored**, startup **1 passed**.
  The ignored helper is explicitly exercised by its parent desktop-kill check.
  This passed with the same source/binaries once sandbox restrictions were lifted.

No actual desktop or native-provider workflow was rerun. No provider submission,
authentication change or RAT inspection occurred. Original screenshots and
completed milestone evidence remain retained. Ticket 03's verification gate is
now implemented; its repository-registration behavior and remaining milestones
remain open.

## Final gate, packaging and source snapshot

- First `corepack pnpm verify`: exit **101**, **1.26 s** wall. Formatting passed;
  Clippy rejected items after the library test module. Moved private test modules
  to the end of their source files. No production policy changed. Units were not
  reached in this failed gate.
- Final `/usr/bin/time -p corepack pnpm verify`: exit **0**, **4.58 s** wall,
  existing default target with warm caches, dev/test profiles. Formatting passed;
  TypeScript and all-target/all-feature Clippy passed (**3.17 s** Cargo-reported
  checking time). Units: **24 Rust + 12 tooling passed**, 0 failed. Unit
  compilation **0.274024 s**; direct Rust execution **0.333102 s**, Node execution
  **0.110871 s**. These single executions are reported separately from the warm
  five-run measurement above. Each retained check ran once in the successful gate.
- `/usr/bin/time -p corepack pnpm build`: exit **0**, **8.28 s** wall, existing
  default target with warm release cache. Supervisor compilation **0.58 s**,
  desktop compilation **6.83 s**; frontend and packaging passed. App bundle
  **10.29 MiB**. Standalone `corepack pnpm compile` was not separately run; the
  gate's Clippy compilation and the separate release build passed.
- A Node assertion harness compared the Cargo release executable, staged binary,
  bundled resource and provenance SHA-256. All matched:
  `be9206c6b185bde3ea2bda5a465617a0bacb3cdc73f27e4e461cc2f40c4bba62`.
  Provenance records source revision `7fbaaa9ae8925df2aa7f1489c319d2b45ffb2a0b`,
  dirty **true**, default target directory, and release/Cargo profile details.
- `git diff --check`: passed. Historical evidence files and the user's concurrent
  Rust-guidance edits were preserved.

Final implementation snapshot (SHA-256; dependency lockfiles unchanged):

```text
a6017e17cdb2902f3481fac72717dcdd1455f5f74a3edfaff578e087b2f739ba  crates/core/src/lib.rs
d3e2b7cd79576ea4cd1639b0f94b1e3e3620205cbff4ac16fda9a26802cd6961  crates/core/src/connection.rs
5286ba1c167b742b0632c4c6ab19560fbc143bd089ff0d4219e8af7af784420e  crates/core/src/connection/tests.rs
05c35760ce52ae8b799cd96ab30543a078fb5ccc6eaaacef5f9acb7235f79adb  crates/core/src/supervisor.rs
8df0fe6d5d08f57b485383c9be145fd63f906bd27b2c58a2aae6f97e6abbd330  crates/core/src/supervisor/tests.rs
ca8f75d797384f2f5051164a9a0929d816405d1ca349371ea8efeb2bd725af5e  crates/supervisor/Cargo.toml
69a078850e13068c62f540b491cbeec7d27248e2f31dbd9917043bdc693cb815  scripts/build.mjs
9c3b6bb02b17b0a551846e0f61a07399a2e1cc674de6708050c3073ee20e0d16  scripts/cargo.mjs
cc8bb14627114dc52293f8d3ed1fd6be872552f1f904ff78c1d7a88fcd4286e4  scripts/cargo.test.mjs
4ee1dea06017d985d5c1486202c5187d60f16e408d71cec28423227d74214ab8  scripts/prepare-supervisor.mjs
52f092eed55f3df11d7f532e0a42812f0565bfcd51ecd5ffa5811d1c4a5aaa65  scripts/prepare-supervisor.test.mjs
26827dd1e6407a61481ca8a42fa1a75a89c0fb3a85ffb6dbf4146864dde8e9d2  scripts/start.mjs
d9cf75103acc23bb2bcbf53ec3be4d3d0ea567ed2fe86d73e95e785c7389e8fd  scripts/test-unit.mjs
cce7396776684541dae9efd6b30a118f04b88a9bd58e065dd0db364b8a5213aa  package.json
ff45325061c347604f13fc0155a8d6a10f57b54f8c0d96d4be6be78001afd8dc  src-tauri/tauri.conf.json
```
