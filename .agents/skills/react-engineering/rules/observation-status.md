---
title: "Represent transport and execution evidence separately"
impact: HIGH
impactDescription: "Keeps disconnection or UI scheduling from inventing an outcome"
tags: observations, state, uncertainty
---

# Represent transport and execution evidence separately

Apply to pending/error/loading/status models. Distinguish the authoritative
observation from transport state and local interaction feedback. Preserve retained
observations with an explicit stale/unknown indication when that is the workflow.

**Incorrect (a read failure claims the operation failed):**

```ts
try { await refreshStatus(); }
catch { setExecutionState("failed"); }
```

**Correct (illustrative presentation model keeps the prior observation):**

```ts
type View<T> = {
  observation?: T;
  transport: "connected" | "disconnected";
  error?: string;
};
setView(previous => ({
  ...previous, transport: "disconnected", error: "Status unavailable",
}));
```

Loading, empty, disconnected, stale, failed, and unknown are distinct where the
feature produces them. A Transition's pending flag is rendering feedback, not
provider receipt or completion. Bind action feedback to its actual domain target;
use discriminated unions when they prevent contradictory combinations.

Source: [MVP evidence and recovery](../../../../docs/MVP.md#starting-architecture).
