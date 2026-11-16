---
title: "Validate current presentation records without losing action identity"
impact: HIGH
impactDescription: "Preserves current-build recovery evidence"
tags: storage, preferences, recovery
---

# Validate current presentation records without losing action identity

Apply when reading/writing desktop preferences or changing their shape. Keep
records small and validate the current format. The MVP development compatibility
policy requires fresh state for incompatible changes; remove retired-format
readers and migrations. Distinguish discardable preferences from retained
identities needed to retrieve authoritative current-build action status.

**Incorrect (a read failure silently discards recovery evidence):**

```ts
try { return decode(JSON.parse(localStorage.getItem(key) ?? "null")); }
catch { localStorage.removeItem(key); return null; }
```

**Correct (illustrative decoder preserves unsupported or corrupt records):**

```ts
type Decoded<T> = { kind: "ok"; value: T } | { kind: "unreadable" };
function readRecord<T>(key: string, decode: (raw: unknown) => T): Decoded<T> {
  try {
    return { kind: "ok", value: decode(JSON.parse(localStorage.getItem(key) ?? "null")) };
  } catch { return { kind: "unreadable" }; }
}
```

The caller selects visible recovery for unreadable records; successful decoding is
not authoritative execution evidence. Preserve original action IDs/payloads through same-build recovery and handle
read/write failure according to the workflow. Unreadable action evidence must
remain visible and block submission until deliberately resolved. A cache miss
cannot prove non-admission. Presentation storage does not replace Rust-owned
durable transcripts/metadata. Avoid repeated synchronous reads during render;
initialize or subscribe through an explicit owner.

Sources: [localStorage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage),
[MVP durability](../../../../docs/MVP.md#starting-architecture).
