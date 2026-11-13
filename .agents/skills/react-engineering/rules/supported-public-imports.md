---
title: "Preserve typed public imports until actual cost warrants change"
impact: MEDIUM
impactDescription: "Keeps package contracts intact while allowing measured bundle improvements"
tags: imports, lucide, bundles
---

# Preserve typed public imports until actual cost warrants change

Apply when investigating barrel-import, development-start, or bundle cost. Inspect
package exports/types and actual emitted code before changing paths. Next server
cold-start numbers do not establish a Tauri/Vite bundle problem.

**Incorrect (an assumed internal icon path can break exports or types):**

```ts
import Folder from "lucide-react/dist/esm/icons/folder";
```

**Correct (supported named icon import):**

```ts
import { Folder } from "lucide-react";
```

Lucide documents tree-shakable named imports and Vite prebundles dependencies.
These facts are not a guarantee that every package's barrels are free. If actual
analysis identifies a costly entry, choose a supported narrower export or other
measured remedy. Do not add untyped declarations merely to silence a deep-import
problem. Icon composition/styling remains governed by
[shadcn](../../shadcn/SKILL.md) and the component guide.

Sources: [Lucide React](https://lucide.dev/guide/react/),
[Vite dependency prebundling](https://vite.dev/guide/dep-pre-bundling).
