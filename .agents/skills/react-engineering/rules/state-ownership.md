---
title: "Place state with its coordinating owner"
impact: HIGH
impactDescription: "Prevents divergent drafts and duplicated observations"
tags: state, ownership, subscriptions
---

# Place state with its coordinating owner

Apply when several controls coordinate selection, drafts, or observations. Keep
interaction state near its consumers and lift it to their closest coordinating
owner. Calling a custom Hook twice creates two independent instances unless it
deliberately connects to a shared source.

**Incorrect (the list and panel cannot share this selection):**

```tsx
function RepositoryList() {
  const [selected, setSelected] = useState<string | null>(null);
  // Owns a selection unrelated to RepositoryPanel's selection.
}
function RepositoryPanel() {
  const [selected] = useState<string | null>(null);
}
```

**Correct (illustrative child interfaces receive one owner's state):**

```tsx
function RepositoryScreen() {
  const [selected, setSelected] = useState<string | null>(null);
  return <>
    <RepositoryList selected={selected} onSelect={setSelected} />
    <RepositoryPanel selected={selected} />
  </>;
}
```

A dialog can own disclosure and an unsent draft. Shared supervisor/conversation
observations need an explicit owner; see [shared observations](shared-observation-owner.md).

Source: [Sharing state](https://react.dev/learn/sharing-state-between-components).
