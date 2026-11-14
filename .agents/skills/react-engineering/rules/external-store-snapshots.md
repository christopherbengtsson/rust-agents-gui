---
title: "Give external stores stable immutable snapshots"
impact: HIGH
impactDescription: "Avoids render loops and missed external updates"
tags: stores, snapshots, subscriptions
---

# Give external stores stable immutable snapshots

Apply when integrating an actual subscribable frontend store or a library's React
bindings. For `useSyncExternalStore`, keep subscription identity stable where possible and
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

For adopted libraries, use their supported bindings inside feature interfaces
rather than rebuilding their subscription adapters or mirroring snapshots in an
Effect. Select the part needed for rendering. Illustrative integration with an
already app-owned actor and vanilla presentation store:

```tsx
import { useSelector } from "@xstate/react";
import { useStore } from "zustand";

function useReadingStatus() {
  const live = useSelector(feedActor, (snapshot) => snapshot.matches("live"));
  const panelWidth = useStore(presentationStore, (state) => state.panelWidth);
  return { live, panelWidth };
}
```

Actor/store names and selected fields are illustrative, not current app interfaces.
An object-producing selector needs the selected binding's supported
equality/stability handling; React Compiler does not define
external-store snapshot identity. Query feature Hooks likewise select what their
consumer needs, preserving loading/error status where it affects presentation.
See [ownership guidance](../../../../docs/agents/observations.md#cache-actor-and-presentation-ownership)
for where each record belongs.

Sources: [React useSyncExternalStore](https://react.dev/reference/react/useSyncExternalStore),
[XState React selectors](https://stately.ai/docs/xstate-react#useselectoractorref-selector-compare-getsnapshot),
[Zustand vanilla-store bindings](https://github.com/pmndrs/zustand/blob/v5.1.0/src/react.ts),
[Query render subscriptions](https://tanstack.com/query/v5/docs/framework/react/guides/render-optimizations).
