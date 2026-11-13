---
title: "Let Compiler handle routine memoization"
impact: MEDIUM
impactDescription: "Keeps optimizations deliberate without relying on cache persistence"
tags: compiler, memoization, diagnostics
---

# Let Compiler handle routine memoization

Apply when adding render calculations/callbacks or investigating repeated work.
Check current compiler configuration and diagnostics; coverage is not universal.
Write ordinary pure code first. Add `useMemo`, `useCallback`, or `memo` for measured
benefit or a documented integration need, with complete dependencies.

**Incorrect (routine wrappers obscure a cheap calculation):**

```tsx
const label = useMemo(() => `${name} · ${branch}`, [name, branch]);
```

**Correct (routine new code with Compiler enabled):**

```tsx
const label = `${name} · ${branch}`;
```

Preserve existing deliberate memoization unless the touched change justifies
removing it and checking behavior. A skipped observation Hook may still require
stable callbacks for Effect synchronization. Memo caches are disposable, so use
proper state/ref/resource ownership for correctness. Avoid expensive calculations
in branches that will not display them; extraction is one possible technique.
An omitted default parameter does not change the incoming props compared by that
component's memo boundary, although downstream identity can matter on a render.

Sources: [Compiler defaults](https://react.dev/blog/2025/10/07/react-compiler-1#what-should-i-do-about-usememo-usecallback-and-reactmemo),
[Compiler debugging](https://react.dev/learn/react-compiler/debugging),
[memo](https://react.dev/reference/react/memo).
