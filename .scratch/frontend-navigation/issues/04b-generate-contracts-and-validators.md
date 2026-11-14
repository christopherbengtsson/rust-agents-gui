# 04b: Generate Rust-owned contracts and validators

Status: ready-for-agent

Blocked by: [F04a](04a-publish-library-refactor-plan.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read the
[owning requirements](../spec.md#approved-libraries-and-generated-contracts) and predecessor evidence. Read the MVP,
milestone, verification policy and applicable repository engineering guidance.
[Observation guidance](../../../docs/agents/observations.md) owns shared engineering
pitfalls; the parent spec owns behavior and limits. This ticket is one session;
successors remain separate. Preserve unrelated edits and completed evidence.

## Module and caller interface

Rust Serde wire types are authoritative. Feature observation modules consume
generated declarations and structural validators, then apply named contextual
assertions. Screens learn neither generator configuration nor wire decoding.

Add ts-rs/Schemars as optional application `codegen` dependencies and one explicit
exporter. Use controlled ts-rs Config, draft-2020-12 serialization schemas,
Ajv2020 standalone ESM and registered UUID formats. Commit artifacts under
`src/generated/contracts/`; retain the current CSP. Native protocol remains 7
until F04d. Export frontend roots and dependencies through owning-module helpers,
without exposing native resource state for generation.

## Implementation and removal

- Cover snapshot/identity, owner/result/error, repository/workspace/inspection and
  feed observations plus required request declarations. Keep request-deserialization
  schema semantics distinct if exporting them. Use agreed versions from the spec.
- Share preparation/drift handling, with explicit write and check modes. Reuse it
  in dev/check/build/Tauri/CI; drift must fail before any repairing generation.
  Keep codegen separate from application units, typechecking, bundling and packaging.
- Replace repeated TypeScript wire shapes and structural field/variant guards.
  Keep exact owner/target, state/sequence, safe-number, UTF-8 and custom path assertions.
  Preserve currently rejected contradictory variants without blanket additional-field
  rejection around flattened types. Pending preferences retain their own migrations.
- Keep unsupported Serde diagnostics visible. Avoid `#[ts(export)]` file-writing
  tests. Pure exports/serializers and generated validators form the contract test seam.

Source footholds at F04: `crates/core/src/{lib,connection,workspaces,observations}.rs`,
`connection/subscription.rs`, `workspaces/inspection.rs`, `src/lib/{observation-contracts,
observation-feed,repositories,workspaces,changes}.ts`, `contracts/README.md`,
`package.json`, `scripts/{cargo,build,start,test-unit}.mjs`, Tauri hooks and CI.
Follow moved definitions rather than inventing per-command forwarding modules.

## Required scenarios

- Rust serialization, generated types/schemas and current corpus agree for flattening,
  tagged unions, UUID newtypes, `Result`, null/omitted/default fields and safe u64 values.
- Malformed structure fails; structurally valid wrong-owner/target/cursor/state and
  byte-over-budget payloads still fail semantic assertions. Retained legacy preferences
  remain readable without erasing uncertainty.
- Deterministic regeneration, stale/missing output rejection, clean-checkout preparation,
  and effect-substituted tooling command ordering. Prove tests are selected by the gate.
- Generated ESM imports bundle without runtime schema compilation or CSP relaxation.

## Acceptance

- [x] All selected wire roots use generated declarations/validators; removed duplication and retained semantic assertions are mapped.
- [x] Explicit generation/drift commands serve every required entry point with target ownership/provenance preserved.
- [x] Relevant pure contract/frontend/tooling checks and configured static checks pass; bundle result is recorded separately.
- [x] Evidence records versions, commands, source/artifact identity, failures and unperformed native qualification.

F04c owns admission/framing. This ticket changes no socket protocol, provider adapter,
storage layout, routing or lifecycle controller.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.

### 2026-11-14 — F04b implementation complete

Confirmed F04a's checked acceptance/evidence and continued the clean
`feature/m01-frontend-navigation` branch from `f3d325f`. Added optional Rust codegen,
committed Rust-owned declarations/serialization schemas/standalone ESM validators,
replaced frontend structural duplication and retained named contextual assertions
and preference migrations. Shared explicit write/read-only drift preparation serves
dev/check/build/Tauri/CI while retaining target ownership and sidecar provenance.

[Evidence](../../../docs/evidence/frontend-navigation/f04b.md) records source/artifact
identities, pinned versions, failure/red checks, deterministic and clean-source
preparation, gate selection, 144 Rust + 40 tooling + 126 frontend checks, warm
measurements, separate bundle cost and independent Standards/Spec reviews (0 findings
each). Final configured verification passed; both corpus files and prior evidence
are preserved. Packaging, actual sockets/fixture-desktop/WebView/accessibility,
hosted CI and native-provider qualification were not performed; no provider
submissions were made. Protocol remains 7. F04c is next and was not started;
successors and whole-foundation acceptance remain open. The containing commit owns
this ticket's changes and evidence.
