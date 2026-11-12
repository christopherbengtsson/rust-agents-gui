# Presentation framework and desktop performance

Historical research follow-up to [UI library options](ui-foundation-library-options.md). The user is experienced with React and prioritizes performance. Decision status is resolved by the accepted [React decision](../adr/0001-react-presentation-with-vite.md) and [component foundation decision](../adr/0002-shadcn-base-ui-presentation-components.md); the comparison below preserves their supporting research.

## What primary sources establish

Tauri serves frontend assets in its webview and recommends Vite for SPA frameworks including React, Vue, Svelte and Solid. A Rust backend does not remove frontend JavaScript, DOM, layout or paint work. Source: [Tauri frontend configuration](https://v2.tauri.app/start/frontend/). The latter implication follows from the documented webview arrangement; it is not a measurement of this application.

React normally re-renders components when their parent renders. Memoization can skip unchanged props, while state and consumed context can still cause updates. React recommends localizing transient state and applying optimization where work is expensive. Re-rendering is component computation, not evidence that every DOM node was replaced or repainted. Source: [React memo](https://react.dev/reference/react/memo).

React Compiler is a separately configured build-time optimizer that automates memoization work. Choosing React does not automatically configure that tool, and it does not establish performance for this product. Source: [React Compiler introduction](https://react.dev/learn/react-compiler/introduction).

React's external-store interface rechecks a subscribed snapshot and renders when that value changes. Stable, appropriately scoped snapshots can connect application observations to a view without exposing the entire application as one changing value. A store mutation cannot be assumed to become non-blocking merely by wrapping it in a transition. Source: [useSyncExternalStore](https://react.dev/reference/react/useSyncExternalStore). The scoped-snapshot recommendation is our application-design inference.

Solid's signal/observer model targets reactive dependencies rather than re-executing whole component bodies for ordinary reactive updates. This is a plausible advantage for frequent small updates, not proof that a Solid implementation of this application would be faster overall. Source: [Solid fine-grained reactivity](https://docs.solidjs.com/advanced-concepts/fine-grained-reactivity).

Svelte's compiler syntax and deeply reactive state support granular updates to consumers of changed properties. It is another credible alternative for an interactive desktop. Sources: [Svelte runes](https://svelte.dev/docs/svelte/what-are-runes), [Svelte state](https://svelte.dev/docs/svelte/$state). These mechanisms do not establish relative bundle size, memory, startup time or interaction latency for our final dependencies and screens.

Virtualization can limit mounted list content to the relevant viewport range; TanStack documents this pattern independently of application layout/styling. Source: [TanStack Virtual introduction](https://tanstack.com/virtual/latest/docs/introduction). Its examples do not qualify variable-height streaming messages, selection behavior or our eventual diff viewer.

Svelte also has a shadcn-style source-component project; selecting Svelte does not necessarily mean writing every common control ourselves. It is a distinct implementation ecosystem, not permission to import React components unchanged. Source: [shadcn-svelte](https://www.shadcn-svelte.com/docs).

## Assessment for this repository

| Candidate | Reason to consider it | Tradeoff for this app |
| --- | --- | --- |
| React | User expertise; access to the React UI options already researched; explicit tools for controlling expensive rendering. | Careful state/subscription placement matters under sustained streaming. Compiler setup, if selected, is an additional build choice. |
| Svelte | Granular reactivity and compiler-driven authoring; a source-component ecosystem exists. | New authoring model for this user and a different component stack; a speed improvement is not measured here. |
| Solid | Fine-grained reactive updates suit frequent localized changes; JSX syntax may be familiar. | JSX familiarity does not make its reactivity identical to React. The researched React libraries are not automatically compatible, and actual component availability needs checking if chosen. |
| Continue vanilla TypeScript | No framework runtime added and the present status page is small. | The growing transcript, navigation and interaction model would require us to maintain more manual view coordination. This is a maintainability judgment, not a claim that vanilla cannot perform well. |

Recommendation: React remains a sound default given user experience and the known component requirements. The evidence does not justify declaring React performance irrelevant, guaranteeing the targets, or choosing another framework solely on a general speed claim.

## Performance direction as features arrive

These are proposed implementation practices, not extra acceptance gates or a new test framework:

- Keep shell/navigation, composer, active response and completed history updates localized. A streaming delta should not unnecessarily recalculate unrelated panels.
- Coalesce text-only presentation updates when needed; preserve all authoritative events and history in the Rust core. Never coalesce away approvals, delivery uncertainty, errors or terminal-state semantics.
- Parse/render the changing message incrementally or cache completed content as needed. Avoid repeatedly parsing the entire growing transcript, and keep expensive syntax highlighting away from each incoming chunk where possible.
- Bound loaded data and mounted content. Use paging first under the accepted product contract; add virtualization when the actual content size warrants it, accounting for variable-height text and selection/accessibility needs.
- Keep Git/process/disk work in its owning Rust modules, while remembering that webview layout, markdown parsing and rendering remain frontend work.
- Preserve responsiveness during actual streaming, history opening and diff navigation; framework choice alone does not bound any of those costs.

The product already targets navigation feedback within 100 ms and recent history opening within one second. Treat those as observed end-to-end desktop targets under the milestone workload; later concurrency requires its own qualification. Source: [MVP](../MVP.md), [Milestone 01](../MILESTONE-01.md).

Use the existing actual-desktop qualification path for measurements as representative screens arrive. Record build/profile, workload, hardware, versions, latency and relevant CPU/memory observations separately from unit results. Compare representative production builds rather than inferring shipped behavior from development instrumentation. Any proposed checks must follow the [repository verification policy](../agents/verification.md), including its separation of unit tests and qualification. No comparative prototype or automated integration suite is authorized or introduced by this note.

## Limits and decision status

No framework benchmark, app build, runtime measurement or package installation was performed. This is primary-source research plus product-specific engineering judgment. The accepted decisions do not establish measured performance for the final workload.
