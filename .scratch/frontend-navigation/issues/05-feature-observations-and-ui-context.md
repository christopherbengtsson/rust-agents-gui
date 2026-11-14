# 05: Complete retained presentation context and integration

Status: ready-for-agent

Blocked by: [F04g](04g-qualify-library-refactor.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read
[Zustand context requirements](../spec.md#zustand-presentation-context), predecessor
actual qualification and [observation guidance](../../../docs/agents/observations.md).
F04e/F04f now own complete feature observation/cache/freshness/polling migration;
consume their actual interfaces and evidence instead of repeating it.

## Context and implementation

Finish the lightweight app-scoped presentation store introduced by F04e. Persist a
validated/migratable allowlist for workspace-selected file/comparison context,
reading position, panel layout and selected remembered navigation defaults. Report
storage failures visibly. Query owns observation bodies, actors own coordination,
and Rust owns execution/persistence/reconciliation authority.

- Retain context across workspace switches and Settings-style interest removal;
  revalidate comparison-specific file/reading context before restoration.
- Preserve pending action records and sticky uncertainty independently of preference
  migration. Stored navigation is a restoration default, not a second current-target
  authority. F06 owns route selection and the old selection-preference-to-route migration.
- Confirm one app-lifetime Query/physical-read/feed/inspection owner and narrow feature
  subscriptions. Resolve integration defects without reintroducing polling or caches.
- Reconcile predecessor evidence against original F05 observation requirements;
  link them as implemented by F04e/F04f/F04g, with unperformed checks still open.

Source footholds: follow F04e–F04g's new app composition, feature observation modules,
actor modules and Zustand store; baseline `App.tsx`, inspection blocks and old Hook
preference helpers identify existing behavior, not required final file organization.

## Required scenarios

- Validated/migrated preferences restore allowed context and report read/write failure.
- Workspace/settings-style switches retain context, while changed comparisons reject
  stale file/scroll restoration and hidden views start no reads.
- Heavy previews/transcripts, authoritative snapshots, credentials and execution
  authority never enter the presentation store; pending identities survive migration.
- Existing observation freshness and physical drain remain covered by current
  predecessor tests; new contextual integration scenarios are selected by the gate.

## Acceptance

- [ ] Allowlisted presentation context persists/migrates with visible storage errors and validated comparison restoration.
- [ ] Pending action identity/uncertainty remains independent of presentation migrations.
- [ ] Shared app-lifetime owners and feature interfaces integrate without duplicate caches, lifecycle state or covered polling.
- [ ] Original observation requirements have linked passing predecessor evidence plus relevant current integration checks.

## Handoff boundary

F06 supplies route authority and final navigation preference migration; F07 supplies
actual Settings. F08 retains full routed desktop/accessibility/packaging/performance
qualification. This ticket creates no new observation framework or provider behavior.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — fresh-session handoff clarified

Expanded source pointers, implementation boundaries and observable completion
scenarios from the agreed plan. No implementation or acceptance is claimed.

### 2026-11-14 — F04a rescope

The user selected complete feature slices: Query/freshness/covered polling replacement
now finishes in F04e/F04f, followed by F04g actual qualification. This ticket retains
presentation persistence and integration. Historical comments describe the former
scope; acceptance above is current and remains unchecked.
