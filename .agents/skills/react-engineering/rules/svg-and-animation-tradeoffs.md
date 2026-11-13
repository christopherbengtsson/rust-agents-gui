---
title: "Treat SVG and animation changes as measured visual tradeoffs"
impact: LOW
impactDescription: "Avoids unsupported GPU claims and damaged asset geometry"
tags: svg, animation, accessibility
---

# Treat SVG and animation changes as measured visual tradeoffs

Apply when SVG complexity or animation is an actual rendering cost. Inspect the
asset and WebView behavior before choosing wrapper transforms, reduced path
precision, or a different asset. Keep accessible naming, layout, focus and reduced
motion behavior under the component contract.

**Incorrect (loading animation ignores the reduced-motion contract):**

```tsx
<span className="inline-flex animate-spin"><LoaderIcon /></span>
```

**Correct (illustrative nonessential animation respects reduced motion):**

```tsx
<span className="inline-flex motion-safe:animate-spin"><LoaderIcon /></span>
```

The icon is an illustrative existing accessible loading indicator. A wrapper can
change layout/semantics and does not establish GPU acceleration. Coordinate
rounding must preserve appearance at intended scales; benchmark/compare the actual
asset. Prefer supported Lucide imports and existing toolkit animation facilities;
their sizing and styling contracts remain in
[the component guide](../../../../src/components/README.md).

Sources: [Tailwind reduced motion](https://tailwindcss.com/docs/hover-focus-and-other-states#prefers-reduced-motion),
[Vercel SVG precision tradeoff](https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-svg-precision.md).
