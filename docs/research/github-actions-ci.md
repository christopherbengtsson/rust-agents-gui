# Basic GitHub Actions CI

Researched 2026-11-13 for the native Oxc migration branch. The repository had no
existing workflow. Scope is verification and frontend asset compilation, with no
packaging, desktop/process qualification, credentials or provider submissions.

## Actions and runner

Official latest release pages were rechecked before implementation:

| Action                                                                          | Release | Pinned release commit                      |
| ------------------------------------------------------------------------------- | ------- | ------------------------------------------ |
| [actions/checkout](https://github.com/actions/checkout/releases/tag/v7.0.1)     | v7.0.1  | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| [actions/setup-node](https://github.com/actions/setup-node/releases/tag/v7.1.0) | v7.1.0  | `949feb2413d6458794dcd2491c4babbbce0c15c1` |

Immutable pins retain version comments for future updates. Checkout credentials
are not persisted and the token has only contents-read permission.

The [GitHub runner reference](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
lists `macos-15` as a standard arm64 M1 runner for public and private repositories.
This follows the product's Apple Silicon scope without maintaining a platform
matrix. The [runner image inventory](https://github.com/actions/runner-images/blob/main/images/macos/macos-15-arm64-Readme.md)
includes Xcode tooling and rustup. Its moving Rust version is not authoritative:
`rustup show active-toolchain` selects/installs the repository toolchain, including
components declared by `rust-toolchain.toml`.

Setup-node reads `.node-version`, with automatic package-manager caching disabled.
[Corepack](https://github.com/nodejs/corepack) is installed explicitly at 0.36.0,
then `corepack pnpm` reads the version from `package.json`. No global pnpm or shim
activation is necessary. Neither setup-node's action-runtime Node version nor the
runner's preinstalled Node replaces the repository's selected Node version.

## Repository command selection

A fresh checkout's Tauri static checks require the sidecar resource. The existing
preparation script builds the debug supervisor and records provenance without
launching it. This precedes the single `corepack pnpm verify` gate.

The final `corepack pnpm exec vite build` verifies asset compilation without
repeating TypeScript checking from `verify`. No caches are introduced initially;
particularly, Cargo's worktree-ownership marker must not be restored from another
checkout. Superseded jobs are cancelled per workflow and PR/branch, with a
20-minute job timeout. No branch-protection or repository settings are changed.

Hosted success is not established by local validation. No workflow has been
pushed, dispatched or observed running on GitHub as part of this change.

## Local validation

Implementation validated on 2026-11-13, macOS 26.2 arm64, Apple M4 Pro,
Node 26.13.1 and Rust 1.99.1. This is a local macOS cohort, not the hosted macOS 15
image. Corepack 0.36.0 was installed under a disposable `/tmp` prefix, with a
private Corepack cache; no global package installation changed the user's setup.
Its first `corepack pnpm --version` downloaded the macOS arm64 pnpm 12.18.0 binary.
The package's Corepack compatibility wrapper supports this bootstrap.
[Corepack engine requirements](https://github.com/nodejs/corepack/blob/v0.36.0/package.json),
[pnpm's published package](https://registry.npmjs.org/pnpm/12.18.0).

Commands executed sequentially using the selected Node and temporary Corepack:

| Command                                                       | Result                                                                                                       |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `corepack pnpm install --frozen-lockfile`                     | Passed; package and lockfile pins unchanged.                                                                 |
| `rustup show active-toolchain`                                | Selected repository Rust 1.99.1.                                                                             |
| `node scripts/prepare-supervisor.mjs debug`                   | Passed; built/staged resource without launching it.                                                          |
| `corepack pnpm verify`                                        | Passed: formatting, native lint, TypeScript/Clippy, 33 Rust units, 19 tooling checks, 12 presentation tests. |
| `corepack pnpm exec vite build`                               | Passed; 2,344 modules transformed.                                                                           |
| `actionlint -shellcheck= -pyflakes= .github/workflows/ci.yml` | Passed with actionlint v1.7.12.                                                                              |
| `git diff --check`                                            | Passed.                                                                                                      |

The actionlint binary was downloaded from its official release into temporary
storage and checked against the published checksum. Its YAML, expression and
workflow checks ran; optional external ShellCheck/Pyflakes checks were disabled.
[Actionlint release](https://github.com/rhysd/actionlint/releases/tag/v1.7.12).
Standards and Spec reviews found no issues. No additional permanent tests or test
framework were introduced. These are correctness runs, not performance benchmarks.
No hosted workflow run, packaging, desktop/process qualification, native-provider
submission or repository-setting change was performed for this CI addition.
