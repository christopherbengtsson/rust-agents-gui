---
title: "Keep optional imports analyzable and preloading intentional"
impact: MEDIUM
impactDescription: "Avoids accidental broad bundles and unsupported loading assumptions"
tags: vite, imports, preloading
---

# Keep optional imports analyzable and preloading intentional

Apply when choosing optional module/asset loading. Use Vite-supported analyzable
paths or an explicit loader map. Preload code on likely user intent when the
latency benefit warrants it; importing a module must not admit application work.

**Incorrect (the bundler cannot enumerate arbitrary runtime module names):**

```ts
const loadRenderer = (moduleName: string) => import(moduleName);
```

**Correct (illustrative modules are known at build time):**

```ts
const renderers = {
  diff: () => import("./diff-renderer"),
  image: () => import("./image-renderer"),
};
const loadRenderer = (kind: keyof typeof renderers) => renderers[kind]();
```

Vite's normal module entry/import graph manages much loading/preloading work.
Inspect emitted assets before adding classic scripts or resource hints. If a
separate script/resource has an actual consumer, select ordering, load/error
handling, CSP, and browser support for that resource. Drop Next/hydration-specific
recipes; neither a preload nor module initialization is permission to send/start.

Source: [Vite dynamic imports and preload handling](https://vite.dev/guide/features.html#dynamic-import).
