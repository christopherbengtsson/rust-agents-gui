# React performance

Use the sections relevant to the touched behavior. These are choices for concrete
workloads, with measurement guiding optimization; they do not call for a frontend
rewrite. The [skill](../SKILL.md) owns shared state, identity, Effect, and Compiler
guidance.

## Responsive rendering and view visibility

For expensive filtering, history, or change inspection, keep controlled input
updates and immediate intent feedback urgent. Consider a Transition when you own
the nonurgent state update, or `useDeferredValue` when deferring a received value.
Make stale displayed content apparent where it affects interpretation or actions.
Deferral changes rendering priority; it neither debounces nor deduplicates reads.
Check that the expensive subtree can skip the urgent render, using Compiler's
optimization first and measuring any remaining work.
Measure calculation cost separately from rendering; scheduling cannot split a long
synchronous calculation.

Keep authoritative action pending, failure, and outcome observations distinct from
Transition pending. Interruptible rendering does not cancel backend work or order
custom async responses. Retain request/target checks when navigation changes.
Follow the installed React version's post-`await` transition requirements when
marking an async result update as nonurgent.

Choose unmounting, CSS visibility, or `<Activity>` according to the required local
state and resource lifetime. Activity preserves state/DOM but cleans up Effects
while hidden and recreates them on reveal. Keep shared observations outside a
visibility-dependent subtree when they must remain active across view switches.
A frontend app-load initializer can own a truly app-wide UI resource; mount-owned
subscriptions still need cleanup and re-setup. A module initialization flag cannot
establish durable once-only action execution across process lifetimes.

Sources: [Transitions](https://react.dev/reference/react/useTransition),
[deferred values](https://react.dev/reference/react/useDeferredValue),
[Activity](https://react.dev/reference/react/Activity).

## Optional code loading

For a substantial optional editor, renderer, or inspection surface, consider
loading code when the feature needs it. Declare React `lazy` components at module
scope; use supported Vite dynamic imports with analyzable paths. Give the surface
a loading fallback and an owner for load failures. Keep usable surrounding UI
available. If offering retry, account for React caching the lazy promise and its
rejection; resetting an error boundary alone does not guarantee another load.
Suspense works with supported sources such as lazy code; an ordinary
Effect/IPC read does not suspend merely because a Suspense boundary surrounds it.

Preload code on user intent only when the latency benefit warrants it. Code
preloading grants no authority to send, approve, or start execution. Preserve
supported typed public imports; investigate actual bundle or development costs
before changing import paths or adding dependencies.

Sources: [React lazy](https://react.dev/reference/react/lazy),
[Suspense](https://react.dev/reference/react/Suspense),
[Vite dynamic imports](https://vite.dev/guide/features.html#dynamic-import),
[Lucide tree shaking](https://lucide.dev/guide/react/).

## Desktop preferences and caches

Keep stored presentation records small and validate them at the read boundary.
When changing a persisted shape, choose a version/migration strategy for the actual
compatibility need and account for malformed records and read/write failures.
Preserve pending action identity and recovery information; a migration or cache
miss cannot establish that an uncertain action was never admitted. Separate
discardable preferences from records needed for authoritative status lookup.

Initialize component-owned preferences once per mount when appropriate, rather
than repeatedly reading synchronous storage during render. Shared storage or
derived-result caches need explicit ownership, invalidation, and bounded lifetime;
account for other writers if they exist. Measure before adding a module cache.

Sources: [localStorage](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage),
[state initialization](https://react.dev/reference/react/useState#avoiding-recreating-the-initial-state),
[local action/history requirements](../../../../docs/MVP.md#starting-architecture).

## Growing data and imperative integrations

Choose algorithms by actual workload. Repeated joins or membership checks can
justify an index; finding an extremum needs no full sort. Account for index rebuild
cost, ownership, and readability before replacing small array operations. Cheap
decisive checks and early exits must preserve validation, error selection, and
observable effects. Choose copying collection APIs compatible with the project's
TypeScript libraries and WebView target.

Keep retained data and rendered history bounded. CSS content visibility can
complement paging or virtualization on supported WebViews, but it does not bound
retained data or DOM. Check the configured browser baseline before relying on it.

Prefer declarative CSS/classes for visual state. When an imperative integration
requires layout measurement, group layout reads and invalidating writes to avoid
repeated forced layout. For wheel/touch listeners that only observe, passive
listeners can permit scrolling; retain active listeners when the interaction
requires `preventDefault`, and clean up either kind.

Sources: [React array updates](https://react.dev/learn/updating-arrays-in-state),
[TypeScript library targets](https://www.typescriptlang.org/tsconfig/lib.html),
[WebKit content visibility support](https://webkit.org/blog/15865/webkit-features-in-safari-18-0/),
[layout cost](https://web.dev/articles/avoid-large-complex-layouts-and-layout-thrashing),
[passive listener semantics](https://developer.mozilla.org/en-US/docs/Web/API/EventTarget/addEventListener#using_passive_listeners).

The [adoption research](../../../../docs/research/vercel-react-best-practices-fit.md)
records the pinned Vercel source rules and reasons for these local choices.
