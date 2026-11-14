# Application observation corpus

`observations-v6.json` is a small synthetic application-owned protocol-6 corpus,
introduced by F02 against application version 0.1.0. It is not captured provider
traffic and establishes no native runtime behavior. Native request framing and
Tauri's owner/result/error boundary are checked separately in connection units.

Pure Rust tests construct typed requests, snapshots, replies and failures and compare
their serialization with these literals. TypeScript tests consume the same literals
through production decoders, with separate malformed/target/authority scenarios.
Keep semantic guards when changing or generating declarations in the future.

F04 adds `observations-v7.json`: the protocol-7 corpus is synthetic and application-owned,
with owner-bound resync/hint batches plus the retained request/result cases. The Rust
serializer and TypeScript decoders select v7; v6 is retained as F02 compatibility evidence.
No fixture establishes native provider behavior. Protocol 7 rejects older peers before
domain dispatch; matching desktop/supervisor builds are required. Status revision and
accepted invalidation sequence are independent cursors. Collection hints cover related
details and pending actions; resync markers mean all retained observation interests are dirty.

## F04b generated contracts

Rust DTOs are authoritative. The optional `workspace-core/codegen` feature enables
pure, in-memory declaration/schema export and the explicit `export-contracts`
binary. Owning modules register wire types and dependencies; native resource and
storage internals are not exported. `DesktopPid` also types the Tauri scalar result.
No export test writes generated files.

Run `corepack pnpm generated:write` after an intentional contract/generator change,
then review and commit `src/generated/contracts/`. Run
`corepack pnpm generated:check` to reject missing, stale or unexpected managed
outputs without repairing them. The preparation owner claims this worktree's Cargo
target and invokes only Cargo's reported exporter executable. The manifest records
pinned generator versions, source hashes and artifact hashes without timestamps or
checkout-specific paths. Managed output has no manual formatter owner.

`types.ts` contains ts-rs declarations with explicit JSON-number configuration.
`schemas.json` describes draft-2020-12 **serialization**, including canonical
outgoing requests: defaulted response fields remain required, nullable fields stay
nullable, and skipped fields can be omitted. These are not request-deserialization
schemas. Callers send canonical request defaults; legacy preferences retain their
separate migration rules. UUID schemas register UUID formats and retain the
application's 36-character identity shape. Unsupported Serde diagnostics remain
visible; the corpus and pure attribute cases verify supported output behavior.

`validators.js` and its generated type predicates are Ajv2020 standalone ESM.
Only generated checks enter the WebView; Ajv/schema compilation runs in preparation.
Generation rejects unresolved CommonJS helpers/dynamic evaluation rather than
relaxing CSP. Current output needs no runtime helper imports. Schemars' numeric
formats do not always provide an upper bound: local assertions retain safe-number
and PID-range checks.

| Consumer                | Generated structure replaces                                        | Retained contextual assertions                                                                                                       |
| ----------------------- | ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| Connection/contracts    | Snapshot, identity, domain/transport errors, owner/Result envelopes | Owner syntax/equality, PID range, safe revision, UTF-8 limits, exact Result variant, delivery classification and uncertainty         |
| Repository observations | Requests/replies, registration, availability, records/pages         | Exact action/path/creation flag/target, contradictory state fields, initial-workspace relationships, names, byte/page bounds         |
| Workspace observations  | Requests/actions/replies, workspace status/artifacts/pages          | Exact action/target/repository links, allocation/state combinations, provider, normalized absolute paths, pinned commit, page bounds |
| Inspection observations | Observations, files, previews, row/phase/unsupported variants       | Exact workspace/comparison/file, retained-error rules, pagination/uniqueness, safe counts/lines, row relationships, byte budgets     |
| Feed observations       | Cursor, targets, sample, acknowledgement, batch/event variants      | Owner/subscription, next batch, safe sequence, watermark/heartbeat relationships, delivery/target bounds, stopped-message bytes      |

Flattened schemas remain open to additional fields. Named semantic checks preserve
previously rejected contradictory states instead of applying blanket closed-object
rules. Presentation errors and preference records remain local contracts.

Development, static checks, build/start, Tauri frontend hooks and CI use read-only
preparation. `scripts/check.mjs` runs preparation once for each complete verify/CI
gate; CI separately identifies resource compilation and bundling before checking.
Application units select pure codegen tests through `--features codegen` and execute
committed validators; they never invoke generation or require a staged sidecar.
The Node tooling gate discovers `*.test.mjs`; frontend contracts use `.test.tsx` to
match the configured Vitest include.

Protocol remains **7**. No native framing/admission/controller, provider adapter,
storage or routing migration is part of F04b. The v6/v7 corpus provenance above
remains unchanged; generated-validator success does not qualify actual desktop,
socket or native-provider behavior.

## F04d protocol 8

`observations-v8.json` is the synthetic application corpus for application version
0.1.0 and the F04d protocol-8 schema/source in the containing commit. It retains
the F04b frontend wire shapes. Current serializer/validator/decoder checks select
v8; v6 and v7 remain unchanged as historical evidence. No corpus is captured
provider traffic.

Native protocol 8 uses the shared length-delimited codec: a four-byte big-endian
u32 length and at most 65,532 payload bytes (65,536 including the prefix).
Controlled stream tests qualify the framing separately from these JSON literals.
Version/owner checks precede domain dispatch; malformed/legacy framing closes
without dispatch or format fallback. Desktop failures stay visible and cannot
authorize replacement of an established supervisor or replay of an action.
Actual socket/process/desktop qualification remains F04g.
