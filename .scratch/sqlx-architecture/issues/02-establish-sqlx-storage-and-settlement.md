# 02: Establish SQLx storage and settlement

Status: ready-for-agent

Blocked by: [01 — Core startup and complete shutdown](01-own-core-startup-and-complete-shutdown.md).

## What to build

Persist existing action records through one private SQLx storage owner, with honest
commit uncertainty, safe connection retirement and fresh settled observations.

## Context and ownership

Read [storage decisions](../spec.md#storage-topology-opening-and-transaction-settlement),
[settings/gates](../spec.md#testing-decisions-and-qualification-gates),
[cleanup source chain](../review-research.md#the-concrete-reuse-hole),
[amendments A6–A8](../amendments.md), [MVP](../../../docs/MVP.md),
[verification](../../../docs/agents/verification.md) and
[observation guidance](../../../docs/agents/observations.md). Use rust-engineering;
confirm 01's evidence. Own private storage, schema and dependency resolution.

**Integration checkpoint:** 02–04 use one integration branch and one selected driver.
Remove rusqlite at this checkpoint; do not retain dual implementations. Intermediate
callers may not compile until 03/04 convert them. Keep focused pure checks runnable
where possible; record unbuildable/unperformed gates, and do not merge/claim a green
cutover until 04 passes the combined gate. Preserve other working-tree edits.

## Acceptance

- [x] Resolve SQLx 0.9.0 with defaults disabled and required SQLite/Tokio features, one SQLite-linked graph and lockfile; record Rust/SQLite source/version and confirm the engine's WAL-reset fix. No driver contest, fork, raw-handle adoption or plugin.
- [x] One exact current-schema initializer, atomic domain-shaped writes and identity/reservation constraints replace retired migration/legacy readers. Follow the [global MVP development compatibility policy](../../../docs/MVP.md#development-compatibility-policy): fresh state only, actionable rejection before a deliberate reset, and no migration, reader or continued-readability obligation for retired app metadata.
- [x] Writer initializes schema/WAL before bounded read-only/no-create readers. Shared initial/replacement validation applies owner-private canonical/file/link/mode checks and effective WAL/FULL/fullfsync/foreign-key settings, with the accepted same-user race limitation documented precisely.
- [x] Retain an acquired connection outside borrowed transaction scope. Failed statement/commit/rollback retires it explicitly before reuse/reconciliation, preserving original and close failures. Return guard orders queued cleanup with ping, then rejects unresolved tracked transaction/ping failure. No default-ping cleanliness assumption.
- [x] Own writer completion/retirement independently of the requester. Status waits for relevant settlement and uses a fresh consistent snapshot; stale absence and lost commit acknowledgment grant no replay authority. Unknown retirement fences replacement/effects.
- [x] Bound admitted acquisition waiters, rows and bytes, driver/decode/result copies and undelivered results. Short read transactions return committed revision/position; oversized data is explicit. SQL/pools/transactions/closure scheduling stay private.
- [x] Observe incomplete checkpoints/WAL pressure and all operation failures before both close barriers. Pool close is not an aggregate error audit or literal OS-thread join. Partial open cleans all resources while lease remains held.
- [x] Supply substituted decision checks and a separate bounded storage qualification entry point/report for fresh opens, replacement, effective settings/read-only rejection, commit/rollback faults, cancellation, pinned readers, acquisition and close. Failed/unperformed physical fault probes remain explicit and block only their claimed qualification.

## Verification and limits

Q2 begins here and closes with 04. Units substitute SQLite/filesystem outcomes;
actual SQLx/disk fault runs use fresh disposable state outside the unit gate. Keep
original research/evidence intact. Record settings as provisional versus durability
requirements, actual graph, commands and failed/unperformed checks. No live provider
or authentication action is authorized.

## Comments

2026-11-16: Published as the first checkpoint of the indivisible driver cutover; acceptance is unchecked.


2026-11-16: Implemented ticket 02 on the unmerged `sqlx-storage-cutover` integration branch. [Evidence, coverage map and final review](../../../docs/evidence/sqlx-architecture/02/README.md) record runtime source `e4bc30e`, SQLx 0.9.0 / bundled SQLite 3.51.3 with matching WAL-reset-fix provenance, 17 passing substituted storage units, 14 passing actual-disk cases, focused Clippy/TypeScript and 51 tooling / 260 UI tests. Both fresh GPT-6.1 (`gpt-6.1-sol`) High reviewers report zero unresolved findings after fixes. Failed probes/gates are preserved. Checked acceptance records this storage-local checkpoint, not closure of Q2: the complete app gate stops at four deliberately unconverted callers; combined compilation, generated drift completion, packaging/desktop and physical disk/I/O/rollback-close fault qualification remain open through 03/04. Ticket 03 and later tickets remain untouched. No live-provider, authentication, user-state reset or merge action occurred.
