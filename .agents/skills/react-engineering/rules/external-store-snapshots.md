---
title: "Give external stores stable immutable snapshots"
impact: HIGH
impactDescription: "Avoids render loops and missed external updates"
tags: stores, snapshots, subscriptions
---

# Give external stores stable immutable snapshots

Apply when integrating an actual subscribable frontend store with
`useSyncExternalStore`. Keep subscription identity stable where possible and
return the same snapshot reference until the observed data changes.

**Incorrect (every read invents a changed snapshot):**

```ts
const getSnapshot = () => ({ ...currentSnapshot });
```

**Correct (illustrative store publishes a new immutable snapshot on change):**

```ts
type Snapshot = Readonly<{ revision: number; text: string }>;
let currentSnapshot: Snapshot = { revision: 0, text: "" };
const getSnapshot = () => currentSnapshot;
function publish(next: Snapshot) {
  currentSnapshot = next;
  for (const notify of listeners) notify();
}
```

`listeners` represents the store's subscribed callback set; subscription returns
its cleanup. Module ownership in this example is intentional, not a prescription
for global state. Keep snapshots immutable and publish only meaningful changes.
Simple owned observations can remain ordinary React state; introducing a store is
not a prerequisite for calling IPC.

Source: [React useSyncExternalStore](https://react.dev/reference/react/useSyncExternalStore).
