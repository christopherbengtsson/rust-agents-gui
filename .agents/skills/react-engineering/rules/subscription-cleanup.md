---
title: "Own subscription setup including late registration"
impact: HIGH
impactDescription: "Prevents leaked listeners after cleanup or re-setup"
tags: effects, tauri, subscriptions
---

# Own subscription setup including late registration

Apply to Tauri/browser listeners, timers, and other Effect-owned resources. Cleanup
must cover registration resolving after unmount, not just an already available
handle. Setup/cleanup/re-setup must leave one valid owner.

**Incorrect (cleanup can run before the promise supplies its handle):**

```tsx
useEffect(() => {
  let stop: (() => void) | undefined;
  void listen<string>("notice", event => setNotice(event.payload))
    .then(unlisten => { stop = unlisten; });
  return () => stop?.();
}, []);
```

**Correct (late registration tears itself down; rejection is observed):**

```tsx
useEffect(() => {
  let disposed = false;
  let stop: (() => void) | undefined;
  void listen<unknown>("notice", event => {
    if (!disposed && typeof event.payload === "string") setNotice(event.payload);
  }).then(unlisten => {
    if (disposed) unlisten();
    else stop = unlisten;
  }).catch(error => { if (!disposed) setError(String(error)); });
  return () => { disposed = true; stop?.(); };
}, []);
```

`listen` is from `@tauri-apps/api/event`; this sketch uses stable React setters.
An event generic supplies types, not payload validation. Reactive subscription
targets still belong in dependencies. Mock registration completion/rejection and
cleanup order in DOM units; real desktop listener behavior needs separate evidence
under [verification policy](../../../../docs/agents/verification.md).

Source: [Tauri frontend communication](https://v2.tauri.app/develop/calling-frontend/).
