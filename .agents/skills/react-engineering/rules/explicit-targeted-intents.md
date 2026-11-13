---
title: "Keep execution and reconciliation in the Rust owner"
impact: HIGH
impactDescription: "Avoids duplicate work after uncertain replies or reconnect"
tags: tauri, intent, recovery
---

# Keep execution and reconciliation in the Rust owner

Apply to send, approval, resume, archive, and retry controls. Tauri forwards an
explicitly targeted action and presents its observed status. The owning Rust core
module decides admission, reconciliation, and retries under the
[MVP](../../../../docs/MVP.md#starting-architecture).

**Incorrect (an uncertain reply becomes an automatic new submission):**

```ts
try { await submit(action); }
catch { await submit({ ...action, id: crypto.randomUUID() }); }
```

**Correct (illustrative adapter methods retrieve the original action's status):**

```ts
try { await submit(action); }
catch (error) {
  showTransportError(error);
  await readActionStatus(action.id);
}
```

Handle status-read failure visibly too; a second failure grants no retry authority.
The example's methods are placeholders for existing typed owning-module commands,
not new APIs. Retain exact approval identity and the original action payload.
Frontend pending guards improve feedback but do not replace durable duplicate
protection. View mounts and reconnects only retrieve observations.

Source: [Repository execution contract](../../../../docs/MVP.md#starting-architecture).
