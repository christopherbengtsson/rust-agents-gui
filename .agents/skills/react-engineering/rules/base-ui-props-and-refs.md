---
title: "Preserve Base UI composition props and refs"
impact: HIGH
impactDescription: "Keeps focus, event handling, and accessible trigger behavior intact"
tags: base-ui, composition, refs
---

# Preserve Base UI composition props and refs

Apply when a custom React component receives Base UI `render` composition props.
Forward received props and ref to the intended single DOM element. The selected
foundation uses `render`; Radix `asChild` examples require translation.

**Incorrect (drops the library's handlers, attributes, and ref):**

```tsx
function Trigger({ children }: React.ComponentPropsWithRef<"button">) {
  return <button>{children}</button>;
}
```

**Correct (React 19 forwards the ref through these props):**

```tsx
function Trigger(props: React.ComponentPropsWithRef<"button">) {
  return <button {...props} />;
}
```

When adding handlers or styles, use supported composition/merging behavior rather
than overwriting incoming behavior. Base UI `render` merges refs, whereas its
`mergeProps` utility alone retains the rightmost ref. Preserve the intended
interactive element and check keyboard/focus behavior. Layer/style choices remain
owned by [the component guide](../../../../src/components/README.md) and
[shadcn](../../shadcn/SKILL.md); this rule adds no wrapper requirement.

Source: [Base UI composition](https://base-ui.com/react/handbook/composition).
