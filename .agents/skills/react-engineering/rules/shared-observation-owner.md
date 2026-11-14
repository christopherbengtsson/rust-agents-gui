---
title: "Share one observation owner among consumers"
impact: HIGH
impactDescription: "Avoids duplicate subscriptions and divergent snapshots"
tags: subscriptions, ownership, state
---

# Share one observation owner among consumers

Apply when multiple views need the same supervisor/conversation source. Place its
observation Hook or store at an explicit common owner and distribute its snapshot
and targeted callbacks. A custom Hook's name does not imply shared state.

**Incorrect (two instances install two independent sources):**

```tsx
function Sidebar() {
  const connection = useConnection(); /* ... */
}
function Diagnostics() {
  const connection = useConnection(); /* ... */
}
```

**Correct (illustrative child props share one source):**

```tsx
function Shell() {
  const connection = useConnection();
  return (
    <>
      <Sidebar connection={connection} />
      <Diagnostics connection={connection} />
    </>
  );
}
```

Use context or an [actual external store](external-store-snapshots.md) if consumers
need that distribution model. Multiple feature Hook calls can correctly subscribe
to the same app-scoped Query client or actor; share the underlying owner rather
than requiring prop distribution for every consumer. The example's defect is
independent source creation, not repeated subscription to a shared source.

Follow [observation ownership](../../../../docs/agents/observations.md#cache-actor-and-presentation-ownership)
for the approved Query/XState/Zustand split. Feature interfaces hide cache keys and
coordination; consumers select observations and submit targeted callbacks instead
of copying records into component or presentation state. Place a continuing source
outside [visibility-owned subtrees](view-resource-lifetimes.md).

Source: [Custom Hooks share logic, not state](https://react.dev/learn/reusing-logic-with-custom-hooks).
