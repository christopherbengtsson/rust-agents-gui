# F04b review

2026-11-14. Two independent fresh-context reviewers, following the code-review
skill's separate axes. The baseline is
`f3d325f52699d5eaf99bd3d6b48c47dace92049d`; the implementation was staged and
uncommitted, so review used `git diff <baseline>` rather than an empty HEAD-only
diff. Each received the selected ticket, parent spec, repository authority and
engineering/verification pointers. Reviews were read-only and ran no builds/tests.

## Standards

No Standards findings in the F04b diff against the baseline.

The implementation follows Rust-owned wire generation, additive codegen,
feature-local semantic validation, separate preparation/unit gates, Cargo target
ownership, and protocol-7 preservation. No actionable baseline code smells
identified.

## Spec

No Spec findings.

The diff implements F04b's generated contracts, standalone validators and shared
preparation checks while retaining contextual validation and preference migrations.
Protocol remains 7; no successor transport, controller, routing, storage or provider
changes were identified. Pending evidence and acceptance updates were excluded.

## Dispositions and self-review

Standards: 0 findings. Spec: 0 findings. Neither axis has an unresolved defect.
A non-finding observation noted repeated preparation across outer build/start
launchers and their Tauri hooks. Both independently accessible entry points retain
their checks; the complete verification/CI gate runs preparation once.

Before independent review, self-review found a missing snapshot PID upper-bound
assertion after replacing the old structural PID helper. A schema-valid `2 ** 32`
regression failed, then passed after sharing the PID range assertion with desktop
identity. Focused decoders/typechecking and the subsequent complete measured unit
runs pass. Generated/preparation inputs are unchanged by that frontend correction.
New TypeScript contains no `any`, unchecked shape casts, suppression comments,
declaration shims or strictness reductions; literal `as const` preserves request
operation discrimination. Generated validator type predicates correspond to the
same exported schema names and declarations.

Final delta review: both independent reviewers also inspected the preparation
owner's normalization of ts-rs declaration line-end spaces and updated artifact
identities. Standards: no new finding. Spec: no new issue. Schema and executable
validator bytes are unchanged; deterministic/private-source preparation and
affected static/tooling checks were repeated.
