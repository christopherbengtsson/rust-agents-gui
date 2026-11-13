---
title: "Preserve component and domain identity"
impact: HIGH
impactDescription: "Prevents state, focus, and resource resets or reassignment"
tags: components, keys, identity
---

# Preserve component and domain identity

Apply to reusable component definitions, reordered lists, or navigation between
domain objects. Keep component types stable, normally at module scope, and use
stable domain keys. Nested component definitions create a new type on each render;
array positions cannot identify records that move.

**Incorrect (reordering can attach a draft to a different conversation):**

```tsx
conversations.map((conversation, index) =>
  <Composer key={index} conversationId={conversation.id} />
);
```

**Correct (identity survives reordering):**

```tsx
conversations.map(conversation =>
  <Composer key={conversation.id} conversationId={conversation.id} />
);
```

React does not pass `key` as a prop: pass action targets explicitly. Ordinary inline
callbacks and JSX expressions are not nested component definitions. Changing a
key can intentionally reset a draft; select that behavior from the feature's
contract rather than prescribing reset or preservation for every screen.

Source: [Preserving/resetting state](https://react.dev/learn/preserving-and-resetting-state).
