# React Compiler setup and verification

2026-11-12 · Frontend tooling and skill update. Baseline
`4fddafac4639028058b36c785668f5faa6d3dab3` plus working changes. Existing
milestone desktop qualification remains open.

## Implementation cohort

macOS 26.2 (25C56), Darwin arm64, Apple M4 Pro; Node 26.13.1, pnpm 12.18.0.
React/React DOM 19.3.1, Vite 8.3.8, React plugin 6.3.0, TypeScript 7.0.2,
Vitest 5.1.0. Versions are pinned in the manifest and lockfile.

Installed with:

```sh
corepack pnpm add --save-dev --save-exact @babel/core@8.1.2 @rolldown/plugin-babel@0.3.1 babel-plugin-react-compiler@1.0.0
```

Exit 0; no peer warnings. Babel 8 supplies its own TypeScript declarations.
The Vite configuration uses `reactCompilerPreset()` through the Babel plugin in
the existing client pipeline. Default inferred compilation applies in development,
production builds, and the existing Vitest configuration. React 19 supplies
`react/compiler-runtime`; no additional production runtime or native compiler
dependency was added. The existing Safari 16.4 target and Tauri asset setup remain.
See [adoption research](../research/react-compiler-adoption.md).

The [React engineering skill](../../.agents/skills/react-engineering/SKILL.md)
defaults to compiler memoization for new calculations/callbacks, allows explicit
memoization for measured or documented needs, preserves existing wrappers pending
focused verification, and separates correctness from cache persistence. Its
[research rationale](../research/react-compiler-memoization-guidance.md) records
the official guidance and exceptions.

## Frontend unit and build checks

Each command ran once, sequentially in the existing worktree, with installed
dependencies and existing caches. Elapsed command times below are reported by the
execution tool; this is a correctness check, not a comparative benchmark. No Cargo
target or Rust harness was used.

| Check                          | Result                                                                     | Elapsed                         |
| ------------------------------ | -------------------------------------------------------------------------- | ------------------------------- |
| `corepack pnpm test:ui`        | Exit 0; 1 file, 3 mocked-IPC presentation units pass; maximum workers 1    | 0.876 s; Vitest reports 0.770 s |
| `corepack pnpm frontend:build` | Exit 0; TypeScript check and minified production build pass; 2,251 modules | 0.577 s; Vite reports 0.372 s   |

The existing units cover explicit observation retry, pending/stale feedback, and
remounts retrieving status without product actions. They use substituted IPC
outcomes and do not run the supervisor or native providers.

Production assets: `index-DcNlVMhu.js` is 328.59 kB (106.50 kB gzip); CSS is
41.39 kB (8.40 kB gzip). Output inspection found generated cache-slot access in
`ConnectionDetails` and the local UI components. The presence of React's runtime
sentinel alone would not establish that application code was compiled.

## Compiler coverage inspection

An in-memory diagnostic transform inspected the two components in `src/App.tsx`:

```sh
node --input-type=module <<'JS'
import { transformFileAsync } from '@babel/core';
import compiler from 'babel-plugin-react-compiler';
const events = [];
await transformFileAsync('src/App.tsx', {
  babelrc: false,
  configFile: false,
  parserOpts: { plugins: ['jsx', 'typescript'] },
  plugins: [[compiler, { logger: { logEvent(filename, event) {
    events.push({filename, event});
  } } }]],
});
console.log(JSON.stringify(events, null, 2));
JS
```

Exit 0, 0.064 s. `ConnectionDetails`: `CompileSuccess`, 24 memo slots, 9 memo
blocks, 9 memo values. `App`: skipped with a recoverable `CompileError`, category
`Todo`, reason `(BuildHIR::lowerStatement) Handle TryStatement with a finalizer
('finally') clause`, referring to its existing refresh handler. The production
output corroborates this partial coverage: the root remains unoptimized and the
details component contains generated caches. This is an unsupported construct,
not a failing application build or evidence of a product defect. No application
refactor was made to force compilation.

## Source identity and qualification limits

SHA-256 values at the successful build:

```text
0fb031f987dcbbcb7c126d537400508da14c06fdf9acc0cd0be1a0dd2a78fccf  package.json
96a9a4d1691769ded56c598d1674a23c040d4388b1e11bb936b416ab4e1d0824  pnpm-lock.yaml
af2a15206b4e54854f67efe9129ecf85c74209dacc41bfaeefa4dc32d2120d60  vite.config.ts
4f7de44dd009beb0ab14c98777b56394732064f18c672c06bbfa656ba48a4266  src/App.tsx
290beb0bce491453606144a9775a84ad66f8e738ded74ece650e2118866213dc  dist/assets/index-DcNlVMhu.js
```

Frontend units and build pass. Complete Rust/tooling verification, desktop
packaging, fixture-desktop qualification, native-provider checks, and performance
comparisons were not run for this change. No latency improvement or full component
coverage is claimed. Native provider allowances and the deferred desktop gate are
unchanged.

Final documentation/tooling checks: targeted Prettier checking passed for
`package.json`, `vite.config.ts`, the skill, this evidence, and the memoization
research. Skill Creator's `quick_validate.py` passed using isolated `uv` PyYAML;
all 26 local links checked across the skill and three related documents resolved.
`git diff --check` passed. All commands exited 0. Application and build inputs match
the successful build cohort above.
