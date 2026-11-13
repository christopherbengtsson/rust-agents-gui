---
title: "Separate app-load initialization from mount synchronization"
impact: MEDIUM
impactDescription: "Preserves re-setup without duplicate app-wide initialization"
tags: initialization, lifecycle, effects
---

# Separate app-load initialization from mount synchronization

Apply to frontend resources intended to exist once per loaded application. An
explicit entry/module initializer can own that lifetime. Mount-owned resources
still need cleanup and re-setup; a flag must not suppress a required second setup.

**Incorrect (after cleanup, remount can leave no observer):**

```tsx
let didSubscribe = false;
useEffect(() => {
  if (didSubscribe) return;
  didSubscribe = true;
  return subscribeToStatus();
}, []);
```

**Correct (the mounted owner can set up again):**

```tsx
useEffect(() => subscribeToStatus(), []);
```

The illustrative module function returns an unsubscribe handle. Truly app-wide
frontend initialization belongs outside this mount Effect when its contract calls
for that lifetime. Neither a module flag nor browser initialization establishes
durable once-only execution across desktop/supervisor restarts; status reads on
mount/reconnect remain valid.

Source: [Application initialization](https://react.dev/learn/you-might-not-need-an-effect#initializing-the-application).
