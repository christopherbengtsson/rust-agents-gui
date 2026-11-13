---
title: "Budget optional long tasks and own scheduled callbacks"
impact: MEDIUM
impactDescription: "Keeps optional work responsive without delaying required effects"
tags: scheduling, timers, browser-support
---

# Budget optional long tasks and own scheduled callbacks

Apply when measured noncritical work blocks input. Bound/chunk work or consider an
appropriate worker. If idle scheduling fits, check WebView support, provide a
fallback and cancellation, and retain target/generation checks. A timeout only
defers a long task; it does not make that task interruptible.

**Incorrect (an unsupported idle API becomes the only execution path):**

```ts
window.requestIdleCallback(() => buildOptionalPreview());
```

**Correct (illustrative cancellable fallback for small optional work):**

```ts
function scheduleOptional(work: () => void): () => void {
  if (typeof window.requestIdleCallback === "function") {
    const id = window.requestIdleCallback(work, { timeout: 1000 });
    return () => window.cancelIdleCallback(id);
  }
  const id = window.setTimeout(work, 0);
  return () => window.clearTimeout(id);
}
```

The owner calls the returned cancellation function on cleanup and guards already
started work. Do not rely on idle opportunity for required identity persistence,
execution, reconciliation, or cleanup. Use controlled time in applicable DOM units;
that evidence does not measure actual WebView responsiveness.

Source: [Idle callback support and semantics](https://developer.mozilla.org/en-US/docs/Web/API/Window/requestIdleCallback).
