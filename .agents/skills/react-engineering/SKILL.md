---
name: react-engineering
description: Write, refactor, or review this repository's React/TypeScript frontend, including component boundaries, state ownership, effects, and Tauri observations.
---

# React engineering

Read [testing and verification](../../../docs/agents/verification.md) before
applying this skill. For behavior, architecture, or execution ownership, also read
[MVP.md](../../../docs/MVP.md) and the relevant milestone. Repository policy owns
scope and verification; this skill guides implementation and review within it.
Apply improvements to the requested feature and touched boundaries.

## Component workflow

Before component work, read the [component guide](../../../src/components/README.md).
Inspect existing components, theme, and the relevant official toolkit docs before
choosing a layer, customization, or new abstraction. The guide owns these decisions;
use [shadcn](../shadcn/SKILL.md) for toolkit APIs and detailed composition rules.

## Component boundaries

- Extract a component for a coherent UI contract, repeated composition, or useful
  update boundary. Extract a Hook for a coherent React capability whose state or
  synchronization obscures the component or needs reuse. These are independent
  choices: a simple feature can keep its logic and markup together.
- Use container/view separation when it clarifies dependencies. Choose boundaries
  by responsibility rather than paired files, line counts, or a Hook per component.
  Keep ordinary transformations and command construction in plain functions when
  they need no React lifecycle. Keep component type definitions stable, normally at
  module scope: defining a component during render can reset its state, focus, and
  resource lifecycle. Ordinary inline callbacks and JSX are separate choices.

## State and identity

- Keep drafts, disclosure, and selection near their consumers; lift coordinated
  state to an explicit common owner. A visual component may own interaction state.
  Compute derived values during render. Use discriminated unions when they prevent
  contradictory states rather than spreading related state across optional fields.
- Treat props, state, and observations as immutable; copy before mutating. Use pure
  functional updates when the next value depends on pending state, and pure lazy
  initializers for expensive mount initialization. Initializers and updaters can be
  replayed in development; application actions belong in explicit intent paths.
  Use boolean conditions or ternaries when a numeric JSX guard could render `0`.
- Separate authoritative observations from transport status and local pending UI.
  Present stale, disconnected, error, empty, or unknown states when the workflow
  produces them. A lost connection alone cannot establish an execution outcome.
- Use stable domain IDs for list identity and explicit action targets. React's
  `key` is not passed as a prop. Bind async results and intent callbacks to their
  actual workspace/conversation/turn so a selection change cannot retarget work.

## Effects and the Tauri boundary

- Keep rendering pure. Put interaction-driven commands in explicit intent paths;
  use Effects to synchronize observations or external UI resources. View mounts,
  reconnects, and observation errors retrieve status rather than replay execution.
  Execution, reconciliation, and retry decisions remain with the owning Rust core.
- Give IPC functions and event payloads explicit TypeScript contracts matching
  the Rust interface. A generic return type or assertion is not runtime validation;
  narrow untrusted payloads where the boundary requires it. Add a small adapter or
  observation Hook when it makes the feature's dependencies clearer.
- Give each subscription and request lifecycle an owner. Account for reactive
  dependencies, cleanup, stale responses, and errors when inputs change or the
  consumer unmounts. Tolerate setup/cleanup/re-setup, including asynchronous listener
  registration. Suppressing an outdated response does not cancel backend work.
- Structure Effects around their actual synchronization inputs; read relevant
  primitives and separate independent synchronization processes. Keep dependencies
  complete. Use Effect Events for nonreactive logic called from an Effect
  or its subscription callback. They see latest committed values but have changing
  identity; keep them local to their Effect owner and out of dependency arrays.
  Retain dependencies that should resynchronize the resource.
- Calling a custom Hook in several components creates separate state and Effects.
  When consumers need shared observations, choose one owner for that source.
  Use `useSyncExternalStore` when integrating an actual subscribable external store;
  local React state remains suitable for simple observations. UI pending guards
  provide interaction feedback; durable duplicate protection belongs in Rust.
- Check cheap relevant guards before unnecessary reads and run independent
  observations concurrently. Preserve required ordering, target identity, error
  handling, and cleanup; read concurrency does not establish safe parallel mutations.

## Performance and completion

Bound rendered history and other growing data, and localize frequent updates to
their consumers. Subscribe to the values that drive rendering; read event-only
values on demand when that preserves their intended identity and timing. Avoid
expensive work in branches that will not display its result.

Use the configured React Compiler for routine memoization: write
render calculations and callbacks directly. Add `useMemo`, `useCallback`, or `memo`
when measurement or a documented integration need justifies explicit control;
keep dependencies complete. Preserve existing manual memoization unless the
touched change warrants removing it and checking behavior. Use state for values
driving output and refs for nonrendering mutable information or owned imperative
resources; memo caches can be discarded and are not resource lifecycle guarantees.
For skipped compilation or remaining slow interactions, inspect diagnostics and
measure before selecting an optimization or new state library. Check the current
build configuration before assuming compiler coverage or framework features.

When a change touches expensive interactive rendering, view visibility, optional
code loading, desktop preference schemas/caches, or growing-data algorithms, read
the relevant sections of [React performance](references/react-performance.md).

For every changed stateful or effectful boundary, account for its owner, target
identity, and applicable stale, cleanup, and failure paths. For every new or changed
component, account for its layer, token/variant choices, guide entry, and affected
accessible interaction contract. Resolve review findings or explain the concrete
tradeoff. Select checks from repository policy and
report their limits; static checks and mocked outcomes do not establish desktop
responsiveness or native-provider compatibility. Review findings should identify
concrete defects or tradeoffs, rather than enforce stylistic extraction preferences.
Report checks performed, unperformed qualification, and justified convention
exceptions in the completion summary.

For the reasoning and primary sources behind these choices, consult
[component boundary research](../../../docs/research/frontend-component-boundaries.md).
For compiler memoization tradeoffs and exceptions, consult
[memoization guidance](../../../docs/research/react-compiler-memoization-guidance.md).

## Provenance

Selected guidance is locally adapted from Vercel's
[React best practices at 1a2c79d](https://github.com/vercel-labs/agent-skills/tree/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices),
checked 2026-11-13, and official React documentation. For the source-rule mapping,
overlap decisions, and stack-specific corrections, consult the
[adoption research](../../../docs/research/vercel-react-best-practices-fit.md).
Review source changes deliberately; the local skill and its references own the
maintained guidance.
