---
title: "Derive display values during render"
impact: MEDIUM
impactDescription: "Avoids redundant state and synchronization renders"
tags: state, effects, rendering
---

# Derive display values during render

Apply when a value is fully determined by current props/state. Calculate it during
render instead of storing a second copy and synchronizing it with an Effect.
Subscribe only to observations that drive output; read event-only values on demand
when that preserves the event's intended target and timing.

**Incorrect (a second state value can lag behind the query):**

```tsx
const [filtered, setFiltered] = useState(items);
useEffect(() => {
  setFiltered(items.filter(item => item.name.includes(query)));
}, [items, query]);
```

**Correct (one source of truth):**

```tsx
const filtered = items.filter(item => item.name.includes(query));
```

An intentionally captured baseline, editable draft, or externally owned observation
is not automatically redundant state. For expensive derivation, consult
[Compiler memoization](compiler-first-memoization.md) and
[responsive rendering](urgent-and-deferred-rendering.md).

Source: [You might not need an Effect](https://react.dev/learn/you-might-not-need-an-effect).
