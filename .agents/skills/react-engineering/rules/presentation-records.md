---
title: "Validate and evolve presentation records without losing action identity"
impact: HIGH
impactDescription: "Keeps schema migration from erasing uncertain work"
tags: storage, preferences, migration
---

# Validate and evolve presentation records without losing action identity

Apply when reading/writing desktop preferences or changing their shape. Keep
records small, validate decoded data, and choose migration/version behavior for
the actual compatibility need. Distinguish discardable preferences from retained
identities needed to retrieve authoritative action status.

**Incorrect (every decode/migration failure deletes recovery identity):**

```ts
try { return migrate(JSON.parse(localStorage.getItem(key) ?? "null")); }
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
not authoritative execution evidence. Preserve original action IDs/payloads across
migration and handle read/write failure according to the workflow. A cache miss
cannot prove non-admission. Presentation storage does not replace Rust-owned
durable transcripts/metadata. Avoid repeated synchronous reads during render;
initialize or subscribe through an explicit owner.

Sources: [localStorage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage),
[MVP durability](../../../../docs/MVP.md#starting-architecture).
