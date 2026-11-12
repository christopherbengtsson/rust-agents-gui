# Frontend agent conventions — 2026-11-13

## Change and environment

Source baseline: `75bc36faa080e54c6a6ed543819d3d1e2c0a27fa` plus the working-tree
convention/lint changes. Apple M4 Pro, arm64, macOS 26.2; Node 26.13.1,
pnpm 12.18.0. Package versions are pinned in package.json and pnpm-lock.yaml.
Existing default Cargo target: `target/`, warm cache, dev profile for Clippy and
test profile for units. No staged sidecar required by unit execution. No runtime
logic or component relocation changed; two narrow inline lint exceptions preserve
existing Base UI composition and post-mount breakpoint initialization.

## Checks

| Command/check                                                                                                                                            | Outcome                                                                                                                             |
| -------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `corepack pnpm exec eslint src --max-warnings 0` (initial)                                                                                               | Failed: Base UI-injected heading content and upstream post-mount setState. Both reviewed and narrowly documented inline.            |
| `node --test scripts/frontend-lint.test.mjs`                                                                                                             | Passed: 4 configuration behavior tests.                                                                                             |
| `corepack pnpm peers check`                                                                                                                              | Passed: no peer dependency issues after compatible package selection.                                                               |
| `corepack pnpm verify`                                                                                                                                   | Passed, exit 0: Prettier/rustfmt, ESLint, TypeScript/Clippy, then unit execution. Each gate ran once in this complete verification. |
| Rust units within verify                                                                                                                                 | 33 passed; one harness thread. Compilation 0.105981 s; compiled executable execution 0.003963 s.                                    |
| Tooling units within verify                                                                                                                              | 16 passed; Node test concurrency 1. Execution 0.449396 s. Includes 4 lint configuration tests.                                      |
| Mocked-IPC UI units within verify                                                                                                                        | 8 passed; Vitest maxWorkers 1. Execution 1.253522 s.                                                                                |
| Skill validator (initial system Python)                                                                                                                  | Failed before validation: missing PyYAML.                                                                                           |
| `uv run --with pyyaml python /Users/christopherbengtsson/.codex/skills/.system/skill-creator/scripts/quick_validate.py .agents/skills/react-engineering` | Passed in isolated dependency environment.                                                                                          |
| `git diff --check`                                                                                                                                       | Passed.                                                                                                                             |

The times above are single executions, not warm-run medians or performance claims.
Complete gate wall time was not measured. Fresh-private-target compilation,
packaging, process/fixture-desktop qualification, actual desktop accessibility,
screen-reader checks and native-provider qualification were not run. Existing
milestone evidence and incomplete acceptance status remain unchanged.

## Instruction scenario review

A manual walkthrough checked the instruction path from AGENTS.md through the
component guide and React skill. This evaluates discoverability and decision
coverage; it is not an independent agent behavioral test.

| Request                       | Required decision and completion evidence                                                                                                    |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| Choose a control              | Inspect existing product primitives first; use upstream UI directly when its contract fits. No redundant wrapper needed.                     |
| Add a compact shared action   | Put its concrete shared customization in primitives; document why/when/alternative and variant consumers. Verify composition props/refs.     |
| Compose repository navigation | Use a coherent product block, with parent-owned placement and explicit observations/intents. Keep execution policy in Rust.                  |
| Add a status color or variant | Reuse a semantic role/variant or introduce the missing shared contract in the existing theme/primitive; review consumers and contrast.       |
| Update a customized component | Preview CLI differences, account for recorded local deviations and consumers, and verify affected primitives/blocks without blind overwrite. |

## Limits and compatibility

Babel syntax parsing preserves the TypeScript 7 baseline; tsc remains the type
checker. ESLint 9 satisfies the accessibility plugin's peer range. The package is
marked deprecated upstream; the research note records the reason for this pin and
when to revisit it. Static accessibility lint does not establish contrast,
keyboard behavior, screen-reader output, or Base UI composition correctness.
Tauri import restrictions cover direct static/dynamic imports and re-exports,
including type imports. Review is still required for indirect IPC access,
architectural layer selection, visual quality and variant usefulness.
