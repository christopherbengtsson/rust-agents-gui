---
name: rust-engineering
description: Write, review, or refactor this repository's Rust code, including ownership, typed boundaries, durable I/O, and execution lifetimes.
license: MIT
---

# Rust engineering

Read [testing and verification](../../../docs/agents/verification.md) before
applying this skill. For behavior, architecture, or execution ownership, also read
[MVP.md](../../../docs/MVP.md). These policies govern the work; the Rust guidance
here supports their implementation.

## Choose references by task

Read each reference whose branch the change touches; leave unrelated references
unloaded.

| Task | Reference |
| --- | --- |
| Identities, state transitions, input boundaries, or recovery errors | [Types and errors](references/types-and-errors.md) |
| Numeric conversions, lengths, offsets, budgets, or cursor arithmetic | [Numeric boundaries](references/numeric-boundaries.md) |
| Storage writes, transcript/attachment files, archive/restore, or durable shutdown | [Durable I/O](references/durable-io.md) |
| Resource guards, standard threads/processes, or destruction order | [Synchronous resources](references/synchronous-resources.md) |
| Async I/O, queues, task ownership, or shutdown | [Async lifecycle](references/async-lifecycle.md) |
| Unsafe operations or diagnostic instrumentation | [Unsafe code and diagnostics](references/unsafe-and-diagnostics.md) |

## Ownership and interfaces

- Borrow for temporary access; accept ownership when storing or transferring a
  value. Prefer `&str`, `&[T]`, and `&Path` when those views provide the operations
  needed. Retain owned snapshots and cheap `Arc::clone` where independent lifetime
  or shared ownership is intentional.
- Use `Option::take`, `mem::take`, or `mem::replace` to transfer a field when the
  replacement is a valid state. Account for every fallible exit after replacement;
  an error must leave the owner in a defined state with its recovery data retained.
- Choose concrete types, generics, or `dyn Trait` by the interface's actual needs.
  Generics suit statically known implementations; trait objects suit runtime
  heterogeneity or intentional type erasure. Borrowed trait objects need no box.
  Keep dependency seams that enable the repository's required controlled outcomes;
  a single production implementation is not evidence that a seam is unnecessary.
- Measure a suspected bottleneck before adding specialized collections, allocators,
  unsafe optimizations, or build-profile tuning. Use existing workspace lint and
  verification configuration as the starting point for enforcement changes.

## Completion

For each changed boundary, account for ownership and the applicable failure,
cancellation, or stale-state path. Select verification through the repository
policy and report its limits. Review findings should identify a concrete defect
or tradeoff in the code, rather than treating every guideline as a mandatory edit.

## Provenance

Adapted from Leonardo Maldonado's
[rust-skills at fd2a861](https://github.com/leonardomso/rust-skills/tree/fd2a861ab0406a4ac536a55274d14ea6fd1ca9c9).
Retained upstream notice: [LICENSE](LICENSE). References identify their source rules;
examples and recommendations are curated rather than copied wholesale.

Ownership/interface sources: `own-borrow-over-clone`, `own-slice-over-vec`,
`mem-take-replace`, `trait-dyn-vs-generic`, `anti-over-abstraction`,
`perf-profile-first`, and `lint-workspace-lints` in the pinned upstream `rules/`.
