# 04a: Publish the library refactor architecture

Status: ready-for-agent

Blocked by: [F04](04-push-observation-delivery.md)

## Start here

Follow [fresh-session instructions](../spec.md#starting-a-fresh-session), read the
[owning requirements](../spec.md#1-purpose-scope-and-publication) and predecessor evidence. Read the MVP,
milestone, verification policy and applicable repository engineering guidance.
[Observation guidance](../../../docs/agents/observations.md) owns shared engineering
pitfalls; the parent spec owns behavior and limits. This ticket is one session;
successors remain separate. Preserve unrelated edits and completed evidence.

## Documentation-only scope

This is the current publication session, not authority to implement F04b onward.
Product source, manifests, lockfiles, tooling scripts, generated assets, and original
F01–F04 evidence remain unchanged. Correct current summaries without rewriting
historical outcomes. Use writing-for-agents to keep requirements, rationale,
evidence and execution criteria in their respective owners.

## Deliverables

- Amend the parent spec with shared mechanisms, feature-specific interfaces,
  generated contracts, async transport, ownership, complete feature slices and gates.
- Publish F04b–F04g as self-contained sessions and rescope F05–F09/milestone handoffs.
- Publish researched stable versions, maintenance/fit sources and unperformed checks;
  add ADR 0007 and the amendment pointer from ADR 0006.
- Update applicable engineering/TanStack guidance and distinguish current component
  lifetimes from their planned replacement. Record the F04 source/evidence baseline.

## Verification and acceptance

- [x] Spec, ADR and research agree on the selected modules/libraries and physical-work guarantees.
- [x] Tickets have resolvable blockers, source footholds, replacement scope, scenarios and completion evidence requirements.
- [x] F05 and later handoffs consume predecessor results without repeating their migrations.
- [x] Changed Markdown/link/consistency checks pass; only intended documentation changes are present, unrelated edits and historical evidence preserved.

Record commands, failures/unperformed checks and scope in
[F04a evidence](../../../docs/evidence/frontend-navigation/f04a.md). Only these
documentation acceptance items may be checked here. F04b is next.

## Comments

### 2026-11-14 — F04a publication

Published after the user approved library refactors and all four deepening
opportunities. Acceptance remains unchecked until this ticket's implementation
and verification produce linked evidence. No provider submissions are authorized.

### 2026-11-14 — F04a documentation complete

Published the amended spec, seven session tickets, ADR/research and guidance.
Rescoped F05 and updated later handoffs without changing product/tooling/dependency
source or historical results. [Evidence](../../../docs/evidence/frontend-navigation/f04a.md)
records formatting, links/anchors, blocker order, scope checks and unperformed
implementation qualification. F04b is next; F04b–F04g/F05–F08 acceptance stays open.
