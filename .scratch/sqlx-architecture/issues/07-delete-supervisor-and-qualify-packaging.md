# 07: Delete supervisor and qualify packaging

Status: ready-for-agent

Blocked by: [06 — Native core action and observation slice](06-host-direct-core-actions-and-observations-in-tauri.md).

## What to build

Ship and develop the native-owned app without application-supervisor resources,
discovery/peer transport or obsolete staging, while retaining process qualification
and safe private development state.

## Context and ownership

Read [spec reset/shutdown decisions](../spec.md#bounds-observations-and-diagnostics),
[amendments A4/A7/A8](../amendments.md), [MVP](../../../docs/MVP.md),
[milestone](../../../docs/MILESTONE-01.md), [verification](../../../docs/agents/verification.md)
and [observations](../../../docs/agents/observations.md). Use rust-engineering for
native/tooling changes and relevant installed guidance. Confirm 06's native evidence.

Own retired host/transport removal, workspace/bundle/tooling references, developer
startup/state derivation and qualification command replacement. Preserve others'
edits and all historical evidence. Provider/bridge executables retain provenance.

## Acceptance

- [x] Remove supervisor crate/binary, application discovery/private socket/frame/peer negotiation, stale host DTO/diagnostic exports, bundle resource, staging and dead script/CI assumptions. One production native path remains; no compatibility host or alternate SQLite driver.
- [x] Inspect every old transport assertion before deletion against the coverage map. Preserve owner exclusion, physical completion, action uncertainty, generated validation, exact provider framing and current recovery assertions in their new owners.
- [x] Replace retired qualify:process selection with identifiable bounded storage/native lifecycle qualification. Unit entry point selects substituted units explicitly and needs no staged sidecar. Preparation/check/build/package remain separate without duplicate full-suite runs.
- [x] Audit dev launcher's escalation now that native exit loses the owner: ordinary stop waits for retained shutdown; deadline reports pending/failure, with force-kill recorded as recovery rather than success. No stale-PID signaling.
- [x] Publish and exercise fresh-private-state reset/start instructions under the [global MVP development compatibility policy](../../../docs/MVP.md#development-compatibility-policy): exact overrides, explicit current-schema namespace independent of supervisor hash/protocol, same lease inode on same-root handover, old-owner exclusion and no silent initialize-over-incompatible-state. Retired app metadata/history/preferences need no migration or preserved reader/build; an investigation copy of a stopped database/WAL/SHM set is optional. Preserve every worktree/branch/provider home/native history/evidence set.
- [x] Verify dev and packaged startup, same-root competing owner rejection, WebView/native distinction and owned Quit without supervisor resources. Re-run affected provider login fixture process/desktop paths, without real authentication or agent work.
- [x] Run relevant current generated drift/static/compilation/isolated units/full verification/package gates once per complete gate, preserving private Cargo target safeguards. Record package contents, source/versions, commands, failures, build/bundle deltas and remaining qualification gaps.

## Verification and limits

Q4 deletion/package closure needs actual package/native process evidence; a source
search alone is insufficient. A disposable reset rehearsal grants no authority to
delete the user's existing state/worktrees. Record unperformed fault probes and
retain old supervisor evidence under its original cohort. Stop before 08.

## Comments

2026-11-16: Published for removal after the native slice passes; acceptance is unchecked.

2026-11-16: Implemented only ticket 07. Removed the supervisor/application transport/staging path, retained mapped current ownership/uncertainty/framing coverage, replaced opt-in qualification commands, and qualified final native fixtures, storage/process effects, normal package, exact dev override, same-inode exclusion/handover and disposable incompatible-state/fresh-root rehearsal. [Ticket 07 evidence](../../../docs/evidence/sqlx-architecture/07/README.md) records passing gates, failures/transient-lock cohorts, package deltas and remaining limits. No live-provider/authentication/agent work or existing-state reset; ticket 08 and parent/milestone remain open. Independent review is recorded in the follow-up comment.

2026-11-16: Implementation commits `dbebd8a` and `5f217e2` pass fresh-context [final Standards and Spec reviews](../../../docs/evidence/sqlx-architecture/07/review.md), **0 findings on each axis**, with both model `gpt-6.1-sol` and reasoning `high` explicitly selected and accepted. Initial Spec P2 (foreground Ctrl-C bypassing launcher ordering) was fixed and qualified with actual process-group pending cleanup and real dev Ctrl-C. Final complete gate: **172 Rust / 48 tooling / 261 UI tests**; native/storage/process/package gates pass with recorded limits. Ticket 07 is complete; stop before ticket 08. No live provider/authentication/agent actions or existing-user-state reset.

2026-11-16: Ticket 07 was squash-merged to `main`, as confirmed by the user. The commit IDs above identify the pre-squash implementation and review cohort.
