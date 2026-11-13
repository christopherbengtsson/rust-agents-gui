# Vercel React guidance: repository fit and decisions

2026-11-13 · Research and decision snapshot. Repository baseline:
`5c16386f3d77c8665d9af342c17bcfe06cf13550` (`main`). Upstream:
`vercel-labs/agent-skills` at
`1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62`.

## Decision and authority

Maintain the existing [React engineering skill][local-skill] as the single entry
point, with an indexed library of 44 individual `rules/` files containing
explanations, incorrect/correct examples and primary sources. Provide a
[generated full guide][local-guide] from that same inventory. The index and full
guide are two reading formats, not competing policy or separately edited copies.

**Follow-up criterion, 2026-11-13:** cover the installed stack and intended MVP,
including useful existing conventions and conditional optimization recipes. This
supersedes the earlier gap-only/compact-reference criterion and its omissions
based solely on today's small UI or lack of an observed current defect. The
[rule index][local-index] replaces the former `references/react-performance.md`
organization. The finite inventory follows coherent decisions rather than
matching upstream's count or making every sentence a separate file.

Choices follow supported APIs, platform evidence and intended application
responsibilities. The [MVP][local-mvp] owns Rust/Tauri responsibilities; the
[component guide][local-components] owns layers, composition decisions, tokens and
styling; [verification policy][local-verification] owns checks and qualification.
The [shadcn skill][local-shadcn] owns toolkit lookup and detailed component
workflows. Rules link to these authorities rather than duplicate their policy.
No recipe authorizes new product behavior, dependencies or provider execution.
This note records provenance, coverage and decisions; maintained instructions live
in the index and its rule sources.

## Inspection scope and relevant baseline

In the original comparison, both remote `main` SHAs were checked with
non-mutating `git ls-remote` reads.
The repository main object was available locally, so its files could be inspected
without treating the active workspace-creation feature branch as main. The
comparison read the upstream skill index, all 70 individual rules, section
metadata, generated full guide, metadata, README and guide builder. It inspected
repository AGENTS, MVP, verification policy, component guide, React skill,
package/compiler/lint/TypeScript configuration, architecture ADR and a bounded
sample of the existing observation and preference code. These are source reads,
not a frontend performance audit. The follow-up re-read repository policy, MVP,
component guide, package/compiler/TypeScript/WebView settings and this comparison,
and consulted the official integration sources mapped below. It did not inspect
RAT or exercise the application or providers.

At the baseline, [package.json][local-package] specifies React/React DOM 19.3.1,
Vite 8.3.8, TypeScript 7.0.2 and Tauri 2. Installed integrations also include
Base UI 1.9.0, Tailwind 4.3.5, the Tauri dialog plugin, Lucide React,
class-variance-authority and `cn`; verification uses Vitest 5, jsdom 31, React
Testing Library 16 and user-event 14. The checked-in manifest and lockfile, not
a rule example, determine available dependencies. The [Vite configuration][local-vite]
enables React Compiler in infer mode, targets React 19, reports diagnostics and
allows recoverable compiler bailouts. The browser build target is Safari 16.4;
[TypeScript library declarations][local-tsconfig] are ES2022 plus DOM.
[The React presentation ADR][local-adr] selects Vite, and
[src/main.tsx][local-main] uses client `createRoot`. There is no Next.js, SSR/RSC,
SWR or server-action dependency in the manifest. Tauri's official guidance describes
a static frontend host, recommends Vite for React SPAs, and does not natively
provide SSR. [Tauri frontend configuration][tauri-frontend]

The inspected main baseline already uses shared observation ownership, concurrent
independent reads,
lazy preference initialization, validated preference shapes and request identity
checks. Repository pages are bounded to 32 items in the inspected adapter. These
observations establish existing coverage; they do not remove those practices from
the maintained inventory or establish that the growing MVP will be fast.
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

## Complete upstream rule disposition

Each of the 70 pinned upstream IDs appears once in the first column. A retained
local choice now maps to named maintained files as well. **Retain** preserves the
useful mechanism; **Adapt** gives its stack/ownership qualifications; **Exclude**
identifies an actual architecture mismatch. Conditional optimization guidance is
retained for relevant workloads without asserting a measured improvement or
requiring speculative rewrites. Rule links are the maintained destinations.

| Pinned upstream rules | Maintained rule files | Disposition and rationale |
| --- | --- | --- |
| [async-cheap-condition-before-await][u-async-cheap-condition-before-await], [async-defer-await][u-async-defer-await] | [guard-before-read][r-guard-before-read] | Adapt. Guard unnecessary reads and defer awaits while preserving validation, errors, effects and required ordering. |
| [async-api-routes][u-async-api-routes], [async-dependencies][u-async-dependencies], [async-parallel][u-async-parallel], [server-parallel-nested-fetching][u-server-parallel-nested-fetching] | [schedule-independent-reads][r-schedule-independent-reads] | Adapt the general dependency graph to bounded, independent observations. Use native promises; do not import route/RSC ownership or mandate better-all. Preserve failure behavior and mutation order. |
| [async-suspense-boundaries][u-async-suspense-boundaries] | [lazy-surfaces-and-suspense][r-lazy-surfaces-and-suspense], [observation-status][r-observation-status] | Adapt. Put fallbacks around unavailable supported surfaces; ordinary Effect/IPC reads do not automatically suspend. Retain explicit observation loading/stale/error states. [React Suspense][react-suspense] |
| [bundle-analyzable-paths][u-bundle-analyzable-paths], [bundle-conditional][u-bundle-conditional], [bundle-defer-third-party][u-bundle-defer-third-party], [bundle-dynamic-imports][u-bundle-dynamic-imports], [bundle-preload][u-bundle-preload] | [lazy-surfaces-and-suspense][r-lazy-surfaces-and-suspense], [vite-imports-and-preloading][r-vite-imports-and-preloading], [render-and-load-failures][r-render-and-load-failures] | Adapt for substantial optional MVP surfaces, using module-scope lazy declarations and analyzable Vite imports. Include loading/load failure ownership and conditional intent preloading, without Next APIs or assumed telemetry. Importing code cannot admit application work. [React lazy][react-lazy], [Vite features][vite-features] |
| [bundle-barrel-imports][u-bundle-barrel-imports] | [supported-public-imports][r-supported-public-imports] | Adapt. Use supported typed exports and measure bundle/build/dev costs before changing paths. Named Lucide imports are tree-shakable; Next/server cold-start figures do not establish desktop cost. [Lucide React][lucide-react], [Vite features][vite-features] |
| [server-after-nonblocking][u-server-after-nonblocking], [server-auth-actions][u-server-auth-actions], [server-cache-lru][u-server-cache-lru], [server-cache-react][u-server-cache-react], [server-dedup-props][u-server-dedup-props], [server-hoist-static-io][u-server-hoist-static-io], [server-no-shared-module-state][u-server-no-shared-module-state], [server-parallel-fetching][u-server-parallel-fetching], [server-serialization][u-server-serialization] | No corresponding frontend recipe | Exclude these Next/SSR/RSC/request-specific recipes by architecture. Frontend [typed IPC][r-typed-ipc-contracts] and [targeted intents][r-explicit-targeted-intents] cover their actual boundary without relocating Rust execution, storage, admission or recovery policy. [Tauri frontend][tauri-frontend], [MVP][local-mvp] |
| [client-event-listeners][u-client-event-listeners], [client-swr-dedup][u-client-swr-dedup] | [shared-observation-owner][r-shared-observation-owner], [subscription-cleanup][r-subscription-cleanup], [external-store-snapshots][r-external-store-snapshots] | Adapt. One owner serves shared observations; repeated custom Hook calls do not share state. Integrate an actual subscribable source with useSyncExternalStore and cached immutable snapshots. No SWR requirement. [React external stores][react-external] |
| [client-passive-event-listeners][u-client-passive-event-listeners] | [passive-event-listeners][r-passive-event-listeners] | Retain conditionally for observing wheel/touch events. Preserve active listeners where preventDefault is required and clean up either kind. [DOM options][dom-events] |
| [client-localstorage-schema][u-client-localstorage-schema], [js-cache-storage][u-js-cache-storage] | [presentation-records][r-presentation-records], [owned-bounded-caches][r-owned-bounded-caches], [lazy-initialization][r-lazy-initialization] | Adapt. Keep validated minimal records, explicit evolution/failure behavior and owned initialized reads. Preserve pending action/status-lookup identity through cache misses or migrations; silent deletion cannot prove an uncertain action never ran. [MVP][local-mvp], [local preference boundary][local-adapter] |
| [rerender-defer-reads][u-rerender-defer-reads], [rerender-derived-state][u-rerender-derived-state] | [state-ownership][r-state-ownership], [shared-observation-owner][r-shared-observation-owner], [explicit-targeted-intents][r-explicit-targeted-intents] | Adapt. Subscribe to output-relevant observations; read genuinely event-only values when their timing and target meaning permit. CSS still owns purely visual responsiveness. Latest selection cannot replace an admitted action target. [React external stores][react-external], [component guide][local-components] |
| [rerender-derived-state-no-effect][u-rerender-derived-state-no-effect], [rerender-move-effect-to-event][u-rerender-move-effect-to-event] | [derive-during-render][r-derive-during-render], [events-vs-effects][r-events-vs-effects] | Retain existing local guidance as named recipes: derive display values during render, submit interaction commands from explicit intent paths and use Effects for synchronization. [React Effect guidance][react-no-effect] |
| [rerender-dependencies][u-rerender-dependencies], [rerender-split-combined-hooks][u-rerender-split-combined-hooks] | [synchronization-dependencies][r-synchronization-dependencies], [component-responsibilities][r-component-responsibilities] | Adapt. Depend on actual synchronization inputs, keep dependencies complete and separate independent processes when lifetimes differ. A Hook per component and routine calculation splitting are unnecessary. [React dependencies][react-deps], [Compiler][react-compiler] |
| [rerender-functional-setstate][u-rerender-functional-setstate], [rerender-lazy-state-init][u-rerender-lazy-state-init] | [immutable-state-updates][r-immutable-state-updates], [lazy-initialization][r-lazy-initialization] | Retain. Pure functional updates apply when next state depends on pending state; pure lazy initializers avoid repeated expensive mount initialization. Neither is an action execution path, and development may replay them. [React state][react-state] |
| [rerender-memo][u-rerender-memo], [rerender-memo-with-default-value][u-rerender-memo-with-default-value], [rerender-simple-expression-in-memo][u-rerender-simple-expression-in-memo], [rendering-hoist-jsx][u-rendering-hoist-jsx] | [compiler-first-memoization][r-compiler-first-memoization], [component-responsibilities][r-component-responsibilities] | Adapt. Compiler handles routine render memoization; keep measured/manual integration exceptions and inspect skipped compilation. Do not prescribe constant defaults, manual memo or JSX hoisting universally. An omitted default created inside a function does not itself invalidate that function’s incoming-prop memo comparison. [React memo][react-memo], [Compiler][react-compiler] |
| [rerender-no-inline-components][u-rerender-no-inline-components] | [component-and-list-identity][r-component-and-list-identity] | Retain. Stable component types, normally module-scoped, protect state/focus/resources; domain keys express list identity. Ordinary inline callbacks and JSX expressions are separate choices. [React identity][react-identity] |
| [rerender-transitions][u-rerender-transitions], [rerender-use-deferred-value][u-rerender-use-deferred-value], [rendering-usetransition-loading][u-rendering-usetransition-loading] | [urgent-and-deferred-rendering][r-urgent-and-deferred-rendering], [async-result-targets][r-async-result-targets], [observation-status][r-observation-status] | Adapt for expensive history/filter/change surfaces. Controlled inputs and immediate intent feedback stay urgent; explain stale content. Rendering priority does not cancel backend work, deduplicate requests or order custom async responses. Apply supported post-await transition handling. [Transitions][react-transition], [deferred values][react-deferred] |
| [rerender-use-ref-transient-values][u-rerender-use-ref-transient-values], [advanced-event-handler-refs][u-advanced-event-handler-refs], [advanced-use-latest][u-advanced-use-latest], [advanced-effect-event-deps][u-advanced-effect-event-deps] | [refs-and-resources][r-refs-and-resources], [effect-events][r-effect-events], [async-result-targets][r-async-result-targets] | Adapt. State drives output, refs retain nonrendering mutable values/resources, and Effect Events handle genuinely nonreactive Effect logic. Effect Events have changing identity, remain local to their Effect owner and cannot hide dependencies or retarget admitted work. [Refs][react-ref], [Effect Events][react-effect-event] |
| [advanced-init-once][u-advanced-init-once] | [app-initialization][r-app-initialization], [subscription-cleanup][r-subscription-cleanup] | Adapt. Distinguish app-wide frontend initialization from mount synchronization. A module flag cannot replace cleanup, reconnect reads or durable once-only execution. [React initialization][react-no-effect], [MVP lifecycle][local-mvp] |
| [rendering-activity][u-rendering-activity] | [view-resource-lifetimes][r-view-resource-lifetimes], [shared-observation-owner][r-shared-observation-owner] | Adapt. Choose unmounting, CSS visibility or Activity according to state/DOM/effect lifetimes. Activity cleans up hidden Effects and recreates them on reveal; keep continuing observations above visibility-dependent subtrees. [Activity][react-activity] |
| [rendering-content-visibility][u-rendering-content-visibility] | [bounded-streams-and-history][r-bounded-streams-and-history] | Adapt conditionally. Paging/bounded retention remain primary; CSS content visibility neither bounds retained data/DOM nor supports the Safari 16.4 baseline. It can complement a qualified fallback on supporting WebViews. [WebKit Safari 18][webkit-content] |
| [rendering-conditional-render][u-rendering-conditional-render] | [boolean-render-conditions][r-boolean-render-conditions] | Retain explicit boolean comparisons or ternaries for numeric guards that could leak 0/NaN; boolean && remains valid. [React conditional rendering][react-conditional] |
| [rendering-animate-svg-wrapper][u-rendering-animate-svg-wrapper], [rendering-svg-precision][u-rendering-svg-precision] | [svg-and-animation-tradeoffs][r-svg-and-animation-tradeoffs] | Adapt as conditional asset/animation guidance for the installed icon and CSS stack. A wrapper does not establish universal GPU acceleration; precision changes require visual evidence. Scope by rendering/animation cost and preserve accessibility/reduced motion. Absence of today’s defect is not an omission criterion. [SVG][svg-spec], [CSS transforms][css-transforms], [component guide][local-components] |
| [rendering-hydration-no-flicker][u-rendering-hydration-no-flicker], [rendering-hydration-suppress-warning][u-rendering-hydration-suppress-warning] | No corresponding frontend recipe | Exclude hydration recipes by client createRoot architecture; no hydration lifecycle exists here to repair or suppress. [Client entry][local-main], [Tauri frontend][tauri-frontend] |
| [rendering-resource-hints][u-rendering-resource-hints], [rendering-script-defer-async][u-rendering-script-defer-async] | [vite-imports-and-preloading][r-vite-imports-and-preloading] | Adapt to Vite-managed module entry, assets and code preloading. Explain resource-specific hints only when useful and CSP-compatible; do not copy Next or classic remote-script defaults into the desktop build. [Vite features][vite-features], [Vite assets][vite-assets], [HTML scripts][html-scripts] |
| [js-batch-dom-css][u-js-batch-dom-css] | [layout-measurement][r-layout-measurement] | Adapt. Prefer declarative visual state; real imperative integrations should avoid interleaving layout reads with invalidating writes. This does not mandate a class rewrite or layout Effect for every style. [Rendering cycle][html-rendering], [component guide][local-components] |
| [js-index-maps][u-js-index-maps], [js-set-map-lookups][u-js-set-map-lookups], [js-min-max-loop][u-js-min-max-loop] | [indexes-and-extrema][r-indexes-and-extrema] | Retain conditional algorithm choices for growing history/change data: repeated joins/membership may justify indexes; an extremum needs no full sort. Account for rebuild cost, ownership and readability; examples establish complexity, not application speedups. [ECMAScript collections][ecma-collections] |
| [js-early-exit][u-js-early-exit], [js-length-check-first][u-js-length-check-first], [js-cache-property-access][u-js-cache-property-access], [js-combine-iterations][u-js-combine-iterations], [js-flatmap-filter][u-js-flatmap-filter] | [behavior-preserving-hot-loops][r-behavior-preserving-hot-loops], [guard-before-read][r-guard-before-read] | Adapt as measured hot-path recipes rather than universal loop/property/array rewrites. Preserve empty-string handling, validation, first/last-error selection, ordering and effects. Fewer source passes do not establish lower cost. [ECMAScript arrays][ecma-arrays], [verification policy][local-verification] |
| [js-cache-function-results][u-js-cache-function-results] | [owned-bounded-caches][r-owned-bounded-caches], [compiler-first-memoization][r-compiler-first-memoization] | Adapt. Shared caches require an owner, bounds, invalidation and demonstrated benefit. Compiler/render memoization is separate from caching external state or durable decisions. [Compiler scope][react-compiler] |
| [js-hoist-regexp][u-js-hoist-regexp] | [regexp-reuse-and-state][r-regexp-reuse-and-state] | Adapt conditionally for repeated nontrivial work. Preserve dynamic inputs and global/sticky lastIndex semantics; hoisting is not automatically a safe pure optimization. [ECMAScript RegExp][ecma-regexp] |
| [js-tosorted-immutable][u-js-tosorted-immutable] | [immutable-state-updates][r-immutable-state-updates] | Adapt immutability, not a blanket toSorted API choice. Copy then sort fits current ES2022 declarations; check types/runtime before newer APIs. [React arrays][react-arrays], [TypeScript libraries][ts-libraries] |
| [js-request-idle-callback][u-js-request-idle-callback] | [timers-and-long-tasks][r-timers-and-long-tasks] | Adapt into capability-aware cooperative work with fallback, cancellation and ownership. Required identity persistence, execution, recovery and cleanup cannot depend on opportunistic idle execution; scheduling also does not make one large synchronous calculation interruptible. [Idle callbacks][idle-spec], [HTML timers][html-timers] |

## Stack and intended-MVP coverage

This map includes existing core React guidance and substantive integrations beyond
the upstream collection. Capability coverage determines inclusion; an unused API
does not require an added library or a speculative component. The installed
manifest/configuration owns availability. Official documentation supplies API facts,
while local product and verification documents supply requirements.

| Dependency or intended capability | Maintained rule coverage | Official API evidence and local authority |
| --- | --- | --- |
| React 19.3 components, state and identity | [component-responsibilities][r-component-responsibilities], [state-ownership][r-state-ownership], [derive-during-render][r-derive-during-render], [immutable-state-updates][r-immutable-state-updates], [lazy-initialization][r-lazy-initialization], [component-and-list-identity][r-component-and-list-identity], [refs-and-resources][r-refs-and-resources] | [State structure][react-state-structure], [Effect guidance][react-no-effect], [state][react-state], [identity][react-identity], [refs][react-ref]; [component guide][local-components]. |
| React synchronization and app lifecycle | [events-vs-effects][r-events-vs-effects], [synchronization-dependencies][r-synchronization-dependencies], [effect-events][r-effect-events], [subscription-cleanup][r-subscription-cleanup], [async-result-targets][r-async-result-targets], [shared-observation-owner][r-shared-observation-owner], [external-store-snapshots][r-external-store-snapshots], [app-initialization][r-app-initialization] | [Effects][react-effect], [dependencies][react-deps], [Effect Events][react-effect-event], [external stores][react-external]; [MVP][local-mvp]. |
| Tauri 2 typed commands/events and native dialog results | [typed-ipc-contracts][r-typed-ipc-contracts], [explicit-targeted-intents][r-explicit-targeted-intents], [observation-status][r-observation-status], [guard-before-read][r-guard-before-read], [schedule-independent-reads][r-schedule-independent-reads], [async-result-targets][r-async-result-targets], [subscription-cleanup][r-subscription-cleanup] | [Calling Rust][tauri-rust], [frontend events/channels][tauri-events], [dialog API][tauri-dialog]. Dialog cancellation returns null; closing/changing a React dialog does not cancel its pending native result. Capture request/target identity and ignore late results for an obsolete owner. Core reconciliation and retry remain Rust-owned under [MVP][local-mvp]. |
| Compiler-enabled React and interactive history/change surfaces | [compiler-first-memoization][r-compiler-first-memoization], [urgent-and-deferred-rendering][r-urgent-and-deferred-rendering], [view-resource-lifetimes][r-view-resource-lifetimes], [bounded-streams-and-history][r-bounded-streams-and-history], [boolean-render-conditions][r-boolean-render-conditions] | [Compiler][react-compiler], [transitions][react-transition], [deferred values][react-deferred], [Activity][react-activity], [conditions][react-conditional], [WebKit support][webkit-content]; [MVP responsiveness/bounds][local-mvp]. |
| Vite 8 static desktop build, optional large surfaces and import costs | [lazy-surfaces-and-suspense][r-lazy-surfaces-and-suspense], [render-and-load-failures][r-render-and-load-failures], [vite-imports-and-preloading][r-vite-imports-and-preloading], [supported-public-imports][r-supported-public-imports] | [React lazy][react-lazy], [Suspense][react-suspense], [boundaries][react-boundary], [Vite features][vite-features], [Vite assets][vite-assets], [Lucide][lucide-react]; [Tauri frontend][tauri-frontend]. |
| WebView layout, gestures, image attachments and rendered provider output | [layout-measurement][r-layout-measurement], [passive-event-listeners][r-passive-event-listeners], [owned-attachment-resources][r-owned-attachment-resources], [safe-rendered-content][r-safe-rendered-content], [timers-and-long-tasks][r-timers-and-long-tasks], [svg-and-animation-tradeoffs][r-svg-and-animation-tradeoffs] | [React DOM][react-dom-common], [DOM listeners][dom-events], [File API URL lifetime][file-api], [HTML timers][html-timers], [idle tasks][idle-spec], [SVG][svg-spec], [CSS transforms][css-transforms]. Respect configured CSP and the Safari 16.4/ES2022 baseline. The [MVP][local-mvp] justifies attachment/history handling without choosing a Markdown, editor, worker or virtualization dependency. |
| Base UI 1.9, generated shadcn consumers, CVA/cn and Tailwind 4 | [base-ui-props-and-refs][r-base-ui-props-and-refs], [controlled-inputs-and-submission][r-controlled-inputs-and-submission], [tailwind-static-classes][r-tailwind-static-classes], [base-ui-csp-integration][r-base-ui-csp-integration] | [Composition][base-composition], [forms][base-forms], [React inputs][react-input], [Tailwind class detection][tailwind-detection], [CSP Provider][base-csp]. Forward composition props/ref and preserve handler semantics; map variants to complete classes. The [component guide][local-components] and [shadcn skill][local-shadcn] own design/composition workflow, including CVA/cn usage; no extra rule merely for a utility dependency. |
| Desktop preferences and growing data transformations | [presentation-records][r-presentation-records], [owned-bounded-caches][r-owned-bounded-caches], [indexes-and-extrema][r-indexes-and-extrema], [behavior-preserving-hot-loops][r-behavior-preserving-hot-loops], [regexp-reuse-and-state][r-regexp-reuse-and-state] | [Web Storage][web-storage], [ECMAScript arrays][ecma-arrays], [collections][ecma-collections], [RegExp][ecma-regexp], [TypeScript libraries][ts-libraries]; [MVP durable identity][local-mvp]. |
| Vitest 5, jsdom 31, React Testing Library 16 and user-event 14 | [subscription-cleanup][r-subscription-cleanup], [async-result-targets][r-async-result-targets], [controlled-inputs-and-submission][r-controlled-inputs-and-submission], [render-and-load-failures][r-render-and-load-failures], [layout-measurement][r-layout-measurement] | [Vitest environments][vitest-environments], [RTL render/rerender/unmount/cleanup][rtl-api], [user-event interactions][user-event], [jsdom limitations][jsdom-readme]. Existing [verification policy][local-verification] governs test selection, doubles and qualification; these tools provide controlled application-decision observations, not native dialog, WebView layout, screen-reader or responsiveness proof. No new test category or dependency is introduced. |

The map covers all 44 maintained rule IDs. Rule files provide examples for these
decisions; the generated full guide contains the same material. Baseline-local
examples demonstrate a seam and do not prove future workload performance.

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
comparison and selected ideas. The maintained local rules use original prose and
examples with official API references rather than vendoring upstream examples or
its full guide. The local full guide is generated from those local rule sources.
Record the pinned SHA and rule IDs when revisiting a choice; review relevant source
diffs and current supported-version documentation
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

The original comparison mechanically checked all 70 upstream IDs against the
pinned GitHub tree. The follow-up keeps that exact upstream inventory, maps every
retained/adapted choice to the maintained rules, and adds a stack-capability map
covering all 44 local IDs. Documentation validation should check matrix coverage,
local links, rule/index agreement and generated-guide reproducibility; it does
not establish API execution or application performance.

This follow-up changes documentation and agent guidance. It used read-only
source/network research followed by the authorized documentation edits. It did
not run application static/build checks, units, performance benchmarks,
desktop/accessibility qualification or native-provider checks, and did not inspect
RAT. No such results or measured improvement are claimed. Existing milestone and
compiler evidence remains unchanged; later implementation selects checks under
[repository verification policy][local-verification].

[local-skill]: ../../.agents/skills/react-engineering/SKILL.md
[local-index]: ../../.agents/skills/react-engineering/SKILL.md#rule-index
[local-guide]: ../../.agents/skills/react-engineering/AGENTS.md
[local-shadcn]: ../../.agents/skills/shadcn/SKILL.md
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

[r-component-responsibilities]: ../../.agents/skills/react-engineering/rules/component-responsibilities.md
[r-state-ownership]: ../../.agents/skills/react-engineering/rules/state-ownership.md
[r-derive-during-render]: ../../.agents/skills/react-engineering/rules/derive-during-render.md
[r-immutable-state-updates]: ../../.agents/skills/react-engineering/rules/immutable-state-updates.md
[r-lazy-initialization]: ../../.agents/skills/react-engineering/rules/lazy-initialization.md
[r-component-and-list-identity]: ../../.agents/skills/react-engineering/rules/component-and-list-identity.md
[r-refs-and-resources]: ../../.agents/skills/react-engineering/rules/refs-and-resources.md
[r-events-vs-effects]: ../../.agents/skills/react-engineering/rules/events-vs-effects.md
[r-synchronization-dependencies]: ../../.agents/skills/react-engineering/rules/synchronization-dependencies.md
[r-effect-events]: ../../.agents/skills/react-engineering/rules/effect-events.md
[r-subscription-cleanup]: ../../.agents/skills/react-engineering/rules/subscription-cleanup.md
[r-async-result-targets]: ../../.agents/skills/react-engineering/rules/async-result-targets.md
[r-shared-observation-owner]: ../../.agents/skills/react-engineering/rules/shared-observation-owner.md
[r-external-store-snapshots]: ../../.agents/skills/react-engineering/rules/external-store-snapshots.md
[r-app-initialization]: ../../.agents/skills/react-engineering/rules/app-initialization.md
[r-typed-ipc-contracts]: ../../.agents/skills/react-engineering/rules/typed-ipc-contracts.md
[r-explicit-targeted-intents]: ../../.agents/skills/react-engineering/rules/explicit-targeted-intents.md
[r-observation-status]: ../../.agents/skills/react-engineering/rules/observation-status.md
[r-guard-before-read]: ../../.agents/skills/react-engineering/rules/guard-before-read.md
[r-schedule-independent-reads]: ../../.agents/skills/react-engineering/rules/schedule-independent-reads.md
[r-compiler-first-memoization]: ../../.agents/skills/react-engineering/rules/compiler-first-memoization.md
[r-urgent-and-deferred-rendering]: ../../.agents/skills/react-engineering/rules/urgent-and-deferred-rendering.md
[r-view-resource-lifetimes]: ../../.agents/skills/react-engineering/rules/view-resource-lifetimes.md
[r-bounded-streams-and-history]: ../../.agents/skills/react-engineering/rules/bounded-streams-and-history.md
[r-boolean-render-conditions]: ../../.agents/skills/react-engineering/rules/boolean-render-conditions.md
[r-lazy-surfaces-and-suspense]: ../../.agents/skills/react-engineering/rules/lazy-surfaces-and-suspense.md
[r-render-and-load-failures]: ../../.agents/skills/react-engineering/rules/render-and-load-failures.md
[r-vite-imports-and-preloading]: ../../.agents/skills/react-engineering/rules/vite-imports-and-preloading.md
[r-supported-public-imports]: ../../.agents/skills/react-engineering/rules/supported-public-imports.md
[r-layout-measurement]: ../../.agents/skills/react-engineering/rules/layout-measurement.md
[r-passive-event-listeners]: ../../.agents/skills/react-engineering/rules/passive-event-listeners.md
[r-owned-attachment-resources]: ../../.agents/skills/react-engineering/rules/owned-attachment-resources.md
[r-safe-rendered-content]: ../../.agents/skills/react-engineering/rules/safe-rendered-content.md
[r-timers-and-long-tasks]: ../../.agents/skills/react-engineering/rules/timers-and-long-tasks.md
[r-svg-and-animation-tradeoffs]: ../../.agents/skills/react-engineering/rules/svg-and-animation-tradeoffs.md
[r-base-ui-props-and-refs]: ../../.agents/skills/react-engineering/rules/base-ui-props-and-refs.md
[r-controlled-inputs-and-submission]: ../../.agents/skills/react-engineering/rules/controlled-inputs-and-submission.md
[r-tailwind-static-classes]: ../../.agents/skills/react-engineering/rules/tailwind-static-classes.md
[r-base-ui-csp-integration]: ../../.agents/skills/react-engineering/rules/base-ui-csp-integration.md
[r-presentation-records]: ../../.agents/skills/react-engineering/rules/presentation-records.md
[r-owned-bounded-caches]: ../../.agents/skills/react-engineering/rules/owned-bounded-caches.md
[r-indexes-and-extrema]: ../../.agents/skills/react-engineering/rules/indexes-and-extrema.md
[r-behavior-preserving-hot-loops]: ../../.agents/skills/react-engineering/rules/behavior-preserving-hot-loops.md
[r-regexp-reuse-and-state]: ../../.agents/skills/react-engineering/rules/regexp-reuse-and-state.md

[react-state-structure]: https://react.dev/learn/choosing-the-state-structure
[react-effect]: https://react.dev/reference/react/useEffect
[react-arrays]: https://react.dev/learn/updating-arrays-in-state
[react-boundary]: https://react.dev/reference/react/Component#catching-rendering-errors-with-an-error-boundary
[react-dom-common]: https://react.dev/reference/react-dom/components/common
[react-input]: https://react.dev/reference/react-dom/components/input
[tauri-rust]: https://v2.tauri.app/develop/calling-rust/
[tauri-events]: https://v2.tauri.app/develop/calling-frontend/
[tauri-dialog]: https://v2.tauri.app/reference/javascript/dialog/
[vite-assets]: https://vite.dev/guide/assets.html
[base-composition]: https://base-ui.com/react/handbook/composition
[base-forms]: https://base-ui.com/react/handbook/forms
[base-csp]: https://base-ui.com/react/utils/csp-provider
[tailwind-detection]: https://tailwindcss.com/docs/detecting-classes-in-source-files
[file-api]: https://www.w3.org/TR/FileAPI/
[html-timers]: https://html.spec.whatwg.org/multipage/timers-and-user-prompts.html#timers
[html-scripts]: https://html.spec.whatwg.org/multipage/scripting.html#the-script-element
[html-rendering]: https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering
[idle-spec]: https://w3c.github.io/requestidlecallback/
[svg-spec]: https://www.w3.org/TR/SVG2/
[css-transforms]: https://www.w3.org/TR/css-transforms-1/
[ecma-arrays]: https://tc39.es/ecma262/multipage/indexed-collections.html
[ecma-collections]: https://tc39.es/ecma262/multipage/keyed-collections.html
[ecma-regexp]: https://tc39.es/ecma262/multipage/text-processing.html#sec-regexp-regular-expression-objects
[ts-libraries]: https://www.typescriptlang.org/tsconfig/lib.html
[web-storage]: https://html.spec.whatwg.org/multipage/webstorage.html
[vitest-environments]: https://vitest.dev/guide/environment.html
[rtl-api]: https://testing-library.com/docs/react-testing-library/api/
[user-event]: https://testing-library.com/docs/user-event/intro/
[jsdom-readme]: https://github.com/jsdom/jsdom#unimplemented-parts-of-the-web-platform
