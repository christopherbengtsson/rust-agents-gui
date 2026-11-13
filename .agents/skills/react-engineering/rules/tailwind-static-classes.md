---
title: "Keep Tailwind class candidates discoverable"
impact: MEDIUM
impactDescription: "Prevents runtime variants whose CSS is absent from the bundle"
tags: tailwind, variants, css
---

# Keep Tailwind class candidates discoverable

Apply to conditional classes and component variants. Tailwind detects complete
class strings in source; it does not evaluate JavaScript interpolation. Map finite
choices to complete semantic classes or existing `cva` variants.

**Incorrect (the full class name never appears in source):**

```tsx
const className = `text-${muted ? "muted-foreground" : "foreground"}`;
```

**Correct (each possible utility is statically present):**

```tsx
const className = muted ? "text-muted-foreground" : "text-foreground";
```

For actual composition use existing `cn`/`cva` APIs and preserve their types. Truly
dynamic values may need a supported CSS variable/style strategy; check CSP and
the property's ownership. Source registration and theme changes belong in the
current Tailwind 4 configuration, not a parallel legacy config. The
[component guide](../../../../src/components/README.md) owns token/variant and
styling decisions; this rule concerns CSS generation, not new visual conventions.

Source: [Tailwind source detection](https://tailwindcss.com/docs/detecting-classes-in-source-files).
