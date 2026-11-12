# Milestone 01 — sidecar staging review fix

2026-11-12 · Addresses the P2 sidecar staging finding from the tickets 01–02
review against `main` at `7fbaaa9ae8925df2aa7f1489c319d2b45ffb2a0b`.
The working tree was dirty with this fix during verification. Earlier milestone
qualification remains retained; the milestone is still incomplete.

## Change and source

Preparation selects exactly one non-test supervisor executable from Cargo's
`compiler-artifact` messages, matching its package ID and binary target. Missing
or ambiguous artifacts and build failure stop preparation before staging.
This uses Cargo's documented [artifact messages](https://doc.rust-lang.org/cargo/reference/external-tools.html#artifact-messages)
and [metadata output](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html).

The executable is staged under `src-tauri/gen/sidecar` alongside
`provenance.json`: Git revision, dirty state, Cargo target directory, selected
profile, Cargo profile details, original executable path and staged SHA-256.
The Tauri resource now consumes that staged file. Development startup uses it;
release startup locates the bundle using Cargo's target directory and uses its
bundled supervisor by default. This supports private `CARGO_TARGET_DIR` builds
without copying or launching an old default-target supervisor.

Verified implementation files (SHA-256):

```text
ed7ddd1504946039fae6c68473c5080f4bcd75595c18af8f786460ba42078fa0  scripts/cargo.mjs
b3c23a2dcf3479631d0dd59e2ca726bfa3af8b50047a6e02494fb1ef998cf440  scripts/prepare-supervisor.mjs
f667d6e16787b292edc71f93091d8b491ebb9cd0b6fa9364f6671a17ad429178  scripts/prepare-supervisor.test.mjs
2a0db5fe5ab439c032c233008d892735c0132d5fd3356ab467c34391e4d55dc2  scripts/start.mjs
ff45325061c347604f13fc0155a8d6a10f57b54f8c0d96d4be6be78001afd8dc  src-tauri/tauri.conf.json
```

## Environment and unit results

Same Mac/toolchain cohort as the [bootstrap evidence](milestone-01-bootstrap.md):
MacBook Pro Mac16,8, M4 Pro, 48 GB RAM (retained hardware record); this run confirmed
macOS 26.2 (25C56), arm64, Node 26.13.1, Cargo 1.99.1 and Rust 1.99.1. A fresh
hardware `sysctl` query was denied by the sandbox; no new hardware measurement is
claimed. Lockfiles and dependency versions are unchanged.

Command: `node --test --test-concurrency=1 scripts/prepare-supervisor.test.mjs`.
Six unit tests passed, zero failed; commands, Git, filesystem reads, copying and
writes were substituted. Real artifact selection, build command construction and
provenance/hash policy were retained. Coverage: private-target selection over a
foreign/stale artifact, cached artifact acceptance, dirty/clean source, release/dev
profiles, missing/test/ambiguous executables, and failed build without staging.
These checks require neither compiled Rust artifacts nor a staged sidecar.

An initial untimed run passed. Then a Python `subprocess.run`/`perf_counter`
harness measured one warm-up and five warm runs of the exact command above,
sequentially, each with exit 0 and six passing tests. Warm-up: **0.066308 s**.
Warm runs: **0.058719, 0.060586, 0.062312, 0.062327, 0.061705 s**.
Median: **0.061705 s**; range: **0.058719–0.062327 s**. These are Node unit
execution wall times; no Cargo compilation is included.

## Compilation, packaging and static checks

Commands ran sequentially in the source worktree. Timed commands used
`/usr/bin/time -p`; download caches were warm. All listed checks exited 0 unless
explicitly stated otherwise.

- `env CARGO_TARGET_DIR=/private/tmp/rag-sidecar-review-7fbaaa9-target node scripts/prepare-supervisor.mjs debug`:
  **3.76 s** wall, fresh private target, dev profile. Cargo compilation itself
  reported **3.66 s**. This compiled the supervisor but did not execute it.
- A `node --input-type=module` assertion harness confirmed the recorded private
  target/executable paths, source revision, dirty flag, dev profile, byte equality
  between Cargo's executable and the staged file, and matching recorded SHA-256:
  `7faa591993d5620decedcea815c855088d475213736d55e0af20e29dd4f4ecd4`.
- `corepack pnpm build`: **9.20 s** wall, existing worktree-default target and
  warm release cache. Supervisor compilation **0.64 s**, desktop compilation
  **7.54 s**; TypeScript/Vite frontend and macOS app packaging passed. The app
  bundle was **10.30 MiB**. Preparation replaced the earlier debug staging with
  the default-target release artifact and corresponding provenance.
- A second `node --input-type=module` assertion harness confirmed byte equality
  among Cargo's release executable, staged binary and bundled
  `Contents/Resources/workspace-supervisor`; their recorded SHA-256 matched:
  `4de498d6d15d719e8f0440cc598acaf8bf34ada4fa2cf5cb672b1b8d54920920`.
- `corepack pnpm check`: **1.97 s** wall. TypeScript, locked workspace Cargo
  checking and Clippy on all targets with warnings denied passed using the
  existing default target/dev cache. This compiled/checked fixtures but did not
  execute them.
- `node --check` on each of `scripts/cargo.mjs`,
  `scripts/prepare-supervisor.mjs`, `scripts/prepare-supervisor.test.mjs` and
  `scripts/start.mjs`: passed.
- `corepack pnpm format:check`: **0.30 s** wall; Prettier and rustfmt passed.
- `git diff --check`: passed.

## Qualification limits

Unit checks establish staging decisions with supplied dependency outcomes; actual
private-target compilation and default-target bundle inspection separately
establish artifact selection/copying for these runs. Full private-target desktop
packaging, development/release desktop launch, inherited process fixtures, full
workspace test execution and complete milestone verification were not rerun.
No actual desktop workflow or native-provider qualification was performed; no
supervisor/runtime was launched and no provider submission or authentication
change occurred. The ticket 03 unit-gate work remains open.
