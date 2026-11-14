---
title: "Place state with its coordinating owner"
impact: HIGH
impactDescription: "Prevents divergent drafts and duplicated observations"
tags: state, ownership, subscriptions
---

# Place state with its coordinating owner

Apply when several controls coordinate selection, drafts, or observations. Identify
the authoritative owner before choosing where state lives. Keep local interaction
state near its consumers and lift it to their closest coordinating owner. Calling
a custom Hook twice creates two independent instances unless it deliberately
connects to a shared source.

**Incorrect (the editor and preview cannot share this unsent draft):**

```tsx
function DraftEditor() {
  const [draft, setDraft] = useState("");
  // Owns a draft unrelated to DraftPreview's draft.
}
function DraftPreview() {
  const [draft] = useState("");
}
```

**Correct (illustrative children share the dialog's local draft):**

```tsx
function NoteDialog() {
  const [draft, setDraft] = useState("");
  return (
    <>
      <DraftEditor draft={draft} onDraftChange={setDraft} />
      <DraftPreview draft={draft} />
    </>
  );
}
```

Current destinations and navigable views belong to Router. Retained presentation
memory supplies restoration defaults rather than a second current-selection owner;
see [TanStack guidance](../../../../docs/agents/tanstack.md). A dialog can own
disclosure and an unsent draft. Shared observations use their feature interfaces;
see [shared observations](shared-observation-owner.md) for Query, actor and
presentation-store ownership.

Source: [Sharing state](https://react.dev/learn/sharing-state-between-components).
