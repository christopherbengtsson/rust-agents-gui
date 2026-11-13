---
title: "Load substantial optional surfaces behind local fallbacks"
impact: MEDIUM
impactDescription: "Defers optional code while keeping surrounding UI usable"
tags: lazy, suspense, bundles
---

# Load substantial optional surfaces behind local fallbacks

Apply to substantial optional editors/renderers/inspection surfaces. Declare lazy
component types at module scope, with a local loading fallback and an owner for
load failure. Small always-used controls need not be split speculatively.

**Incorrect (a new lazy type is created on every parent render):**

```tsx
function DiffPanel() {
  const Diff = lazy(() => import("./diff-renderer"));
  return <Diff />;
}
```

**Correct (illustrative optional module exports a default React component):**

```tsx
const Diff = lazy(() => import("./diff-renderer"));
function DiffPanel() {
  return <SurfaceBoundary>
    <Suspense fallback={<p>Loading diff…</p>}><Diff /></Suspense>
  </SurfaceBoundary>;
}
```

`SurfaceBoundary` denotes a render error boundary, not an installed library/API.
Keep navigation and usable surrounding content available. Suspense recognizes
supported sources such as lazy code; adding a boundary does not make ordinary
Effect/IPC reads suspend. See [failure handling](render-and-load-failures.md).

Sources: [React lazy](https://react.dev/reference/react/lazy),
[Suspense](https://react.dev/reference/react/Suspense).
