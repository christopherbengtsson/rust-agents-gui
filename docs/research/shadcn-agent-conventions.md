# shadcn agent conventions: source evidence

Research inspected on 2026-11-13. This note supports the repository's eight UI
principles; it does not add product scope or verification gates. Official API
requirements, upstream recommendations, and repository design choices are
distinguished below.

## Inspection baseline

Read-only inspection found `base-nova`, `rsc: false`, CSS variables enabled, and
`src/style.css` configured in [components.json](../../components.json).
[package.json](../../package.json) pins Base UI 1.9.0, React/React DOM 19.3.1,
Tailwind and its Vite plugin 4.3.5, and Vite 8.3.8. The
[Vite configuration](../../vite.config.ts) enables React Compiler. The stylesheet
already defines semantic colors and radius mappings through `@theme inline`.
The inspected [Button](../../src/components/ui/button.tsx) uses Base UI and CVA;
[Dialog](../../src/components/ui/dialog.tsx) uses Base UI's `render` composition.
These observations establish the starting point, not a complete component audit
or evidence of accessible runtime behavior. This baseline is recorded for the convention change; inspect configuration for current values.

## Eight principles and their basis

1. **Own the component source.** shadcn distributes editable source and explicitly
   supports customization. Maintaining local component APIs and reviewing upstream
   changes are consistent with that model. Separate `ui`, `primitives`, and
   `blocks` directories are our organizational choice, not an upstream requirement;
   `ui` need not remain an untouched upstream snapshot. Source:
   [shadcn introduction](https://ui.shadcn.com/docs).

2. **Use semantic theme tokens.** shadcn recommends CSS variables with semantic
   surface/foreground pairs. Tailwind 4 supports CSS-first `@theme inline` mapping.
   Prefer existing role utilities and extend shared roles for recurring product
   meaning. Exact colors, density, radius scale, and supported themes are product
   choices. Sources: [theming](https://ui.shadcn.com/docs/theming),
   [Tailwind 4 and React 19](https://ui.shadcn.com/docs/tailwind-v4).

3. **Give product abstractions a purpose.** Use a primitive when it centralizes a
   real product convention or behavior; compose a block when it owns meaningful
   product structure. A mandatory `AppButton` wrapper around every Button is not
   an upstream rule: official examples import Button directly. Our directory and
   import conventions express repository policy. Sources:
   [shadcn introduction](https://ui.shadcn.com/docs),
   [Base UI Button examples](https://ui.shadcn.com/docs/components/base/button).

4. **Compose coherent blocks.** shadcn makes composition a central principle.
   Choosing which product regions become blocks, and having parents own the layout
   between them, are repository design decisions. Base UI's API contract is more
   specific: compose through `render`, and pass received props and the ref through
   custom components to their DOM element. React 19 does not remove that ref
   requirement. Sources: [shadcn introduction](https://ui.shadcn.com/docs),
   [Base UI composition](https://base-ui.com/react/handbook/composition).

5. **Preserve and complete accessibility.** Base UI handles many ARIA, keyboard,
   and focus interactions. Application authors still supply accessible names,
   visible focus, and suitable contrast. Keep appropriate native semantics and
   avoid nested interactive elements; preserve labels, handlers, and refs when
   composing. Accessibility needs review after customization, rather than an
   assumption that generated code makes every screen accessible. Sources:
   [Base UI accessibility](https://base-ui.com/react/overview/accessibility),
   [composition](https://base-ui.com/react/handbook/composition).

6. **Keep styling and variants intentional.** Shared variants and predictable
   component APIs help keep product styling consistent. Exact variant counts,
   parent-owned external layout, and rules about when to add a primitive are our
   choices. Upstream supports variants and customization; it does not impose the
   article's numeric or structural limits. Sources:
   [Button variants](https://ui.shadcn.com/docs/components/base/button),
   [shadcn introduction](https://ui.shadcn.com/docs).

7. **Use CSS for visual states and primitives for interaction.** Base UI exposes
   state attributes for styling, allowing hover, focus, and open-state presentation
   without duplicate React state. Keep actual interaction behavior in the owning
   primitive. This is a design preference, not proof that fewer lines of JavaScript
   make a screen faster; performance claims need measurements. Source:
   [Base UI styling](https://base-ui.com/react/handbook/styling).

8. **Document decisions and review updates.** Component purpose, usage boundaries,
   deviations, and update rationale are repository documentation conventions.
   shadcn's CLI supports `add --dry-run`, `--diff`, and `--view` for inspecting
   incoming source. Review source updates separately from Base UI dependency
   upgrades, whose release notes include focus and accessibility fixes. Sources:
   [shadcn CLI](https://ui.shadcn.com/docs/cli),
   [Base UI releases](https://base-ui.com/react/overview/releases).

## Article applicability and limits

The [Medium article](https://medium.com/write-a-catalyst/shadcn-ui-best-practices-for-2026-444efd204f44)
provides the discussion outline, not authoritative API guidance. Its Radix and
`asChild` assumptions require translation to Base UI's `render` API. Its JavaScript
Tailwind configuration example does not match this project's Tailwind 4 CSS-first
setup. Blanket wrappers, a three-tier directory layout, parent layout ownership,
and documentation requirements become rules only through repository agreement.

The official sources above were reviewed for this research; they are living pages,
not immutable snapshots. The Base UI release page identifies 1.9.0; check the
installed version's API before adopting later examples. No verification commands,
tests, builds, desktop checks, or native-provider runs were performed for this
note. Verification remains governed by
[testing and verification](../agents/verification.md).

## Lint implementation compatibility

The initial package resolution found eslint-plugin-jsx-a11y 6.10.2 supports ESLint
through version 9, while typescript-eslint 8.72.2 supports TypeScript below 6.1.
The application uses TypeScript 7.0.2. The gate therefore pins ESLint 9.39.5 and
uses Babel's parser for TypeScript/JSX syntax alongside the existing Babel 8
toolchain. Type checking and unused-symbol checking remain with `tsc`; this gate
does not claim type-aware lint coverage. ESLint 9 is marked deprecated in package
metadata; revisit the pin when the accessibility plugin supports ESLint 10.
Sources: [Babel ESLint parser](https://babeljs.io/docs/babel-eslint-parser),
[Babel parser syntax plugins](https://babeljs.io/docs/babel-parser#plugins),
[React Hooks lint](https://react.dev/reference/eslint-plugin-react-hooks),
[JSX accessibility flat configuration](https://github.com/jsx-eslint/eslint-plugin-jsx-a11y).
Installed dependency peer ranges were checked with `corepack pnpm peers check`.
