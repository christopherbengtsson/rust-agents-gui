---
title: "Keep updates immutable and updater functions pure"
impact: HIGH
impactDescription: "Preserves snapshots and queued update semantics"
tags: state, immutability, updates
---

# Keep updates immutable and updater functions pure

Apply when changing props/state/observations or computing next state. Treat input
snapshots as immutable. Use a functional updater when the new value depends on
pending state; direct assignments from an event's value remain suitable.

**Incorrect (mutates a retained snapshot):**

```tsx
setRows(previous => {
  previous.push(newRow);
  return previous;
});
```

**Correct (each queued update receives its actual prior state):**

```tsx
setRows(previous => [...previous, newRow]);
// ES2022-compatible copying sort:
const sorted = [...rows].sort((a, b) => a.name.localeCompare(b.name));
```

Updater functions can be replayed in development: place no IPC, persistence, or
required side effect inside them. Choose newer copying APIs only after checking
TypeScript library declarations and the WebView baseline; compiler optimization
does not make mutation safe.

Sources: [React updaters](https://react.dev/reference/react/useState#updating-state-based-on-the-previous-state),
[array updates](https://react.dev/learn/updating-arrays-in-state),
[TypeScript lib](https://www.typescriptlang.org/tsconfig/lib.html).
