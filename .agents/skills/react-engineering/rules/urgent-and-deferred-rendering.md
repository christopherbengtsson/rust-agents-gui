---
title: "Defer expensive presentation while keeping inputs urgent"
impact: MEDIUM
impactDescription: "Keeps interactions responsive without changing action semantics"
tags: rendering, transitions, deferred-values
---

# Defer expensive presentation while keeping inputs urgent

Apply to slow filtering, history, and change inspection. Keep controlled inputs
and immediate intent feedback urgent. Use a Transition for a nonurgent state
update you own, or a deferred value for an expensive consumer of received input.

**Incorrect (controlled input state is updated as nonurgent):**

```tsx
<input value={query} onChange={event => {
  startTransition(() => setQuery(event.target.value));
}} />
```

**Correct (illustrative results subtree can lag behind typing):**

```tsx
const deferredQuery = useDeferredValue(query);
return <>
  <input value={query} onChange={event => setQuery(event.target.value)} />
  <HistoryResults query={deferredQuery} stale={query !== deferredQuery} />
</>;
```

Ensure the expensive subtree can skip unchanged urgent renders, using Compiler
first. Explain stale content where it affects actions. Measure calculation versus
render cost: scheduling cannot split one long synchronous calculation. Deferral
does not debounce/deduplicate reads or cancel backend work. Retain authoritative
action status and response ordering. Current React requires another Transition
around post-`await` setters intended to be nonurgent.

Sources: [useDeferredValue](https://react.dev/reference/react/useDeferredValue),
[useTransition](https://react.dev/reference/react/useTransition).
