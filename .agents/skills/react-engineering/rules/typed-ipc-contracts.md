---
title: "Validate IPC observations at typed boundaries"
impact: HIGH
impactDescription: "Prevents malformed or mismatched observations from becoming UI truth"
tags: tauri, typescript, validation
---

# Validate IPC observations at typed boundaries

Apply to command arguments, replies, and event payloads crossing the Rust/frontend
boundary. Match the Rust contract explicitly. A generic argument or assertion
cannot validate a runtime payload; narrow untrusted data and verify expected kind
and target where the boundary requires it.

**Incorrect (the type parameter silently trusts the reply):**

```ts
type Status = { state: "ready"; owner: string };
const status = await invoke<Status>("example_status");
```

**Correct (illustrative command and minimal schema are explicitly checked):**

```ts
const value = await invoke<unknown>("example_status");
if (typeof value !== "object" || value === null ||
    !("state" in value) || value.state !== "ready" ||
    !("owner" in value) || typeof value.owner !== "string") {
  throw new Error("Invalid status observation");
}
```

`example_status` is an illustrative command, not a new application API. Real
adapters validate the actual schema and domain identities, preserving structured
errors needed by consumers. Toolkit components receive data/callbacks rather than
importing Tauri directly or through application Hooks; see the
[component guide](../../../../src/components/README.md).

Sources: [Tauri invoke](https://v2.tauri.app/reference/javascript/api/namespacecore/),
[TypeScript narrowing](https://www.typescriptlang.org/docs/handbook/2/narrowing.html).
