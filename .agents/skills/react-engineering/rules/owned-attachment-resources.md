---
title: "Own attachment preview resources through cleanup"
impact: MEDIUM
impactDescription: "Avoids leaking URLs or confusing previews with durable content"
tags: attachments, urls, lifecycle
---

# Own attachment preview resources through cleanup

Apply to image/file preview URLs and subscriptions. Create external resources in
an owned lifecycle and release them when replaced/unmounted, after consumers no
longer need them. A preview URL is not durable attachment storage.

**Incorrect (every render allocates a URL that is never revoked):**

```tsx
return <img src={URL.createObjectURL(file)} alt="Attachment preview" />;
```

**Correct (resource ownership sketch; use only if blob previews are allowed):**

```ts
function openPreview(file: File): () => void {
  const url = URL.createObjectURL(file);
  try {
    const closePreview = mountPreview(url);
    return () => {
      try { closePreview(); }
      finally { URL.revokeObjectURL(url); }
    };
  } catch (error) {
    URL.revokeObjectURL(url);
    throw error;
  }
}
```

`mountPreview` is an illustrative preview adapter that returns its teardown; the
caller owns the returned cleanup, including replacement/unmount and setup failure.
Gate stale preview results, expose load failure, and retain accessible names.
Check Tauri CSP and asset permissions before choosing blob/data/native asset URLs;
the current image policy does not automatically allow blob URLs. Choose a
compatible preview path rather than broadly relaxing policy. Durable transcript
and attachment writes remain with Rust storage under the
[MVP](../../../../docs/MVP.md#starting-architecture).

Sources: [File API object URL lifetime](https://www.w3.org/TR/FileAPI/#lifeTime),
[Tauri CSP](https://v2.tauri.app/security/csp/).
