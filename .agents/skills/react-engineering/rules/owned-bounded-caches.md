---
title: "Give shared caches an owner, invalidation, and bounds"
impact: MEDIUM
impactDescription: "Avoids stale results and unbounded lifetime outside React"
tags: caching, ownership, memory
---

# Give shared caches an owner, invalidation, and bounds

Apply to shared computation caches and external observation caches, including
architecture-required Query caches. Their correctness obligations apply whether
they exist for shared freshness, retained observations or performance.
Define the key's complete identity/revision, invalidation, capacity/lifetime, and
other writers. Use measurement to justify optional computation-cache optimizations
and claimed performance gains; an approved observation cache needs no new
performance case. Component-owned initialized state is often enough for local
preferences.

**Incorrect (a file name alone aliases different revisions forever):**

```ts
const cache = new Map<string, string>();
function remember(path: string, rendered: string) {
  cache.set(path, rendered);
}
```

**Correct (illustrative one-entry cache is bounded and revision-specific):**

```ts
type Entry = {
  workspace: string;
  path: string;
  revision: string;
  rendered: string;
};
let last: Entry | undefined;
function cached(workspace: string, path: string, revision: string) {
  return last?.workspace === workspace &&
    last.path === path &&
    last.revision === revision
    ? last.rendered
    : undefined;
}
function remember(entry: Entry) {
  last = entry;
}
```

The actual cache belongs to a chosen resource owner and clears on relevant context
changes. A one-entry cache is an example, not a required capacity. Suppressing
duplicate reads cannot replace Rust action deduplication or snapshot/cursor
reconciliation. Account for storage events or other writers if that source exists.
Use the approved Query cache for observations rather than adding the illustrative
Map beside it. [Observation guidance](../../../../docs/agents/observations.md#cache-actor-and-presentation-ownership)
owns feature freshness and the separation from actors and presentation memory.

Sources: [React memoization scope](https://react.dev/learn/react-compiler/introduction),
[storage event semantics](https://developer.mozilla.org/en-US/docs/Web/API/Window/storage_event).
