# Current version, revision and schema sweep

2026-11-16 · Read-only source research against commit `4c58b37` after the
current-format cleanup. This note changes no product code, tests, configuration,
application state or historical evidence. No providers, tests, generators or
qualification cohorts were run. Repository source and retained Git history are
the primary sources for these application-owned decisions; this is not a new
external-provider compatibility assessment.

## Governing interpretation

The [MVP development compatibility policy](../MVP.md#development-compatibility-policy)
requires one current intended implementation and fresh development state, without
retired-build readers, migrations or compatibility-only tests. It preserves
current-build recovery, ownership, cleanup, external-provider compatibility and
historical evidence. The [native-core spec](../../.scratch/sqlx-architecture/spec.md)
and [amendments A3/A7/A10](../../.scratch/sqlx-architecture/amendments.md) apply that
policy to contracts, storage and the development namespace.

The requested baseline is application metadata schema **1**, presentation format
**1**, and default namespace **`native-schema-1`**. Runtime revisions, generations,
cursors and incarnation identities are separate concepts: their numbers describe
changes or ownership during execution, rather than successive architectural choices.

## Confirmed current-documentation errors

| Finding                                                                        | Primary sources                                                                                                                                                                                                                                                                                                                                       | Recommended cleanup                                                                                                                                                                        |
| ------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| The root README still describes current metadata as schema 4.                  | [README.md](../../README.md), line 94, contradicts the current schema constant and initialization SQL in [storage/opening.rs](../../crates/core/src/storage/opening.rs), lines 9 and 32. Current default paths are in [main.rs](../../src-tauri/src/main.rs), line 139, and [development-state.mjs](../../scripts/development-state.mjs), line 25.    | Say schema 1, or link to the current development-state owner to avoid another duplicated numeric claim. Preserve schema-4 evidence from its original checkpoint.                           |
| The contract README still assigns legacy preferences separate migration rules. | [contracts/README.md](../../contracts/README.md), lines 46–47. Its opening historical notice at lines 3–6 refers to numbered protocol sections, while the F04b section at line 26 mixes still-current generation commands and contract guidance with this superseded preference rule. The MVP policy forbids retaining that rule as current guidance. | Separate current generation guidance from historical cohort descriptions; remove the legacy-preference migration instruction from current guidance. Keep historical checkpoint provenance. |

Neither finding establishes a remaining production compatibility implementation.
The production readers/writers inspected below already use the current baseline.

## Naming and historical-boundary housekeeping

| Item                                                                | Evidence and classification                                                                                                                                                                                                                                                                                                                                                           | Recommended treatment                                                                                                                                                                                                      |
| ------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Active native qualification retains milestone labels.               | [qualify-native.py](../../scripts/qualify-native.py), lines 2, 11 and 15–16, names Q4, `aw-native07-`, `Native07.app` and a `native07` bundle identifier. These label a qualification cohort; they do not select an application schema or retired format.                                                                                                                             | Use neutral native-lifecycle names for future runs if removing architectural-history terminology is desired. Never rename old reports, bundles or referenced evidence. Lower priority than incorrect current instructions. |
| The discovery fixture driver has an incomplete historical boundary. | [qualify-discovery.py](../../fixtures/codex-readiness/qualify-discovery.py), lines 2–8, advertises runnable usage, while lines 52, 56, 64 and 66 require the removed supervisor/protocol seam. The [fixture README](../../fixtures/codex-readiness/README.md), lines 5–7, explicitly labels only `qualify.py` historical. Current package/scripts do not select the discovery driver. | Explicitly label both supervisor drivers historical, or place them under a clearly historical evidence boundary. Do not restore the retired protocol or port this driver solely to preserve it.                            |

The `protocol >= 11` special cases in historical
[qualify.py](../../fixtures/codex-readiness/qualify.py), lines 470–475, are likewise
retained checkpoint logic, not an active production version switch. Existing
superseded schema-3 descriptions in [ADR 0005](../adr/0005-readable-workspace-reservations.md),
line 16, and the dated follow-up at [MILESTONE-01.md](../MILESTONE-01.md), lines
153–161, describe historical decisions/results. Their old numbers should not be
mass-replaced; improve historical labeling only where a reader could mistake them
for current instructions.

## Current baseline and guards to retain

| Surface                          | Reader/writer trace                                                                                                                                                                                                                                                            | Assessment                                                                                                                                                                                                                                                                                                                                                                                                                            |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Metadata schema                  | [opening.rs](../../crates/core/src/storage/opening.rs):9 declares `SCHEMA_VERSION = 1`; line 32 initializes `user_version=1`; lines 222–229 read the marker, initialize an empty unmarked database, and reject other identities/versions with deliberate fresh-state guidance. | Correct baseline. There is one schema and no migration ladder. Empty-database initialization is necessary current behavior. Rejection is a guard against reinterpretation, not old-format support.                                                                                                                                                                                                                                    |
| Development root                 | [main.rs](../../src-tauri/src/main.rs):134–139 honors an exact explicit override or selects `AgentWorkspace/native-schema-1`; [development-state.mjs](../../scripts/development-state.mjs):25 agrees.                                                                          | Correct baseline; no peer/binary-version namespace negotiation.                                                                                                                                                                                                                                                                                                                                                                       |
| Presentation                     | [presentation-store.ts](../../src/lib/presentation-store.ts):129,133,243 uses format 1.                                                                                                                                                                                        | Correct baseline; no need to increment it to reflect retired choices.                                                                                                                                                                                                                                                                                                                                                                 |
| Database application identity    | [opening.rs](../../crates/core/src/storage/opening.rs):10,31 writes `0x41575332` / `1096241970`; lines 223 and 228 compare that identity separately from `user_version`.                                                                                                       | Identity magic, not schema version 2. It decodes to ASCII `AWS2`, but the originating commit `c85add8a` already paired it with schema 1; [ticket-02 evidence](../evidence/sqlx-architecture/02/README.md):15 records the same pair. No source establishes the trailing digit as a schema counter. Retain unless deliberately choosing a different application magic; do not normalize it as a revision or add compatibility handling. |
| Invalid state/path guards        | [opening.rs](../../crates/core/src/storage/opening.rs):58–63 rejects missing metadata in a nonempty root; [records.rs](../../crates/core/src/workspaces/records.rs):219–242 validates normalized absolute workspace paths.                                                     | Retain current ownership/resource protection and actionable rejection. Neither path reads or transforms a retired format.                                                                                                                                                                                                                                                                                                             |
| Optional current snapshot fields | [lib.rs](../../crates/core/src/lib.rs):56–61 uses optional/defaulted diagnostic and login-pending fields.                                                                                                                                                                      | Current contract omission behavior; no retired-version branch. Defaults alone are not evidence of compatibility debt.                                                                                                                                                                                                                                                                                                                 |

## Runtime revisions and identities to retain

| Value                                   | Producer and consumer                                                                                                                                                                                                                                                                      | Why it remains                                                                                                                                                                             |
| --------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Storage commit revision                 | [operations.rs](../../crates/core/src/storage/operations.rs):23 increments within the write transaction; lines 439 and 495 read and return the committed snapshot revision.                                                                                                                | Orders current-build writes and identifies a consistent read snapshot. Fresh storage starts at zero before its first write ([opening.rs](../../crates/core/src/storage/opening.rs):12–13). |
| Snapshot/observation revision           | [workspaces.rs](../../crates/core/src/workspaces.rs):190,218–221 initializes/increments it; [lib.rs](../../crates/core/src/lib.rs):382–395 compares the snapshot cursor.                                                                                                                   | Detects changes and establishes whether observations must reset.                                                                                                                           |
| Native incarnation                      | [native.rs](../../crates/core/src/native.rs):103 creates a UUID; lines 217–222 reject requests addressed to another incarnation. [OwnedReply](../../crates/core/src/connection.rs):77–82 returns serving identity.                                                                         | Fences stale authority and results after a new native lifetime. It is not a build version.                                                                                                 |
| Workspace generation                    | [workspaces.rs](../../crates/core/src/workspaces.rs):415–425 changes it on fencing; [coordinator.rs](../../crates/core/src/workspaces/coordinator.rs):76–85 checks it before continuation.                                                                                                 | Prevents delayed work from starting an effect after its owner is fenced.                                                                                                                   |
| Inspection generation                   | [inspection.rs](../../crates/core/src/workspaces/inspection.rs):410,446 mutates it; lines 1029–1047 compare it to publish invalidation.                                                                                                                                                    | Detects changes to inspection work and records.                                                                                                                                            |
| Provider configuration revision         | [provider_connection.rs](../../crates/core/src/runtimes/provider_connection.rs):463 checks the expected revision; [login.rs](../../crates/core/src/runtimes/provider_connection/login.rs):173–188 binds repeated login identities and admission to the configuration revision.             | Protects exact action/configuration identity and rejects stale actions.                                                                                                                    |
| Feed cursor, sequence, watermark, batch | [observations.rs](../../crates/core/src/observations.rs):30–32,88–124 produces and interprets bounded journal positions; [subscription.rs](../../crates/core/src/connection/subscription.rs):251–285,305–327 binds delivery and acknowledgements to incarnation/subscription/batch/cursor. | Supports catch-up, resync, bounded delivery and exact acknowledgement.                                                                                                                     |
| Naming/allocation cursors               | [naming.rs](../../crates/core/src/workspaces/naming.rs):27–38 traverses candidates; [operations.rs](../../crates/core/src/storage/operations.rs):327–335 persists monotonic continuations.                                                                                                 | Resumes current workspace allocation without retrying already exhausted candidates.                                                                                                        |
| Frontend cache/freshness counters       | [feature-query-cache.ts](../../src/lib/feature-query-cache.ts):66,107 and [feature-freshness.ts](../../src/lib/feature-freshness.ts):4,28,36 advance cache revisions and per-feature invalidation clocks.                                                                                  | Current freshness coordination. Normalizing these to 1 would change behavior.                                                                                                              |

## External and tooling versions to retain

The application version **0.1.0** is consistent across [Cargo.toml](../../Cargo.toml):7,
[package.json](../../package.json):3 and [tauri.conf.json](../../src-tauri/tauri.conf.json):4.
[Core::status](../../crates/core/src/lib.rs):372–376 reports that package version;
the current [native corpus](../../contracts/observations-native.json):8 records it.
This is release identity, not a retired architecture counter.

The generated [manifest](../../src/generated/contracts/manifest.json):2–9 records
`format: 1` plus actual generator versions; its writer is
[prepare-generated.mjs](../../scripts/prepare-generated.mjs):114. Source/artifact
hashes are reproducibility evidence, not a protocol negotiation mechanism.
[codegen.rs](../../crates/core/src/codegen.rs):26–30 deliberately emits JSON Schema
draft 2020-12. Preserve the dialect, dependency versions, lockfile formats and
external Cargo `metadata --format-version 1` arguments: these belong to their tools.

Codex **0.169.0**, its pinned source revision, upstream schema `v1`/`v2` paths,
synthetic **0.170.0** scenarios and incompatible/prerelease cases describe the
external provider contract. The [fixture provenance](../../fixtures/codex-readiness/README.md):9–24,59–77
identifies the source cohort and synthetic nature. The production
[version parser](../../crates/core/src/runtimes/codex/protocol.rs):287–304 enforces
minimum stable admission separately from required protocol probes. Keep these
checks and unmodified upstream schemas, including upstream descriptions containing
“legacy”; they do not support retired application builds.

The current corpus is selected by [generated-contracts.test.tsx](../../src/generated-contracts.test.tsx):2,
[observation-contracts.test.tsx](../../src/observation-contracts.test.tsx):4,
[observation-feed-bridge.test.tsx](../../src/observation-feed-bridge.test.tsx):5,
[observation-feed.test.tsx](../../src/observation-feed.test.tsx):144,
[connection tests](../../crates/core/src/connection/tests.rs):17 and
[subscription tests](../../crates/core/src/connection/subscription/tests.rs):118.
No active source selects a numbered historical corpus.

## Recommended follow-through and research limits

Correct the two active documentation defects first. Clarify the discovery-driver
historical boundary and consider neutral future qualification names separately.
No additional production version-normalization change is supported by this sweep.
Preserve numbered historical corpora and completed evidence, and continue to select
only the current native corpus and native qualification path.

Research covered production Rust, generated contract sources/artifacts, frontend
presentation/cache/freshness version markers, active preparation/qualification
tooling, provider fixtures and current versus historical documentation. Local
Intent discovery succeeded; its listed skills concern TanStack implementation,
which this documentation-only task does not change. The Rust wire-contract guidance
was applied under repository verification policy. Archived RAT was not inspected.
The result is static source research, not a new unit, fixture-desktop, native-provider,
storage-durability or clean-start qualification claim.
