# React Compiler adoption for the Tauri presentation

2026-11-12 · Research and repository inspection; no compiler installation or execution.

This report retains the pre-adoption baseline. The compiler was subsequently added;
see [implementation and verification evidence](../evidence/react-compiler-adoption.md)
for the selected dependency cohort, emitted-code inspection, and qualification limits.

## Recommendation

**Adding React Compiler now is reasonable for this young React application. Use the stable Babel implementation first.** Its main value here is reducing future manual memoization work as chat and workspace screens grow. React released Compiler 1.0 as production-ready on 2025-10-07 and explicitly recommends it for new applications. This recommendation does not establish a measured benefit in this repository. The optional native compiler exposed by this repository's Vite plugin is still marked experimental; its shorter setup should not be confused with the stable status of React Compiler itself. [React release announcement](https://react.dev/blog/2025/10/07/react-compiler-1), [plugin-react 6.3.0 README](https://github.com/vitejs/vite-plugin-react/blob/plugin-react@6.3.0/packages/plugin-react/README.md#react-compiler)

Treat adoption as a small, separately reviewable tooling change, alongside React linting. It is also defensible to defer until the next substantive UI slice: the present status screen has no demonstrated rendering bottleneck, and Vite's current template deliberately leaves Compiler disabled because of development/build overhead. Waiting does not require an architectural migration later. [Local screen](../../src/App.tsx), [Vite template](https://github.com/vitejs/vite/blob/main/packages/create-vite/template-react-ts/README.md), [incremental adoption](https://react.dev/learn/react-compiler/incremental-adoption)

## Observed repository cohort

The checked-out [package manifest](../../package.json) and [lockfile](../../pnpm-lock.yaml) pin React/React DOM 19.3.1, Vite 8.3.8, and `@vitejs/plugin-react` 6.3.0. The installed plugin's manifest, README, and declaration files identify the same plugin version. Its official `plugin-react@6.3.0` source tag was accessible and agrees with those files; these versions were therefore examined directly rather than inferred from older tutorials. The plugin accepts Vite `^8.0.0`; its optional peers include Babel Compiler `^1.0.0` and `oxc-transform-react >=0.152.0 <0.157.0`. [Versioned package source](https://github.com/vitejs/vite-plugin-react/blob/plugin-react@6.3.0/packages/plugin-react/package.json)

[Vite configuration](../../vite.config.ts) currently uses `react()` and Tailwind, with no Compiler option or Babel stage. Neither Compiler implementation nor React ESLint tooling is declared. TypeScript uses automatic JSX and strict checking. [App.tsx](../../src/App.tsx) owns a connection observation, makes explicit status requests through Tauri, and has no manual `useMemo`/`useCallback` usage. This inspection does not establish that every component is compilable. [TypeScript configuration](../../tsconfig.json), [UI sources](../../src/components/ui/)

## Why it fits Tauri

Compiler runs during the frontend build and emits JavaScript. Tauri serves frontend assets and executes them in its operating-system WebView; its documentation recommends Vite for React SPAs. Wry, Tauri's WebView library, uses WKWebView on macOS. The resulting compatibility conclusion is an **architectural inference**, not a compiler-enabled desktop result: there is no special Tauri compilation target required by these documented layers. [React build-time tool](https://react.dev/learn/react-compiler/introduction), [Tauri frontend model](https://tauri.app/start/frontend/), [Tauri process model](https://v2.tauri.app/concept/process-model/), [Wry macOS WebView implementation](https://github.com/tauri-apps/wry/blob/dev/src/wkwebview/mod.rs)

React 19 already supplies the generated code's `react/compiler-runtime` API. Compiler is a development dependency; this app needs no standalone `react-compiler-runtime` dependency. React 17/18 instead require that package as a production dependency and a matching target. No compiler process needs to ship alongside the supervisor. [React target reference](https://react.dev/reference/react-compiler/target)

Locally, Tauri runs `frontend:build` and embeds `../dist`; Vite targets Safari 16.4 and the bundle declares macOS 13.3. Retain those browser/OS constraints and qualify actual compiled assets in the supported desktop environment. Compiler does not replace browser-target lowering or establish WebView compatibility merely by completing a build. [Tauri configuration](../../src-tauri/tauri.conf.json), [Vite configuration](../../vite.config.ts)

## Correct integration for this Vite version

**Recommended path:** add compatible, pinned development versions of `babel-plugin-react-compiler`, `@rolldown/plugin-babel`, and `@babel/core`; the installed plugin README additionally requires `@types/babel__core` for TypeScript with Babel 7. Its documented shape is:

```ts
import babel from "@rolldown/plugin-babel";
import react, { reactCompilerPreset } from "@vitejs/plugin-react";

// Relevant portion of the existing Vite configuration:
plugins: [react(), babel({ presets: [reactCompilerPreset()] }), tailwindcss()];
```

This is illustrative, not an applied edit or resolved dependency set. The versioned helper supplies a React-oriented filter and runtime prebundling configuration. [plugin README](https://github.com/vitejs/vite-plugin-react/blob/plugin-react@6.3.0/packages/plugin-react/README.md#babel-react-compiler), [helper source](https://github.com/vitejs/vite-plugin-react/blob/plugin-react@6.3.0/packages/plugin-react/src/reactCompilerPreset.ts)

Avoid the older `react({ babel: { plugins: [...] } })` recipe: plugin-react 6 removed its inline Babel integration. React Compiler must see original source before other Babel transformations; preserve its documented ordering if further transforms are added. [React installation guide](https://react.dev/learn/react-compiler/installation), [Vite 6.0 migration record](https://github.com/vitejs/vite-plugin-react/blob/plugin-react@6.3.0/packages/plugin-react/CHANGELOG.md#600-2026-03-12)

Two alternatives now exist; older statements that Vite can only use Babel are outdated:

| Path | Current evidence and implication |
| --- | --- |
| Native Oxc | `react({ compiler: true })` needs the optional `oxc-transform-react` peer. Plugin 6.3.0 explicitly labels this experimental. Oxc reports faster compilation, which is build throughput evidence, not this app's interaction latency. [Versioned source](https://github.com/vitejs/vite-plugin-react/blob/plugin-react@6.3.0/packages/plugin-react/src/index.ts), [Oxc announcement](https://oxc.rs/blog/2026-08-18-react-compiler-support) |
| SWC | Current `@vitejs/plugin-react-swc` documentation exposes `compiler: true`, but also recommends the default React plugin for most usage. Switching plugin families has no demonstrated benefit here. This is current upstream documentation, not a qualified installed SWC cohort. [Official SWC plugin README](https://github.com/vitejs/vite-plugin-react/blob/main/packages/plugin-react-swc/README.md#compiler) |

React's general introduction still describes first-class Oxc integration as future work; the versioned Vite sources and dated Oxc announcement provide the more specific current integration evidence. [React introduction](https://react.dev/learn/react-compiler/introduction)

## Correctness, rollout, and limits

Add `eslint-plugin-react-hooks` with a compatible ESLint configuration. Its current recommended rules expose Compiler diagnostics even without enabling compilation, covering Hooks, purity, mutation, refs, dependencies, and unsupported patterns. Diagnosed components can be skipped while other components compile. Static analysis does not prove every external library or effect is correct. [React lint reference](https://react.dev/reference/eslint-plugin-react-hooks)

For a cautious rollout, use `compilationMode: "annotation"` with `"use memo"`, or restrict compilation to selected files. Expand after observing behavior. For a suspected regression, temporarily mark the function `"use no memo"`, compare compiled/uncompiled behavior, and investigate the Rules of React or a minimal compiler reproduction. Confirm actual compilation through emitted runtime/cache code or the React DevTools memoization badge where available; configuration alone is insufficient evidence. [Incremental adoption](https://react.dev/learn/react-compiler/incremental-adoption), [debugging guide](https://react.dev/learn/react-compiler/debugging), [installation verification](https://react.dev/learn/react-compiler/installation)

Compiler primarily avoids repeated rendering and calculations in components/Hooks when inputs permit reuse. It does not memoize every ordinary function or share its caches across component instances. It cannot make an expensive first calculation free. Existing manual memoization can coexist; removing it wholesale is unnecessary for adoption. [React introduction](https://react.dev/learn/react-compiler/introduction)

For this app, keep stream bounds, history pagination, localized updates, IPC ownership, and uncertain-action reconciliation in their existing architectural owners. Automatic memoization supplies none of those policies and does not accelerate provider/network work, Rust execution, or disk I/O. These are conclusions from Compiler's documented scope and the [MVP architecture](../MVP.md), not new implementation requirements. The existing [React engineering guidance](../../.agents/skills/react-engineering/SKILL.md) still calls for measurement before targeted performance work.

No dependency, configuration, or product code changed in this research. Static/build checks, unit tests, fixture-desktop checks, native-provider checks, and compiler performance measurements were **not performed**. A future adoption change should report its build and behavior checks separately from actual macOS desktop compatibility and responsiveness evidence, following [repository verification policy](../agents/verification.md).
