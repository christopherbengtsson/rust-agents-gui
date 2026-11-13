---
title: "Handle render failures and async failures at their actual boundaries"
impact: HIGH
impactDescription: "Keeps action errors visible and lazy retries deliberate"
tags: errors, boundaries, lazy
---

# Handle render failures and async failures at their actual boundaries

Apply to failure isolation for optional surfaces and user actions. Render boundaries
handle descendant render/lazy-load failures. Ordinary event-handler promises and
async callbacks need explicit rejection handling and target-bound feedback.

**Incorrect (a surrounding render boundary cannot handle this ordinary rejection):**

```tsx
<button onClick={() => { void saveDraft(); }}>Save draft</button>
```

**Correct (illustrative async callback handles its own failure):**

```tsx
<button onClick={() => {
  void saveDraft().catch(error => setSaveError(String(error)));
}}>Save draft</button>
```

Capture target/version when the callback can outlive its view, and preserve
uncertain outcomes rather than infer that a failed reply means non-admission.
Use an accessible local error state. React caches a lazy loader's promise and
resolved value; a rejected load is not guaranteed to retry merely by resetting an
error boundary. Design the retry/reload path deliberately; do not recreate lazy
types on every render to simulate recovery. Never replay application actions as
part of render-boundary recovery.

Sources: [React error boundaries](https://react.dev/reference/react/Component#catching-rendering-errors-with-an-error-boundary),
[lazy caching](https://react.dev/reference/react/lazy).
