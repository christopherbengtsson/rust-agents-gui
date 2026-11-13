---
title: "Send interaction commands through explicit intent paths"
impact: HIGH
impactDescription: "Prevents view synchronization from replaying user commands"
tags: effects, events, intent
---

# Send interaction commands through explicit intent paths

Apply when deciding where a command or synchronization belongs. Handle work caused
by a particular interaction in its event/intent path. Effects synchronize external
systems with current inputs; render computes output.

**Incorrect (mounts and dependency changes can replay this command):**

```tsx
useEffect(() => {
  void submitDraft(conversationId, draft);
}, [conversationId, draft]);
```

**Correct (illustrative command callback runs on an explicit submit):**

```tsx
function handleSubmit(event: React.FormEvent) {
  event.preventDefault();
  void submitDraft(conversationId, draft);
}
```

The callback still needs [target capture and result handling](async-result-targets.md).
An Effect may read status or install a subscription. Mount, reconnect, and error
recovery retrieve observations; execution/retry decisions belong to the Rust owner
under [the MVP](../../../../docs/MVP.md#starting-architecture).

Source: [Separating events from Effects](https://react.dev/learn/separating-events-from-effects).
