# UI foundation: component-library options

Historical research for the incremental Conductor-inspired desktop UI. Decision status is resolved by the accepted [React decision](../adr/0001-react-presentation-with-vite.md) and [component foundation decision](../adr/0002-shadcn-base-ui-presentation-components.md). The comparison below records the reasoning before those decisions; implementation follows the ADRs and current spec.

## Starting point

The current frontend uses vanilla TypeScript, Vite, pnpm and Tauri, with no React, Tailwind or component library dependency. Adopting the React options below therefore includes a presentation-framework decision. Sources: [package manifest](../../package.json), [current frontend](../../src/main.ts).

The intended shell and feature progression are described in the [Conductor UI research](conductor-incremental-ui-foundation.md). The product's execution and reconciliation policies remain owned by the Rust core regardless of UI tooling. Source: [MVP](../MVP.md).

## Primary-source findings

| Option | What the source establishes | Project assessment |
| --- | --- | --- |
| shadcn/ui | Distributes editable component source with styled defaults and composition patterns. Its Vite guide supports React/TypeScript and Tailwind setup in an existing project. | Recommended starting layer if we choose React: use a small subset and adapt shared styles for the desktop. Owning copied source also means reviewing upstream fixes and preserving local changes. |
| Base UI | Unstyled React components; no bundled CSS or prescribed styling system. Includes dialogs, menus, tabs, tooltips and other interaction primitives. | Good directly if we want to own more styling work. Also usable underneath shadcn/ui. |
| Radix Primitives | Unstyled components with documented focus, keyboard and ARIA behavior; incremental adoption. | Viable primitive alternative. No existing Radix investment here requires choosing it. |
| React Aria | Style-free React components with keyboard, selection, drag/drop and internationalization facilities. | Credible alternative for richer collection interactions. Those capabilities alone do not establish a present need to choose it for this milestone. |
| Uber Base Web (`baseui`) | React implementation of Uber's Base design language, with Styletron dependencies. Repository warns that external engagement is limited while internal development is mirrored. | Different from Base UI. Not recommended as this project's starting point given its stated maintenance posture and our custom desktop design. |

Sources: [shadcn introduction](https://ui.shadcn.com/docs), [shadcn Vite installation](https://ui.shadcn.com/docs/installation/vite), [Base UI overview](https://base-ui.com/react/overview/about), [Radix introduction](https://www.radix-ui.com/primitives/docs/overview/introduction), [React Aria](https://react-aria.adobe.com/), [Uber Base Web repository](https://github.com/uber/baseweb). Assessment is our inference from those features and the product requirements, not a benchmark or comparative accessibility certification.

shadcn/ui explicitly supports Base UI and Radix implementations. Its July 2026 announcement makes Base UI the default for new projects while retaining Radix support. Thus choosing shadcn/ui does not imply choosing Radix, and shadcn/ui versus Base UI is partly a choice between using styled source components and using their underlying primitives directly. Sources: [Base UI documentation announcement](https://ui.shadcn.com/docs/changelog/2026-01-base-ui), [Base UI default announcement](https://ui.shadcn.com/docs/changelog/2026-07-base-ui-default).

## Recommendation at research time

Choose React for presentation, retain Vite/pnpm/Tauri, and use a small set of shadcn/ui components backed by Base UI with Tailwind and shared CSS variables. This recommendation favors readily adaptable source components and a consistent interaction foundation for the known shell, registration forms and later approval/chat controls.

Establish application-specific typography, density, spacing, surfaces, borders, focus and status styling. A library supplies useful pieces; it does not decide workspace/conversation navigation, error meaning, approval validity or recovery behavior. Keep those product semantics faithful to the existing Rust ownership model.

Add components when actual screens need them: initially buttons, inputs, labels, a registration dialog and tooltips if used. Layout can begin with ordinary CSS grid/flex; add specialized resizing only when change-inspection behavior calls for it. Avoid installing a complete component catalog or adopting a chat/agent execution framework as part of this decision.

Migration scope should be presentation-only: move the existing status exchange into the shell and preserve its behavior before adding registration. Exact dependency/CLI versions and generated component provenance should be recorded when implementation begins.

## UI foundation delivery

The subsequently agreed compact dark baseline and project-specific reference-view deliverables are defined in [ticket 02a](../../.scratch/milestone-01/issues/02a-establish-incremental-ui-foundation.md).

## Evidence limits

This research used official documentation/repositories and read-only inspection of the current frontend. No packages were installed and no code, spec or ticket was changed. Library accessibility/browser-support statements are upstream claims; they do not qualify our eventual composition, focus behavior, portals, scrolling or resizing inside the Tauri webview. Any checks must follow the [repository testing policy](../agents/verification.md).
