# One current application design

Status: ready-for-agent

Accepted 2026-11-16 in this task. Implement the approved cleanup plan and user
clarification: production and active guidance should read as one coherent design,
without transitional adapters, migration machinery or special cases for earlier
choices. This is a bounded cleanup, not a new architecture selection.

Authority: AGENTS.md, docs/MVP.md development compatibility policy, native-core/
SQLx spec and amendments, and docs/agents/verification.md. Baseline:
`a00a243259ff238bbb73e8dd9c0682572b72a9d1`.

## Required result

- Remove retired-choice compatibility readers/defaults/aliases and compatibility-only
  tests wherever found; preserve current-build recovery and external-provider support.
- Require complete current pending registration/workspace envelopes and explicit
  uncertainty. Unreadable action evidence stays visible, preserves bytes, and fences
  submission until deliberately resolved and reopened. Absence is distinct.
- Require initial-workspace fields in registration requests/records, including
  presence of the nullable ID. Require presentation contexts and use presentation revision 1. Regenerate current
  contracts together and retain meaningful current-format recovery coverage.
- Repair the no-op workspace recovery test: assert exact status lookup and
  settlement with zero creation submissions. Remove retired-shape reconstruction
  from the registration serialization test.
- Use one current schema/revision 1 and native-schema-1 default roots. Explicit
  overrides remain exact; incompatible/unrecognized state fails with guidance.
  No migration ladder, old filename detection or automatic reset/fallback.
- Update active guidance; preserve historical evidence separately identified.
  Append correction to earlier cleanup completeness claims in new evidence.

## Fresh start and verification

User explicitly selected cleanup plus fresh start of normal app state. Inventory
owners/roots, complete matching native Quit, preserve old roots/worktrees/branches,
start fresh current roots, clear only presentation and the two pending-action
WebView keys. Preserve provider credentials/native history and unrelated storage.
No live-provider allowance is supplied. Report any reset or qualification limitation.

Use repository React/Rust engineering, UX/component guidance for affected controls,
and writing-for-agents for skill guidance. Existing hook/codec/storage seams govern
focused red-green tests; repository policy allows private owning-module seams.
Run focused checks during implementation, one final complete verify, separate
fresh native/storage qualification, independent Standards and Spec reviews against
the baseline, then commit the completed work. No full-MVP qualification claim.
