---
title: "Schedule reads by their actual dependency graph"
impact: MEDIUM
impactDescription: "Avoids unnecessary waterfalls while retaining ordering and budgets"
tags: async, concurrency, errors
---

# Schedule reads by their actual dependency graph

Apply when a view needs several observations. Start independent reads together;
start dependent work when its input becomes available. Use native promises and
bound fan-out for growing inputs.

**Incorrect (unrelated observations wait for each other):**

```ts
const snapshot = await invoke<unknown>("supervisor_status");
const pid = await invoke<number>("desktop_pid");
```

**Correct (existing independent command names):**

```ts
const [snapshot, pid] = await Promise.all([
  invoke<unknown>("supervisor_status"),
  invoke<number>("desktop_pid"),
]);
```

Validate replies at the adapter boundary. `Promise.all` rejects on a rejection but
does not cancel sibling work. Choose aggregate versus partial error presentation
from the feature contract; preserve cleanup and target checks. Independent reads
do not establish safe parallel mutations. No dependency helper library is required.

Sources: [Promise.all semantics](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Promise/all),
[Vercel dependency scheduling](https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-dependencies.md).
