---
title: "Use pure lazy initialization for expensive initial state"
impact: MEDIUM
impactDescription: "Avoids recomputing initialization on each render"
tags: state, initialization, purity
---

# Use pure lazy initialization for expensive initial state

Apply when initial state requires a substantial calculation. Pass an initializer
function rather than evaluating the calculation on every render.

**Incorrect (the calculation runs even after state is initialized):**

```tsx
const [draft] = useState(buildInitialDraft(initialText));
```

**Correct (initializes this mounted state owner):**

```tsx
const [draft] = useState(() => buildInitialDraft(initialText));
```

Initialization is per mount, not once per application lifetime. React can call
initializers extra times in development. Keep them pure and move commands,
subscriptions, and required writes to their proper lifecycle/intent owners.
Changing `initialText` later does not reset this draft: choose an explicit reset
or identity boundary when that is the desired behavior. Cheap literals need no
lazy wrapper. Preference reads additionally follow
[presentation record handling](presentation-records.md).

Source: [React initializers](https://react.dev/reference/react/useState#avoiding-recreating-the-initial-state).
