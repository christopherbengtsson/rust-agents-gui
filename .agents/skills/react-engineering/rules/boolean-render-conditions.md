---
title: "Make numeric JSX conditions explicitly boolean"
impact: LOW
impactDescription: "Prevents stray zero or NaN output"
tags: jsx, conditionals, rendering
---

# Make numeric JSX conditions explicitly boolean

Apply when the left side of JSX `&&` can be a number or another renderable falsy
value. React renders the expression's result, so a count of zero can become text.

**Incorrect (an empty list renders a stray zero):**

```tsx
{items.length && <ResultList items={items} />}
```

**Correct (the guard produces a boolean):**

```tsx
{items.length > 0 && <ResultList items={items} />}
```

A ternary is also suitable, especially when both branches have meaning. Boolean
`&&` is valid; this rule is about the value produced, not a stylistic ban on it.

Source: [React conditional rendering](https://react.dev/learn/conditional-rendering#logical-and-operator-).
