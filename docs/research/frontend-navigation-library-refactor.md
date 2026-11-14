# Accepted frontend foundation library refactor

2026-11-14 · Documentation publication only · Implementation baseline
`7dc4eac26ab51a2146805b0aa07bcfb84d03417c` (F04).

## Decision and evidence boundary

Adopt maintained, stable libraries to remove duplicated contracts, transport
plumbing, scheduling, and lifecycle machinery before F05. The motivation is
maintenance, robustness, and deleting mechanisms the libraries can own; a measured
performance bottleneck is not a prerequisite. This accepted direction supersedes
the earlier recommendation to defer an async transport until scaling or blocking
pressure appears. The [crate checkpoint](frontend-navigation-rust-crates.md) and
[transport audit](frontend-navigation-tokio-and-refactor-audit.md) remain historical
evidence, including their narrower recommendations.

The [spec](../../.scratch/frontend-navigation/spec.md) owns the accepted plan;
[MVP](../MVP.md) owns product behavior and Rust responsibilities. F01–F04 have
implemented unit-covered increments. [F04 evidence](../evidence/frontend-navigation/f04.md)
does not establish actual socket or fixture-desktop qualification. Nothing in this
note completes those checks. No dependency installation, resolution, export,
compilation, bundling, unit execution, desktop qualification, provider submission,
or RAT inspection was performed for this publication.

## Sequence and replacement boundaries

| Increment   | Accepted deliverable                                                                                                                                                      |
| ----------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F04a        | Publish the accepted plan, tickets, architecture corrections, and this research.                                                                                          |
| F04b        | Generate TypeScript wire declarations and structural runtime validators from Rust DTOs.                                                                                   |
| F04c        | Replace manual admission with Tokio Semaphore and introduce one private framed-exchange implementation.                                                                   |
| F04d        | Convert client, supervisor server, and feed transport together to Tokio async I/O with explicit runtime ownership.                                                        |
| F04e        | Move repository/workspace observations to Query with the shared physical queue gate, complete feature freshness, and covered polling removal in the same increment.       |
| F04f        | Move feed and inspection lifecycles to app-lifetime XState actors; migrate inspection Query/freshness and covered polling together; consume Zustand presentation context. |
| F04g        | Qualify the actual separate-supervisor/desktop seam with focused disposable process fixtures.                                                                             |
| Revised F05 | Integrate retained context and remaining feature presentation after the replacements and qualification.                                                                   |

Each implementation ticket remains separately selected work. This publication
authorizes no successor implementation or native-provider cohort. Preserve original
requirements: exact owner/target/approval matching, sticky uncertain outcomes,
bounded lanes and queues, physical completion, journal/cursor/resync semantics,
deadlines, retained pending interests, route-independent reader lifetimes, and
visible failure. Reconciliation and retry policy remain in owning Rust modules.

## Generated wire contracts

Pin **ts-rs 12.0.1**, **Schemars 1.2.3**, **Ajv 8.20.0**, and
**ajv-formats 3.0.1**. Generate declarations with the current ts-rs
`TS::export_to_string(&Config)` API through an explicit preparation owner, commit
artifacts, and check drift before repair. Avoid `#[ts(export)]` side-effecting
generated tests in the application unit gate. Keep unsupported Serde warnings
visible. Verify tagged/flattened/generic DTOs, UUID newtypes, optional versus null
fields, and JSON-safe integer representation against the retained corpus.
[ts-rs API](https://docs.rs/ts-rs/12.0.1/ts_rs/trait.TS.html),
[published source](https://crates.io/api/v1/crates/ts-rs/12.0.1/download)

Schemars defaults describe deserialization. Supervisor **output** schemas must use
`SchemaSettings::draft2020_12().for_serialize()` before constructing the generator;
otherwise defaults and omitted fields can describe the wrong contract. Use the
appropriate input contract separately. This correction is established by the
[1.2.3 source archive](https://crates.io/api/v1/crates/schemars/1.2.3/download)
(`src/generate.rs`), not a claim that representative DTO export has passed.

Use Ajv's draft-2020-12 constructor (`ajv/dist/2020`) at preparation time and
standalone generation with `code: {source: true, esm: true}`. Ship generated ESM
validators, without schema compilation or dynamic `Function` evaluation in the
WebView. Keep UUID format validation enabled with ajv-formats; inspect and bundle
any generated helper imports. CSP and ESM compatibility require actual build and
WebView evidence. [Ajv dialects](https://ajv.js.org/json-schema.html),
[standalone generation](https://ajv.js.org/standalone.html),
[formats](https://ajv.js.org/packages/ajv-formats.html)

Generated shape checks replace repeated structural guards; retain semantic checks
for owner, exact target, authority, state combinations, safe sequences, cursor and
acknowledgement identity, delivery bounds, and UTF-8 **byte** counts. JSON Schema
string length does not establish the existing byte ceiling. Query accepts results
only after structural and semantic validation. No generator owns action recovery.

## Shared Rust transport mechanism

Pin **Tokio 1.54.1** and **tokio-util 0.7.20**. Replace atomic permit accounting
behind the existing admission boundary with `Arc<Semaphore>` and
`try_acquire_owned`. Preserve nonwaiting Busy, acquire-before-spawn, lane counts,
and owned permits retained until actual work finishes. Do not turn admission into
an awaiting queue. [Semaphore](https://docs.rs/tokio/1.54.1/tokio/sync/struct.Semaphore.html),
[owned permit](https://docs.rs/tokio/1.54.1/tokio/sync/struct.OwnedSemaphorePermit.html)

Use one private shared framed exchange for short requests and the feed. Adopt a
four-byte big-endian u32 length prefix with **65,532 payload bytes**, so prefix plus
payload remains within the **65,536-byte total** ceiling. This deliberately changes
v7 newline framing and requires a protocol bump with synchronized compatibility
evidence. Configure limits explicitly on encoding and decoding; reject oversized,
malformed and incomplete frames terminally. The codec replaces byte mechanics,
not the protocol's two-phase idle/transfer budget or absolute write deadlines.
[LengthDelimitedCodec source](https://docs.rs/crate/tokio-util/0.7.20/source/src/codec/length_delimited.rs),
[timeout_at](https://docs.rs/tokio/1.54.1/tokio/time/fn.timeout_at.html)

**Strict EOF needs a small adapter.** The codec consumes a complete header before
its payload arrives. Default `Decoder::decode_eof` can then observe an empty byte
buffer and return clean EOF while the codec still awaits a nonempty payload.
Track pending-frame state in the adapter; checking only buffer emptiness is
insufficient. Reuse the codec's buffer rather than adding a second byte buffer.
Retain a focused header-only truncation scenario alongside partial-header and
partial-payload scenarios. This follows the published `decode_head`, `decode_data`
and default EOF implementation, not observed application qualification.
[Decoder source](https://docs.rs/crate/tokio-util/0.7.20/source/src/codec/decoder.rs),
[published archive](https://crates.io/api/v1/crates/tokio-util/0.7.20/download)

The supervisor binary owns a `current_thread` runtime with I/O and time enabled;
the core library creates no runtime. Tauri uses its existing runtime. Convert both
ends and the feed coherently. Keep Git/storage/domain blocking work behind bounded
blocking dispatch, retaining permits with physical work. Started `spawn_blocking`
work cannot be aborted; logical waiter cancellation proves no physical completion.
Own cancellation and joins explicitly; preserving access control, exclusive owner
admission, discovery, pending acknowledgements and uncertain-send recovery is part
of the refactor. [Runtime builder](https://docs.rs/tokio/1.54.1/tokio/runtime/struct.Builder.html),
[blocking work](https://docs.rs/tokio/1.54.1/tokio/task/fn.spawn_blocking.html),
[cancellation safety](https://docs.rs/tokio/1.54.1/tokio/macro.select.html#cancellation-safety)

## Frontend mechanisms and feature ownership

Use **TanStack Query 5.107.1** as the sole observation cache and **p-queue 10.0.0**
for shared physical scheduling. Retain a small registry of unsettled physical
promises for same-target coalescing and drain accounting across logical Query
cancellation. Keep two active observation reads and at most 64 queued targets.
Pending obsolete tasks can be aborted; once started, a queue task must await its
actual IPC promise. Do not configure active abort/timeout behavior that releases a
slot early. The pinned p-queue source races running work against abort and timeout
and cleans up its slot when that race settles; it cannot prove the underlying IPC
finished. Query cancellation can likewise settle logical state without stopping
native work. [p-queue v10 source](https://github.com/sindresorhus/p-queue/blob/v10.0.0/source/index.ts),
[Query cancellation](https://tanstack.com/query/v5/docs/framework/react/guides/query-cancellation)

Repository and workspace observation modules each own their Query definitions,
decoding, interests, dirty generations and freshness policy. Complete each feature's
cache/freshness/poll replacement together. Preserve owner-scoped keys, baseline
before reads, follow-up after hints during reads, retained last-good data and
pending-action interests. Disable automatic retries and focus/network refetch;
use local-IPC network mode and agreed retention. Hint-driven repository reads use
`refresh=false`. Preserve visible inspection's two-second external-Git refresh.
[Query defaults](https://tanstack.com/query/v5/docs/framework/react/guides/important-defaults),
[network mode](https://tanstack.com/query/v5/docs/framework/react/guides/network-mode),
[local guidance](../agents/tanstack.md)

Use **XState 5.33.4** with **@xstate/react 6.1.0** for app-lifetime feed and
inspection actors above routed layouts. Actors own lifecycle transitions and
subscription/reader coordination. Feed acceptance records validated deliveries
and acknowledges independently of refetch completion; it is separate from each
feature's dirty generations. Migrate inspection Query/freshness/polling together.
Components select actor snapshots directly; do not mirror snapshots into Zustand.
Actor stopping is not evidence that an already started native operation stopped.
[Actor lifecycle](https://stately.ai/docs/actors),
[React actor selectors](https://stately.ai/docs/xstate-react)

Use **Zustand 5.1.0** only for retained presentation context, with an app-scoped
store and narrow selectors. Router owns current destination; Query owns cached
observations; actors own lifecycle state; Rust owns durable execution authority.
Persist only the validated/migrated allowlist and show storage failures.
[Scoped vanilla stores](https://zustand.docs.pmnd.rs/reference/hooks/use-store),
[persistence](https://zustand.docs.pmnd.rs/reference/integrations/persisting-store-data)

Share mechanics once where they have one owner. Keep feature decisions local.
Do not introduce a generic public observation host or one forwarding module per
library. A small private adapter is justified by a concrete policy gap such as
strict EOF or physical draining; a dependency alone does not justify a new layer.

## Maintenance and adoption evidence

All selected versions are stable releases. Crates.io publication records checked
2026-11-14 date [ts-rs 12.0.1](https://crates.io/api/v1/crates/ts-rs/12.0.1) to
January 31, [Schemars 1.2.3](https://crates.io/api/v1/crates/schemars/1.2.3) to
October 12, [Tokio 1.54.1](https://crates.io/api/v1/crates/tokio/1.54.1) to October
21, and [tokio-util 0.7.20](https://crates.io/api/v1/crates/tokio-util/0.7.20) to
October 9, all in 2026. npm registry metadata checked 2026-11-14
records Query 5.107.1 and Zustand 5.1.0 published November 6, XState 5.33.4 November
7, p-queue 10.0.0 October 12, @xstate/react 6.1.0 February 26, Ajv 8.20.0 April 24,
and ajv-formats 3.0.1 March 30, 2024. An older companion-format release alone is not
evidence of abandonment. Sources: version/time records from the npm registry for
[Query](https://registry.npmjs.org/@tanstack%2freact-query),
[Zustand](https://registry.npmjs.org/zustand),
[XState](https://registry.npmjs.org/xstate),
[React bindings](https://registry.npmjs.org/@xstate%2freact),
[p-queue](https://registry.npmjs.org/p-queue),
[Ajv](https://registry.npmjs.org/ajv), and
[formats](https://registry.npmjs.org/ajv-formats).

The npm downloads API reported substantial package distribution for all seven
over November 7–13; these include transitive/CI downloads and do not count unique
users or validate fit. The endpoints provide auditable adoption signals without a
pass/fail popularity threshold: [Query](https://api.npmjs.org/downloads/point/2026-11-07:2026-11-13/@tanstack%2freact-query),
[p-queue](https://api.npmjs.org/downloads/point/2026-11-07:2026-11-13/p-queue),
[XState](https://api.npmjs.org/downloads/point/2026-11-07:2026-11-13/xstate),
[React bindings](https://api.npmjs.org/downloads/point/2026-11-07:2026-11-13/@xstate%2freact),
[Zustand](https://api.npmjs.org/downloads/point/2026-11-07:2026-11-13/zustand),
[Ajv](https://api.npmjs.org/downloads/point/2026-11-07:2026-11-13/ajv),
[formats](https://api.npmjs.org/downloads/point/2026-11-07:2026-11-13/ajv-formats).

Rust evidence includes stable published archives and maintained upstream source:
Tauri 2.12.5 directly uses Tokio, and Wasmtime's inspected manifest uses Tokio and
rustix. These establish adoption of the libraries, not this refactor's correctness.
ts-rs has an upstream test workflow; its existence is a maintenance signal, not a
claim about current CI success. [Tauri manifest](https://github.com/tauri-apps/tauri/blob/tauri-v2.12.5/crates/tauri/Cargo.toml),
[Wasmtime manifest, checked 2026-11-14](https://github.com/bytecodealliance/wasmtime/blob/main/Cargo.toml),
[ts-rs workflow](https://github.com/Aleph-Alpha/ts-rs/blob/main/.github/workflows/test.yml)

## Remaining proof

The library boundaries reduce handwritten mechanisms while leaving semantic
policy explicit; generated artifacts, async ownership and actor modeling add
preparation and dependency costs. No deletion count, performance gain, binary-size
change, or compatibility success has been measured. Representative exports,
schema/corpus parity, standalone ESM/helper/CSP bundling, exact-version compilation,
deterministic replacement scenarios, and F04g actual socket/desktop/cleanup
qualification remain unperformed. Report static checks, units, fixture-desktop,
packaging, and native-provider results separately under the
[verification policy](../agents/verification.md); native qualification retains its
separately agreed allowance.
