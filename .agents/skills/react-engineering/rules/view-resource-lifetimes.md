---
title: "Choose visibility by state and resource lifetime"
impact: HIGH
impactDescription: "Preserves required observations while views are hidden"
tags: activity, visibility, lifecycle
---

# Choose visibility by state and resource lifetime

Apply when tabs/panels hide or unmount. Unmounting discards local state and cleans
up Effects; CSS hiding retains mounted resources. Activity preserves state/DOM but
cleans up hidden Effects and re-creates them on reveal.

**Incorrect (a required continuing stream belongs to a hidden Activity):**

```tsx
<Activity mode={visible ? "visible" : "hidden"}>
  <PanelThatOwnsTheSharedStream />
</Activity>
```

**Correct (illustrative props separate observation and presentation ownership):**

```tsx
const stream = useSharedStream();
return <Activity mode={visible ? "visible" : "hidden"}>
  <ConversationPanel stream={stream} />
</Activity>;
```

Draft retention/reset is a feature decision: lift per-target drafts or preserve
appropriate view identity when required. Bound the number of retained views.
Activity is optional, not a universal menu/tab wrapper. Supervisor work and
conversation lifetime are independent of UI visibility.

Source: [React Activity](https://react.dev/reference/react/Activity).
