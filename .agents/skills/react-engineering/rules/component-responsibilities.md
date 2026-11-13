---
title: "Choose boundaries by responsibility"
impact: MEDIUM
impactDescription: "Keeps dependencies and update boundaries understandable"
tags: components, hooks, architecture
---

# Choose boundaries by responsibility

Apply when adding or restructuring a component or Hook. Extract a coherent UI
contract, reusable composition, or useful update boundary. A Hook extracts React
state/synchronization; a plain transformation needs no Hook. Simple features can
keep logic and markup together. Container/view separation is optional.

**Incorrect (a formatting function gains a misleading React contract):**

```ts
function useRepositoryLabel(name: string, branch: string) {
  return `${name} · ${branch}`;
}
```

**Correct (ordinary calculation stays independently usable):**

```ts
function repositoryLabel(name: string, branch: string) {
  return `${name} · ${branch}`;
}
```

Choose toolkit/product layers and custom component documentation through the
[component guide](../../../../src/components/README.md). Component extraction and
Hook extraction are independent decisions; neither file length nor a naming suffix
establishes the right boundary.

Source: [React custom Hooks](https://react.dev/learn/reusing-logic-with-custom-hooks).
