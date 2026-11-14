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

**Correct (illustrative generated shape check precedes contextual assertions):**

```ts
// Stand-ins for a generated declaration/validator and a local contextual assertion.
declare const validateStatus: (value: unknown) => value is Status;
declare const assertStatusContext: (
  value: Status,
  expectedOwner: string,
) => void;

const value = await invoke<unknown>("example_status");
if (!validateStatus(value)) {
  throw new Error("Invalid status observation");
}
assertStatusContext(value, expectedOwner);
// Only now may the feature observation module publish/cache value.
```

`example_status`, `Status` and the helper names illustrate a validation boundary;
they are not product APIs or manually maintained wire shapes.
Rust-generated declarations/standalone validators establish structure. Local
assertions establish exact owner/target, state and delivery semantics before Query
accepts a result. Preserve structured errors rather than adopting this sketch's
generic error message. See [contract preparation](../../../../docs/agents/observations.md#contract-preparation)
for generator settings, CSP and numeric/byte pitfalls.

Toolkit components receive data/callbacks rather than
importing Tauri directly or through application Hooks; see the
[component guide](../../../../src/components/README.md).

Sources: [Tauri invoke](https://v2.tauri.app/reference/javascript/api/namespacecore/),
[TypeScript narrowing](https://www.typescriptlang.org/docs/handbook/2/narrowing.html).
