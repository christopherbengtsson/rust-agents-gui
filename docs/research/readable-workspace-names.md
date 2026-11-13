# Readable workspace names and compiler cleanup

2026-11-13. Accepted follow-up: [implementation plan](../../.scratch/milestone-01/readable-workspaces-plan.md). The 500-name roster and identity corrections are in [footballer catalog](footballer-catalog.md).

## Reservation policy

Conductor documents numbered variants such as warsaw-v2 in its [city reference](https://www.conductor.build/docs/reference/cities), and describes workspaces as separate branches/worktrees in [workspaces and branches](https://www.conductor.build/docs/concepts/workspaces-and-branches). Its private allocation algorithm is unknown. Our agreed policy permanently reserves historical labels rather than reusing archived names, preserving readable history and recovery targets.

Git branches belong to the shared repository. [Git worktree documentation](https://raw.githubusercontent.com/git/git/v2.50.0/Documentation/git-worktree.adoc) also establishes that a missing directory does not remove a registered worktree location. Therefore allocation inventories refs, wrapper entries and registered paths before reserving a name. The [update-ref transaction](https://git-scm.com/docs/git-update-ref/2.50.0) and exclusive folder creation remain authoritative race guards; preflight cannot eliminate concurrent external changes. A collision after reservation requires explicit reconciliation with the same artifacts.

No UUID is needed in a human label. UUIDs still identify saved objects, actions and ownership internally. Numeric tiers avoid a permanent catalog-size limit. Repository folders have a separate durable reservation scope within application state.

## Size and selection investigation

A standalone Rust 1.99.1 release probe on macOS 26.2 ARM64 used synthetic distinct 32-byte ASCII names, a HashSet of reservations, a scan of the unused pool and one selection. One warm-up plus five 10,000-selection runs: at 500 names, median selection cost ranged from 1.035 microseconds (empty history) to 5.007 microseconds (all plain names used); at 1,000 names, 1.236–9.930 microseconds. This excludes Git, SQLite, IPC and UI. Synthetic 500-name logical storage was 24,000 bytes including string-slice entries. Standalone link padding obscured small binary deltas; these figures do not establish actual application/package cost. The follow-up evidence records the actual curated catalog's size and a controlled application binary comparison.

## Installed React compiler

Installed versions remain @vitejs/plugin-react 6.3.0 and oxc-transform-react 0.156.0. The [Vite plugin source](https://github.com/vitejs/vite-plugin-react/blob/plugin-react%406.3.0/packages/plugin-react/src/index.ts) exposes diagnostics; the [native compiler's lowering](https://github.com/oxc-project/oxc/blob/75efdca4747c4ad9b252542743098d4894f8005b/crates/oxc_react_compiler/src/react_compiler_lowering/build_hir.rs) rejects finalizers and throws inside a lexical try/catch. These diagnostics skip optimization; they do not cause a supervisor wire-version mismatch. [React documents recoverable compiler failures](https://react.dev/reference/react-compiler/panicThreshold).

An in-memory native transform of the prior source reported 1/2/3/1 recoverable diagnostics in supervisor/repositories/workspaces/picker respectively. `use no memo` did not suppress these lowering diagnostics in this installed port. Moving the async try/catch/finally envelope into an ordinary non-React helper produced zero diagnostics and compiler cache calls in all four files. The checked-in tooling probe exercises that exact installed transform with the application's compiler options. Owners retain their guards and callbacks; the helper only guarantees error propagation and settlement, including when an error handler throws. This retains [JavaScript finally semantics](https://tc39.es/ecma262/multipage/ecmascript-language-statements-and-declarations.html#sec-try-statement-runtime-semantics-evaluation). Historical compiler evidence remains unchanged.

## Presentation

Existing shadcn/Base UI components suffice. [Collapsible composition](https://ui.shadcn.com/docs/components/base/collapsible) and the [Base UI API](https://base-ui.com/react/components/collapsible.md) support an explicit keyboard-accessible disclosure for legacy paths. No foundation source, dependency or compiler version changes are needed.
