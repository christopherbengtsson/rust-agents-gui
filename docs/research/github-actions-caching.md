# GitHub Actions caching

Researched 2026-11-13 against the checked-out CI workflow, repository verification
policy, and primary sources. This note proposes configurations; it does not modify
the workflow or establish a measured cache benefit.

## Recommendation

Prioritize a compiled Rust dependency cache plus Cargo downloads through
`actions/cache`. Restore explicit dependency directories into each fresh hosted
job's private target, excluding its ownership marker by construction. Defer pnpm
store caching until evidence justifies its overhead. This recommendation treats
an immutable CI archive restored into a fresh VM as distinct from the live mutable
target sharing prohibited between local worktrees; the ownership guard remains
unchanged.
The repository sources are [verification policy](../agents/verification.md),
[Cargo wrapper](../../scripts/cargo.mjs), and
[supervisor preparation](../../scripts/prepare-supervisor.mjs).

The parent investigation observed successful hosted run 43342525574 at revision
`9861fca`: dependency installation took approximately 9 seconds, resource
preparation 15 seconds, and verification 88 seconds, including approximately 77
seconds of Clippy compilation. These observations prioritize Rust compilation
over pnpm download caching, but one uncached run cannot predict a cached saving.
This researcher did not independently run that workflow or benchmark caches.
The parent confirmed that the
[hosted CI run](https://github.com/christopherbengtsson/rust-agents-gui/actions/runs/43342525574)
was green, with approximately 147 seconds total wall time.

## Action versions

Official latest-release API responses and tag references were checked on the
research date. Resolve annotated tags to their commit before pinning.

| Action                                                                            | Release | Immutable commit                           |
| --------------------------------------------------------------------------------- | ------- | ------------------------------------------ |
| [actions/cache](https://github.com/actions/cache/releases/tag/v6.1.0)             | v6.1.0  | `55cc8345863c7cc4c66a329aec7e433d2d1c52a9` |
| [Swatinem/rust-cache](https://github.com/Swatinem/rust-cache/releases/tag/v2.9.3) | v2.9.3  | `d1527c8f67ef9af357290475d4af4b0fab8f12e0` |

Both action manifests use Node 24 as their action runtime, independently of the
repository's Node 26.13.1 selection. Sources:
[cache manifest](https://github.com/actions/cache/blob/v6.1.0/action.yml),
[rust-cache manifest](https://github.com/Swatinem/rust-cache/blob/v2.9.3/action.yml).

## Cargo download cache

The Cargo Book recommends retaining registry index data, downloaded crate
archives, and bare Git repositories without also retaining unpacked sources and
Git checkouts. The latter can be recreated. Its broader example includes
installed Cargo binaries, which this workflow does not need to cache.
[Cargo home and CI caching](https://doc.rust-lang.org/cargo/guide/cargo-home.html).

For downloads alone, add this after checkout and before the first Cargo command.
For the recommended combined cache, use the later configuration instead:

```yaml
- name: Cache Cargo downloads
  uses: actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9 # v6.1.0
  with:
    path: |
      ~/.cargo/registry/index
      ~/.cargo/registry/cache
      ~/.cargo/git/db
    key: cargo-downloads-v1-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('Cargo.lock') }}
    restore-keys: |
      cargo-downloads-v1-${{ runner.os }}-${{ runner.arch }}-
```

These paths assume the workflow's current default `CARGO_HOME`; adjust them if
that environment changes. `--locked` remains on existing Cargo commands. The
cache reduces dependency fetching, not Rust compilation or rustup installation.
The explicit paths exclude target ownership, staged sidecars, credentials, and
the Rust toolchain. Cache keys permit download reuse across lockfile changes.
The primary cache action saves on successful jobs and leaves existing exact-key
entries unchanged; restoration also searches the configured prefixes.
[Cache inputs and behavior](https://github.com/actions/cache/blob/v6.1.0/README.md),
[save implementation](https://github.com/actions/cache/blob/v6.1.0/src/saveImpl.ts).

## Compiled cache and ownership

`scripts/cargo.mjs` binds `target/.agent-workspace-owner.json` to the checkout's
real path. It rejects a target reached through a symlink or claimed by a different
checkout. A cached marker from an identical hosted checkout path might happen to
match; that does not make it portable to a different checkout location. Keep the
claim check intact, and never repair a restored mismatch by deleting the marker.
[Repository implementation](../../scripts/cargo.mjs).

The standard macOS runner creates a new VM per job. Restoring an immutable cache
snapshot into its private filesystem differs operationally from mounting one
mutable target into concurrent local worktrees. The recommended configuration
uses this isolation and leaves the repository's ownership check unchanged.
[Hosted runner isolation](https://docs.github.com/en/actions/reference/runners/github-hosted-runners).

For a compiled snapshot, cache only `target/debug/deps`,
`target/debug/.fingerprint`, and `target/debug/build` alongside Cargo downloads.
The ownership marker is outside the selected paths,
and the existing wrapper creates the current checkout's marker before compiling.
Cargo places the dev and test profiles in `debug` when no explicit target triple
is selected; the checked-in commands use that layout. Revisit the paths when
changing profiles, targets, Cargo build-directory configuration, or toolchains.
[Cargo build-cache layout](https://doc.rust-lang.org/cargo/reference/build-cache.html).

An illustrative configuration, after selecting Rust and before resource
preparation, is:

```yaml
- name: Cache Rust dependencies
  uses: actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9 # v6.1.0
  with:
    path: |
      ~/.cargo/registry/index
      ~/.cargo/registry/cache
      ~/.cargo/git/db
      target/debug/deps
      target/debug/.fingerprint
      target/debug/build
    key: rust-deps-v1-${{ runner.os }}-${{ runner.arch }}-macos-15-${{ hashFiles('rust-toolchain.toml', 'Cargo.lock', '**/Cargo.toml', '.cargo/config.toml', '**/.cargo/config.toml') }}
```

This intentionally has no broad restore prefix. The key must also incorporate
any future Rust flags, Cargo configuration, or compiler/SDK choices affecting the
build. A stable runner label does not freeze its installed SDK image. Cargo still
decides which restored artifacts are reusable. Always run resource preparation
again so the staged executable and its provenance are derived from this job's
Cargo report. Do not cache `src-tauri/gen/sidecar` or skip verification on a hit.
These paths omit incremental artifacts and top-level binaries; workspace artifacts
inside `deps` can still be included. Larger archives and restore/upload time can
offset the benefit. No speedup is
asserted without a cold/warm hosted comparison including post-job upload time.

## Why rust-cache is not a drop-in choice

The action defaults to caching workspace targets, Cargo registry/Git data, and
Cargo binaries. Its keys account for runner OS/architecture, compiler identity,
Rust-related environment, manifests, lockfiles, and toolchain configuration. It
normally keeps dependency artifacts rather than workspace crates. Select the
repository toolchain first.
[Versioned defaults](https://github.com/Swatinem/rust-cache/blob/v2.9.3/README.md),
[key and path construction](https://github.com/Swatinem/rust-cache/blob/v2.9.3/src/config.ts).

Its cleanup has a material interaction with this repository: `cleanTargetDir`
deletes target-root files except `CACHEDIR.TAG`, including the ownership marker.
The save path calls this cleanup before upload and catches errors. Consequently,
a normal cleaned archive omits the marker, but the omission relies on successful
cleanup. Cleanup runs even with `cache-targets: 'false'`, so that input avoids
target archival without preventing target mutation.
[Cleanup source](https://github.com/Swatinem/rust-cache/blob/v2.9.3/src/cleanup.ts),
[save source](https://github.com/Swatinem/rust-cache/blob/v2.9.3/src/save.ts).

`cache-targets: 'false'`, `cache-bin: 'false'`, and
`cache-directories` listing only those three dependency directories would omit the root marker from the
archive while retaining compiled dependencies. It still performs the marker
cleanup locally in the post action. Direct `actions/cache` avoids depending on
this third-party cleanup behavior and does not change target ownership files.
The Rust action also disables incremental compilation through
`CARGO_INCREMENTAL=0` during restore.
[Restore source](https://github.com/Swatinem/rust-cache/blob/v2.9.3/src/restore.ts).

## pnpm with the existing Corepack bootstrap

Keep the current setup-node step's `package-manager-cache: false`. Adding
`cache: pnpm` there would execute `pnpm store path --silent`, before this workflow
installs Corepack and without the plain `pnpm` shim the action expects. Adding a
second setup-node invocation or enabling shims solely for caching adds machinery.
[Pinned setup-node cache implementation](https://github.com/actions/setup-node/blob/v7.1.0/src/cache-utils.ts),
[Corepack manual installation and invocation](https://github.com/nodejs/corepack/blob/v0.36.0/README.md).

If pnpm caching proves useful, discover its actual store after Corepack
installation and use the generic action:

```yaml
- name: Locate pnpm store
  id: pnpm-store
  run: echo "path=$(corepack pnpm store path --silent)" >> "$GITHUB_OUTPUT"

- name: Cache pnpm store
  uses: actions/cache@55cc8345863c7cc4c66a329aec7e433d2d1c52a9 # v6.1.0
  with:
    path: ${{ steps.pnpm-store.outputs.path }}
    key: pnpm-store-v1-${{ runner.os }}-${{ runner.arch }}-${{ hashFiles('package.json', 'pnpm-lock.yaml', 'pnpm-workspace.yaml') }}
    restore-keys: |
      pnpm-store-v1-${{ runner.os }}-${{ runner.arch }}-

- name: Install dependencies
  run: corepack pnpm install --frozen-lockfile
```

The discovery command follows pnpm's own path selection rather than assuming a
versioned native-store subdirectory. pnpm 12.18.0's published package selects
platform executable dependencies, including Darwin arm64; Corepack remains the
existing invocation mechanism. This is separate from caching Corepack's own
download of pnpm.
[Store path command](https://pnpm.io/cli/store#path),
[published pinned package metadata](https://registry.npmjs.org/pnpm/12.18.0).

pnpm also exposes `corepack pnpm cache path` for its metadata cache. Its current CI
documentation recommends considering this in addition to the package store, but
explicitly says store caching may fail to improve installation time. The observed
9-second dependency step makes this optional after the larger Rust opportunity.
Current pnpm documentation recommends native installation over Corepack; changing
the repository's established bootstrap is a separate decision and is unnecessary
for the generic cache configuration above.
[pnpm CI guidance](https://pnpm.io/continuous-integration),
[metadata cache command](https://pnpm.io/cli/cache).

Use GitHub's cache branch isolation and ordinary pull-request events; do not
introduce a privileged untrusted-code workflow or a writable shared host store.
pnpm's store integrity setting does not turn an attacker-writable store into a
trusted one. These observations concern cache scope, without adding a new
application approval or qualification requirement.
[pnpm store trust model](https://pnpm.io/settings/store#storedir),
[GitHub cache access restrictions](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows#restrictions-for-accessing-a-cache).

## Verification limits

This work inspected sources and official release metadata only. No package was
installed, no workflow was changed or dispatched, no provider was invoked, and no
cache speed, restore portability, archive size, or hosted warm-run behavior was
measured. Any implementation should retain existing locked install/build gates,
check workflow syntax, then compare cold and warm hosted runs including cache
restoration and post-job upload costs.

## Implementation follow-up

Implemented the combined six-directory Rust cache in the existing CI workflow.
The environment fingerprint includes actual compiler identity, Xcode version,
SDK path/version, runner image identity when available, and sorted Rust/Cargo
environment flags. The key also hashes Cargo manifests/lockfile, toolchain and
legacy/TOML Cargo configuration, and the workflow itself. It has no restore prefix.
The effective Cargo home is resolved rather than assumed. Neither source changes
nor documentation edits alone invalidate otherwise compatible dependency caches.

A post-restore assertion rejects regular or symlink ownership markers, including
dangling links, without deleting them. Existing resource preparation still claims
the target and stages Cargo's reported executable with fresh provenance. All
checks execute on cache hits. pnpm caching remains deferred.

Local Node 26.13.1 verification passed: formatting, lint, TypeScript/Clippy,
33 Rust units, 19 tooling checks and 12 presentation tests. Actionlint v1.7.12
passed. A disposable shell check confirmed absent markers pass and regular-file
and dangling-link markers fail without deletion. The full environment fingerprint
could not run locally because this Mac uses Command Line Tools without full
Xcode; its `xcodebuild -version` requirement will be exercised on the hosted macOS
image. No fallback identity was substituted. This is an environment limitation,
not a local cache-hit qualification.

Hosted cache miss/save and exact-hit results are pending. No performance saving
is claimed. Standards and Spec reviews cover this scoped implementation before
publication; runtime cache behavior requires the hosted runs.
