# React component boundaries and a frontend engineering skill

2026-11-12 · Research supporting the accepted [React engineering skill](../../.agents/skills/react-engineering/SKILL.md). Research involved source and repository inspection; no application tests or desktop/native qualification were run.

## Recommendation

The user accepted a **repository-specific React engineering companion to the existing shadcn skill**, with extraction for concrete reasons, direct shadcn usage, and improvements applied as code is touched. The maintained instructions live in the skill; this report retains their rationale and primary-source evidence. These are local engineering choices, rather than a React-prescribed architecture.

The strict boundary is execution ownership: the supervisor's Rust core owns execution, reconciliation, retries, approval validity, and persistence. Moving those decisions into a Hook still violates the [MVP](../MVP.md). React owns presentation and transient interaction state; Tauri forwards targeted requests and returns observations. [Repository authority](../../AGENTS.md), [ADR 0001](../adr/0001-react-presentation-with-vite.md), and [ADR 0002](../adr/0002-shadcn-base-ui-presentation-components.md) govern over external examples.

## Observed repository context

The checked-out [package manifest](../../package.json) pins React/React DOM 19.3.1, Base UI 1.9.0, Tailwind 4.3.5, Vite 8.3.8, TypeScript 7.0.2, and Tauri API 2.12.4. These are declared versions, not a runtime qualification result. The official documentation examined labels React v19.3, Base UI npm 1.9.0, and Tailwind v4.3.

[components.json](../../components.json) selects `base-nova`, `rsc: false`, semantic CSS variables, and `src/components/ui`. [vite.config.ts](../../vite.config.ts) uses React and Tailwind plugins; [main.tsx](../../src/main.tsx) mounts with `createRoot`. No Compiler configuration is present in the inspected config/manifest. [App.tsx](../../src/App.tsx) combines the shell with read-only supervisor status requests and connection state; `ConnectionDetails` receives props. This starting point does not itself justify refactoring.

The [installed shadcn skill](../../.agents/skills/shadcn/SKILL.md) covers toolkit composition, styling, and accessibility. A companion should reference it. [verification.md](../agents/verification.md) owns testing and qualification.

## Is Container/Presentational obsolete?

**As a mandatory taxonomy, it is dated. As an optional separation of responsibilities, it remains useful.** Dan Abramov's original 2015 post is accessible and includes his 2019 correction: “I don’t suggest splitting your components like this anymore.” He explains that Hooks provide the stateful-logic extraction he wanted without an arbitrary division, while allowing the pattern when it feels natural. Even the historical description allowed presentational components to hold UI state. This is the original author's revised guidance, not a React deprecation of container components. [Original post and update](https://medium.com/@dan_abramov/smart-and-dumb-components-7ca2f9a7c7d0).

Current React documentation presents Hooks as stateful-logic reuse and leaves boundaries to developers. Hook extraction and container/view separation are independent choices: a focused observation Hook can simplify a component without a separate view. Separate Hook calls create independent state/Effects unless they use an explicitly shared source; extraction does not establish one observer. Prefer meaningful capabilities over lifecycle wrappers such as `useMount`. [Custom Hooks](https://react.dev/learn/reusing-logic-with-custom-hooks).

The useful distinction is therefore **what a unit owns and depends on**, rather than whether its name ends in `Container` or `View`. A dialog can own whether it is open; a form can own an unsent draft; a list can receive observations and intent callbacks. Lift state to the closest common parent when components must coordinate, and deliberately choose controlled versus uncontrolled ownership. [Sharing state](https://react.dev/learn/sharing-state-between-components).

## What current React guidance changes

| Concern | Official guidance | Fit for this repository |
| --- | --- | --- |
| Rendering | Render must be idempotent; side effects run outside render; props and state are immutable. [Purity rules](https://react.dev/reference/rules/components-and-hooks-must-be-pure). | No IPC, runtime start, mutation, or subscription installation during rendering. Plain formatting/selectors remain ordinary functions. |
| State shape | Avoid contradictory, redundant, and duplicated state. [Choosing state structure](https://react.dev/learn/choosing-the-state-structure). | Distinguish an authoritative observation from connection/pending/error display state. Do not turn a frontend guess into execution truth. Prefer a union when it prevents impossible UI combinations. |
| Event or Effect | Derive display data during render; handle interaction-caused work in the corresponding event handler. Effects synchronize with external systems. [You Might Not Need an Effect](https://react.dev/learn/you-might-not-need-an-effect). | Send/approve/archive follows explicit user intent and a targeted command. An Effect may retrieve status or subscribe to observations; mounting a view must not admit execution. |
| Async lifetime | Effect dependencies include reactive inputs; cleanup precedes re-setup and unmount. React's manual-fetch example ignores outdated responses to avoid races. [useEffect](https://react.dev/reference/react/useEffect). | Name the owner of each listener/request. Dispose subscriptions and ignore or cancel stale view results where supported. Frontend response suppression does not prove a backend action was canceled. |
| Shared external data | `useSyncExternalStore` reads a subscribable store using cleanup, stable subscription identity, and cached immutable snapshots. [API contract](https://react.dev/reference/react/useSyncExternalStore). | Use it when a shared frontend observation store actually exists. It is not required for every one-off `invoke`, nor a reason to add a store now. |
| Performance | `memo` is an optimization, not a correctness guarantee. `useMemo` caches calculations; React Compiler can reduce manual memoization when installed. [memo](https://react.dev/reference/react/memo), [useMemo](https://react.dev/reference/react/useMemo), [Compiler setup](https://react.dev/learn/react-compiler/installation). | Avoid blanket memoization. Localize high-frequency updates, bound displayed data, then measure a real slow interaction before selecting an optimization. React 19 alone does not establish that this app has Compiler optimization. |

Effects must tolerate setup/cleanup; Strict Mode deliberately exercises this during development, although current `main.tsx` does not enable it. [StrictMode](https://react.dev/reference/react/StrictMode). The [MVP's](../MVP.md) bounded streams, paged history, and desktop latency still need actual qualification.

### Server Components do not replace this application's boundary

React Server Components execute in a separate environment before client bundling; they can run at build time without an always-running web server. They require supporting bundler/framework integration, and their framework implementation APIs have version caveats. [Official Server Components reference](https://react.dev/reference/rsc/server-components).

The inspected Vite configuration and `rsc: false` show no RSC pipeline. Consequently, Next.js advice about Server Actions or `'use client'` is not a default here. A Rust backend alone does not supply an RSC environment. This is a repository-fit inference, not a claim that desktop software cannot support RSC.

## Assessment of the supplied 2026 article

The [Medium article](https://medium.com/write-a-catalyst/shadcn-ui-best-practices-for-2026-444efd204f44), dated February 2, 2026, was readable in full. It is a useful checklist of opinions, not the authority for this stack. The following paraphrases and evaluation distinguish its advice from primary documentation.

| Article advice | Assessment against primary sources |
| --- | --- |
| Own component source and review updates deliberately. | Keep. shadcn distributes editable source with composition as a core principle. Underlying Base UI and other packages still remain dependencies. [shadcn introduction](https://ui.shadcn.com/docs). |
| Establish tokens early. | Keep semantic CSS variables. Translate its JavaScript theme-config example to the installed Tailwind 4 CSS-based setup; do not copy it mechanically. [shadcn theming](https://ui.shadcn.com/docs/theming), [Tailwind theme variables](https://tailwindcss.com/docs/theme). |
| Always wrap primitives and maintain `ui/primitives/blocks` layers. | Do not adopt as a universal rule. shadcn explicitly enables direct source edits; its Vite example imports Button directly. Introduce a product composition when it expresses real repeated semantics or behavior. A pass-through `AppButton` for hypothetical future needs is optional overhead. [shadcn introduction](https://ui.shadcn.com/docs), [Vite usage](https://ui.shadcn.com/docs/installation/vite). |
| Compose useful product blocks. | Keep the principle, scoped to working screens. It does not require a blocks directory or a generic component framework. Official composition is compatible with feature-level components. [shadcn composition rationale](https://ui.shadcn.com/docs#composition). |
| Preserve accessibility; Radix and `asChild` supply the primitives. | Preserve accessibility, but use the selected Base UI API. This repo uses `render`, with a custom element forwarding its ref and spreading received props. Base UI supplies behavior while developers still supply labels, visible focus, and contrast. [Base UI composition](https://base-ui.com/react/handbook/composition), [accessibility](https://base-ui.com/react/overview/accessibility). |
| Prefer simple styles, CSS interaction, and documented decisions. | Sensible defaults, not proof of universal performance gains or a reason to remove required interactive behavior. Centralize design through tokens; retain meaningful architectural decisions in the repository's ADR system. [Theming](https://ui.shadcn.com/docs/theming), [domain documentation policy](../agents/domain.md). |

Base UI's `render` merges handlers/refs onto one rendered element. Wrappers must preserve them, semantics, names, focus, and keyboard behavior rather than nesting another interactive control. Calling a component “presentational” does not remove these obligations. [Composition](https://base-ui.com/react/handbook/composition), [accessibility](https://base-ui.com/react/overview/accessibility).

## Skill direction

The research led to these areas of guidance; the linked skill above owns the maintained instructions.

1. **Read authority first.** Follow the MVP, relevant milestone, verification policy, pinned versions, and existing shadcn skill.
2. **Make ownership explicit.** Keep toolkit components independent of IPC. Features or focused Hooks connect observations and targeted callbacks. Keep drafts and disclosure state near their users; leave recovery in Rust.
3. **Split for a reason.** Extract coherent capabilities or UI contracts. Allow simple combined components. No mandatory container/view pair, hooks-first rule, stateless-view rule, universal wrapper, or fixed file length.
4. **Review effectful edges.** Identify dependencies, cleanup, target identity, stale responses, and errors. Keep execution commands in intent paths. UI pending guards do not establish durable duplicate protection.
5. **Keep observations honest.** Show relevant loading, empty, error, stale, disconnected, and unknown states. Transport failure must not silently determine execution outcome or trigger replay.
6. **Preserve accessible composition.** Use shadcn/Base UI APIs, semantic tokens, and variants; preserve labels, focus, and keyboard behavior.
7. **Measure performance.** Bound growing data and localize frequent updates before adding optimizations. Report desktop qualification separately from static checks and mocked outcomes.

A possible seam is a typed IPC adapter, an observation Hook/owner, and a feature view, only when the feature warrants it. Connection observations could eventually separate from shell markup without mandatory paired files. This report establishes no current defect and authorizes no refactor.

Of the plausible scopes, a separation-only skill is too narrow to address async ownership; a broad visual-design skill would overlap shadcn and require additional accepted visual goals. A React engineering companion best addresses this repository's concrete risks while leaving visual design and product scope to their existing authorities.
