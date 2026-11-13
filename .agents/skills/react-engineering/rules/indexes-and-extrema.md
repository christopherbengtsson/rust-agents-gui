---
title: "Choose data structures for repeated lookup and selection work"
impact: MEDIUM
impactDescription: "Avoids unnecessary repeated scans or sorts on growing data"
tags: algorithms, maps, performance
---

# Choose data structures for repeated lookup and selection work

Apply to measured repeated joins/membership or growing history/change data. An
index can avoid scanning the same collection for each item. Account for rebuild
cost, key uniqueness, ownership, and readability before replacing short arrays.

**Incorrect (large repeated membership scans):**

```ts
const visible = rows.filter(row => selectedIds.includes(row.id));
```

**Correct (when the workload warrants building the index):**

```ts
const selected = new Set(selectedIds);
const visible = rows.filter(row => selected.has(row.id));
```

Measure index construction as well as lookups. Maps need a duplicate-key policy;
silently keeping the last value may change a join that previously selected the
first. An extremum can be found in one pass rather than sorting the entire input;
preserve tie, empty-input and comparison semantics. Compiler can optimize repeated
render work, but it does not establish the right domain algorithm or index bounds.

Sources: [Set](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set),
[Vercel extrema example](https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-min-max-loop.md).
