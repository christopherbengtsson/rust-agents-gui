---
title: "Reuse regular expressions only with explicit state semantics"
impact: LOW
impactDescription: "Avoids allocation costs without introducing stateful matching bugs"
tags: regexp, performance, correctness
---

# Reuse regular expressions only with explicit state semantics

Apply when repeated expression construction is a measured cost. A constant pattern
can live outside a hot loop, but global/sticky expressions carry mutable `lastIndex`.
Choose that state deliberately rather than blindly hoisting every expression.

**Incorrect (repeated tests can alternate because the shared regex is global):**

```ts
const hasDigit = /\d/g;
const matches = values.map(value => hasDigit.test(value));
```

**Correct (a stateless membership test needs no global flag):**

```ts
const hasDigit = /\d/;
const matches = values.map(value => hasDigit.test(value));
```

When global/sticky iteration is required, reset or isolate matching state according
to its owner. Dynamic patterns need correct escaping and bounded/admissible input;
turning arbitrary user text into a regex changes literal-search semantics. A
shared cache of dynamic expressions additionally follows
[cache ownership and bounds](owned-bounded-caches.md).

Source: [RegExp test and lastIndex](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/RegExp/test#description).
