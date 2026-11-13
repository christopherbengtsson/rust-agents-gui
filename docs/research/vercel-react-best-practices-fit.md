# Vercel React guidance: repository fit and decisions

2026-11-13 · Research and decision snapshot. Repository baseline:
`5c16386f3d77c8665d9af342c17bcfe06cf13550` (`main`). Upstream:
`vercel-labs/agent-skills` at
`1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62`.

## Decision and authority

Extend the existing [React engineering skill][local-skill] as the single entry
point for everyday implementation and review in this repository. Bring in material
gaps and resolve competing recommendations explicitly; do not install a second
React skill or turn every applicable upstream suggestion into a requirement.
Keep maintained instructions compact and put supporting recipes in the
[React performance reference][local-reference]. This note records the comparison,
rather than becoming another set of instructions to load for every frontend task.

Choices below follow the API or platform evidence and the intended application,
not a preference for whichever text already exists. The [MVP][local-mvp] owns
Rust/Tauri responsibilities; the [component guide][local-components] owns component
layers and styling; [verification policy][local-verification] owns checks and
qualification. Those requirements remain authoritative. The maintained React skill
owns the implementation guidance after this change; this note is a dated source
and decision record.

## Inspection scope and relevant baseline

Both remote `main` SHAs were checked with non-mutating `git ls-remote` reads.
The repository main object was available locally, so its files could be inspected
without treating the active workspace-creation feature branch as main. The
comparison read the upstream skill index, all 70 individual rules, section
metadata, generated full guide, metadata, README and guide builder. It inspected
repository AGENTS, MVP, verification policy, component guide, React skill,
package/compiler/lint/TypeScript configuration, architecture ADR and a bounded
sample of the existing observation and preference code. These are source reads,
not a frontend performance audit.

At the baseline, [package.json][local-package] specifies React/React DOM 19.3.1,
Vite 8.3.8, TypeScript 7.0.2 and Tauri 2. The [Vite configuration][local-vite]
enables React Compiler in infer mode, targets React 19, reports diagnostics and
allows recoverable compiler bailouts. The browser build target is Safari 16.4;
[TypeScript library declarations][local-tsconfig] are ES2022 plus DOM.
[The React presentation ADR][local-adr] selects Vite, and
[src/main.tsx][local-main] uses client `createRoot`. There is no Next.js, SSR/RSC,
SWR or server-action dependency in the manifest. Tauri's official guidance describes
a static frontend host, recommends Vite for React SPAs, and does not natively
provide SSR. [Tauri frontend configuration][tauri-frontend]

Main already uses shared observation ownership, concurrent independent reads,
lazy preference initialization, validated preference shapes and request identity
checks. Repository pages are bounded to 32 items in the inspected adapter. These
observations prevent selling familiar practices as discovered defects.
[Connection observations][local-connection], [repository observations][local-repositories],
[repository adapter/preferences][local-adapter]

The intended MVP includes concurrent streamed conversations, retained history and
change inspection. Those needs justify guidance about visibility/resource
lifetimes, bounded growing data, responsive rendering and optional substantial
code. The sampled main shell does not establish those future workloads' costs,
compiler coverage or responsiveness. No source recommendation establishes a
measured speedup in this application. [MVP][local-mvp],
[prior compiler coverage research](react-compiler-app-coverage.md)

## What the upstream package represents

The pinned [SKILL.md][upstream-skill] lists 70 rules across eight categories. It
links individual `rules/` files and a full generated `AGENTS.md`; copying only the
index would leave those relative references missing. The [generated guide][upstream-guide]
is 108,261 bytes and 3,810 lines. [Its builder][upstream-builder] compiles rules
and metadata, so the guide duplicates their presentation rather than providing
independent project policy.

The package is partly Compiler aware: several rules acknowledge automatic
memoization and JSX optimization. It would be inaccurate to reject it as wholly
pre-Compiler guidance. Its caveats and metadata are nevertheless inconsistent:
[metadata.json][upstream-metadata] still says version 1.0.0, January 2026 and 40+
rules, while the index enumerates 70. The SHA, rather than that version string,
identifies this comparison.

## Complete rule disposition

Each upstream rule appears in exactly one matrix row. Cross-category grouping is
intentional when the same actual decision resolves the recommendations. In
particular, the nested-fetch server rule contains a general dependency-scheduling
idea useful beyond RSC.

- **Local** retains the existing local choice with the rationale shown.
- **Upstream** adopts the underlying upstream recommendation, with the stated scope.
- **Merge** combines useful mechanisms with necessary local or API qualifications.
- **Omit** leaves the recipe out of maintained everyday guidance. It does not ban
  a later use justified by an actual feature or measurement.

| Upstream rules | Existing local guidance or gap | Decision, rationale and primary evidence |
| --- | --- | --- |
| [async-cheap-condition-before-await][u-async-cheap-condition-before-await], [async-defer-await][u-async-defer-await] | Explicit intent paths and lifecycle checks; no specific unnecessary-await guidance. | **Merge.** Put cheap relevant guards before unnecessary reads and await only where needed. Preserve required authorization checks, errors, effects and ordering; reordered async work can change behavior. The upstream cheap-condition rule itself preserves fixed effect order. |
| [async-api-routes][u-async-api-routes], [async-dependencies][u-async-dependencies], [async-parallel][u-async-parallel], [server-parallel-nested-fetching][u-server-parallel-nested-fetching] | Shared observation ownership; independent reads already use `Promise.all`. | **Merge.** Schedule independent observations concurrently and start dependent work as its input becomes available. Use native promises without a `better-all` mandate. Preserve target identity, failure handling, lifecycle and intentional sequencing; this is no authorization for parallel mutations or unlimited fan-out. The upstream dependency rule provides a native-promise alternative; [local observation code][local-repositories] demonstrates the concrete seam. |
| [async-suspense-boundaries][u-async-suspense-boundaries] | Explicit stale/loading/error observations; no Suspense recipe. | **Merge.** Keep usable surrounding UI visible and locate a fallback around the unavailable surface. Use Suspense only with a supported source, such as lazy component code. Effect/event-handler IPC reads do not automatically suspend. Omit async RSC examples and uncached requests created during render. [React Suspense][react-suspense] |
| [bundle-analyzable-paths][u-bundle-analyzable-paths], [bundle-conditional][u-bundle-conditional], [bundle-defer-third-party][u-bundle-defer-third-party], [bundle-dynamic-imports][u-bundle-dynamic-imports], [bundle-preload][u-bundle-preload] | No specific optional-code recipe. | **Merge.** For substantial optional UI, use module-scope React `lazy` or statically analyzable Vite `import()` paths, with loading/error ownership. Preload code on likely intent when useful; importing must not admit application work. Drop Next APIs, hydration framing and assumptions that telemetry is required. Do not split today's small controls speculatively. [React lazy][react-lazy], [Vite features][vite-features] |
| [bundle-barrel-imports][u-bundle-barrel-imports] | Supported toolkit APIs and measure-before-optimizing guidance. | **Local.** Keep supported typed public imports and inspect actual bundle/build/dev costs before changing paths. Lucide documents named imports as tree-shakable; Vite prebundles dependencies. Upstream's Next/server cold-start figures do not establish desktop runtime cost. Its own warning about untyped deep paths supports checking exports rather than blindly rewriting imports. [Lucide React][lucide-react], [Vite features][vite-features] |
| [server-after-nonblocking][u-server-after-nonblocking], [server-auth-actions][u-server-auth-actions], [server-cache-lru][u-server-cache-lru], [server-cache-react][u-server-cache-react], [server-dedup-props][u-server-dedup-props], [server-hoist-static-io][u-server-hoist-static-io], [server-no-shared-module-state][u-server-no-shared-module-state], [server-parallel-fetching][u-server-parallel-fetching], [server-serialization][u-server-serialization] | Rust core owns execution/storage; React presents typed observations. | **Omit.** These prescribe Next/SSR/RSC/request facilities absent from this frontend. Keep their recipes out rather than translating backend ownership into React. Rust caching/admission and IPC payload design remain decisions for their actual modules. This is a platform/ownership decision, not a claim that server authentication or payload bounds are unimportant. [Tauri][tauri-frontend], [MVP][local-mvp], [manifest][local-package] |
| [client-event-listeners][u-client-event-listeners], [client-swr-dedup][u-client-swr-dedup] | One owner for shared observations; an actual external store may use `useSyncExternalStore`. | **Local.** Retain explicit shared ownership and correct subscription cleanup; do not mandate SWR to solve duplicated listeners or requests. SWR remains a possible separately justified dependency choice. Ordinary React state fits simple owned observations, while `useSyncExternalStore` specifically integrates an existing subscribable source. [React external stores][react-external], [local skill][local-skill] |
| [client-passive-event-listeners][u-client-passive-event-listeners] | Cleanup/accessibility guidance; no wheel/touch detail. | **Upstream.** When introducing wheel/touch observations that do not cancel default behavior, use passive listeners. Preserve nonpassive behavior where a real gesture/zoom contract requires `preventDefault`, and retain cleanup. This is a conditional recipe, not a directive to introduce listeners. [DOM event listener options][dom-events] |
| [client-localstorage-schema][u-client-localstorage-schema], [js-cache-storage][u-js-cache-storage] | Preferences retain selection/action identity; observations and action results remain authoritative elsewhere. | **Merge.** Store minimal validated shapes, plan schema evolution, and handle failure according to the workflow. Prefer owned initialized state over repeated synchronous reads. Shared caches need explicit ownership/invalidation. Preserve pending action IDs and recovery across migrations; do not copy silent catch/delete examples that can lose uncertain work. [Current preference boundary][local-adapter], [MVP durability][local-mvp] |
| [rerender-defer-reads][u-rerender-defer-reads], [rerender-derived-state][u-rerender-derived-state] | State near consumers; localize frequent updates; CSS for visual responsive states. | **Merge.** Subscribe to the smallest observation that drives output; read genuinely event-only external values on demand when that preserves meaning. A breakpoint observation can avoid continuous pixel updates, but purely visual responsiveness still belongs in CSS. Never substitute the latest selection for an admitted action's original target. [React external stores][react-external], [component guide][local-components], [MVP targeting][local-mvp] |
| [rerender-derived-state-no-effect][u-rerender-derived-state-no-effect], [rerender-move-effect-to-event][u-rerender-move-effect-to-event] | Already derives values during render and sends interaction commands through explicit intent paths. | **Local.** Retain those recommendations without duplicate instructions. They match React's separation of render calculations, event logic and synchronization, with the local skill adding targeted commands and Rust recovery ownership. [React Effect guidance][react-no-effect] |
| [rerender-dependencies][u-rerender-dependencies], [rerender-split-combined-hooks][u-rerender-split-combined-hooks] | Complete reactive dependencies and owned setup/cleanup; coherent Hooks rather than one Hook per component. | **Merge.** Depend on actual synchronization inputs; read relevant primitives and split independent synchronization processes when their lifetimes differ. Keep dependencies complete. Do not remove an object dependency while still reading its other fields, force custom Hook extraction, or manually split routine computations merely to duplicate Compiler. [React dependencies][react-deps], [React Compiler][react-compiler] |
| [rerender-functional-setstate][u-rerender-functional-setstate], [rerender-lazy-state-init][u-rerender-lazy-state-init] | State ownership exists, but updater/initializer rules are implicit. | **Upstream.** Add pure functional updates when the next value depends on pending state, and pure lazy initialization for expensive mount initialization. Direct assignments from arguments remain fine. Initialization is per mount, and development can replay initializers/updaters; no commands or required effects belong there. This correctness rule survives Compiler adoption. [React useState][react-state] |
| [rerender-memo][u-rerender-memo], [rerender-memo-with-default-value][u-rerender-memo-with-default-value], [rerender-simple-expression-in-memo][u-rerender-simple-expression-in-memo], [rendering-hoist-jsx][u-rendering-hoist-jsx] | Compiler-first calculations/callbacks; measured/manual integration exceptions; responsibility-based extraction. | **Local.** Keep those defaults. Skip unnecessary work before guards, but extraction is only one possible solution. Do not prescribe routine `memo`, constant defaults or JSX hoisting. An omitted default parameter inside a function does not itself change the incoming props compared by that function's memo boundary; downstream identity can matter during an actual render. [React memo][react-memo], [Compiler scope][react-compiler], [local memoization research](react-compiler-memoization-guidance.md) |
| [rerender-no-inline-components][u-rerender-no-inline-components] | Domain keys/identity and useful boundaries; Oxlint already checks static components. | **Upstream.** Make stable component-type definitions explicit, normally at module scope. Nested definitions can reset state/focus and restart resources; this does not forbid inline JSX expressions or ordinary event functions. [React state identity][react-identity], [lint configuration][local-lint] |
| [rerender-transitions][u-rerender-transitions], [rerender-use-deferred-value][u-rerender-use-deferred-value], [rendering-usetransition-loading][u-rendering-usetransition-loading] | Localize updates and measure; preserve explicit pending/error/authoritative observations. | **Merge.** Use transitions/deferred values for justified nonurgent rendering, keeping controlled inputs and immediate intent feedback urgent. Explain stale presentation. Retain action identity, stale-result handling and authoritative outcomes; transition pending does not replace them or cancel Rust work. Correct post-`await` handling and avoid compulsory manual memoization. [React transitions][react-transition], [deferred values][react-deferred], [MVP][local-mvp] |
| [rerender-use-ref-transient-values][u-rerender-use-ref-transient-values], [advanced-event-handler-refs][u-advanced-event-handler-refs], [advanced-use-latest][u-advanced-use-latest], [advanced-effect-event-deps][u-advanced-effect-event-deps] | State/ref identity, complete dependencies, resource cleanup and target binding. | **Merge.** Use state for React output, refs for nonrendering mutable values/owned imperative resources, and Effect Events for genuinely nonreactive logic invoked from Effects or their subscriptions. Effect Event identity is intentionally unstable; keep actual reactive dependencies. Do not use latest-value indirection to retarget admitted actions. Ref bridges remain available for concrete integrations, not a universal default. [React refs][react-ref], [Effect Events][react-effect-event] |
| [advanced-init-once][u-advanced-init-once] | Setup/cleanup/re-setup and reconnect status reads; durable execution belongs to Rust. | **Merge.** Distinguish actual app-load initialization from mount-owned synchronization. Explicit entry/module initialization can fit an app-wide frontend resource. A module flag must not hide broken cleanup or suppress required reconnect reads, and cannot establish durable once-only execution. [React application initialization][react-no-effect], [MVP lifecycle][local-mvp] |
| [rendering-activity][u-rendering-activity] | Shared observation owners and local UI state; visibility/resource tradeoffs implicit. | **Merge.** Choose unmounting, CSS visibility or Activity according to state and resource lifetimes. Activity preserves state/DOM but destroys Effects while hidden and recreates them on reveal. Keep observations that must continue outside visibility-owned subtrees; switching views must not stop admitted conversation work. No universal Activity requirement for menus/tabs. [React Activity][react-activity], [MVP concurrency][local-mvp] |
| [rendering-content-visibility][u-rendering-content-visibility] | Bound growing history/data and localize frequent updates. | **Local.** Retain those primary controls. Content visibility can be progressive enhancement, but does not bound retained DOM/data and Safari 18 introduced support, later than this repo's 16.4 baseline. Paging/virtualization is a workload/accessibility choice, not an automatic translation of the upstream 1,000-node example. [WebKit Safari 18][webkit-content], [Vite target][local-vite], [MVP bounds][local-mvp] |
| [rendering-conditional-render][u-rendering-conditional-render] | No explicit numeric-condition guidance. | **Upstream.** Use an explicit boolean comparison or ternary when a numeric/falsy renderable condition could leak `0`/`NaN`. Boolean `&&` remains valid. [React conditional rendering][react-conditional] |
| [rendering-animate-svg-wrapper][u-rendering-animate-svg-wrapper], [rendering-svg-precision][u-rendering-svg-precision] | Toolkit composition, visual contract, CSS/reduced-motion and measurement guidance. | **Omit.** Adding a wrapper does not establish a universal GPU guarantee, and rounding is an asset-specific visual tradeoff. No inspected application evidence requires either recipe. Keep them available for a scoped asset/animation problem rather than changing qualified controls by rule. [Component contract][local-components], [measurement policy][local-verification] |
| [rendering-hydration-no-flicker][u-rendering-hydration-no-flicker], [rendering-hydration-suppress-warning][u-rendering-hydration-suppress-warning] | Client-rendered desktop frontend. | **Omit.** There is no hydration lifecycle here to repair or suppress. Do not introduce inline DOM-patching scripts for that absent problem. [Client entry][local-main], [Tauri frontend model][tauri-frontend] |
| [rendering-resource-hints][u-rendering-resource-hints], [rendering-script-defer-async][u-rendering-script-defer-async] | Vite-managed module entry; no intended MVP remote-script/resource-hint requirement. | **Omit.** Classic parser-blocking script recipes and remote resource hints are not everyday requirements for this application. Revisit when an actual resource demands them; this is no ban on scripts or preloading. [HTML entry][local-html], [Vite features][vite-features] |
| [js-batch-dom-css][u-js-batch-dom-css] | CSS for visual states; preserve component styling ownership. | **Merge.** Prefer declarative CSS/classes for meaningful visual state; when a real imperative integration measures layout, avoid interleaving reads with invalidating writes. Do not convert every inline style to a class or add layout effects without a consumer. [Upstream concrete read/write examples][u-js-batch-dom-css], [component guide][local-components] |
| [js-index-maps][u-js-index-maps], [js-set-map-lookups][u-js-set-map-lookups], [js-min-max-loop][u-js-min-max-loop] | Bounded data and measurement; no specific algorithm examples. | **Merge.** Match structures/algorithms to the real workload: repeated joins/membership can justify an index and extrema do not require sorting. Account for rebuilding, ownership, data size and readability. These are useful options for intended history/change surfaces, not mandatory rewrites of current short arrays. The upstream examples establish the algorithmic distinction, not a measured application speedup. |
| [js-early-exit][u-js-early-exit], [js-length-check-first][u-js-length-check-first] | Preserve application decisions and failure paths. | **Merge.** Use cheap decisive checks and stop once the required result is known, preserving equality semantics, validation requirements, error selection and effects. The upstream first-error rewrite changes its last-error example's behavior; do not treat that as a mechanical optimization. [Upstream examples][u-js-early-exit], [verification policy][local-verification] |
| [js-cache-function-results][u-js-cache-function-results], [js-hoist-regexp][u-js-hoist-regexp] | Compiler/owned state first; performance changes need evidence. | **Local.** Avoid default unbounded module caches and routine manual render memoization. A shared cache needs demonstrated benefit, an owner, bounds and invalidation. Dynamic regex inputs and global/sticky regex state require correctness care; the upstream regex rule itself warns about mutable `lastIndex`. [Compiler scope][react-compiler], [upstream regex caveat][u-js-hoist-regexp] |
| [js-cache-property-access][u-js-cache-property-access], [js-combine-iterations][u-js-combine-iterations], [js-flatmap-filter][u-js-flatmap-filter] | Plain transformations and measurement before optimization. | **Omit.** Do not impose universal loop/property/array-expression rewrites without a relevant workload. Fewer source-level passes do not establish lower runtime cost, and transformations must preserve values/effects. The flatMap example changes empty-string filtering semantics and still constructs callback result arrays. [Upstream examples][u-js-flatmap-filter], [local skill][local-skill] |
| [js-tosorted-immutable][u-js-tosorted-immutable] | Pure rendering and compiler immutability lint; wording can be more explicit. | **Merge.** Treat state/props/observations as immutable; copy before mutation. Do not prescribe `toSorted` specifically with current ES2022 library declarations: copying then sorting is immediately compatible. Check types/runtime before newer APIs and omit routine `useMemo`. [React state arrays][react-state], [TypeScript config][local-tsconfig], [compiler lint][local-lint] |
| [js-request-idle-callback][u-js-request-idle-callback] | Owned effects and required intent-before-effect/durable recovery. | **Omit.** No concrete intended MVP noncritical task currently warrants this scheduler recipe. Any later use needs target support, cancellation, fallback and lifecycle analysis. Required action identity persistence, execution, recovery and cleanup must not depend on opportunistic idle execution. [MVP ownership][local-mvp], [upstream scope/caveats][u-js-request-idle-callback] |

## Important source corrections

The following conclusions are based on comparing the actual examples and current
primary API documentation; they are not stylistic objections to upstream.

**Effect Events.** Upstream's handler-ref rule calls their reference stable, while
its Effect Event dependency rule says identity changes. Current React 19.3 docs
explicitly specify changing identity, exclusion from dependency arrays and
Effect-only usage. Keep the useful separation of reactive synchronization from
nonreactive Effect logic, with React's restrictions. An Effect Event is not a
replacement for target capture or a callback passed to arbitrary consumers.
[React useEffectEvent][react-effect-event]

**Transitions.** Upstream's loading-state rule describes subsequent transitions
as cancelling pending ones, and its post-`await` update lacks the nested transition
currently required by React. React distinguishes interruptible rendering from
async execution, batches concurrent transitions, and says custom async Actions
need their own ordering/abort handling. Deferred values also do not deduplicate
requests. Retain the local action/request model and use these APIs only for their
actual presentation scheduling behavior.
[React useTransition][react-transition], [React useDeferredValue][react-deferred]

**Memoized default parameters.** React memo compares incoming props before
invoking the wrapped component. Therefore, creating an omitted default function
inside that invocation does not by itself defeat that component's parent-prop
memo boundary. When the component does render for another reason, a newly created
default can affect a downstream consumer or Effect. The local decision follows
that actual boundary and Compiler behavior rather than adopting the upstream
example as a reason to hoist all defaults. This is an inference from the documented
memo comparison contract and the position of default evaluation in the example,
not a performance measurement. [React memo][react-memo]

**Behavior-preserving optimization.** In the flatMap rule, the original
`filter(Boolean)` removes an active user's empty name, while the replacement
retains it. In the early-exit rule, the original returns the last observed error,
while the replacement returns the first. Either new behavior could be desirable,
but a performance convention cannot silently choose it. Likewise, changing async
order, dropping Effect dependencies, caching external state or rounding SVG
coordinates requires checking the actual behavioral contract.

**Visibility and platform support.** Activity hides DOM and destroys Effects;
CSS visibility/unmounting have different lifetime consequences. Its state
preservation does not keep hidden subscriptions alive. Content visibility is also
not a cross-baseline substitute for bounded history: WebKit documents its
introduction in Safari 18. These distinctions matter for the intended concurrent
chat UI even though the sampled main shell does not yet demonstrate that workload.
[React Activity][react-activity], [WebKit Safari 18][webkit-content], [MVP][local-mvp]

## Provenance and maintenance

This note credits Vercel Engineering's rule collection as the source of the
comparison and selected ideas. The maintained local guidance uses original prose
and official API references rather than copying upstream examples or vendoring
its generated full guide. Record the pinned SHA and rule IDs when revisiting a
choice; review relevant source diffs and current supported-version documentation
instead of overwriting local guidance with an automatic update.

Upstream declares MIT in [SKILL.md frontmatter][upstream-skill] and its
[repository README][upstream-readme]. The inspected tree did not contain a LICENSE
file, and the repository API reported `license: null` during inspection. No
bundled license text or copyright notice was found to reproduce, and none is
invented here. This is a record of the source's declarations, not an independent
license determination. The unchanged version metadata alone is insufficient to
identify a future update. [Pinned upstream tree][upstream-tree],
[GitHub repository metadata](https://api.github.com/repos/vercel-labs/agent-skills)

## Evidence limits

A mechanical documentation check compared the matrix against the pinned GitHub
tree: all 70 rule IDs occur once in its first column across 31 grouped rows; all
111 used reference labels resolve, and all six local reference targets exist.

This work changes documentation and agent guidance, not application behavior or
requirements. Research used read-only source/network inspection before the
separately authorized documentation implementation. It did not run application
static/build checks, units, performance benchmarks, desktop/accessibility
qualification or native-provider checks. No such results or measured improvement
are claimed. Existing milestone/compiler evidence remains unchanged; any later
implementation selects its checks under [repository verification policy][local-verification].


[local-skill]: ../../.agents/skills/react-engineering/SKILL.md
[local-reference]: ../../.agents/skills/react-engineering/references/react-performance.md
[local-mvp]: ../MVP.md
[local-components]: ../../src/components/README.md
[local-verification]: ../agents/verification.md
[local-package]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/package.json
[local-vite]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/vite.config.ts
[local-tsconfig]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/tsconfig.json
[local-main]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/src/main.tsx
[local-html]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/index.html
[local-lint]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/.oxlintrc.json
[local-adr]: ../adr/0001-react-presentation-with-vite.md
[local-connection]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/src/hooks/use-supervisor-connection.ts
[local-repositories]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/src/hooks/use-repositories.ts
[local-adapter]: https://github.com/christopherbengtsson/rust-agents-gui/blob/5c16386f3d77c8665d9af342c17bcfe06cf13550/src/lib/repositories.ts
[upstream-skill]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/SKILL.md
[upstream-guide]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/AGENTS.md
[upstream-builder]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/packages/react-best-practices-build/src/build.ts
[upstream-metadata]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/metadata.json
[upstream-readme]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/README.md
[upstream-tree]: https://github.com/vercel-labs/agent-skills/tree/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62
[react-compiler]: https://react.dev/learn/react-compiler/introduction
[react-memo]: https://react.dev/reference/react/memo
[react-state]: https://react.dev/reference/react/useState
[react-identity]: https://react.dev/learn/preserving-and-resetting-state
[react-deps]: https://react.dev/learn/removing-effect-dependencies
[react-no-effect]: https://react.dev/learn/you-might-not-need-an-effect
[react-ref]: https://react.dev/reference/react/useRef
[react-effect-event]: https://react.dev/reference/react/useEffectEvent
[react-transition]: https://react.dev/reference/react/useTransition
[react-deferred]: https://react.dev/reference/react/useDeferredValue
[react-activity]: https://react.dev/reference/react/Activity
[react-lazy]: https://react.dev/reference/react/lazy
[react-suspense]: https://react.dev/reference/react/Suspense
[react-external]: https://react.dev/reference/react/useSyncExternalStore
[react-conditional]: https://react.dev/learn/conditional-rendering
[vite-features]: https://vite.dev/guide/features.html
[tauri-frontend]: https://v2.tauri.app/start/frontend/
[lucide-react]: https://lucide.dev/guide/react/
[webkit-content]: https://webkit.org/blog/15865/webkit-features-in-safari-18-0/
[dom-events]: https://dom.spec.whatwg.org/#dom-addeventlisteneroptions-passive
[u-advanced-effect-event-deps]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/advanced-effect-event-deps.md
[u-advanced-event-handler-refs]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/advanced-event-handler-refs.md
[u-advanced-init-once]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/advanced-init-once.md
[u-advanced-use-latest]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/advanced-use-latest.md
[u-async-api-routes]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-api-routes.md
[u-async-cheap-condition-before-await]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-cheap-condition-before-await.md
[u-async-defer-await]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-defer-await.md
[u-async-dependencies]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-dependencies.md
[u-async-parallel]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-parallel.md
[u-async-suspense-boundaries]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/async-suspense-boundaries.md
[u-bundle-analyzable-paths]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/bundle-analyzable-paths.md
[u-bundle-barrel-imports]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/bundle-barrel-imports.md
[u-bundle-conditional]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/bundle-conditional.md
[u-bundle-defer-third-party]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/bundle-defer-third-party.md
[u-bundle-dynamic-imports]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/bundle-dynamic-imports.md
[u-bundle-preload]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/bundle-preload.md
[u-client-event-listeners]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/client-event-listeners.md
[u-client-localstorage-schema]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/client-localstorage-schema.md
[u-client-passive-event-listeners]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/client-passive-event-listeners.md
[u-client-swr-dedup]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/client-swr-dedup.md
[u-js-batch-dom-css]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-batch-dom-css.md
[u-js-cache-function-results]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-cache-function-results.md
[u-js-cache-property-access]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-cache-property-access.md
[u-js-cache-storage]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-cache-storage.md
[u-js-combine-iterations]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-combine-iterations.md
[u-js-early-exit]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-early-exit.md
[u-js-flatmap-filter]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-flatmap-filter.md
[u-js-hoist-regexp]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-hoist-regexp.md
[u-js-index-maps]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-index-maps.md
[u-js-length-check-first]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-length-check-first.md
[u-js-min-max-loop]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-min-max-loop.md
[u-js-request-idle-callback]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-request-idle-callback.md
[u-js-set-map-lookups]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-set-map-lookups.md
[u-js-tosorted-immutable]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-tosorted-immutable.md
[u-rendering-activity]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-activity.md
[u-rendering-animate-svg-wrapper]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-animate-svg-wrapper.md
[u-rendering-conditional-render]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-conditional-render.md
[u-rendering-content-visibility]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-content-visibility.md
[u-rendering-hoist-jsx]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-hoist-jsx.md
[u-rendering-hydration-no-flicker]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-hydration-no-flicker.md
[u-rendering-hydration-suppress-warning]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-hydration-suppress-warning.md
[u-rendering-resource-hints]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-resource-hints.md
[u-rendering-script-defer-async]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-script-defer-async.md
[u-rendering-svg-precision]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-svg-precision.md
[u-rendering-usetransition-loading]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rendering-usetransition-loading.md
[u-rerender-defer-reads]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-defer-reads.md
[u-rerender-dependencies]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-dependencies.md
[u-rerender-derived-state]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-derived-state.md
[u-rerender-derived-state-no-effect]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-derived-state-no-effect.md
[u-rerender-functional-setstate]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-functional-setstate.md
[u-rerender-lazy-state-init]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-lazy-state-init.md
[u-rerender-memo]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-memo.md
[u-rerender-memo-with-default-value]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-memo-with-default-value.md
[u-rerender-move-effect-to-event]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-move-effect-to-event.md
[u-rerender-no-inline-components]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-no-inline-components.md
[u-rerender-simple-expression-in-memo]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-simple-expression-in-memo.md
[u-rerender-split-combined-hooks]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-split-combined-hooks.md
[u-rerender-transitions]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-transitions.md
[u-rerender-use-deferred-value]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-use-deferred-value.md
[u-rerender-use-ref-transient-values]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/rerender-use-ref-transient-values.md
[u-server-after-nonblocking]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-after-nonblocking.md
[u-server-auth-actions]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-auth-actions.md
[u-server-cache-lru]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-cache-lru.md
[u-server-cache-react]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-cache-react.md
[u-server-dedup-props]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-dedup-props.md
[u-server-hoist-static-io]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-hoist-static-io.md
[u-server-no-shared-module-state]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-no-shared-module-state.md
[u-server-parallel-fetching]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-parallel-fetching.md
[u-server-parallel-nested-fetching]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-parallel-nested-fetching.md
[u-server-serialization]: https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/server-serialization.md
