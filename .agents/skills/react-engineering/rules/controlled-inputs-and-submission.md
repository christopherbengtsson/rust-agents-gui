---
title: "Keep controlled input updates synchronous and submission explicit"
impact: HIGH
impactDescription: "Preserves typing, validation, and native form semantics"
tags: forms, inputs, accessibility
---

# Keep controlled input updates synchronous and submission explicit

Apply to composer/configuration fields and form controls. A controlled text input
keeps a defined string value and synchronously reflects user edits. Keep form
submission in an explicit intent path, and mark non-submit buttons accordingly.

**Incorrect (an unrelated picker button submits the containing form):**

```tsx
<form onSubmit={submit}>
  <button onClick={browse}>Choose folder</button>
</form>
```

**Correct (the two interactions have distinct semantics):**

```tsx
<form onSubmit={submit}>
  <button type="button" onClick={browse}>Choose folder</button>
  <button type="submit">Register</button>
</form>
```

Do not switch between controlled and uncontrolled values; use `value={text ?? ""}`
when absence should mean an empty controlled draft. Keep the input update urgent
and defer expensive consumers instead. Preserve labels, help/error associations,
focus, and pending feedback through existing Field/Input contracts. Native dialog
results only update the intended draft; they do not implicitly submit it. DOM
units use semantic queries and controlled dependency outcomes under
[verification policy](../../../../docs/agents/verification.md).

Sources: [React input](https://react.dev/reference/react-dom/components/input),
[Base UI forms](https://base-ui.com/react/handbook/forms).
