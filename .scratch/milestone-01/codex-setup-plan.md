# Ticket 06 — Settings and actionable Codex setup

Status: ready-for-agent

Accepted 2026-11-13. This preparation changes documentation only. Product
implementation and qualification remain open. Preparation branch: `main`;
review base: `b3f8cf00db6965786db6ace294a1a85fd7d01817`.

## Authority and starting a session

This plan elaborates [parent 06](issues/06-connect-codex-with-actionable-setup-diagnostics.md)
under the [milestone spec](spec.md), [MVP](../../docs/MVP.md),
[Milestone 01](../../docs/MILESTONE-01.md),
[verification policy](../../docs/agents/verification.md) and
[component guide](../../src/components/README.md). Those owners remain authoritative.
Apply the repository-local [React](../../.agents/skills/react-engineering/SKILL.md),
[Rust](../../.agents/skills/rust-engineering/SKILL.md) and
[shadcn](../../.agents/skills/shadcn/SKILL.md) skills where applicable.

The agreed preparation saves this plan, the
[navigation research](../../docs/research/settings-navigation.md), child tickets,
and parent links; reviews the documentation; and commits the preparation changes
on the current branch. It implements no product behavior. Preserve existing user
edits and predecessor evidence. Do not treat a later request for one child as
authorization to start the following child.

Inspect the current checkout and predecessor completion comments before each
session. Continue on the preparation branch unless the user explicitly chooses
another branch. Each implementation child ends with relevant verification, review,
evidence, a commit, and an explicit stop. Parent 06 closes only against its original
acceptance evidence; child completion alone does not check the parent boxes.

## Delivery sequence

| Session | Ticket | Deliverable | Blocked by |
| --- | --- | --- | --- |
| 1 | [06a: Settings and retained workspace context](issues/06a-settings-and-retained-workspace-context.md) | Small navigation module, Settings composition, focus/return, retained workbench and late-result fencing. | Existing shell/workspace foundation |
| 2 | [06b: Codex setup end to end](issues/06b-check-codex-setup-end-to-end.md) | Recorded cohort, deep runtime module, production/fixture adapters, executable persistence and actionable setup observations through existing typed transport. | 06a |
| 3 | [06c: Protocol and process hardening](issues/06c-harden-codex-protocol-and-process-recovery.md) | Complete adversarial framing, identity, delayed-result, persistence uncertainty, reconnect, timeout and cleanup coverage; fix discovered defects. | 06b |
| 4 | [06d: Qualification and review](issues/06d-qualify-codex-setup.md) | Actual desktop/process fixture qualification, accessibility/responsiveness evidence, packaging, final review/fixes and parent acceptance cross-check. | 06c |

06b includes production bounds and owned cleanup from its first working check.
06c expands adversarial coverage; it is not permission to leave a known unsafe
intermediate implementation. Tickets 07–13, Claude integration and whole-milestone
qualification remain separate.

## Settings and application navigation

### Accepted experience

- Add an accessible Settings gear in the left sidebar footer.
- Settings replaces the workbench/sidebar visually. Show **Settings**, Back,
  category navigation, and **Agent runtimes** as the first category.
- Show functioning Codex setup content. Claude joins this category when its
  integration is implemented; add no unavailable provider tabs or unrelated
  settings placeholders.
- Preserve the current workspace, active tab, selected diff file, panel
  configuration and reading position while visiting Settings. Back restores that
  retained workbench and appropriate focus.

### Small navigation module

Use no router. The accepted MVP uses workspace selection and document tabs,
without general Back/Forward history or addressable application URLs. The linked
research records the full-MVP reasoning, alternatives and reconsideration triggers.
The accepted architecture is recorded in
[ADR 0006](../../docs/adr/0006-application-owned-desktop-navigation.md).

The navigation module coordinates workbench/Settings transitions, return context,
focus intent, and whether an earlier asynchronous completion may redirect
selection. The sidebar is its main caller; Settings Back uses the same rules.
Keep it close to `App`, extracting a focused Hook when it concentrates these
responsibilities rather than merely renaming a screen setter.

Existing repository/workspace hooks retain canonical selections and observations.
Introduce no mirrored selection store or effect-based two-way synchronization.
Entering Settings invalidates an earlier registration/creation action's permission
to redirect selection. That action still settles against its exact target and
updates observations. Cover navigation away and back to the same selected identity.
Workspace presentation keeps owning tabs, file selection, panel configuration and
reading position.

### Retention and accessible composition

Retain one workbench through React Activity at a stable identity. Preserve the
physical changes-reader lane across hide/reveal. Hidden inspection effects pause;
shared supervisor/repository/workspace observations remain outside the hidden
Activity. Future continuing conversation observations follow the same rule.
React documents Activity's retained state/DOM and hidden-effect cleanup:
[Activity](https://react.dev/reference/react/Activity).

Opening Settings focuses its heading. Back restores focus to the Settings trigger
or an appropriate workbench fallback. Maintain one active main landmark, unique
IDs and a working skip link. Account for hidden dialogs, Sheets, tooltips and
portal focus cleanup.

Compose existing shadcn/Base UI controls and semantic tokens: sidebar navigation,
native button semantics for screen actions, labelled FieldGroup/Field inputs,
associated help/errors, explicit Save submission, separate Check/reset/copy
actions, Spinner with visible pending text, Alert errors and Badge status. Keep
existing compact variants and CSP integration, composition props/refs and qualified
customizations. Document new product blocks in the component guide.

Preserve link semantics for documentation. The local shadcn skill's anchor-through-
Button example conflicts with installed-version official Base UI guidance; use
the supported link treatment rather than copying that example:
[Base UI Button](https://base-ui.com/react/components/button#rendering-links-as-buttons).
Add no blanket CSP relaxation or unrelated foundation regeneration.

## Deep Rust runtime module

### Ownership and interface

Add `runtimes` beside `workspaces` in Core. Its interface hides setup sequencing,
protocol interpretation, process ownership, bounds, deadlines and cleanup
uncertainty. Keep future durable sends, approval validity and execution
reconciliation in `conversations` as required by the MVP.

Expose typed requests/replies for observing runtime configuration/setup, saving or
resetting the executable preference, explicitly admitting a check and retrieving
its status. Bind observations to runtime, supervisor owner, configuration revision
and check identity. Validate reply shape and exact targets in TypeScript. Tauri
forwards targeted requests and presents observations. Update transport versioning
and compatibility checks as the concrete contract requires.

Distinguish transport availability, local pending feedback and runtime evidence.
A disconnected read does not establish a failed check. Repeated requests for the
same admitted check retrieve its status rather than spawn again. Preserve identity
and required recovery evidence before runtime effects; a changed owner or missing
observation never grants automatic replay authority.

Use a bounded runtime-owned worker for slow process work. Admission/status retrieval
stays short; process work holds no presentation or shared observation locks.
Keep process/stdio, protocol, controlled-time and storage substitutions private
where possible. Production and deterministic fixture adapters share protocol
interpretation. Add concrete mechanics required now; later tickets extend them,
without a generic provider registry, scheduler or workflow framework.

### Executable preference and saving

Persist the application-owned executable preference through existing supervisor-
owned SQLite storage using a narrow runtime storage seam. Do not make runtime
settings depend on repository/creation policy or repository-specific errors.

Resolve Codex from a saved absolute override, otherwise inherited PATH, then
`~/.local/bin`, `/opt/homebrew/bin`, `/usr/local/bin`. Invoke directly without a
shell. An invalid override produces guidance rather than silent fallback.

Serialize preference saves and setup checks. Draft editing remains available while
work is pending; Save and another Check wait for settlement and confirmed cleanup.

- Save launches no Codex process. A successful save invalidates prior readiness
  evidence.
- Check setup tests the authoritative saved configuration and is disabled while
  unsaved edits remain.
- A Save result updates the saved baseline without replacing edits typed after
  submission.
- Definite rejection preserves the previous saved preference. An uncertain commit
  or unavailable reply requires authoritative configuration retrieval before
  further setup work; never assume the previous value remains saved.
- Handle actual SQLite transaction state and rollback/cleanup outcomes before
  reporting certainty. Keep recovery identity separate from discardable draft
  presentation.

### Authentication and diagnostics

**Settings diagnostics never gate normal use.** Codex retains authentication across
restarts. In later chat tickets, explicit Send/Resume connects using existing
authentication and reports actionable failures. Users need not visit Settings
first. This is a constraint on later admission, not chat implementation in ticket 06.

Opening Settings and reconnecting retrieve observations only. **Check setup**
explicitly performs one bounded application-wide metadata check, then closes its
owned process. Inspect executable/version, initialization, existing authentication
and application-wide configuration. Do not claim project-override validation or
successful model execution.

Successful diagnostic results are session observations. Their absence after a
supervisor restart requires neither login nor a Settings visit. Persist evidence
needed to recover unresolved process ownership.

Show resolved executable/version, progress/timestamp, evidenced authentication and
provider information, and billing route when established; otherwise unknown. Keep
missing-executable, incompatible-version/protocol, unavailable-authentication,
configuration, timeout and cleanup guidance distinct. A successful diagnostic is
**Setup check passed**, not proof that a model request will succeed.

Provide official recovery instructions and copyable external commands. Copy/open
failures stay visible; the existing copy helper's swallowed rejection is not a
sufficient failure path. Open documentation externally through narrowly scoped
Tauri opener support using application-owned official destinations:
[Opener](https://v2.tauri.app/plugin/opener/).

Invoke no login/logout, credential extraction/editing, runtime configuration
writes, threads or turns. Retain runtime-owned authentication and billing choices.

### Protocol and process lifecycle

Initial supported cohort: **Codex 0.167.2**, official source revision
`ce21072c90d32c40af7a55b60b2fb35c84a43a6a`.
Before adapter/fixture changes, inspect that cohort's official schema and retained
evidence. Record runtime/schema/source versions, hashes and captured-versus-synthetic
provenance beside fixtures. Sources:
[pinned schema](https://github.com/openai/codex/tree/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/app-server-protocol/schema),
[app-server documentation](https://learn.chatgpt.com/docs/app-server).

Metadata inspection includes initialization, account reads without forced token
refresh and configuration reads. Preserve exact provider identifier types and
operation/request/thread/turn/item matching; ground approval assumptions without
implementing approval flows or weakening matching. Synthetic faults prove
application recovery decisions, not native provider behavior.

Bound queued work, frames, total retained output and deadlines; include in-flight
work in budgets and drain stderr independently. Unexpected requests never grant
permission. Explicitly own worker and child-process handles, with bounded stop/join
and cleanup outcomes including owned descendants. Dropping handles or reaching a
timeout does not establish cleanup. Retain uncertainty and block conflicting
replacement until reconciled; saved PIDs alone grant no signaling authority.

Expose and retain selected non-secret diagnostic fields. Raw configuration,
credentials and provider payloads must not leak through logging, error formatting
or ordinary setup observations. Keep process work, durable persistence and their
uncertain outcomes distinguishable.

## Deferred shared supervisor exchange improvement

The user chose to document this opportunity for later tickets, not make it a
prerequisite of 06. Current `connection.rs` repeats privacy checks, socket setup,
framing and response handling in repository/workspace exchange paths; their
substitution paths also differ. Runtime requests would add another caller.

Assess consolidation when a later ticket benefits. Keep transport mechanics local
to `connection` and typed domain errors/decisions in their owning modules.
Preserve definite local rejection versus unavailable-reply distinctions, version
compatibility, private access checks and no automatic replay/start-on-action.
One private shared exchange seam can earn depth if it actually removes repeated
transport knowledge. Do not introduce a generic RPC framework or broaden this
ticket into a transport refactor. Adding the necessary runtime forwarding and
compatibility checks remains part of 06b.

## Verification and evidence

Use the agreed test-first seams under repository policy:

- Rust setup/protocol decisions with supplied process, stdio, discovery/filesystem,
  storage and controlled-time outcomes; retain real policy, parsers and command
  construction. Private owning-module test surfaces are valid.
- Schema-grounded success/authentication/configuration; version/exit failures;
  unrelated/differently typed IDs; malformed/oversized frames; unexpected requests;
  EOF; deadlines; uncertain cleanup and no duplicate admission.
- Preference commit/rollback failure, lost Save replies, authoritative rereads,
  serialized admission and preservation of newer draft edits.
- Controlled frontend navigation/setup: retained context, focus intent, late
  registration/creation completion, hide/reveal cleanup, stale observations,
  disconnect/reconnect and absence of automatic check replay.
- Recovery copying and scoped external documentation opening, including failures.

Units use no real SQLite, Git, supervisor/runtime processes or production waits;
add no integration-test framework. Unit mocks are not desktop/process evidence.
Inspect current package scripts before selecting commands. Run focused units and
typechecking during implementation, then the configured complete gate at completed
implementation gates without duplicate full-suite execution. Record compilation,
unit execution, required measurements and packaging separately under policy.

06d qualifies actual WebView keyboard/accessibility, minimum-window layout,
retained context, responsive navigation, CSP compatibility and production
supervisor-to-fixture setup/error states with private disposable state. Retain
failed/unperformed results and source/version/build provenance.

Keep unit, fixture-desktop/process and permitted native metadata results separate.
**No live agent submissions are authorized.** Native execution compatibility,
later chat behavior, Claude and whole Milestone 01 qualification remain open.
No product checks or qualification are claimed by this documentation preparation.
