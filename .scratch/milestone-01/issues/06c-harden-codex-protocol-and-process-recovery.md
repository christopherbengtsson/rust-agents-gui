# 06c: Harden Codex protocol and process recovery

**What to build:** Complete adversarial setup coverage and recovery presentation,
fixing defects in the existing bounded runtime owner and adapters.

**Blocked by:** [06b: Setup end to end](06b-check-codex-setup-end-to-end.md).

**Status:** ready-for-agent

**Parent:** [06: Codex setup](06-connect-codex-with-actionable-setup-diagnostics.md).

## Start here

Read the [accepted plan](../codex-setup-plan.md), 06b's completion evidence and
actual interfaces, [MVP](../../../docs/MVP.md),
[Milestone 01](../../../docs/MILESTONE-01.md),
[verification policy](../../../docs/agents/verification.md) and
[component guide](../../../src/components/README.md). Apply local Rust, React and
shadcn skills where applicable. Reinspect the pinned official cohort before
adapter/fixture changes and retain all provenance.

Switch to `feature/m01-codex-setup` and continue from 06b's completed commit. Keep
this child's commits on that branch; do not create a new branch or implement on
`main`.

## Acceptance

- [ ] Complete schema-grounded coverage of version/exit failure, authentication and
      configuration, unrelated/differently typed request IDs, duplicate/late frames,
      malformed/oversized input, unexpected server requests, EOF and deadline paths.
      Unknown or unexpected requests never grant permission.
- [ ] Exact operation/request/thread/turn/item/approval identities survive
      interpretation without lossy normalization or permissive matching. Synthetic
      approval/recovery inputs are labelled and establish no native outcome.
- [ ] Controlled time/events or immediate timeout outcomes exercise production
      deadline decisions without production waits. Bounds account for queued and
      in-flight work and retained stdout/stderr/frame bytes before allocation.
- [ ] Worker/process/stdio ownership, stop/join, early failures and cleanup
      uncertainty have deterministic coverage. Child exit does not alone establish
      descendant cleanup; saved PIDs confer no authority. Unresolved ownership
      survives restart through required evidence and fences conflicting replacement.
- [ ] Preference commit/rollback uncertainty, lost Save replies, authoritative
      rereads, save/check serialization, invalidated readiness, and newer draft
      preservation remain correct. Reconnect and changed owner never replay work.
- [ ] Frontend retained/stale/unknown/auth/setup/transport states stay distinct;
      settings entry/exit, target/owner/config changes, hidden effects, delayed
      responses and error cleanup cannot steal context or invent check failure.
- [ ] Diagnostics remain optional for normal runtime use. Process work continues
      independently of UI visibility. No credentials/raw config/provider payloads
      leak through displayed errors, diagnostic records or Debug/Display paths.
- [ ] Copy/open error paths, narrow-window readability, accessible pending/error
      feedback and existing CSP integration are covered or explicitly handed to
      desktop qualification. No unrelated component/source regeneration is added.
- [ ] Configured checks and review pass; retained fixture instructions cover the
      adversarial/error scenarios 06d must demonstrate. Record coverage, failures,
      commands, source/version identity and remaining qualification separately.

## Completion and handoff

Commit and add a dated evidence comment mapping parent requirements to controlled
coverage and executable fixture scenarios. Record any production fixes and
unperformed desktop/OS/native checks without converting mock evidence into those
claims. Keep fixture faults deterministic and explicitly synthetic where applicable.

Stop after this child. [06d](06d-qualify-codex-setup.md) performs actual qualification
and final review/fixes. Parent 06 stays open. No live submissions, login changes or
native execution compatibility claim are authorized.
