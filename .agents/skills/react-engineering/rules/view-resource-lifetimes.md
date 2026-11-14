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
return (
  <Activity mode={visible ? "visible" : "hidden"}>
    <ConversationPanel stream={stream} />
  </Activity>
);
```

Draft retention/reset is a feature decision: lift per-target drafts or preserve
appropriate view identity when required. Bound the number of retained views.
Activity is optional, not a universal menu/tab wrapper. Supervisor work and
conversation lifetime are independent of UI visibility.

With app-owned actors, a view releases its feature interest on hide/unmount while
the owner drains actual work. Creating a fresh `useMachine`/`useActor` instance in
each routed screen would bind that actor to the screen's lifetime. Subscribe to
the existing owner through feature interfaces instead. Keep actor construction,
replacement and awaited native cleanup at the app boundary described in
[observation guidance](../../../../docs/agents/observations.md#cache-actor-and-presentation-ownership).
Stopping a React subscription or promise actor is not evidence of native cleanup.

Sources: [React Activity](https://react.dev/reference/react/Activity),
[XState React actor lifetimes](https://stately.ai/docs/xstate-react#useactoractorlogic-options).
