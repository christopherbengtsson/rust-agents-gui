---
title: "Use passive listeners when the interaction only observes"
impact: LOW
impactDescription: "Lets scrolling proceed without waiting for cancellable handlers"
tags: events, scrolling, cleanup
---

# Use passive listeners when the interaction only observes

Apply when introducing wheel/touch listeners. If the handler never needs to cancel
the default interaction, mark it passive and retain owned cleanup.

**Incorrect (an observation-only wheel listener remains potentially blocking):**

```ts
element.addEventListener("wheel", observeWheel, { passive: false });
```

**Correct (for a handler that only observes):**

```ts
element.addEventListener("wheel", observeWheel, { passive: true });
// On owner cleanup:
element.removeEventListener("wheel", observeWheel);
```

A gesture/zoom implementation that requires `preventDefault` needs an active
listener instead; passive handlers cannot cancel it. Keep handler and capture
identity consistent during removal. This rule does not introduce global listeners
or override a toolkit's accessible interaction contract.

Source: [DOM passive listener option](https://dom.spec.whatwg.org/#dom-addeventlisteneroptions-passive).
