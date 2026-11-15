# 06: Connect Codex with actionable setup diagnostics

**What to build:** Deliver the Codex connection outcome through the current
[Codex connection feature](../../codex-connection/spec.md): provider readiness,
official browser sign-in, compatibility checks and cumulative qualification.
Connection 03's separate CLI/terminal capability is deferred by user decision.
The separately selected
[workspace-provider cleanup](../../provider-independent-workspaces/issues/01-retire-workspace-provider.md)
precedes those sessions; this parent does not implement it.

**Blocked by:** [02a — UI foundation](02a-establish-incremental-ui-foundation.md);
[F08 — Frontend qualification](../../frontend-navigation/issues/08-qualify-frontend-foundation.md);
[connection 04 — Qualified connection journey](../../codex-connection/issues/04-qualify-connection-journey.md)

**Status:** ready-for-agent

## Parent and session index

This is the milestone parent/index, not one implementation session. Shared behavior
is authoritative in the [connection spec](../../codex-connection/spec.md); select
one child ticket per fresh Codex session. Children inherit the foundation
prerequisites directly through connection 01, so this parent is not their blocker.
Connection 01 also requires the provider-independent workspace prerequisite.
The [MVP](../../../docs/MVP.md), [milestone](../../../docs/MILESTONE-01.md) and
[verification policy](../../../docs/agents/verification.md) retain their authority.

| Session | Deliverable | Blocked by |
| --- | --- | --- |
| [Connection 01](../../codex-connection/issues/01-provider-readiness-and-diagnostics.md) | Providers page, existing authentication/status and setup diagnostics | Workspace-provider cleanup 01, Milestone 02a, F08 |
| [Connection 02](../../codex-connection/issues/02-official-browser-sign-in.md) | Official browser sign-in, retained progress and cancellation/reconciliation | Connection 01 |
| [Connection 02a](../../codex-connection/issues/02a-runtime-compatibility-policy.md) | Minimum stable runtime policy and required capability checks | Connection 02 |
| [Connection 03](../../codex-connection/issues/03-embedded-cli-login.md) | Deferred CLI/terminal proposal; not part of current acceptance | Not actionable |
| [Connection 04](../../codex-connection/issues/04-qualify-connection-journey.md) | Cumulative desktop, lifecycle and accessibility qualification | Connection 02a |

## Aggregate acceptance

- [x] Connection 01, 02 and 02a each have verified acceptance, focused unit/fixture-desktop evidence and dated successor handoffs. Record 03 as deferred, without claiming implementation or qualification.
- [x] Connection 04 records cumulative acceptance/results, real-authentication allowance/results or explicit limits, and the A3 outcome without claiming native-agent execution qualification.
- [x] The supported schema/fixture provenance and real/fixture adapter seam remain available to later chat/approval tickets. Claude actions stay disabled; API-key setup and general terminals stay deferred.
- [x] The provider-independent workspace prerequisite is verified, and 01, 02, 02a and 04 record the agreed provider-connection ownership, adapter locality and shared observation design, including synthetic-target isolation evidence and its limits.

The next milestone ticket remains [06a — Storage evaluation](06a-evaluate-storage-before-durable-chat.md),
which waits for connection 04 and this aggregate outcome before 07. The connection fixture aggregate is recorded below; native authentication, the
hands-on trial and whole-milestone qualification were open at that fixture checkpoint;
the separate trial below adds existing-authentication readiness and participant
feedback, with its remaining native limits. This documentation split grants no sign-in,
installation or provider submission allowance.

## Comments

### 2026-11-14 — frontend foundation prerequisite

Follow the [agreed foundation](../../frontend-navigation/spec.md). Runtime setup belongs at Settings → Providers → Codex, with relevant workspace status linking there. Keep SupervisorConnection transport diagnostics separate. This does not authorize provider sessions.

### 2026-11-14 — library refactor handoff

The amended foundation adds F04b–F04g before revised F05–F08; F08 remains this
ticket's direct blocker. Consume the qualified async connection and feature
observation/actor interfaces without creating a new runtime, queue or cache in
Tauri/screens. Provider protocol/fixtures still require their own supported-version
sources and allowance; application protocol 8 does not establish Codex compatibility.

### 2026-11-14 — F07 Settings extension seam

[F07](../../frontend-navigation/issues/07-routed-settings.md) supplies the routed
Settings layout and General placeholder. Extend that composition with Settings →
Providers → Codex and link relevant workspace status there when implementing this
ticket. Keep runtime setup distinct from SupervisorConnection transport diagnostics.
F08 remains the prerequisite; this pointer implements no provider setup.

### 2026-11-15 — connection journey agreed; documentation-only publication

The shared Providers page above supersedes the earlier **Settings → Providers →
Codex** successor wording: Codex and Claude Code are tabs on one page, with Codex
selected by default and all Claude connection actions disabled. The agreed browser
and embedded CLI sign-in actions replace the former blanket prohibition on login
changes; authentication and credentials remain official-runtime-owned. API-key
setup is omitted; existing runtime configuration remains observable. The terminal
ends at login completion and preserves progress across navigation, unlike
Conductor's nonpersistent interactive-shell login wrapper.

This publication updates requirements and linked guidance only. The
[research note](../../../docs/research/conductor-provider-connection.md) retains
source provenance and limits. F08 and 02a blockers, 06 → 06a → 07 sequencing,
completed evidence, strict approval matching and native-agent allowance boundaries
are unchanged. No application code, generated contracts, runtime/login state or
implementation acceptance is changed.

Publication checks: local links and heading anchors across all eight changed/new
Markdown files passed; `git diff --check` passed. Ticket 06 acceptance remains
unchecked and F07's previous content was preserved verbatim. Historical links
outside this workspace were not inspected. No application units, builds,
fixture-desktop, real-authentication, native-agent or user-trial checks were run
for this documentation-only publication.

### 2026-11-15 — split into separate connection sessions

The approved [connection spec](../../codex-connection/spec.md) now owns the shared
requirements previously held in this ticket. Connection 01–04 are separate
implementation sessions with explicit blockers, focused evidence and completion
criteria. This ticket is their milestone parent/index; implement a selected child
rather than the whole feature at once. The comments above retain their original
publication history, including the first eight-file link-check report.

This supersedes single-session execution of 06 while preserving its behavior and
milestone sequence: connection 01 → 02 → 03 → 04 / parent 06 → existing 06a → 07.
All acceptance remains unchecked. No application implementation or qualification
was performed by this split.

Split-publication checks passed: 212 local links and 47 heading anchors across
15 changed/new Markdown files; the four-child dependency chain and parent/06a/07
gates; unchecked implementation acceptance; preserved F07 content and prior parent
comments; and `git diff --check`. Seven historical outside-workspace links were
skipped without inspecting their targets. Application tests/builds, desktop,
authentication and native-agent qualification were not run for this docs-only split.

### 2026-11-15 — provider architecture and workspace prerequisite

The user agreed one deep provider-connection owner with concrete adapters and
provider-scoped observations reusing the foundation. The
[architecture constraints](../../codex-connection/spec.md#architecture-and-extension-constraints)
now govern every child. The separately selected
[workspace-provider cleanup](../../provider-independent-workspaces/spec.md) precedes
connection 01; the four connection sessions and existing 06a identifier remain.
This amendment documents future work only. Historical publication checks and
completed milestone/foundation evidence above remain checkpoint results.

### 2026-11-15 — browser-only authentication scope

The user approved existing-authentication detection plus one official-runtime
browser sign-in action. Separate CLI login, embedded terminals, PTY/xterm and a
read-only command-output replacement are deferred. The current sequence is
01 -> 02 -> 02a -> 04 / parent 06 -> 06a -> 07. Ticket 03 and its plan retain the
superseded proposal with unchecked acceptance; they no longer block this aggregate.
This amendment changes current requirements only. Completed evidence and the
historical publication comments above remain unchanged; cumulative 04 and parent
acceptance remain open. No application implementation or qualification occurred.

### 2026-11-15 — connection fixture aggregate recorded

Connection 01, 02, 02a and [04](../../codex-connection/issues/04-qualify-connection-journey.md)
now have checked fixture acceptance and dated evidence. [Cumulative A3/architecture mapping](../../../docs/evidence/codex-connection/04.md)
links their supported-schema provenance, concrete adapter seam, shared observation
module, provider-independent workspace handoff and synthetic-target isolation.
03 is deferred, not passed. Real authentication and Christopher's hands-on trial
move to a separate selected session starting with existing authentication; no
credential replacement, native agent thread/turn or terminal work occurred.
Whole-milestone/native execution qualification remains open. Existing 06a retains
its separate storage-evaluation scope before 07; neither successor is implemented.

### 2026-11-15 — separate hands-on trial recorded

[Christopher's existing-authentication trial](../../../docs/evidence/codex-connection/04-trial.md)
used the qualified unchanged packaged desktop with private app state. Real Codex
0.169.0 readiness detected existing ChatGPT authentication and billing; Christopher
reported clear presentation except for the sign-in button remaining visible while
Connected. He requested recording that finding and fixing it in a separate session.
No product code changed; normal Quit and process cleanup were observed. Existing
fixture acceptance/evidence is preserved. Browser sign-in and native-agent execution
remain unperformed, and whole-milestone qualification remains open. No credential
replacement or agent thread/turn was performed. Storage evaluation 06a and chat 07
were not started.
