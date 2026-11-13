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
function Sidebar() { const connection = useConnection(); /* ... */ }
function Diagnostics() { const connection = useConnection(); /* ... */ }
```

**Correct (illustrative child props share one source):**

```tsx
function Shell() {
  const connection = useConnection();
  return <>
    <Sidebar connection={connection} />
    <Diagnostics connection={connection} />
  </>;
}
```

Use context or an [actual external store](external-store-snapshots.md) if consumers
need that distribution model. SWR or another state library is a separate justified
choice, not a required fix for repeated Hook calls. Place a continuing source
outside [visibility-owned subtrees](view-resource-lifetimes.md).

Source: [Custom Hooks share logic, not state](https://react.dev/learn/reusing-logic-with-custom-hooks).
