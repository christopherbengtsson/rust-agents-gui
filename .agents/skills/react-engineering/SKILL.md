---
name: react-engineering
description: Write, refactor, or review this repository's React/TypeScript frontend, including component boundaries, state ownership, effects, and Tauri observations.
---

# React engineering

Use these rules for this repository's React/Vite/Tauri frontend, including supported
stack capabilities and intended MVP surfaces that are not implemented yet.
Applicability determines which rules to read; measurement determines whether a
conditional optimization is warranted.

Read [verification policy](../../../docs/agents/verification.md) before applying
this skill. Read [the MVP](../../../docs/MVP.md) for behavior, architecture, or
execution ownership, and the relevant milestone for its acceptance criteria.
Before component work, read [the component guide](../../../src/components/README.md)
and use [shadcn](../shadcn/SKILL.md) for toolkit APIs and conventions. These owners
remain authoritative; rule examples support their implementation.

## How to use

Use the categorized summaries to select each rule whose decision the task touches,
then read that rule's applicability, examples, and caveats. Higher-impact correctness
and ownership concerns take precedence over optional performance recipes. Impact
labels describe consequences here, not measured speedup estimates.

Examples are focused sketches: ordinary imports and surrounding feature code may
be omitted; illustrative adapters/components are identified rather than presented
as existing product APIs. Use current installed-version docs when implementing.
For a complete reference or broad review, [AGENTS.md](AGENTS.md) expands all rules.
Individual rules are the editable source; the full guide is generated.

## Rule index

### Components, state and identity

- [component-responsibilities](rules/component-responsibilities.md) — Extract components, Hooks, and plain functions for different responsibilities.
- [state-ownership](rules/state-ownership.md) — Coordinate state through one explicit owner.
- [derive-during-render](rules/derive-during-render.md) — Compute display values instead of synchronizing redundant state.
- [immutable-state-updates](rules/immutable-state-updates.md) — Preserve immutable snapshots and pure queued updates.
- [lazy-initialization](rules/lazy-initialization.md) — Initialize expensive mounted state without repeated work or commands.
- [component-and-list-identity](rules/component-and-list-identity.md) — Preserve component types, domain keys, and deliberate resets.
- [refs-and-resources](rules/refs-and-resources.md) — Distinguish rendered state, mutable handles, and resource lifetimes.

### Effects and observations

- [events-vs-effects](rules/events-vs-effects.md) — Separate interaction commands from synchronization.
- [synchronization-dependencies](rules/synchronization-dependencies.md) — Track actual reactive inputs and independent processes.
- [effect-events](rules/effect-events.md) — Read nonreactive Effect values without hiding dependencies.
- [subscription-cleanup](rules/subscription-cleanup.md) — Clean up resources even when listener registration resolves late.
- [async-result-targets](rules/async-result-targets.md) — Keep late results bound to their original target and generation.
- [shared-observation-owner](rules/shared-observation-owner.md) — Share a source instead of duplicating Hook instances.
- [external-store-snapshots](rules/external-store-snapshots.md) — Integrate actual stores through stable immutable snapshots.
- [app-initialization](rules/app-initialization.md) — Separate app-load work from mount-owned synchronization.

### Rust/Tauri presentation boundary

- [typed-ipc-contracts](rules/typed-ipc-contracts.md) — Match Rust interfaces and validate untrusted observations.
- [explicit-targeted-intents](rules/explicit-targeted-intents.md) — Leave admission, reconciliation, and retries with the Rust owner.
- [observation-status](rules/observation-status.md) — Separate execution evidence, transport, pending feedback, and uncertainty.

### Async coordination

- [guard-before-read](rules/guard-before-read.md) — Skip unnecessary reads while preserving required effects and errors.
- [schedule-independent-reads](rules/schedule-independent-reads.md) — Schedule actual dependencies with bounded, intentional concurrency.

### Rendering and responsiveness

- [compiler-first-memoization](rules/compiler-first-memoization.md) — Prefer Compiler for routine work and inspect measured exceptions.
- [urgent-and-deferred-rendering](rules/urgent-and-deferred-rendering.md) — Keep inputs urgent while deferring expensive presentation.
- [view-resource-lifetimes](rules/view-resource-lifetimes.md) — Choose visibility without losing required observations.
- [bounded-streams-and-history](rules/bounded-streams-and-history.md) — Bound retained data, rendered history, and stream processing.
- [boolean-render-conditions](rules/boolean-render-conditions.md) — Prevent numeric guards from leaking text into JSX.

### Loading and bundles

- [lazy-surfaces-and-suspense](rules/lazy-surfaces-and-suspense.md) — Load optional surfaces with appropriate local fallbacks.
- [render-and-load-failures](rules/render-and-load-failures.md) — Distinguish render boundaries, async errors, and cached lazy failures.
- [vite-imports-and-preloading](rules/vite-imports-and-preloading.md) — Keep imports analyzable and preloading free of execution intent.
- [supported-public-imports](rules/supported-public-imports.md) — Preserve typed package exports while investigating actual bundle costs.

### Browser integrations

- [layout-measurement](rules/layout-measurement.md) — Group layout reads and invalidating writes in imperative integrations.
- [passive-event-listeners](rules/passive-event-listeners.md) — Match listener cancellation behavior to the interaction.
- [owned-attachment-resources](rules/owned-attachment-resources.md) — Release preview resources without confusing them with durable content.
- [safe-rendered-content](rules/safe-rendered-content.md) — Keep provider text and URLs within explicit rendering contracts.
- [timers-and-long-tasks](rules/timers-and-long-tasks.md) — Budget optional work with supported scheduling and cleanup.
- [svg-and-animation-tradeoffs](rules/svg-and-animation-tradeoffs.md) — Preserve visual/accessibility contracts when optimizing assets.

### Installed UI integrations

- [base-ui-props-and-refs](rules/base-ui-props-and-refs.md) — Forward the complete composed behavior and DOM ref.
- [controlled-inputs-and-submission](rules/controlled-inputs-and-submission.md) — Keep typing synchronous and form intents distinct.
- [tailwind-static-classes](rules/tailwind-static-classes.md) — Make complete variant class candidates discoverable.
- [base-ui-csp-integration](rules/base-ui-csp-integration.md) — Integrate injected elements with the actual desktop CSP.

### Storage and algorithms

- [presentation-records](rules/presentation-records.md) — Validate evolving preferences while preserving uncertain-action identity.
- [owned-bounded-caches](rules/owned-bounded-caches.md) — Define cache identity, invalidation, capacity, and ownership.
- [indexes-and-extrema](rules/indexes-and-extrema.md) — Choose algorithms for actual growing-data workloads.
- [behavior-preserving-hot-loops](rules/behavior-preserving-hot-loops.md) — Optimize measured loops without changing values, effects, or errors.
- [regexp-reuse-and-state](rules/regexp-reuse-and-state.md) — Reuse expressions without leaking mutable matching state.

## Completion

For changed stateful/effectful boundaries, account for owner, target identity, and
applicable stale, cleanup, and failure paths. Review components through their guide
for layer, tokens/variants, documentation, and accessible interaction contracts.
Resolve findings or explain concrete tradeoffs. Apply improvements within the
requested scope; a rule is not an instruction to rewrite unrelated working code.

Select checks through repository policy. Static checks and controlled DOM units
establish different evidence from actual WebView responsiveness, accessibility,
and native-provider qualification. Report performed checks, unperformed
qualification, and justified convention exceptions separately.

## Provenance and maintenance

Adapted selectively from [Vercel at 1a2c79d](https://github.com/vercel-labs/agent-skills/tree/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices)
and primary stack documentation, checked 2026-11-13. The
[coverage research](../../../docs/research/vercel-react-best-practices-fit.md)
records every upstream disposition, dependency coverage, and source corrections.
Rules own maintained recommendations; research records the rationale.

Edit the index/rules, then run `node .agents/skills/react-engineering/scripts/build-guide.mjs`
from the repository root. Use `--check` to validate without writing. Review source
updates deliberately rather than reinstalling over local decisions.
