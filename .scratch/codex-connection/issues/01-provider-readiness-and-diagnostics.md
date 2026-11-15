# 01: Provider readiness and diagnostics

**What to build:** Open Providers settings, see existing Codex authentication and
configuration, and understand how to fix missing or incompatible setup. Claude
Code is inspectable with disabled connection actions. This is a working read-only
setup slice through the production supervisor and real/fixture adapters.

**Blocked by:** [Milestone 02a — UI foundation](../../milestone-01/issues/02a-establish-incremental-ui-foundation.md);
[F08 — Frontend qualification](../../frontend-navigation/issues/08-qualify-frontend-foundation.md)
and [provider-independent workspaces 01](../../provider-independent-workspaces/issues/01-retire-workspace-provider.md)

**Status:** ready-for-agent

## Start here

Follow the [fresh-session procedure](../spec.md#authority-and-fresh-sessions).
Read [connection journey](../spec.md#connection-journey) for readiness, diagnostic
and disabled-Claude behavior, [ownership/protocol](../spec.md#ownership-and-protocol)
for the adapter seam and schema provenance, and
[verification policy](../../../docs/agents/verification.md) before choosing checks.
Confirm predecessor evidence and its outstanding limits before product edits.
Read [architecture and extension constraints](../spec.md#architecture-and-extension-constraints).
Consume the provider-independent workspace handoff; retain milestone setup defaults
in explicit presentation/admission rather than restoring a workspace provider field.

## Acceptance

- [x] Select and record the supported installed Codex runtime/schema cohort before adapter or fixture changes. Inspect official supported-version sources and retained evidence; record revision and captured-versus-synthetic provenance beside fixtures, preserving exact approval/request/turn interpretation for successors.
- [x] Providers settings defaults to Codex and displays existing authentication/account/configuration evidence and Refresh before setup choices. Recognize existing API-key configuration, show the known billing route or unknown, and link relevant workspace setup status here. Provider status remains distinct from supervisor transport diagnostics.
- [x] Missing executable, incompatible protocol, unavailable authentication and status-read failures provide the spec's concrete recovery/recheck guidance with secondary technical details. Retain last-good evidence with a stale/unknown label. Keep Codex sign-in/terminal actions absent until 02/03; guidance can explain the currently external official login path.
- [x] Claude Code is selectable with its not-implemented explanation and disabled connection actions. Tab inspection and disabled controls cause no Claude probing, authentication or process startup.
- [x] Implement the concrete Codex setup connection and deterministic fixture adapter at the same intended execution seam. Separate protocol interpretation from process/stdio mechanics with controlled substitutions; expose typed owner-bound observations through supervisor/Tauri and the existing feature-observation composition. Reads, refresh, route preload and reconnect preserve authentication and start no agent threads/turns.
- [x] Establish the deep provider-connection module in supervisor-hosted Rust `runtimes` with only the readiness operations exercised here. Keep version/protocol interpretation and authentication evidence in the concrete Codex adapter; fixtures supply outcomes without copying admission/reconciliation policy. Keep process/stream/time substitutions internal and retain real parsers/command construction under test.
- [x] Add one app-scoped provider-observation feature module using the existing generated validation, Query cache, feed freshness and physical scheduler. Key reads/results by exact supervisor owner and provider/effective-configuration target, keeping inspected-tab presentation separate. Expose cohesive observations to consumers without another cache, retry authority or universal provider interface.
- [x] General/Providers/tab navigation retains Settings return/focus provenance, the correct landmark label and workspace reading context. Use shared components/tokens and document actual new block contracts; keep the page usable by keyboard and at ordinary/minimum window sizes.
- [x] Passing deterministic units cover version/exit failures, authentication/configuration, owner/target mismatches, unrelated IDs, malformed/oversized frames, unexpected requests, EOF, controlled deadlines and failed/uncertain cleanup. Unit execution substitutes process/filesystem outcomes; preserve production deadlines.
- [x] Use controlled synthetic provider targets to prove independent readiness/freshness and rejection of cross-target or late-owner results. Navigation, route preload and reconnect remain observation-only. Keep Claude effects absent; these targets establish application cardinality, not Claude/OpenCode adapter compatibility.
- [x] Focused actual fixture-desktop evidence demonstrates the readiness/error/Refresh/Claude/navigation states through the production supervisor path, including bounded process cleanup. Record source/cohort, exact checks, results, failed/unperformed limits and the actual interfaces 02 will extend in a dated comment.

## Handoff boundary

This session ends at usable setup observation and diagnostics. Browser sign-in is
[02](02-official-browser-sign-in.md); embedded CLI login is
[03](03-embedded-cli-login.md). Add only the concrete setup operations exercised
here; later chat/approval tickets extend the same seam without placeholder methods.
Keep native compatibility unqualified until its recorded cohort. No sign-in or
agent submission is authorized by these read-only checks. Stop after this ticket.
Record the actual provider-connection and shared observation interfaces and the
captured effective-configuration identity for 02; add no placeholder login/chat
operations or new provider implementations.

## Comments

2026-11-15: Published from the approved four-session split. Implementation and
qualification acceptance remain unchecked.

2026-11-15: Added the separately selected workspace-provider cleanup prerequisite
and the agreed owner/adapter/observation constraints. The readiness slice still
establishes a complete working path; no architecture-only setup ticket is added.


### 2026-11-15 — implemented and qualified for the agreed fixture scope

Implemented on `main`, code cohort `9c770dd326361ee61d8e09d0f39d919ad8645831`
(baseline `38a863d`, implementation `7faf3bc`, review correction `57bd4a6`).
[Evidence, exact commands, provenance, failures and handoff](../../../docs/evidence/codex-connection/01.md)
record the selected installed Codex **0.169.0** / official source
`d32063f96c2dff1dc96af839b3952abb6acfeda4`, synthetic-versus-captured provenance,
protocol **11**, generated preparation/drift, static/type/Clippy gates, **174 Rust /
51 tooling / 223 frontend** units with warm measurements, production packaging,
18 actual process scenarios and actual fixture-desktop readiness/error/Refresh/
Claude/navigation/keyboard/ordinary-minimum-size checks. Independent Standards and
Spec review and re-reviews have zero unresolved findings. Physical group cleanup,
reactive Query snapshots and skip-link return defects were fixed and requalified;
failed attempts remain linked. Owned fixture process audits show no remaining PIDs.
The existing `docs/IDEAS.md` edit was preserved byte-for-byte.

The user selected **user-level configuration**, read from a neutral private cwd,
and **fixture-only qualification**. No real authentication read/login/logout,
credential mutation, native thread/turn, Claude probing or agent submission was
performed. Native compatibility, escaped-descendant and full owned-Quit lifecycle,
a hands-on user trial, the parent milestone and tickets 02–04 remain unqualified
or separately scoped. All acceptance above is checked for this authorized slice;
no successor implementation was included.

02 extends the actual supervisor-hosted `runtimes::ProviderConnections` owner and
its captured private Codex `Launch`; public readiness operations are
`ProviderRequest::Target` and `Observe { target_id, refresh }`, returning typed
`ProviderReply` / `ProviderObservation` with retained evidence and sanitized
diagnostics. Exact identity is supervisor owner plus `ProviderTarget.id` (opaque
configuration scope); the descriptor records selected executable/Codex home/
neutral cwd/version while the inherited environment stays private in Rust.
`ProviderObservations` / `useProviderSetup` compose existing Query/freshness/feed/
physical scheduling. Provider tab search is presentation only. No workspace
provider field, login/chat placeholder or universal provider interface was added.
Use fresh protocol-11 private development state and matching peers; prior formats
need no migration. Stop after this ticket.

### 2026-11-15 — agreed readiness presentation refinement

The user approved the [shared presentation refinement](../spec.md#presentation-refinement--2026-11-15):
automatically verify on committed Codex Providers activation; use an inline glowing
status indicator, compact account facts and collapsed details/help; remove Refresh
from healthy presentation entirely. Failed/missing setup retains contextual
Check again; uncertain cleanup and disconnected status retain their recovery paths.
This prospectively supersedes the Refresh and retained-status-only navigation
parts of the completed acceptance above, preserving that cohort as historical evidence.

The readiness wire contract and Rust owner remain unchanged. `useProviderSetup`
requests one targeted `Observe { refresh: true }` per actual Codex-tab activation,
using the existing shared observation module. Preload gets only the descriptor;
rerenders, invalidations and reconnect do not replay that activation. 02 adds visible
ChatGPT sign-in below readiness; 03 adds CLI alongside it, outside collapsed help.
Both automatically verify login completion and must keep check/login admission in
the same Rust owner. 04 qualifies entry checks and the healthy screen without
Refresh; all sessions keep their focused verification. Successor work is not
implemented by this refinement.

### 2026-11-15 — presentation refinement implemented and fixture-qualified

[Refinement evidence](../../../docs/evidence/codex-connection/01-ui.md) records the
implemented inline status primitive, automatic committed-entry verification,
compact facts, collapsed help/details and contextual Check again. Healthy setup
has no Refresh. The existing readiness wire contract and Rust owner are unchanged;
the shared spec and 02–04 handoffs explicitly retain their sequence and visible
login controls. Historical ticket evidence remains intact.

Formatting, lint, generated drift/types/Clippy, measured **174 Rust / 51 tooling /
237 frontend** units and production packaging passed. Actual packaged fixture
checks covered ordinary/minimum layout, keyboard/focus, entry freshness after
external fixture changes, pending/stale evidence, contextual recovery, API-key and
missing-authentication setup, Claude isolation, wrapped paths/long account values
and Settings return. Both private fixture cohorts ended with no remaining fixture
PIDs. One automation window-binding error was recovered and recorded. Native
authentication/agent compatibility, screen-reader speech and a hands-on user trial
remain unperformed. Tickets 02–04 and parent milestone acceptance remain open.

### 2026-11-15 — review corrections remain within readiness scope

The [shared correction clarification](../spec.md#readiness-correction-clarification--2026-11-15)
keeps both reviewed failures in this ticket: preserve an undispatched readiness
check through same-owner transport detachment, scope entry/attachment markers by
owner/target, and show failed-read/disconnected recovery independently of retained
Checking. An actual retry shows progress and blocks another retry until settlement;
cleanup uncertainty still fences replacement. Rust readiness admission, generated
contracts and the wire interface are unchanged. Browser/CLI login, attempt
reconciliation and terminal ownership remain exclusively in 02/03; 04 retains
cumulative qualification. Correction evidence and re-review are recorded separately
from the original ticket and first UI checkpoint.

2026-11-15: [Correction evidence](../../../docs/evidence/codex-connection/01-ui-fixes.md)
records the reviewed readiness fixes, replacement-scope and retry-feedback variants,
and the actual stopped-delivery issue exposed during qualification. Final gates
passed **174 Rust / 51 tooling / 246 frontend** units, static checks and production
packaging. Corrected actual fixture-desktop outage/reconnection/healthy/minimum-size
checks passed with no remaining fixture PIDs; failed attempts are retained.
Independent Standards and Spec re-reviews each have **0 unresolved findings**.
No Rust/protocol, authentication, login/terminal lifecycle or 02–04 implementation
was added. Native authentication/agent compatibility and user-trial limits remain.
