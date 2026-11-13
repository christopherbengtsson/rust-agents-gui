# Settings navigation for the desktop MVP

Research inspected and decision accepted on 2026-11-13. Preparation baseline:
`b3f8cf00db6965786db6ace294a1a85fd7d01817`. This note supports the
[ticket 06 plan](../../.scratch/milestone-01/codex-setup-plan.md); the
[MVP](../MVP.md), [milestone](../MILESTONE-01.md) and
[verification policy](../agents/verification.md) remain authoritative.

## Accepted decision

Use a small typed application navigation module without a router for the stated
MVP. Settings replaces the workbench visually, and Back returns to its retained
context. The first category is **Agent runtimes**, with Codex now and Claude when
its integration is implemented. The user explicitly chose selection/document tabs
over general workspace/conversation Back/Forward history.
The repository-wide decision is recorded in
[ADR 0006](../adr/0006-application-owned-desktop-navigation.md).

This is our product/architecture decision, not a Tauri restriction. The full MVP
needs persistent workspace/conversation presentation and supervisor execution
independently of which screen is visible. A routing library would still need that
workbench model. It adds limited leverage while navigation history, copyable
internal destinations and native deep links are unspecified. Reconsider when one
of those becomes an actual requirement; do not infer it from future entity count.

## Observed repository starting point

Read-only inspection found React 19.3.1, Base UI 1.9.0, Vite 8.3.8, TypeScript,
Tailwind 4 and no router dependency in [package.json](../../package.json).
[components.json](../../components.json) selects base-nova, Lucide and
`src/style.css`. shadcn CLI planning commands included
`corepack pnpm dlx shadcn@latest info --json` and `docs` lookups for the existing
sidebar, field, input, button, badge, alert, tooltip, spinner and disclosure controls.
These establish available composition, not installed-version desktop qualification.

- [App](../../src/App.tsx) owns one connection, repository and workspace observation
  Hook instance. Keep those owners mounted across Settings transitions.
- [RepositorySidebar](../../src/components/blocks/repository-sidebar.tsx) owns the
  current navigation and footer; its selection callbacks currently call the domain
  hooks directly.
- [useRepositories](../../src/hooks/use-repositories.ts) owns repository selection,
  preference restoration and registration settlement.
  [useWorkspaces](../../src/hooks/use-workspaces.ts) separately owns workspace
  selection and guards action-driven selection using navigation generations.
- [WorkspacePanel](../../src/components/blocks/workspace-panel.tsx) keeps a
  [WorkspaceChangesReader](../../src/lib/workspace-changes-reader.ts) above keyed
  workspace views. Its physical IPC lane survives stale-response invalidation.
  [WorkspaceInspection](../../src/components/blocks/workspace-changes.tsx) owns the
  current tabs, panel state and reading-position references.
- The current skip link targets `#workspace`. Screen switching must supply unique
  IDs and a working current main target rather than leave a hidden destination.

## Official mechanisms and alternatives

Tauri supports static frontend assets and SPA presentation; it does not prescribe
a React router. Its native deep-link plugin delivers startup/open-URL information
that application code interprets. A router is therefore neither required nor
prohibited by a desktop WebView. Sources:
[frontend configuration](https://v2.tauri.app/start/frontend/),
[process model](https://v2.tauri.app/concept/process-model/),
[deep linking](https://v2.tauri.app/plugin/deep-linking/).

| Choice | Documented capability | Assessment for the accepted MVP |
| --- | --- | --- |
| Typed application navigation module | Application-owned screen/selection coordination; no dependency required. | Chosen: direct Settings return and document selection without inventing a history engine. |
| React Router Declarative mode | Route matching, navigation and nested presentation; MemoryRouter keeps location history in memory. | Credible option alongside existing data abstractions if history/addressability is requested. |
| TanStack Router | Registered route-tree typing for navigation/parameters, code/file route definitions and memory history. | Strongest router alternative if typed addressable destinations become useful; route wiring is a modest cost, not grounds for dismissal. |
| Browser/hash histories | Couple location to path/fragment respectively. | No accepted URL requirement. Hash navigation would also need to coexist with skip-link fragments; do not claim Tauri universally requires it. |

Router capabilities above come from primary documentation:
[React Router modes](https://reactrouter.com/start/modes),
[MemoryRouter](https://reactrouter.com/api/declarative-routers/MemoryRouter),
[TanStack type safety](https://tanstack.com/router/latest/docs/guide/type-safety),
[code routing](https://tanstack.com/router/latest/docs/routing/code-based-routing),
[history types](https://tanstack.com/router/latest/docs/guide/history-types).
The assessments are our inference from the accepted workflow, not library rankings
or measured bundle/performance results. No router package was installed or benchmarked.

## Workbench state and lifetime

The full MVP has different owners for durable identities/execution, selected
repository/workspace, per-workspace conversation/Changes tabs, and per-target
draft/file/panel/scroll state. More active conversations do not by themselves
require more application pages. A router would still need those presentation
sessions and exact target validation. This ownership mapping follows the MVP,
not an upstream framework mandate.

React Activity retains child state/DOM while hidden, cleans up hidden Effects and
recreates Effects on reveal. Ordinary unmount discards local state. This supports
one retained workbench without serializing its full presentation into navigation.
Sources: [Activity](https://react.dev/reference/react/Activity),
[preserving/resetting state](https://react.dev/learn/preserving-and-resetting-state).

Keep continuing shared observations outside hidden Activity. Keep the existing
physical changes-reader lane stable so an obsolete pending IPC drains before a
new read. Hidden inspection effects may pause. A CSS-only hiding approach would
leave Effects mounted; retaining an unbounded view per workspace would require a
separate resource policy and is not part of this ticket.

The accepted navigation module owns screen transitions, return/focus and an
explicit-navigation eligibility marker. Existing hooks keep their canonical
selections. Opening Settings must prevent an earlier registration/creation result
from changing the context the user returns to, while still settling that action.
ID equality alone misses intervening navigation back to the same ID. Keep
observation, action and navigation generations distinct. This is a source-grounded
design requirement; no production race was reproduced during research.

Sidebar selection and Settings Back are callers of the same navigation rules.
Workspace tab/file/panel state remains local to workspace presentation. A module
that only forwards a screen setter would add little depth; extract it to
concentrate actual return/focus/eligibility rules, not to build a generic manager.

## Composition and native actions

shadcn's Base UI Sidebar supplies the footer/menu composition and active-state
presentation. It does not select a routing architecture. Keep accessible native
buttons for application screen actions and retain qualified compact source.
Source: [Base Sidebar](https://ui.shadcn.com/docs/components/base/sidebar).

Base UI requires custom render composition to preserve received props/refs and
the intended interactive semantics. Installed-version Button guidance explicitly
rejects anchor links through Button's `render` prop because it enforces button
semantics; `nativeButton={false}` does not restore link semantics. This supersedes
the conflicting anchor example in the local shadcn rule file for implementation.
Sources: [composition](https://base-ui.com/react/handbook/composition),
[Button links](https://base-ui.com/react/components/button#rendering-links-as-buttons).

The app must own focus placement on screen changes and return; a router does not
establish the product's exact policy. Source:
[React Router accessibility guidance](https://reactrouter.com/how-to/accessibility).
Preserve the actual Tauri CSP with compatible control composition. Source:
[Base UI CSP Provider](https://base-ui.com/react/utils/csp-provider).

Recovery documentation opens externally through scoped Tauri opener support, not
as navigation that replaces the application inside its WebView. Destinations are
application-owned official documentation, not authority taken from provider text.
Source: [Tauri Opener](https://v2.tauri.app/plugin/opener/).

## Accepted architecture and deferred exchange improvement

Keep one deep Rust runtime module with concrete production/fixture adapters,
shared protocol interpretation and private effect seams. It owns setup/process
mechanics; `conversations` will own durable delivery/reconciliation, and `storage`
owns writes. Narrow runtime settings persistence must not inherit repository/Git
worker policy. This is the accepted application of the MVP's existing modules.

Source inspection of [connection.rs](../../crates/core/src/connection.rs) found
repeated socket/privacy/framing knowledge in repository/workspace request paths
and differing substitution paths. The user deferred shared supervisor exchange
consolidation. Record it for later tickets that benefit, preserving typed domain
errors, version/private-access checks, definite rejection versus unavailable
replies, and no automatic action replay. It is not a prerequisite architecture
project for 06; necessary runtime forwarding/version checks still belong to 06b.

## Evidence limits

This is documentation/source research and skill validation. No router candidate
was benchmarked. No product implementation, unit suite, actual desktop/OS process
qualification or native provider execution was performed for this preparation.
Read-only Codex version/help and pinned schema inspection during planning did not
start a thread/turn or establish native compatibility. No RAT inspection or
inherited native allowance is used.

The [accepted plan](../../.scratch/milestone-01/codex-setup-plan.md) assigns
controlled behavior checks and actual desktop/process qualification separately.
Settings diagnostics never gate later Send/Resume; retaining runtime-owned
authentication is independent of retaining an old diagnostic result.
