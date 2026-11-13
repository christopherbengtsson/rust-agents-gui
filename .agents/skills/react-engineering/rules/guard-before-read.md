---
title: "Avoid reads that cheap relevant guards can rule out"
impact: MEDIUM
impactDescription: "Removes unnecessary async waits without changing required work"
tags: async, guards, ordering
---

# Avoid reads that cheap relevant guards can rule out

Apply when a branch need not observe a remote/IPC value at all. Evaluate cheap
relevant conditions before starting unnecessary reads and await results only in
the branch that uses them.

**Incorrect (loads details even when the panel is closed):**

```ts
const details = await readDetails(id);
if (!isOpen) return;
showDetails(details);
```

**Correct (for a purely optional details read):**

```ts
if (!isOpen) return;
const details = await readDetails(id);
if (isStillCurrent(id)) showDetails(details);
```

The read and view-check callbacks are illustrative. Preserve error handling and
stale-result checks. A guard cannot skip required authorization, reconciliation,
or persistence. Moving an await can change failure selection or side-effect order;
check the actual contract rather than applying this as a mechanical rewrite.

Source: [Vercel guard rule](https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-cheap-condition-before-await.md).
