# 04e: Complete repository and workspace observation modules

Status: ready-for-agent

Blocked by: [F04d](04d-migrate-native-transport.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read the
[owning requirements](../spec.md#query-freshness) and predecessor evidence. Read the MVP,
milestone, verification policy and applicable repository engineering guidance.
[Observation guidance](../../../docs/agents/observations.md) owns shared engineering
pitfalls; the parent spec owns behavior and limits. This ticket is one session;
successors remain separate. Preserve unrelated edits and completed evidence.

## Module and caller interface

Repository/workspace observation modules hide Query definitions, generated structural
validation plus semantic assertions, keys, exact targets, dirty generations and
interests. Views express feature interest and receive narrow observations/targeted
callbacks. Shared mechanics are implemented once, with feature differences local.
Avoid a public observation-description framework or per-command forwarding modules.

## Implementation and removal

- Create the app-scoped Query client, physical-read module and Zustand presentation
  store above layouts. Query is the sole observation cache. Keep owner/target-scoped
  bounded pages and agreed local-IPC defaults; never use Query success as execution evidence.
- Use p-queue concurrency two and the existing 64-distinct-pending admission policy.
  A small unsettled physical registry owns complete request identity, coalesces new
  interests after old consumers dispose, and disappears on physical settlement.
  Cancel pending-only work; detach abort forwarding at start and use no active-task
  abort/timeout that frees capacity early. Preserve local Busy and effect flags.
- Complete repository/workspace Query cache extraction, hint mapping, baseline-before-
  initial-read, generation capture/follow-up and retained-action interests together.
  A small synchronous sink lets the existing feed controller record invalidations
  into feature modules before acknowledgement until F04f replaces that controller.
  Remove its competing feature-dirtiness ledger; retain bounded inspection
  invalidations in the inspection owner until F04f consumes them, without moving
  its observation cache or lifecycle twice.
- Keep dirty state bounded, including global resync generations for hidden targets.
  Hint-driven repository reads use refresh=false; explicit reprobe stays separate.
  Remove covered 250 ms repository/workspace progress/action/availability polling
  only after replacement assertions pass in this session.
- Consolidate duplicated retained-action parsing/presentation glue where useful;
  persist action identity before submission and preserve sticky uncertainty/legacy
  records. Rust still owns action reconciliation. Keep explicit intents outside
  automatic read scheduling/retries and capture the caller-observed owner/target.
- Move app-lifetime inspection ownership above layouts now, preserving its existing
  behavior and shared physical gate. Inspection cache/lifecycle/polling migration is
  F04f, not a temporary inspection Query cache that is replaced twice.

Footholds: `App.tsx`, `hooks/use-{repositories,workspaces,supervisor-connection}.ts`,
`lib/{observation-coordinator,repositories,workspaces,observation-feed}.ts`,
`blocks/workspace-panel.tsx`. Use F04b validators and F04d connection observations.

## Required scenarios

- Hints before baseline and during initial/follow-up reads remain pending until
  satisfied; errors preserve last-good data; hidden details do not fan out reads.
- Query cancellation/remount and same-key interests retain one physical exchange;
  two-active/64-queued bounds, obsolete pending disposal, active drain and overload.
- Owner replacement, exact pending payload, late settlement, local rejection after
  uncertainty and navigation-safe action completion. No automatic resubmission.
- Repository progress works without covered polling, including faults and explicit
  reprobe without Checking/refetch loops. Existing inspection behavior remains intact.

## Acceptance

- [ ] Query and feature modules replace repository/workspace cache/read/freshness machinery; no second observation cache remains.
- [ ] p-queue replaces manual drain/count/queue mechanics while the small physical registry preserves physical guarantees.
- [ ] Repository/workspace push equivalence passes and their covered polling is removed; inspection replacement remains F04f.
- [ ] App-scoped ownership/store foundation and retained actions survive navigation with configured checks/evidence.

Routing stays pre-F06. F05 finishes allowlisted context persistence/integration;
F04f replaces feed/inspection lifecycle, then F04g qualifies actual behavior.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.
