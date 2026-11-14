# Rust crate checkpoint for the frontend foundation

2026-11-14 · Research only · Source inspected at
`7dc4eac26ab51a2146805b0aa07bcfb84d03417c` (F04).

## Recommendation

Take a short contract-generation checkpoint before F05. Prefer evaluating `ts-rs`
first, with standalone Specta as the alternative; adopt one only after a small
representative export proves useful. Continue the planned frontend foundation after
that decision. The evidence does **not** support pausing it for a new Rust runtime,
channel system, journal, or RPC transport.

This recommendation concerns future maintenance, not merely preserving completed
work. Generated wire declarations could remove repeated knowledge that F05 will
otherwise spread into Query modules. In contrast, the current transport's main
complexity expresses required application semantics that a channel or socket crate
would leave in place. A semaphore is a credible smaller substitution, independent
of code generation, but replacing the whole connection architecture is unnecessary
to use it.

No dependencies were installed, resolved, compiled, or changed. No implementation,
test, desktop, process-fixture, or provider run was performed. Published crate
metadata and source archives were fetched read-only and inspected in memory.
`docs/IDEAS.md` was already modified and was preserved. Specs, tickets, and completed
evidence were not changed.

## Actual starting point

The spec's F01 handoff and the milestone's summary are historical descriptions.
The current ticket acceptance and evidence establish the following:

| Increment  | Current evidence and implementation                                                                                                                                                                                      |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| F02        | Owner-bound requests/replies, runtime guards, transport outcomes, sticky action uncertainty and shared corpus. See [F02 evidence](../evidence/frontend-navigation/f02.md).                                               |
| F03        | Nonwaiting native lane admission before blocking work; physical workers retain permits; frontend read coordination. See [F03 evidence](../evidence/frontend-navigation/f03.md).                                          |
| F04        | Bounded journal, typed invalidation publication, protocol 7 catch-up, exact acknowledgements, deadline/framing/stop ownership and frontend dirty acceptance. See [F04 evidence](../evidence/frontend-navigation/f04.md). |
| F05 onward | Query migration, freshness, polling removal, Zustand, routes and Settings remain unimplemented. See [F05](../../.scratch/frontend-navigation/issues/05-feature-observations-and-ui-context.md).                          |

F04 records 140 Rust, 25 tooling and 119 frontend checks passing. Its actual socket,
Tauri/WebView, fixture-desktop, access-control, packaging and native-provider checks
remain unperformed. Those limits matter for **either** keeping the code or replacing
a mechanism; unit evidence does not qualify a crate migration on this process seam.

Concrete code inspected:

- [Admission](../../crates/core/src/connection/admission.rs) is 52 lines of atomic
  counting, an owned drop guard and the admitted closure boundary.
- [Journal](../../crates/core/src/observations.rs) uses `VecDeque`, `Mutex` and
  `Condvar`; its policy includes 256 retained entries, identity tags, delivery
  count/byte limits, cursor validity, sampled resync watermarks and exhaustion.
- [Subscription](../../crates/core/src/connection/subscription.rs) combines exact
  pending acknowledgement matching, socket shutdown, wake/join ownership and
  native/frontend flow control. Its larger size includes controlled tests.
- [Tauri](../../src-tauri/src/main.rs) already uses `async_runtime::spawn_blocking`
  and `ipc::Channel`; [connection framing](../../crates/core/src/connection.rs)
  remains synchronous Unix sockets with bounded persistent readers.
- [Feed declarations/guards](../../src/lib/observation-feed.ts) duplicate Rust wire
  shapes while also checking owner, sequence, watermark, count and byte invariants.
  [Other guards](../../src/lib/observation-contracts.ts) distinguish structural
  validity from current authority. Generated types should replace repeated shapes,
  while these decisions remain explicit.

The [workspace manifests](../../Cargo.toml) and [lockfile](../../Cargo.lock) pin
Rust 1.99, Tauri 2.12.5, Serde 1.0.229 and serde_json 1.0.154. Core directly depends
on libc, rusqlite, Serde/serde_json and UUID, with no Tokio dependency. Tokio 1.54.1,
tokio-util 0.7.20 and crossbeam-channel 0.5.15 are already **transitive desktop**
dependencies; that does not make them declared core dependencies or make a
core-only supervisor migration free.

## Version and source check

Registry metadata was checked on 2026-11-14. These are available versions, **not**
proposed lockfile updates. Dates are registry publication dates.

| Candidate         | Registry version/date                                                               | Repository relationship                             |
| ----------------- | ----------------------------------------------------------------------------------- | --------------------------------------------------- |
| ts-rs             | [12.0.1, 2026-01-31](https://crates.io/api/v1/crates/ts-rs/12.0.1)                  | New dependency if selected.                         |
| Specta            | [2.0.0-rc.28, 2026-11-05](https://crates.io/api/v1/crates/specta/2.0.0-rc.28)       | New prerelease family; latest stable is 1.0.5.      |
| tauri-specta      | [2.0.0-rc.29, 2026-11-13](https://crates.io/api/v1/crates/tauri-specta/2.0.0-rc.29) | New prerelease; stable 1.0.2 is the Tauri 1 family. |
| specta-typescript | [0.0.15, 2026-11-05](https://crates.io/api/v1/crates/specta-typescript/0.0.15)      | RC28 exporter pairing.                              |
| Tokio             | [1.54.1, 2026-10-21](https://crates.io/api/v1/crates/tokio/1.54.1)                  | Same version already locked transitively.           |
| crossbeam-channel | [0.5.17, 2026-09-05](https://crates.io/api/v1/crates/crossbeam-channel/0.5.17)      | Newest differs from locked 0.5.15.                  |
| flume             | [0.12.0, 2025-12-08](https://crates.io/api/v1/crates/flume/0.12.0)                  | New dependency.                                     |
| interprocess      | [2.4.8, 2026-11-08](https://crates.io/api/v1/crates/interprocess/2.4.8)             | New dependency.                                     |
| ipc-channel       | [0.23.1, 2026-10-24](https://crates.io/api/v1/crates/ipc-channel/0.23.1)            | New dependency.                                     |

Avoid selecting the Specta family through a generic `latest` documentation link:
the official [Tauri Specta setup](https://specta.dev/docs/tauri-specta) still shows
RC27/.14, whereas the published
[RC29 dependency metadata](https://crates.io/api/v1/crates/tauri-specta/2.0.0-rc.29/dependencies)
requires Specta exactly RC28 and exporter .15. Its
[published source archive](https://crates.io/api/v1/crates/tauri-specta/2.0.0-rc.29/download)
also has a development-workspace Tauri Git patch awaiting upstream PR 16374.
Consumers do not inherit that root patch. Its presence warrants a pinned-version
fit experiment; it does not prove the released crate fails with Tauri 2.12.5.

There is also a nonblocking source discrepancy: ts-rs 12.0.1's published manifest
declares Rust 1.78, while its rendered documentation currently says 1.88. The
repository's 1.99 exceeds both; no MSRV compatibility claim was established by a
build. See the [published archive](https://crates.io/api/v1/crates/ts-rs/12.0.1/download)
and [rendered docs](https://docs.rs/ts-rs/12.0.1/ts_rs/).

## Contracts: strongest incremental opportunity

### ts-rs: evaluate now, adopt if representative export passes

`ts-rs` generates TypeScript declarations from Rust structs/enums, including generic
types. Its default Serde compatibility understands tagged enums, renames, flattening
and defaults, but not every Serde attribute. UUID support is optional. Large Rust
integers default to TypeScript `bigint`, which needs deliberate handling for this
JSON protocol's safe-number representation. Unsupported attributes produce warnings;
keep those visible. [ts-rs 12.0.1 documentation](https://docs.rs/ts-rs/12.0.1/ts_rs/)

The useful seam is **wire DTO declaration generation**, independent of Tauri and
independent of domain state ownership. Start with `FeedCursor`, `Target`, `FeedAck`,
`FeedBatch` and one tagged repository/workspace reply. Check flattened fields,
newtype UUID identities, optional-versus-null fields and safe `u64` sequences against
the existing v7 JSON corpus. Frontend unions currently combine some Rust variants;
some older feature shapes deliberately tolerate optional fields. Review those
differences rather than replacing declarations mechanically.

Use explicit generation plus a drift check, separate from application unit execution.
`TS::export_to_string` permits pure generation assertions; `#[ts(export)]` instead
generates a test that writes bindings, which conflicts with treating the unit gate
as effect-free application verification. Commit generated declarations and keep one
preparation owner, borrowing the established route-generation discipline.
[TS export APIs](https://docs.rs/ts-rs/12.0.1/ts_rs/trait.TS.html)

Expected saving: repeated declarations and some compile-time drift, **not** decoding
unknown JSON, target matching, UUID validity, byte bounds, action uncertainty or
cross-field state validity. This is an assessment of the repository seam, not a
claim that successful type generation validates its runtime protocol.

### Specta / tauri-specta: credible alternative, larger initial commitment

Standalone Specta collects Rust type information for language export. Tauri Specta
adds generated command argument/return bindings and events, potentially removing
more manual invoke signatures than declaration-only generation. Its v2 family uses
Specta v2; its v1 family targets Tauri v1.
[Specta overview](https://docs.rs/specta/latest/specta/),
[Tauri Specta setup](https://specta.dev/docs/tauri-specta),
[published RC29 README/source](https://crates.io/api/v1/crates/tauri-specta/2.0.0-rc.29/download)

Prefer this family if the actual command-wrapper saving outweighs the RC coupling.
For this repository that must include `tauri::State`, `Result<OwnedReply<_>,
RequestFailure>`, `Channel<FeedEvent>`, flattened acknowledgement data and the exact
locked Tauri version. The published Tauri dependency range is `^2`, with its Specta
feature enabled; this is declared compatibility, not a successful build of our
commands. Standalone export avoids attaching the UI-independent core to Tauri.
[RC29 dependency metadata](https://crates.io/api/v1/crates/tauri-specta/2.0.0-rc.29/dependencies)

Do not add both generators. Do not replace F05's target/freshness coordinator with
generic generated Query hooks: owner scoping, dirty generations, no automatic action
retry and explicit refresh policy remain requirements in the local spec. Whether
either generator emits runtime validation suitable for this protocol is not
established here; generated static declarations alone are insufficient evidence.

## Scheduling and delivery: mechanisms to keep narrow

| Mechanism                 | Practical decision and reason                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                 |
| ------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Tokio `Semaphore`         | **Optional small replacement now**, independent of F05. `try_acquire_owned` supplies nonwaiting admission and an owned drop-released permit. It can replace the atomic counter behind `ConnectionSlots` without changing synchronous sockets. Preserve acquire-before-spawn and move the permit into physical work. Core needs an explicit `tokio` dependency with `sync`; Tauri's runtime reexports do not include `Semaphore`. The current 52-line implementation is also reasonable to keep if adding a core dependency saves little. [Tokio 1.54.1 Semaphore](https://docs.rs/tokio/1.54.1/tokio/sync/struct.Semaphore.html), [Tauri 2.12.5 runtime source](https://docs.rs/tauri/2.12.5/src/tauri/async_runtime.rs.html) |
| Tokio `broadcast`         | **Keep the current journal.** Broadcast supplies bounded retention and a lag error, but new subscribers see future messages, not replay from an externally accepted application cursor. It releases entries according to receivers and advances lagging receivers. Our retained catch-up, sampled watermark, tag coalescing and resync rules would still need their own state. Using it only for wakeups duplicates the existing condition variable. [Tokio 1.54.1 broadcast](https://docs.rs/tokio/1.54.1/tokio/sync/broadcast/index.html), [subscribe semantics](https://docs.rs/tokio/1.54.1/tokio/sync/broadcast/struct.Sender.html#method.subscribe)                                                                     |
| crossbeam-channel / flume | **Evaluate later if a real worker queue needs them.** Crossbeam provides bounded channels and selection; clones share a queue, not independent broadcast streams. Flume adds bounded/rendezvous queues, deadlines and sync/async interaction. Neither describes our cursor journal or frontend acceptance. Substituting a queue now would retain the important policy and introduce queue scheduling choices where native requests deliberately return Busy without waiting. [crossbeam-channel 0.5.17](https://docs.rs/crossbeam-channel/0.5.17/crossbeam_channel/), [flume 0.12.0](https://docs.rs/flume/0.12.0/flume/)                                                                                                     |
| Existing Tauri channels   | **Keep using them.** Tauri documents channels as ordered streaming delivery, already used in F04. The application acknowledgement means dirty state was accepted; successful channel send alone does not demonstrate that acceptance. Retain the one-in-flight bridge and explicit stop/deadline logic. [Tauri channel guide](https://v2.tauri.app/develop/calling-frontend/#channels), [current bridge](../../src-tauri/src/main.rs)                                                                                                                                                                                                                                                                                         |
| interprocess              | **Evaluate when another OS/client makes transport portability concrete.** Its local sockets map Unix to Unix-domain sockets and Windows to named pipes; it intentionally adds no framing/metadata. Thus it could replace the socket layer while preserving wire protocol, but not remove frame bounds, deadlines, handshake, owner admission or acknowledgements. macOS-first scope supplies no demonstrated immediate saving. [interprocess 2.4.8 local sockets](https://docs.rs/interprocess/2.4.8/interprocess/local_socket/index.html)                                                                                                                                                                                    |
| ipc-channel               | **Do not adopt for this feed.** Its documented channels are unbounded, use Serde, and require compatible message representations across programs. It also supplies one-shot bootstrap, a different fit from reconnecting to the single existing supervisor owner. Application bounds and access/owner/version policy would still be required. [ipc-channel 0.23.1](https://docs.rs/ipc-channel/0.23.1/ipc_channel/)                                                                                                                                                                                                                                                                                                           |

Tokio already being in the desktop does not remove the physical-work issue:
started `spawn_blocking` tasks cannot be aborted by dropping/aborting their async
waiter. Tokio recommends dedicated threads for long-lived blocking workloads.
That supports the current short exchange versus persistent subscription split; a
future async transport should be motivated by measured workload/cleanup evidence,
not by assuming an async API cancels native work.
[Tokio 1.54.1 blocking-task lifecycle](https://docs.rs/tokio/1.54.1/tokio/task/fn.spawn_blocking.html)

## What remains application-owned

The [MVP](../MVP.md), [frontend spec](../../.scratch/frontend-navigation/spec.md)
and current implementation require these decisions regardless of crate:

1. Match expected owner before effects and serving owner/exact target before cache
   acceptance. A structurally valid message is not execution authority.
2. Retain durable action identity and uncertainty across attempts; reconnect looks up
   outcomes and never silently repeats potentially admitted work.
3. Choose publication points under owning-state protection, including ambiguous
   persistence and fault overlays; release locks before wake/delivery.
4. Interpret missing/expired/future/other-owner cursors, overflow and sampled resync;
   preserve changes beyond the sampled watermark.
5. Validate exact subscription/batch/cursor acknowledgement; distinguish frontend
   acceptance from refetch completion and domain execution.
6. Enforce absolute frame/ack deadlines, heartbeat rules, byte/count bounds,
   cancellation, socket shutdown and join; retain permits until physical completion.

These are concrete module contracts. Using a proven primitive inside them is
compatible with the repository's design guidance; introducing a broad framework
that exposes additional states to every caller may increase maintenance. DRY and
single-choice favor generated wire declarations; simplicity and information hiding
favor keeping domain policy behind current owning modules. These are tradeoffs,
not a rule that fewer dependencies or fewer lines always wins.
[Rust Design Patterns principles, last change 2026-09-21](https://rust-unofficial.github.io/patterns/additional_resources/design-principles.html)

## Bounded next decision and remaining unknowns

Before F05, prepare a small separately scoped codegen experiment: one generator,
the representative DTOs above, one explicit export/drift command, and unchanged
runtime guards/corpus. Adopt if it removes duplicate wire declarations cleanly,
preserves JSON output and independent core ownership, and works with existing
preparation/checking. Otherwise record why and proceed with F05; a generator is
not an architectural prerequisite. No such experiment was run for this report.

Source review cannot establish:

- Compilation and Serde fidelity for all current DTOs, especially Specta RC29 with
  pinned Tauri, Channel, result wrappers and current Rust newtypes.
- The amount of deleted hand-maintained code after generated imports replace
  intentionally different frontend shapes; compile-time benefit may exceed raw
  line reduction.
- Build-time, binary-size, memory, latency or distribution cost of adding a direct
  dependency/features. No benchmark supports a channel speed claim for this app.
- Actual socket deadlines, stalled WebView behavior, cleanup, access control or
  reconnect correctness on the test Mac; existing F08/fixture-desktop qualification
  remains needed whichever mechanism is selected.
- Durable streamed conversation delivery: F04's in-memory hints are explicitly not
  transcript history. That later workload needs its own retention/paging/recovery
  decision and cannot be inferred from a bounded channel or this checkpoint.

Storage-driver work remains in its separate planned checkpoint. No provider or
archived RAT inspection was needed for this question.
