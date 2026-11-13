---
title: "Give shared caches an owner, invalidation, and bounds"
impact: MEDIUM
impactDescription: "Avoids stale results and unbounded lifetime outside React"
tags: caching, ownership, memory
---

# Give shared caches an owner, invalidation, and bounds

Apply when measurement justifies caching beyond routine Compiler memoization.
Define the key's complete identity/revision, invalidation, capacity/lifetime, and
other writers. Component-owned initialized state is often enough for preferences.

**Incorrect (a file name alone aliases different revisions forever):**

```ts
const cache = new Map<string, string>();
function remember(path: string, rendered: string) { cache.set(path, rendered); }
```

**Correct (illustrative one-entry cache is bounded and revision-specific):**

```ts
type Entry = { workspace: string; path: string; revision: string; rendered: string };
let last: Entry | undefined;
function cached(workspace: string, path: string, revision: string) {
  return last?.workspace === workspace && last.path === path && last.revision === revision
    ? last.rendered : undefined;
}
function remember(entry: Entry) { last = entry; }
```

The actual cache belongs to a chosen resource owner and clears on relevant context
changes. A one-entry cache is an example, not a required capacity. Suppressing
duplicate reads cannot replace Rust action deduplication or snapshot/cursor
reconciliation. Account for storage events or other writers if that source exists.

Sources: [React memoization scope](https://react.dev/learn/react-compiler/introduction),
[storage event semantics](https://developer.mozilla.org/en-US/docs/Web/API/Window/storage_event).
