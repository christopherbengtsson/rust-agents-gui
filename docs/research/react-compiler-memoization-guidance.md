# React Compiler and manual memoization guidance

2026-11-12 · Focused follow-up to [compiler adoption](react-compiler-adoption.md).
The research pass consulted official React documentation without running application
checks. Subsequent implementation results are retained in the
[compiler setup evidence](../evidence/react-compiler-adoption.md).

## Recommended skill wording

With React Compiler enabled, write pure render calculations and callbacks directly
and let the compiler handle routine memoization. Add `useMemo`, `useCallback`, or
`memo` when measurement or a documented integration need justifies explicit control;
keep dependencies complete. Retain existing manual memoization unless the touched
change justifies its removal and appropriate verification. Keep state and resource
identity in their proper owners; memo caches are not correctness guarantees.
Structure Effects around their actual reactive inputs, and investigate compiler
diagnostics or incompatible libraries before adding memoization. The
[React engineering skill](../../.agents/skills/react-engineering/SKILL.md) now owns
the maintained wording, grounded in React's recommendation to rely on Compiler for
new code while retaining deliberate manual control. [React Compiler 1.0](https://react.dev/blog/2025/10/07/react-compiler-1#what-should-i-do-about-usememo-usecallback-and-reactmemo)

## Why this default fits

React Compiler performs automatic memoization of components and Hooks during the
build. React's API documentation explicitly reduces the need for manual `useMemo`
and `useCallback`, and describes automatic component optimization equivalent to
`memo`. Therefore, adding wrappers routinely to new code introduces maintenance
without establishing a benefit. Keep the existing skill's requirement to measure
slow interactions and preserve its guidance to bound growing history and localize
updates. Compiler optimizes repeated work; its existence does not establish a
measured improvement in this application. [Compiler introduction](https://react.dev/learn/react-compiler/introduction),
[useMemo](https://react.dev/reference/react/useMemo),
[useCallback](https://react.dev/reference/react/useCallback),
[memo](https://react.dev/reference/react/memo),
[current skill](../../.agents/skills/react-engineering/SKILL.md)

This should remain a default rather than a prohibition. React's release guidance
allows manual Hooks for precise control, including limiting unnecessary Effect
resynchronization. Its `useCallback` reference still discusses returning memoized
functions from custom Hooks so consumers can optimize. These are reasons to
understand the particular boundary, rather than mechanically adding or removing
wrappers. Existing memoization is preserved where the compiler's inference can
match it; incomplete dependencies can prevent compilation. Removal can change
compiled output and deserves focused verification. [Release guidance](https://react.dev/blog/2025/10/07/react-compiler-1#what-should-i-do-about-usememo-usecallback-and-reactmemo),
[custom Hook callbacks](https://react.dev/reference/react/useCallback#optimizing-a-custom-hook),
[preserve-manual-memoization](https://react.dev/reference/eslint-plugin-react-hooks/lints/preserve-manual-memoization)

## Correctness and escape hatches

Both `useMemo` and `useCallback` caches can be discarded, including when initial
mount suspends. `memo` also does not guarantee that a render will be skipped.
Application correctness therefore must not depend on these optimizations retaining
an object or callback forever. Use state for values that drive rendering, refs for
component-owned mutable information that does not, and an explicitly owned Effect
for external resource setup and cleanup. Refs persist across renders of that
mounted owner; this is not durable storage across unmounts or application restarts.
[useMemo caveats](https://react.dev/reference/react/useMemo#caveats),
[useCallback caveats](https://react.dev/reference/react/useCallback#caveats),
[memo reference](https://react.dev/reference/react/memo#reference),
[useState](https://react.dev/reference/react/useState),
[useRef](https://react.dev/reference/react/useRef),
[Effect dependencies](https://react.dev/learn/removing-effect-dependencies)

Prefer expressing Effect dependencies through meaningful reactive inputs, moving
objects/functions inside the Effect when appropriate. A memoized dependency may
reduce redundant work; it must not hide a missing dependency or replace correct
resource lifecycle handling. React explicitly identifies reliance on referential
equality for correctness as a Compiler migration hazard. [Removing dependencies](https://react.dev/learn/removing-effect-dependencies),
[Compiler debugging](https://react.dev/learn/react-compiler/debugging)

Compiler can skip unsupported components while compiling others. Some third-party
APIs contain hidden mutable state and are incompatible with both automatic and
manual memoization; adding `useMemo` is not their general remedy. Investigate
`incompatible-library` diagnostics and use compatible APIs. For a suspected compiler
regression, a scoped, temporary `"use no memo"` directive can isolate the cause;
document the issue and remove the opt-out after resolution. [Compiler lint diagnostics](https://react.dev/reference/eslint-plugin-react-hooks),
[incompatible-library](https://react.dev/reference/eslint-plugin-react-hooks/lints/incompatible-library),
[use no memo](https://react.dev/reference/react-compiler/directives/use-no-memo)

## Evidence limits

React 19 supplies `react/compiler-runtime`; generated imports and cache checks can
confirm transformation, with no separate `react-compiler-runtime` dependency.
Emitted caches or a DevTools memoization badge establish compiler activity, not
coverage of every component, improved interaction latency, or macOS WebView
qualification. Report static/build, unit, fixture-desktop, and native-provider
results separately under repository policy. This research performed none of those
checks and makes no performance claim. [Target reference](https://react.dev/reference/react-compiler/target),
[installation verification](https://react.dev/learn/react-compiler/installation#verify-your-setup),
[repository verification policy](../agents/verification.md)
