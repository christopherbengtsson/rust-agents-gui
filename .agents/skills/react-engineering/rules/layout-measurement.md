---
title: "Separate layout reads from invalidating writes"
impact: MEDIUM
impactDescription: "Avoids repeated forced layout in imperative integrations"
tags: dom, layout, css
---

# Separate layout reads from invalidating writes

Apply when an editor, scroller, or other imperative integration must measure DOM.
Prefer declarative CSS/classes for ordinary visual state. When measurements are
necessary, group reads and writes so each write does not force another read layout.

**Incorrect (each height change can invalidate the next width read):**

```ts
for (const element of elements) {
  element.style.height = `${container.offsetWidth}px`;
}
```

**Correct (one measurement serves the batch):**

```ts
const width = container.offsetWidth;
for (const element of elements) element.style.height = `${width}px`;
```

The example intentionally uses imperative owned elements; it is not a reason to
mutate arbitrary React-managed DOM. Choose an Effect/observer with cleanup when
integrating external layout, and account for resize and hidden/unmounted nodes.
Use layout Effects only when pre-paint measurement is actually required; they
block painting. Actual WebView layout/focus behavior needs desktop evidence.

Sources: [Layout thrashing](https://web.dev/articles/avoid-large-complex-layouts-and-layout-thrashing),
[useLayoutEffect](https://react.dev/reference/react/useLayoutEffect).
