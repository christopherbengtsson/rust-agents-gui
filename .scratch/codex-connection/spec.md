# Codex connection — Milestone 01 ticket 06

## Native core amendment — 2026-11-16

The [accepted native-core/SQLx spec](../sqlx-architecture/spec.md) and
[supersession amendments A2/A5](../sqlx-architecture/amendments.md) govern
native host/owner, setup-attempt crash lifetime, progress contracts and whole-app Quit handoff.
They replace conflicting instructions below for subsequent work. Remaining
policy and recorded checkpoint results retain their scope; publication claims
no production change or new qualification pass.

Status: ready-for-agent

Agreed 2026-11-15; current scope follows the authentication amendment below.
This feature implements the outcome tracked by
[Milestone 01 ticket 06](../milestone-01/issues/06-connect-codex-with-actionable-setup-diagnostics.md)
through 01 -> 02 -> 02a -> 04, after the separately selected
[provider-independent workspace prerequisite](../provider-independent-workspaces/spec.md).
Completed 01/02/02a evidence remains in those tickets; 04 and parent acceptance
remain open. The historical four-session publication is retained below.

## Authentication scope amendment — 2026-11-15

The user approved automatically recognizing existing Codex authentication and one
explicit official-runtime browser sign-in action. A separate CLI option, direct
`codex login` helper, interactive terminal, PTY/xterm and read-only command-output
viewer are deferred. [03](issues/03-embedded-cli-login.md) and its linked plan retain
superseded research with unchecked acceptance; they are not actionable or blockers.
[04](issues/04-qualify-connection-journey.md) follows completed 02a directly.

The normal inspected CLI login also starts browser authentication; no distinct
required recovery case justified a second path in this local-only app. Conductor's
separate product-account connection is not this app's authentication architecture.
Credentials remain owned by local Codex, and existing API-key configuration remains
observable without adding setup controls. Preserve browser progress, cancellation,
reconciliation, authoritative verification and physical cleanup. This amendment
supersedes conflicting prospective terminal requirements and historical publication
comments, not completed evidence. It implements and qualifies no product behavior.

Publication checks passed: 169 local links and 48 heading anchors across ten
changed planning/guidance documents; current dependency/deferral and preserved
history assertions; and `git diff --check`. Seven historical outside-workspace
links were skipped without inspecting their targets. The Git index and existing
IDEAS edits were preserved. Application tests/builds, desktop, authentication and
native-agent qualification were not run for this documentation-only amendment.


## Authority and fresh sessions

[MVP](../../docs/MVP.md) owns product scope and execution ownership;
[Milestone 01](../../docs/MILESTONE-01.md) owns A3 acceptance and qualification;
the [milestone implementation spec](../milestone-01/spec.md) supplies the wider
workflow. This spec owns the shared connection journey and constraints; child
tickets own session deliverables and completion evidence. The
[frontend foundation](../frontend-navigation/spec.md) remains the prerequisite.

1. Work only the selected child ticket. Confirm its blockers using their checked
   acceptance and dated evidence before product edits; the sequence does not
   authorize continuing into the next ticket.
2. Read the selected ticket's requirement pointers and predecessor handoff. Follow
   [verification policy](../../docs/agents/verification.md) before selecting checks,
   [observation guidance](../../docs/agents/observations.md) for IPC/lifetimes, and
   the [component guide](../../src/components/README.md) for UI work. Use applicable
   repository-local UX/UI, React, Rust, shadcn and XState skills under repository
   precedence; inspect installed Intent guidance when the task requires it.
3. Inspect current source/scripts and preserve existing changes. Earlier session
   evidence identifies actual interfaces and versions; a plan is not proof that a
   module, method or dependency already exists.
4. Deliver one working increment through core, supervisor, typed IPC and desktop.
   Add dependencies with their first use. Unit-test the owning decisions with
   controlled dependencies and qualify the ticket's focused fixture-desktop path
   in that session; cumulative ticket 04 does not defer those checks.
5. Append a dated evidence comment to the selected ticket with source identity,
   exact checks/results, unit/fixture/native distinctions and unperformed limits.
   Check only verified acceptance, identify successor interfaces and stop. Required
   failed/unperformed checks stay open. Parent 06 and whole-milestone acceptance
   stay open until their own aggregate gates pass.

## Session sequence

The approved [02a — Runtime compatibility policy](issues/02a-runtime-compatibility-policy.md)
prerequisite follows 02 and blocks 04; 03 is deferred. It replaces exact Codex version admission
with a minimum stable baseline and required per-connection capability checks.
Historical 01/02 qualification cohorts remain unchanged.

First complete [provider-independent workspaces 01](../provider-independent-workspaces/issues/01-retire-workspace-provider.md),
which removes workspace-level provider ownership through current-format records,
generated contracts and desktop observations. It is a bounded prefactor, not a
provider framework or a storage-driver redesign. Connection 01 consumes its dated
handoff alongside the existing foundation prerequisites.

All sessions follow the global [MVP development compatibility policy](../../docs/MVP.md#development-compatibility-policy).
Update the current application model/generated contracts together and use fresh
private state for incompatible changes; retired application compatibility work is
excluded throughout the MVP and every milestone. Current-build recovery/owner
checks and the selected external Codex protocol compatibility remain required.

| Ticket | Working outcome | Blocked by |
| --- | --- | --- |
| [01 — Readiness and diagnostics](issues/01-provider-readiness-and-diagnostics.md) | Shared Providers page, existing Codex setup/status and actionable errors through real/fixture adapters | Provider-independent workspaces 01; Milestone 02a; foundation F08 |
| [02 — Browser sign-in](issues/02-official-browser-sign-in.md) | Explicit official-runtime browser login, progress return, cancellation and reconciliation | 01 |
| [02a — Runtime compatibility](issues/02a-runtime-compatibility-policy.md) | Minimum stable version admission and required per-connection capability checks | 02 |
| [03 — Embedded CLI login](issues/03-embedded-cli-login.md) | Deferred proposal; outside current acceptance | Not actionable |
| [04 — Qualify the connection journey](issues/04-qualify-connection-journey.md) | Cumulative desktop/lifecycle/accessibility evidence and separately recorded authentication limits | 02a |

01 establishes the actual setup adapter seam and supported-schema provenance;
02 extends it with browser authentication and the login-attempt owner;
02a adds compatibility checks, and 04 qualifies their combined journey. Deliver
only the methods exercised by current scope, retaining the real/fixture seam for
later chat/approval tickets.
Keep unavailable Codex actions absent until their implementing ticket; Claude's
explicitly requested disabled actions are the scoped presentation exception.

After 04, complete the [parent 06 evidence](../milestone-01/issues/06-connect-codex-with-actionable-setup-diagnostics.md)
without rerunning unchanged checks, then proceed only when selected to existing
[06a — Storage evaluation](../milestone-01/issues/06a-evaluate-storage-before-durable-chat.md)
and [07 — Durable streamed chat](../milestone-01/issues/07-durable-streamed-chat-and-uncertain-send-recovery.md).
06a retains its identifier; this feature's 01–04 are not milestone ticket numbers.

## Connection journey

The intended benefit to validate is: **“I connected Codex—or understood exactly what to fix—without leaving my workspace behind.”** Conductor supplies the layout and interaction reference; this app retains its own execution ownership and milestone limits.

- Extend routed Settings with one shared **Providers** page and selectable **Codex** / **Claude Code** tabs, defaulting to Codex. Relevant workspace setup/status links open the Codex tab. Keep provider setup distinct from `SupervisorConnection` transport diagnostics.
- Show detected authentication and account/configuration status first. Each actual activation of the Codex Providers tab requests one fresh, bounded read-only readiness check through the existing provider-observation module. This includes first entry and return from another provider, Settings section or workspace. Preloading, rerendering and feed invalidation do not create another activation request. Same-scope reconnect retrieves authoritative status without replaying admitted work; a visible readiness interest preserves any known-undispatched check while baseline admission is unavailable. A replacement supervisor owner or configuration target receives its own fresh readiness check and cannot inherit the previous scope’s entry/attachment markers. Rust admits/coalesces checks and retains cleanup fencing. Show **Checking…** immediately, label retained evidence **Last known**, and show the green **Connected** indicator only when current evidence establishes usable setup. Healthy presentation has no Refresh/recheck control, including in Technical details. Failed or missing setup offers contextual **Check again** only when a new check is permitted. Already-configured users need no sign-in action or login-method choice. Show the observed authentication/billing route when available, otherwise unknown; recognize existing API-key configuration without offering API-key setup controls.
- Use shared controls and semantic status/detail styling. Present one explained **ChatGPT** browser sign-in action beneath readiness, outside Setup help. Existing runtime authentication is recognized automatically; add no CLI method selector, terminal or command viewer. Merely opening Providers starts no authentication and changes no billing route.
- An explicit browser sign-in action starts official Codex-managed authentication, opens its returned URL, and presents pending, completed, failed and cancelled outcomes. Offer reopening the same browser attempt without automatically creating another. Recheck authoritative authentication after completion; opening the browser or receiving a completion notification alone is not proof of readiness.
- Preserve pending browser sign-in across provider tabs, Settings sections and workspace navigation. Display a compact **Codex sign-in in progress** status linking back to Providers. Returning retrieves the same attempt; navigation does not cancel it. Provide an explicit **Cancel sign-in** action.
- Pair failures with their next useful action: missing executable → official installation instructions and explicit recheck; unsupported version/protocol → supported-version guidance and recheck; missing authentication → sign-in; browser-open failure → reopen the current attempt; uncertain delivery/cleanup → status reconciliation before retry. Put technical diagnostics beneath recovery guidance. Retained observations remain visibly stale/unknown when verification fails. Use a compact account table and an inline 6–8 px status light with a soft static green halo on healthy setup; text communicates every state. Model/configuration fields, runtime paths and diagnostics belong in collapsed **Technical details**. General external instructions belong in collapsed **Setup help** when healthy; relevant recovery and future login controls stay visible outside these disclosures.
- After authentication, show the authoritative connection result without automatically starting agent work or navigating away. Preserve workspace reading context and the user's return destination. Back/Escape must retain the established return/focus behavior across General, Providers and provider-tab changes; nested overlay keyboard handling takes precedence. Supply the correct page landmark label, accessible names, focus return, visible focus and usable ordinary/minimum-window layouts.
- Keep Claude's tab inspectable with **“Claude connection isn't implemented yet”** and disabled connection actions. Avoid an elaborate unusable form. Tab inspection and disabled actions trigger no Claude readiness probes, sign-in, terminal, adapter or runtime processes.

## Ownership and protocol

- Keep connection, login-attempt and owned-process lifetimes, admission, cancellation, reconciliation and retries inside the deep provider-connection module in supervisor-hosted Rust `runtimes`, following the [architecture and extension constraints](#architecture-and-extension-constraints). Tauri forwards explicitly targeted actions and observations; screens use the established observation interfaces without creating another runtime, queue or cache. Bind results to the expected owner, provider/configuration and exact attempt; prevent duplicate/competing login starts, ignore stale results, and reconcile lost replies rather than replaying authentication on mount/reconnect.
- Preserve runtime-owned credentials/configuration. Explicit user actions may initiate official-runtime sign-in; extract/store no subscription tokens, silently change no login state, and silently substitute no billing route. Do not collect API keys or change credential-storage settings in this milestone. The sign-in action is not an app-owned credential or provider-mode setting.
- The supervisor owns browser-login processes independently of the routed view. Cancellation, late setup and explicit Quit retain owned handles until physical cleanup is observed; failed/unknown cleanup stays visible and fences replacement. Login presentation remains transient rather than durable chat history. Introduce terminal/process extensions only with a separately selected consumer.
- Before adapter or fixture changes, select/record the supported Codex runtime version, inspect its official protocol/schema and retained evidence, and record schema/source revision and captured-versus-synthetic provenance beside fixtures.
- Ground observer/approval-validation assumptions in the selected official schema and retained local evidence, preserving exact operation/request/turn matching. Qualify the selected cohort explicitly; archived repository inspection is not required.
- Implement the concrete Codex stdio app-server adapter and deterministic fixture adapter at the same execution seam, using supported-version protocol behavior rather than guessed payloads.
- Expose connection/configuration observations and actionable missing-executable, incompatible-protocol, and unavailable-authentication errors in the desktop through core actions/observations.
- Separate protocol interpretation from process/stdio mechanics with small internal substitutions. Test deadline decisions using controlled time/events or immediate timeout outcomes; keep production deadlines intact and unit execution free of runtime processes or waits.

## Architecture and extension constraints

These are supporting parts of one design, not separate architectural systems or
horizontal implementation tickets. The first working readiness slice establishes
the exercised interface; browser sign-in and compatibility work extend it.

- **Provider-connection owner:** one cohesive Rust module hides readiness,
  login-attempt admission, authoritative verification, stale-result rejection,
  reconciliation and cleanup fencing. Browser login and readiness share that owner. Keep
  process handles with the execution owner until physical completion is observed;
  navigation releases presentation interest only. The existing observation shutdown
  path is not a completed provider-Quit mechanism: each login slice adds and
  qualifies cleanup for the resources it introduces.
- **Identity and cardinality:** provider availability, the inspected tab, login
  attempt, conversation binding and process identity remain distinct. Scope
  observations/actions to the expected supervisor and exact provider/configuration;
  bind attempt traffic to its exact identity as well. Readiness and login
  admission exclude conflicting work within the same effective credential scope. An
  unrelated provider's status or attempt must not be overwritten or fenced by
  another's result or failed cleanup, except where an actual shared resource
  requires it. App-wide resource budgets remain valid.
- **Runtime compatibility:** follow the MVP provider policy. Codex initially accepts
  stable releases >= 0.169.0 (including build metadata), rejects prereleases, and
  requires successful initialize/initialized plus account/gatewayOAuth/read on each
  connection before subsequent authenticated requests. Accept additive fields and
  bounded informational notifications; keep required fields and exact targeting
  strict. A probe proves support, not authentication or native qualification.
- **Concrete adapters:** Codex protocol/version interpretation, correlation,
  authentication evidence and diagnostic meaning stay inside its adapter. The
  deterministic fixture adapter substitutes runtime evidence at the same execution
  seam; it does not implement application admission or recovery policy. Internal
  process/stream/time substitutions retain the real parser and command construction
  under test. The supervisor's protocol-8 codec is not evidence for provider framing.
  Extend only operations used by the current slice; derive future provider-specific
  authentication methods from supported evidence instead of requiring Codex's flow.
- **Provider observations:** add one app-scoped feature module to the existing
  observation composition. It hides generated validation, contextual owner/target
  checks, Query identity, freshness and pending-attempt interests from Settings,
  workspace guidance and compact progress. Query remains the sole observation
  cache, using existing feed invalidation and physical scheduling. Pending attempts
  retain observation interest above routes; reconnect retrieves status without
  replay. Presentation memory stores only allowed choices/context, never a second
  copy of execution or authentication state.
- **Conversation ownership:** workspaces remain shared and provider-independent;
  later `conversations` owns each conversation's provider/native mapping, durable
  sends, turns, approvals and exact resume. Provider connection readiness alone
  does not start a conversation. Milestone capability limits belong in admission
  and presentation, not workspace/storage identity.
- **Extension limit:** support several provider targets structurally now, proving
  isolation with synthetic owning-module targets and controlled outcomes. This
  does not implement or probe Claude/OpenCode, add enable switches, dynamic plugin
  loading, a universal capability interface, another cache/scheduler, or multi-account
  configuration management. Concrete future adapters and supported operations arrive
  with their first selected feature. Terminal mechanics remain deferred
  until an explicitly selected consumer requires them.

Use the owning module's interface and small internal effect seams as the test
surface under repository verification policy. Removing the provider-connection
module should redistribute lifecycle decisions among callers; a forwarding-only
module does not earn its place. Existing feature-observation tests supply prior
art for target isolation, retained interests and freshness; existing workspace
effect substitutions supply prior art for deterministic Rust policy tests.

## Verification and qualification

- Unit-test setup with schema-grounded frames and mocked process outcomes: existing authentication/configuration, missing executable, version/exit failure, unrelated IDs, malformed/oversized frames, unexpected requests, EOF, timeout and cleanup uncertainty. Cover duplicate actions, lost replies, stale results, competing readiness/login and duplicate-login rejection, navigation continuity, cancellation during late setup, and browser completion without verified authentication. Metadata/status reads and navigation must not start login, threads or turns; disabled Claude actions have no effects.
- Prove independent provider/configuration targets with controlled synthetic outcomes: distinct readiness records, late-result isolation, same-scope authentication exclusion, unrelated-target progress, scoped cleanup fencing and provider/attempt-specific observation identity. Keep application policy real. Synthetic targets establish cardinality and routing decisions, not Claude/OpenCode protocol support or native concurrency qualification.
- Separately demonstrate actual fixture-desktop setup and browser-handoff states with private test state and controlled fixture commands. Check progress discovery and retained browser attempts after navigation, keyboard/focus/landmark behavior, ordinary/minimum-window sizing and actual cleanup. Controlled DOM/unit success is partial evidence; perform a hands-on connection task with Christopher when feasible and record its limits.
- Keep metadata/schema inspection read-only with respect to authentication; explicit user-requested sign-in is a separate action. Record any real authentication check as its own agreed scenario, runtime/schema/configuration cohort and result, distinct from fixture-desktop and native-agent qualification. Do not silently sign out or replace existing credentials to manufacture test states. No real authentication check may start an agent thread or turn.
- Record inspection sources, versions, exact checks, results, failures and unperformed qualification. Synthetic setup success and real authentication success do not establish native execution compatibility. Starting a real agent session is outside this feature's allowance.

## Scope boundary

Use deterministic fixtures and private test state for execution checks. This
feature covers Codex setup only: API-key setup UI, installer/bundling,
model-selection flows, Claude/OpenCode integration, separate CLI login, embedded
authentication/development terminals and command-output viewers remain deferred. Recognize existing runtime-owned API-key configuration. Real
authentication checks retain separately agreed scenarios/results; these tickets
authorize no live agent submissions. Real authentication success does not establish
native chat/approval/resume compatibility. The current scope amendment changes
requirements only; it does not complete deferred 03 or cumulative 04 acceptance.

## Reference evidence

Read [Conductor connection research](../../docs/research/conductor-provider-connection.md)
when comparing UI, authentication or terminal behavior. It records hash-verified
shipped-source evidence and intentional differences, not successful sign-in,
original source organization, installed Codex compatibility or a user trial.

## Publication record — 2026-11-15 architecture amendment

The user agreed one cohesive provider-connection design with concrete adapters and
shared observations on the existing foundation. Published the separate workspace
provider cleanup as a prerequisite; retained connection sessions 01–04 and their
focused evidence requirements. Updated the MVP, milestone/current successor
guidance, glossary and conversation/storage handoffs without changing historical
checkpoint results or completing implementation acceptance.

Documentation checks passed: 224 local links and 47 Markdown heading anchors in
16 changed/new documents; prerequisite/connection/chat dependency edges and cycle
checks; documentation-only changes relative to the staged baseline; unchecked
feature implementation acceptance; and `git diff --check`. Seven historical
outside-workspace links were skipped without inspecting their targets. Application
units, generation, builds, fixture-desktop, authentication, native-agent and user
trial qualification were not run for this documentation-only amendment.

## Review record — 2026-11-15

Independent Standards and Spec reviews examined the planning publication
`96aa6ff...564e573` against repository policy and the agreed provider architecture,
workspace cleanup, session boundaries and development compatibility policy.
Standards found one precedence ambiguity: the higher-precedence frontend spec
still required preference migrations. Its prospective amendment now adopts the
milestone policy and replaces active migration requirements with current-format
validation/restoration requirements, preserving historical checkpoint evidence.
Standards re-review: zero unresolved findings. Spec review and re-review: zero
findings. This reviews documentation consistency; implementation and qualification
acceptance remain open.

## Presentation refinement — 2026-11-15

The user approved minimal Conductor-inspired readiness presentation, an inline
illuminated status indicator and automatic verification on actual Codex-tab entry.
This supersedes the original always-visible Refresh and retained-status-only
navigation requirements for future work. Historical ticket-01 evidence remains
unchanged. Entry checks use the existing owner/configuration-targeted readiness
operation; navigation starts no authentication, thread or turn. Initial disconnected
entry waits for a usable owner/descriptor. Same-scope reconnect preserves any
known-undispatched readiness intent and retrieves status without replaying admitted
work. Actual owner/target replacement gets its own read-only verification and cannot
inherit another scope’s entry/attachment markers. A new actual tab activation
requests verification again. These readiness rules authorize no authentication
attempt replay; browser/CLI admission and reconciliation remain 02/03.

02 adds prominent ChatGPT sign-in below readiness; 03 adds CLI alongside it while
retaining browser emphasis. Both already require automatic authoritative verification
after login, and must coordinate readiness admission with active authentication and
uncertain cleanup in Rust. 04 qualifies entry freshness and the healthy screen
without Refresh alongside the complete journey. Each ticket retains its focused
checks. Login controls must remain outside collapsed Setup help.

### Readiness correction clarification — 2026-11-15

The UI refinement's review found lost entry verification before baseline admission
and failure recovery hidden by retained Checking. Their correction belongs to 01's
existing readiness/observation presentation, not the browser or CLI lifecycle. Keep
a visible readiness interest bound to its exact retained owner/target through
transport detachment; release it on navigation or actual scope replacement. Baseline
readiness gates queue admission; already admitted physical reads retain their
identity. Scope entry/attachment markers by owner and target. A replacement scope
gets its own readiness verification, without replay to the old scope.

A failed read's retained Checking phase is not current progress. Show recovery and
contextual Check again when permitted; during an actual retry show Checking, retain
the previous failure explanation and withhold another retry until that read settles.
Disconnected guidance and cleanup uncertainty remain visible, and cleanup fencing
continues to block replacement. No new provider wire operations, authentication,
login-attempt owner, terminal lifecycle or successor implementation is introduced.

The Providers view also consumes the existing observation-feed availability. Stopped
or failed delivery labels retained evidence unavailable/stale and directs the user
to the existing workspace supervisor-connection recovery. It offers no ineffective
readiness recheck while delivery is unavailable. This is readiness presentation of
an existing observation, not a new feed/reconnect owner or authentication mechanism.


## Connection UX and discovery amendment — 2026-11-15

The accepted follow-up [05 — Refine Codex connection UX and executable discovery](issues/05-refine-connection-ux-and-discovery.md)
supersedes conflicting prospective presentation/discovery requirements above.
Completed 01–04 and the separate existing-authentication trial remain historical
results; their acceptance is not rewritten.

Detection/rechecking skeleton-loads the entire Codex section body while Settings
navigation and provider tabs remain usable. Verified Connected shows a green
indicator and available account identity only; existing API-key configuration
shows “Connected · Using API key”. Hide sign-in, technical details, billing/plan
and general help while healthy. Missing authentication shows yellow “Not connected”
and Sign in. Pending browser authentication remains actionable: yellow status,
browser instructions, Reopen browser and Cancel sign-in, preserving the exact
attempt across navigation. Verification failure shows yellow “Couldn’t verify
Codex”, contextual Check again and explicitly labeled last verified account;
it does not establish signed-out status. Missing executable shows red “Codex not
found”, Installation guide, Check again and collapsed “Already installed?” with
Locate Codex… inside. Confirmed setup/sign-in failures show red recovery. Text,
accessibility, focus, uncertainty and physical cleanup fencing remain required.

Refresh performs fresh automatic discovery for desktop launches, including bounded
shell PATH recovery and fresh symlink resolution. A native executable chooser is
the only manual selection form. Rust validates version/capabilities, checks auth
automatically and remembers a validated selection in private application state.
An unavailable remembered selection stays selected and requires explicit recovery;
never silently switch installations. Credentials/configuration remain runtime-owned.
Shell discovery updates launcher PATH only, preserving the target credential scope.
No automatic installation or initial-discovery codex doctor is added.

No backward compatibility, migration or mixed-version support is required. Current
build restart recovery, exact owner/target/action identity and external runtime
compatibility remain required. 05 reuses unchanged passing evidence and qualifies
changed deterministic and packaged paths separately. Real browser sign-in requires
a separate exact scenario; sign-out, credential replacement and agent threads/turns
are excluded. Storage 06a and chat 07 remain separately selected work.
