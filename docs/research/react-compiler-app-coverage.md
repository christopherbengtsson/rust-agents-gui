# React Compiler coverage of the application shell

Subsequent source change: the [shell boundary review](../evidence/milestone-01-ui-foundation.md#shell-boundary-review--2026-11-12)
extracts connection observation and diagnostics for responsibility boundaries.
`App` now compiles; the observation Hook retains the finalizer bailout. The
analysis below describes the earlier source and its qualified desktop cohort.

2026-11-12 · Focused research against official documentation, the installed
compiler, and retained qualification evidence. Supported local cohort:
React/React DOM 19.3.1, `babel-plugin-react-compiler` 1.0.0, Babel 8.1.2,
`@rolldown/plugin-babel` 0.3.1, `@vitejs/plugin-react` 6.3.0, Vite 8.3.8,
TypeScript 7.0.2, Tauri JS/CLI 2.12.4 and Rust 2.12.5. Package versions come from
the [manifest](../../package.json) and [desktop cohort](../evidence/milestone-01-ui-foundation.md#cohort-and-retained-artifacts).
Current React documentation identifies itself as v19.3; implementation claims
below are pinned to the installed compiler release rather than upstream `main`.

## Recommendation

Keep the existing `App` implementation and its documented partial compiler
coverage for ticket 02a. Its `try/catch/finally` is valid application cleanup that
Compiler 1.0.0 cannot lower. The bailout removes automatic optimization from this
component; it does not establish an application correctness defect or a failed
build. React explicitly supports gradually increasing coverage. No retained
measurement identifies this empty shell as a rendering bottleneck, and the
[ticket](../../.scratch/milestone-01/issues/02a-establish-incremental-ui-foundation.md)
does not require every component to compile. These are application conclusions
from the evidence below, consistent with the repository's
[measure-before-optimizing guidance](../../.agents/skills/react-engineering/SKILL.md).

If a later working screen makes connection synchronization obscure the shell,
extract that coherent capability while preserving its lifecycle and `finally`.
Confirm the resulting compiler output; do not promise that extraction alone
optimizes every function. A measured rendering problem would also justify a
targeted change. Neither is necessary to close 02a, whose
[desktop qualification is already complete](../evidence/milestone-01-ui-foundation.md#limits-and-closure).

## What the bailout means

The retained [diagnostic transform](../evidence/react-compiler-adoption.md#compiler-coverage-inspection)
reported a recoverable `CompileError` for `App`, category `Todo`, at the nested
refresh handler's finalizer. `ConnectionDetails` reported `CompileSuccess` with
24 memo slots, and production output also contained caches in local UI
components. This is per-component partial coverage, not disabled compilation
throughout the application.

The published npm metadata for
[`babel-plugin-react-compiler` 1.0.0](https://registry.npmjs.org/babel-plugin-react-compiler/1.0.0)
identifies source commit `5c15c1cd3442f28044217703b5625f98c40042ee`.
At that revision, `BuildHIR.ts` explicitly records a `Todo` when a `TryStatement`
has a finalizer. The installed distribution contains the same check at
`dist/index.js:79222`. This is evidence of an unimplemented compiler construct;
the diagnostic alone does not demonstrate a Rules of React violation.
[Versioned BuildHIR implementation, lines 1293–1313](https://github.com/facebook/react/blob/5c15c1cd3442f28044217703b5625f98c40042ee/compiler/packages/babel-plugin-react-compiler/src/HIR/BuildHIR.ts#L1293-L1313)

The installed [Vite configuration](../../vite.config.ts) passes no options to
`reactCompilerPreset()`. Compiler 1.0.0 defaults to `compilationMode: 'infer'`,
`panicThreshold: 'none'`, and target React 19. A failed compilation returns no
replacement function; successfully compiled functions are applied separately.
That explains the observed mix of ordinary and optimized components.
[Versioned defaults](https://github.com/facebook/react/blob/5c15c1cd3442f28044217703b5625f98c40042ee/compiler/packages/babel-plugin-react-compiler/src/Entrypoint/Options.ts#L269-L286),
[versioned error handling](https://github.com/facebook/react/blob/5c15c1cd3442f28044217703b5625f98c40042ee/compiler/packages/babel-plugin-react-compiler/src/Entrypoint/Program.ts#L591-L606),
[versioned replacement step](https://github.com/facebook/react/blob/5c15c1cd3442f28044217703b5625f98c40042ee/compiler/packages/babel-plugin-react-compiler/src/Entrypoint/Program.ts#L750-L785)

React recommends the default skip behavior for production and says unsupported
components can run normally while other components remain compiled. Its lint
reference expressly permits addressing diagnostics incrementally. The
troubleshooting guide distinguishes build failures and runtime regressions;
neither follows merely from this known skipped component.
[panicThreshold](https://react.dev/reference/react-compiler/panicThreshold),
[compiler diagnostics](https://react.dev/reference/eslint-plugin-react-hooks),
[debugging and troubleshooting](https://react.dev/learn/react-compiler/debugging)

## Correctness and performance are separate questions

In [App](../../src/App.tsx), `pending.current` prevents overlapping status reads;
`finally` releases it after the protected success/error handling. The mounted
guard suppresses updates after unmount. These are presentation request-lifecycle
decisions, independent of memoization. Removing `finally` and placing the reset
after `catch` loses cleanup if the catch itself throws or a future branch returns
early. ECMAScript evaluates a finalizer after the preceding block/catch completion,
including abrupt completion. Preserve that guarantee when moving the code.
[ECMAScript TryStatement evaluation](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-try-statement-runtime-semantics-evaluation)

Compiler memoization primarily reduces repeated rendering and render-time
calculations. An uncompiled `App` can repeat work that compilation might reuse,
but the size of that cost needs evidence. This shell contains a status observation,
an empty workspace and diagnostics; its own refreshes happen on mount or explicit
interaction, with no stream/history workload yet. The conclusion that fixing its
coverage has low demonstrated urgency is an inference from those local sources,
not a latency measurement. Compiler coverage cannot remove supervisor/IPC latency
or replace the Rust ownership and reconciliation rules in the
[MVP](../MVP.md).
[Compiler optimization scope](https://react.dev/learn/react-compiler/introduction#what-kind-of-memoization-does-react-compiler-add)

The retained production build and mocked-IPC presentation units passed. The later
actual-Tauri release qualification used the same production asset and covered
ordinary/minimum layout, focus, pending/error/stale states, explicit retry and
desktop reconnect. It explicitly reports the App bailout and makes no measured
speedup or streaming/history performance claim. This establishes the qualified
shell behavior on that Mac, not that all future workloads are fast or all
components are compilable.
[Compiler setup evidence](../evidence/react-compiler-adoption.md),
[desktop results and limits](../evidence/milestone-01-ui-foundation.md#desktop-qualification-follow-up--2026-11-12)

## Minimal options if coverage becomes useful

| Option                                 | Effect and tradeoff                                                                                                                                                                                                                             | Fit here                                                                                                                                                   |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Retain the documented bailout          | Keeps existing cleanup and qualified behavior; App gets no compiler memoization.                                                                                                                                                                | Recommended now.                                                                                                                                           |
| Extract a module-level connection Hook | Moves state, refs, observation Effect and refresh into one coherent capability. The Hook may still be skipped for its finalizer while App becomes independently eligible. Keep one owner and call it once where shared observations are needed. | Reasonable when connection lifecycle obscures a growing shell; proposed, not compiled or behavior-verified here.                                           |
| Extract a module-level async helper    | Can isolate the whole unsupported `try/catch/finally` outside React with explicit per-owner inputs. Moving only the IPC request while leaving `finally` inside App does not remove this bailout.                                                | Smaller syntax isolation in some designs, but passing refs/setters/callbacks may make this simple lifecycle harder to follow. Proposed, not verified here. |
| Extract a presentation component       | Leaves the observation owner intact while a coherent shell/view can compile independently.                                                                                                                                                      | Use when the view contract earns the additional props/boundary. No automatic need for paired container/view files.                                         |

React documents that unsupported syntax can live in a standalone utility, and
that compilation can be rolled out per component/Hook. The specific boundaries
above are design options inferred from those mechanisms and the local
[component guidance](../../.agents/skills/react-engineering/SKILL.md), rather than
upstream prescriptions for this application.
[Unsupported syntax](https://react.dev/reference/eslint-plugin-react-hooks/lints/unsupported-syntax),
[incremental adoption](https://react.dev/learn/react-compiler/incremental-adoption)

Do not duplicate cleanup into success/error branches, flatten it after `catch`,
or replace it with promise chains solely to satisfy this compiler version.
Those rewrites need their own error/settlement reasoning and supply no measured
benefit here. A `"use memo"` directive changes candidate selection; it does not
implement missing finalizer lowering. A `"use no memo"` directive is useful to
isolate a suspected runtime regression, but adds no optimization to an already
skipped App. Stricter panic thresholds expose errors; they do not repair support.
[Directive reference](https://react.dev/reference/react-compiler/directives/use-memo),
[runtime debugging](https://react.dev/learn/react-compiler/debugging),
[panicThreshold](https://react.dev/reference/react-compiler/panicThreshold)

If a future change is justified, inspect compiler events/emitted caches and retain
focused mocked-IPC coverage for pending exclusion, failure then retry, stale data
and unmount behavior. Measure the affected interaction before claiming a speedup;
React's Profiler can identify render cost, with profiling-build overhead and
production configuration accounted for. Actual macOS interaction qualification
remains separate under [repository policy](../agents/verification.md).
[Compiler logger](https://react.dev/reference/react-compiler/logger),
[Profiler](https://react.dev/reference/react/Profiler)

## Tauri fit and research limits

The supplied [Tauri frontend page](https://v2.tauri.app/start/frontend/) describes
Tauri as a static asset host for its webview, recommends Vite for React and other
SPA frameworks, and explains that framework-specific links document configuration
needs. It neither requires full React Compiler coverage nor ranks React below the
listed frameworks. There is no basis there for replacing the accepted React/Vite
foundation because one component is unoptimized. This application recommendation
also follows the prior [Tauri framework research](tauri-frontend-framework-fit.md).

This pass inspected source, package metadata, hashes and retained evidence; it
changed only this research note. At inspection, HEAD was
`ab56a34155ad4ebe0051de60ed8b716cd0d5946d`; App, Vite config, manifest and lockfile
SHA-256 values matched the successful
[compiler cohort](../evidence/react-compiler-adoption.md#source-identity-and-qualification-limits).

The coordinating review independently repeated these current-source checks on
macOS 26.2 (25C56), arm64, Apple M4 Pro, Node 26.13.1 and pnpm 12.18.0, with
installed dependencies and warm caches. Wall times below are single correctness
executions, not a performance benchmark.

| Command                                                                                                                                                    | Result                                                                                                                                                       | Wall time |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------- |
| Exact `node --input-type=module` diagnostic command retained in [coverage inspection](../evidence/react-compiler-adoption.md#compiler-coverage-inspection) | Exit 0; same App `Todo` at lines 61–80; `ConnectionDetails` success, 24 slots / 9 blocks / 9 values.                                                         | 0.0441 s  |
| `corepack pnpm frontend:build`                                                                                                                             | Exit 0; TypeScript and production frontend build pass, 2,251 modules; Vite reports 359 ms.                                                                   | 0.579 s   |
| `corepack pnpm test:ui`                                                                                                                                    | Exit 0; 1 file / 3 mocked-IPC units pass; Vitest 502 ms, maximum workers 1. The first attempt lost its output handle; repeated solely to observe the result. | 0.589 s   |

No Rust/static full gate, performance comparison, fixture-desktop or native-provider
check was run for this review. No desktop/provider action was taken. The fresh
checks corroborate the compiler coverage limit and presentation units; actual
desktop evidence is the retained cohort above. Existing successful and unperformed
qualification results remain preserved in their evidence records.
