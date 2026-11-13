---
title: "Preserve behavior when optimizing measured hot loops"
impact: LOW
impactDescription: "Keeps micro-optimizations from changing selected values or errors"
tags: algorithms, loops, semantics
---

# Preserve behavior when optimizing measured hot loops

Apply when profiling identifies repeated iteration/property access as meaningful
cost. Consider cheap decisive checks, early exits, combined passes, or local reuse
only with equivalent values, ordering, validation, and observable effects.

**Incorrect (the rewrite changes empty-string filtering):**

```ts
// Original:
const names = users.filter(user => user.active).map(user => user.name).filter(Boolean);
// Claimed equivalent, but retains active users' empty names:
const rewritten = users.flatMap(user => user.active ? [user.name] : []);
```

**Correct (if a combined pass is justified and name is a string):**

```ts
const names: string[] = [];
for (const user of users) {
  if (user.active && user.name) names.push(user.name);
}
```

Fewer source-level passes do not prove faster execution. Preserve first-versus-last
error selection, getters/proxy behavior, and comparator semantics. Cache property
access or combine passes only where it matters; clear separate transforms remain
valid. Use application requirements, not a performance recipe, to choose behavior.

Sources: [Array flatMap](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array/flatMap),
[Vercel original example](https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-flatmap-filter.md).
