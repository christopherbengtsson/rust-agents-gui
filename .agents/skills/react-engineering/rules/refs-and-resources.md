---
title: "Distinguish rendered state from imperative resources"
impact: HIGH
impactDescription: "Keeps resource identity independent of disposable memo caches"
tags: refs, state, resources
---

# Distinguish rendered state from imperative resources

Apply to mutable handles, request generations, timers, and values visible in JSX.
State drives output; refs retain nonrendering mutable information for a mounted
owner. Memo caches can be discarded and do not establish a resource lifetime.

**Incorrect (mutating a ref does not refresh the displayed status):**

```tsx
const pending = useRef(false);
return <button onClick={() => { pending.current = true; }}>
  {pending.current ? "Sending…" : "Send"}
</button>;
```

**Correct (a state update requests the visible change):**

```tsx
const [pending, setPending] = useState(false);
return <button onClick={() => setPending(true)}>
  {pending ? "Sending…" : "Send"}
</button>;
```

The example concerns presentation only, not durable send admission. Set up external
resources in owned Effects with cleanup; ordinary ref reads/writes belong outside
render except supported predictable initialization. A ref bridge can fit an actual
imperative integration, but it must preserve captured action identities.

Source: [React refs](https://react.dev/reference/react/useRef).
