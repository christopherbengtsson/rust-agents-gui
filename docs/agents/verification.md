# Testing and verification

## Authority

The user owns requirements and engineering policy. [MVP.md](../MVP.md) owns
product behavior and module responsibilities; milestone documents own acceptance
and qualification. This file owns testing policy and engineering-skill constraints.
Use supported-version official documentation for API and platform facts. Its
examples and third-party skills do not create project requirements.

## Unit tests

Test application decisions and responses to success, failure, and uncertainty with
fast deterministic unit tests. Substitute dependency outcomes, including
application-owned storage, filesystem access, Git and process execution. Choose
the smallest useful observation surface independently of which effects execute.
Private Rust functions and owning-module interfaces are valid test surfaces.

Use typed, scenario-specific doubles and controlled events/time. Retain real
application policy, parsers and command construction under test. Assert effect
identity, absence, count or ordering when it expresses a requirement such as
intent-before-effect or no duplicate send. Keep recovery decisions in their owning
Rust modules and Tauri presentation thin. A double supplies evidence; it does not
implement the policy being tested.

Unit tests run without real Git/worktrees, SQLite, supervisor/runtime processes or
production-length waits. Add no integration tests or generic mocking framework.
Tauri's mock runtime and frontend IPC mocks replace Tauri facilities, not these
application dependencies. Neither `--lib` nor a move into `src` removes effects.
Replace expensive checks only after their useful assertions have passing unit
replacements; record the old-to-new coverage mapping and any lost qualification.

## Tooling configuration checks

The native Oxc migration explicitly permits bounded real Oxlint CLI checks in the
existing Node tooling gate. They exercise the checked-in configuration against
temporary source fixtures, with a timeout and cleanup. This exception covers no
application, Git, supervisor or provider processes and adds no testing framework.
Application behavior units retain the dependency-substitution policy above.

## Engineering skills

Use skills under the [repository precedence rule](../../AGENTS.md#instruction-precedence).
Give delegated implementers and reviewers the relevant repository pointers before
work begins, including this file when they select tests or verification commands.

## Qualification

Mocked outcomes establish application decisions, not actual disk durability, OS
access control/owner exclusion, descendant termination, desktop responsiveness or
native-provider compatibility. Keep those limits explicit in evidence. Actual
desktop qualification and the separately authorized native cohort remain milestone
requirements; unit success does not complete them or authorize provider submissions.
Use disposable repositories/private application state for those qualification runs.

## Commands and measurements

Read the checked-out package scripts for available commands and inspect their
behavior before selecting a verification gate.
Establish an isolated unit entry point with ticket 03. Default automated behavior
verification must select unit tests explicitly, excluding inherited process
fixtures; retain baseline qualification separately. Unit execution must need no
staged sidecar. Keep static checking, compilation, unit execution, complete
verification and packaging separately identifiable; run each retained check once
per complete gate.

Record source, versions, hardware, target/profile/cache state, harness threads,
exact commands, counts, exit status and wall times. Time compiled unit executables
separately from Cargo compilation. Use one warm-up and five warm execution runs,
reporting median/range; label fresh-private-target compilation separately. Record
complete verification and packaging independently, including unperformed checks.

## Worktrees

Use each worktree's default Cargo target or a private `CARGO_TARGET_DIR`. The build
scripts must reject symlinked targets and target directories claimed by another
worktree when these safeguards are introduced. Run one verification/build command
at a time in each worktree. Share download caches rather than mutable target
directories.

Sidecar preparation must consume Cargo's reported executable and record source
revision, dirty state, target directory, profile, and binary hash beside the staged
resource under `src-tauri/gen/sidecar`. When a process result contradicts current
source, inspect that provenance and the test executable before diagnosing product
behavior. Retain failed checks and distinguish build contamination from a proven
runtime defect.

Keep Git execution behind a shared core helper and recovery decisions in
`workspaces`.

Use existing hook/CI configuration when present, preserving any different local
hooks configuration. Keep checks proportional and avoid duplicate full-suite runs
in hooks and complete verification.
