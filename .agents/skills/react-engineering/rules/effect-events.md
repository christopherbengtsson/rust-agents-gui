---
title: "Use Effect Events only for nonreactive Effect logic"
impact: HIGH
impactDescription: "Preserves resource dependencies without stale event reads"
tags: effects, useEffectEvent, closures
---

# Use Effect Events only for nonreactive Effect logic

Apply when an event fired from an Effect should see latest committed values without
resynchronizing the external resource. Effect Events have intentionally changing
identity. Call them only from Effects or their callbacks in the owning component;
keep them out of dependencies and arbitrary component/Hook props.

**Incorrect (mute changes reconnect an otherwise unchanged room):**

```tsx
useEffect(() => {
  const connection = connect(roomId, () => {
    if (!muted) playNotice();
  });
  return () => connection.disconnect();
}, [roomId, muted]);
```

**Correct (for a contract where mute affects notice delivery, not connection):**

```tsx
const onNotice = useEffectEvent(() => {
  if (!muted) playNotice();
});
useEffect(() => {
  const connection = connect(roomId, () => onNotice());
  return () => connection.disconnect();
}, [roomId]);
```

The example's `connect` and `playNotice` are stable illustrative module functions.
`roomId` remains reactive. Latest-value reads must not retarget an admitted action;
use [captured targets](async-result-targets.md) for its completion instead.

Source: [React useEffectEvent](https://react.dev/reference/react/useEffectEvent).
