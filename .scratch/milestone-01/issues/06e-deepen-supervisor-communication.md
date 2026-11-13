# 06e: Deepen supervisor communication

**What to build:** Keep existing Settings, repository and workspace requests
responsive when another request is delayed, concentrating supervisor transport
mechanics before streamed chat adds another caller.

**Blocked by:** [06d: Qualify Codex setup](06d-qualify-codex-setup.md).

**Status:** ready-for-agent

**Parent:** [Milestone 01 implementation spec](../spec.md).

## Start here

Read the [communication decisions](../spec.md#supervisor-communication),
06d's completion evidence, [MVP](../../../docs/MVP.md),
[Milestone 01](../../../docs/MILESTONE-01.md), and
[verification policy](../../../docs/agents/verification.md). Use the repository
Rust skill; read the component guide and React skill if presentation code changes.
Inspect current connection/discovery contracts and their controlled tests first.

Continue on `feature/m01-codex-setup` from completed 06d, retaining predecessor
commits. This is a milestone follow-on, separate from parent 06's original setup
acceptance. Preserve its completed evidence and original acceptance criteria.

## Acceptance

- [ ] Deepen the existing communication module: share privacy checks, socket
      setup, framing and deadlines across domain exchanges. Keep typed domain
      validation, errors and reconciliation with their current owners; preserve
      definite local rejection versus unavailable-reply distinctions.
- [ ] Keep shared connection-state locks short and perform socket I/O outside
      them. A delayed observation permits an independent request to progress.
      Retain serialized initial discovery, correct cursor/owner updates and
      bounded in-flight/queued work compatible with supervisor admission limits.
      Lifecycle/control traffic must not inherit an observation's wait.
- [ ] Preserve owner exclusion, private access, version checks, frame/output
      bounds and existing public commands/wire representations. Actions use the
      discovered owner; failed exchanges never start another owner or replay work.
      Keep discovery and action authority distinct across concurrent reads and
      late results.
- [ ] Exercise all domain exchanges through existing controlled I/O seams,
      retaining production validation/framing. Cover delayed observation alongside
      another request, initial discovery and cursor races, capacity, malformed or
      timed-out frames, structured errors and exact no-replay outcomes. Unit
      checks use substituted filesystem/socket/time outcomes, with no real
      supervisor/provider processes or production waits.
- [ ] Separately demonstrate existing Settings save/check, repository/workspace
      reads and desktop reconnect through the production supervisor with private
      fixture state. Retain actionable stale/error states, exact action settlement
      and navigation responsiveness. Distinguish controlled interleaving evidence
      from actual desktop latency and OS ownership evidence.
- [ ] Run relevant configured checks, required measurements and packaging as
      separate gates under policy; record source/build provenance, failures and
      focused desktop results. Preserve 06d as historical qualification and record
      regressions against the changed communication implementation.

## Completion and handoff

Commit and append a dated completion comment linking controlled checks, focused
desktop/process evidence and remaining limits. Stop after this ticket.
[07: Streamed chat](07-durable-streamed-chat-and-uncertain-send-recovery.md) uses
this implementation in its own session to add bounded stream/catch-up behavior.

This ticket changes existing request progress and transport locality. Stream
subscriptions, transcript delivery, new conversation messages, general scheduling
and a provider framework remain with their owning future work. No live provider
process, thread/turn, login change or native execution qualification is authorized;
fixture-desktop results do not qualify many-agent concurrency or complete M1.
