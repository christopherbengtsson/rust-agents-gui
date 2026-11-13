---
title: "Bind async results to their captured target and generation"
impact: HIGH
impactDescription: "Prevents late responses from overwriting a different view or draft"
tags: async, identity, races
---

# Bind async results to their captured target and generation

Apply to observation reads, sends, native pickers, and other results that can arrive
after navigation or editing. Capture the request's domain identity and relevant
draft/view generation. Separate recording its actual outcome from updating a view
that might no longer own that result.

**Incorrect (selection at completion can identify a different conversation):**

```ts
const result = await send(targetId, text);
recordOutcome(selectedIdRef.current, result);
```

**Correct (illustrative send/outcome interfaces preserve the submitted target):**

```ts
const target = targetId;
const version = draftVersion.current;
const result = await send(target, text);
recordOutcome(target, result);
if (selectedIdRef.current === target && draftVersion.current === version) {
  clearSubmittedDraft();
}
```

Handle rejection and cleanup as well. Suppressing an obsolete view response does
not cancel backend work. Native dialog cancellation returns no selection, and
closing a dialog does not establish that its pending promise was cancelled; guard
late picker results with the dialog/request generation. Durable outcome and retry
ownership remains in Rust, not in this illustrative presentation bookkeeping.

Guard action-result eligibility independently when its lifetime differs from the
current view or read subscription.

Sources: [React async cleanup](https://react.dev/reference/react/useEffect#fetching-data-with-effects),
[Tauri dialog API](https://v2.tauri.app/reference/javascript/dialog/).
