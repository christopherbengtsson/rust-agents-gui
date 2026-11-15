# Component guide

Read this guide before choosing, creating, styling, moving, updating, or reviewing
frontend components. It owns the repository's component conventions. Use the
[React skill](../../.agents/skills/react-engineering/SKILL.md) for state and lifecycle
ownership and the [shadcn skill](../../.agents/skills/shadcn/SKILL.md) for toolkit
APIs, composition rules, and CLI documentation lookup. Product responsibilities
remain governed by [MVP.md](../../docs/MVP.md).

## Upstream shadcn guidance

Keep the shadcn skill upstream-owned; maintain project conventions here under
[repository precedence](../../AGENTS.md#instruction-precedence). Use its relevant
toolkit workflow for the requested component work. Project scope and component
layers govern which additions or customizations are needed.

When the skill's project context contains a shell placeholder rather than actual
JSON, obtain it with `corepack pnpm dlx shadcn@latest info --json` from the repository
root. Use `components.json`, installed component source and dependency versions to
check its output. If lookup fails, report the missing information and inspect
those local sources before selecting an API or adding a component. Fetch the
relevant official documentation and check installed-version compatibility as
described below.

Review upstream skill updates against these conventions and the current stack.
Keep an unavoidable local skill patch small and record its source revision and
reason; prefer contributing a general correction upstream.

## Ownership and selection

| Layer         | Responsibility                                                  | Selection rule                                                                                                                   |
| ------------- | --------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `ui/`         | Upstream-aligned shadcn source backed by Base UI                | Use directly when its existing contract fits. Review and record necessary source-level deviations.                               |
| `primitives/` | Shared product customization of the foundation                  | Prefer an existing primitive when its appearance or interaction contract fits. Add one for a concrete shared product convention. |
| `blocks/`     | Coherent product sections composed from controls and primitives | Own product structure and explicit intent callbacks; keep execution policy in Rust.                                              |

Screens coordinate blocks, observations, and page layout. Extract a block for a
coherent responsibility even before it is reused. A primitive earns its place by
owning a real contract, such as compact action styling or a shared pending-action
presentation; wrapping each upstream export is unnecessary. Preserve predictable,
typed APIs and pass through required composition props and refs.

Keep `ui/` and `primitives/` independent of application IPC, including indirect
access through application hooks. Receive data and callbacks from their consumers.
The lint gate checks direct Tauri imports; review verifies transitive boundaries.
Create layer directories as working features need them.

## Tokens and restrained styling

Use the existing global stylesheet identified by `components.json` as the shared
theme source. Reuse semantic color roles and foreground pairs; define a missing
product role there before using it across controls. Maintain shared typography,
radius, and recurring size/spacing decisions through theme tokens and named
variants. Tailwind 4 uses CSS-first `@theme` mappings; consult current configuration
rather than introducing a second theme system. One-off layout measurements need
not each become a token.

Parents own placement, surrounding spacing, and page layout. Components own their
internal structure: a button owns icon spacing, its toolbar owns action placement.
Use call-site `className` for external layout. Put shared appearance changes in a
primitive's styling or variants, keeping the underlying `ui/` source close to
upstream where composition permits it.

Use existing variants first. Each additional variant must represent a distinct,
recurring visual or interaction need with identifiable consumers. Keep variants
few and understandable; choose an explicit product abstraction when unrelated
conditional styles accumulate. Review tangled class conditions for a missing
contract rather than imposing a numeric variant limit.

Use CSS for visual hover, focus, and responsive states; keep actions and meaningful
interaction state in the owning React/Base UI component. Retain focus-visible,
reduced-motion, and keyboard access when styling. Measure actual performance
before adding JavaScript optimizations.

## Accessible composition and updates

Read installed-version-compatible official docs before changing component APIs.
Our foundation is Base UI: compose with `render`, pass received props and refs to
the intended DOM element, and preserve one appropriate interactive element. Radix
`asChild` examples require translation, not copying or a new dependency.

Account for accessible names, dialog titles/descriptions, visible focus, contrast,
keyboard operation, and focus return. When semantics or composition change, check
the affected keyboard/focus behavior and screen-reader output; static lint and DOM
units provide partial evidence. Report actual desktop/accessibility checks and
unperformed qualification separately under the repository's verification policy.

Before updating source, use the shadcn CLI dry-run and diff workflow. Compare each
upstream change against local deviations and consumers, preserve intentional
customizations, and verify affected primitives and blocks. Treat Base UI dependency
updates separately from generated-source updates. Review internal API changes and
callers together; Git history records their evolution.

## Observation lifetime status

F04f now uses app-scoped Query observations for repository, workspace and inspection
bodies, with one XState feed actor and one inspection actor above layouts.
The [amended foundation](../../.scratch/frontend-navigation/spec.md) and
[observation guidance](../../docs/agents/observations.md) govern their replacement:
F04e establishes app-lifetime construction and repository/workspace feature
observations; F04f replaces feed/inspection lifecycle internals with XState and
completes inspection Query/freshness/polling replacement. Screens receive
feature-specific observations/callbacks; Query owns records, actors own coordination,
and Zustand owns presentation memory. Settings releases inspection interest while
physical work drains. Product source has not changed in F04a; update each entry with
actual interfaces in its implementation ticket, retaining completed UI evidence.

## Component decisions

For each new or materially changed custom primitive or block, maintain a concise
entry here: **why it exists, when to use it, when to choose an alternative**, and
meaningful API constraints. Add the entry in the same change as the contract.
Document implementation-specific caveats beside code; reference them here rather
than duplicating them. Generated components can use official documentation unless
a local deviation changes their contract. Use repository ADRs for architectural
decisions.

### Existing foundation and migration boundary

Current locations and customizations predate the layer convention. Preserve them
until a separately scoped migration inspects upstream differences and consumers.
This guide does not authorize regenerating or relocating the foundation.

- **RepositorySidebar** (`blocks/repository-sidebar.tsx`): owns branding,
  repository navigation, and the connection footer. Use for the application’s
  repository/workspace navigation; use a separate block for workspace content.
  Receives `Connection` and `Repositories` from App. Each repository has a separate
  selection button and Collapsible trigger for path/availability details; closure
  preserves selection. Unavailable records remain selectable. Cursor buttons page
  the bounded list. Add repository sits above the connection summary. Ticket 04
  adds workspace children to these groups.
- **SupervisorConnection** (`blocks/supervisor-connection.tsx`): owns connection
  summary and diagnostics. Use for supervisor observations, not conversation or
  runtime status. Receives `Connection`; dialog disclosure stays local and stale
  observations remain explicitly labeled. Its sidebar parent owns external
  padding and the divider. `ConnectionDetails` remains private to this block.
- **WorkspacePanel** (`blocks/workspace-panel.tsx`): owns the main landmark,
  workspace header, refresh/retry feedback, connection alert, empty content, and
  footer. Receives `Connection`, `Repositories`, and
  `onRefreshStatus: () => Promise<void>`; App's Hooks handle errors and pending
  guards. Displays selected repository context, retained/stale observations,
  action/storage errors and availability. Use for workspace content rather than
  navigation. Long repository names truncate in the header/footer while retaining
  their full accessible text and keeping Refresh visible. Ticket 04 adds
  selected-workspace context. App supplies the stable inspection reader constructed above layouts, sharing
  its physical-read coordinator with repository/workspace observations. Standalone
  consumers must likewise keep the reader above keyed presentation.
  Ticket 05 adds change
  inspection through concrete interfaces in those tickets.

App constructs the Query client, p-queue physical coordinator, feature modules,
Zustand presentation identity store and inspection reader above layouts.
App owns the single connection observation Hook and composes the blocks inside
SidebarProvider with the skip link. Future selection changes must retain that
owner and bind pending work to its actual domain target. Blocks add no duplicate
subscriptions. Shared shell dimensions and typography live in the global theme;
parents control arrangement, while blocks retain internal layout.

- **RepositoryRegistration** (`blocks/repository-registration.tsx`): owns the
  registration dialog, draft, native picker interaction and focus lifecycle. Use
  for explicitly adding an existing repository; creating a workspace is separate.
  Receives registration observations/intents from App's `useRepositories`; it
  never interprets durability or decides recovery. Native picker results only fill
  the path. Late results from a closed dialog are ignored.
- **Repository observations:** App calls `useRepositories` once alongside the
  existing connection Hook. RepositoryObservations hides owner/target/page Query
  definitions, validation, interests and captured dirty generations. The Hook binds
  presentation identities and explicit callbacks without retaining observation bodies.
  Feed baseline acceptance precedes initial reads; hints replace progress polling.
  Desktop preferences retain selection/action IDs; SQLite remains authoritative.
  Mount/reconnect reads status and never resubmits.
- **Registration foundation additions:** Field, Label, Input Group, Textarea,
  Spinner and Collapsible retain official base-nova composition. InputGroupAddon
  has a narrow lint exception for mouse focus assistance; Label's exception
  documents consumer-owned association. Consumers must supply label/help/error
  relationships, native button semantics, loading text and reduced-motion support.

- **Compact foundation:** current `ui/` controls include local compact sizing and
  styling, including Button variants; Sidebar includes application-shell
  adaptations. Preserve these qualified customizations during upstream updates.
  Future work should inspect them for extraction into primitives rather than
  assuming they are untouched generated source. Existing desktop evidence remains
  under `docs/evidence/`.
- **Mobile breakpoint hook:** preserves upstream post-mount initialization; its
  narrow lint exception retains the existing first-render behavior. Revisit with
  a scoped responsive-shell change.

## Completion

Review every new or changed component for layer ownership, token/variant choices,
applicable guide entries, and its accessible interaction contract. Follow the React
skill for state, identity, cleanup, and failure paths. Run the applicable configured
checks; resolve findings or give a narrow, concrete reason for an exception.
Lint detects Hook/compiler issues, some JSX accessibility defects, and direct
Tauri dependencies in toolkit layers. Layer choice, visual quality, contrast,
transitive dependencies, and variant usefulness still require review.

For primary-source rationale and the article's stack adaptations, consult
[component convention research](../../docs/research/shadcn-agent-conventions.md).

### Workspace creation

- **WorkspaceDetails** (`blocks/workspace-details.tsx`) owns selected-workspace
  identity, observed creation stage, pinned base, branch/folder evidence and the
  explicit reconciliation callback. Displays provider-independent workspace facts;
  provider identity belongs to a conversation or explicitly targeted setup status.
  Use it for workspace creation/result/recovery;
  use WorkspacePanel for the screen and RepositorySidebar for navigation. It has
  no IPC or execution policy. Shared appearance uses semantic tokens; long names,
  branches and paths wrap without hiding recovery controls.
- RepositorySidebar now includes independent native-button workspace children and
  an accessible per-repository +. Existing selection/disclosure remains separate.
  WorkspacePanel composes WorkspaceDetails and retained transport/preference errors.
- App calls `useWorkspaces` once. It owns bounded pages, workspace selection and
  pending action status across navigation through WorkspaceObservations and the
  app-scoped Query client. Zustand retains selected identities, never workspace
  records. Reconnect only reads. Creation and
  recovery callbacks retain explicit target IDs; late replies cannot steal changed
  selection. Action settlement uses owner and explicit-navigation generations,
  independent of repository-driven observation resubscription. Admission errors
  remain visible while unrelated workspace progress follows push hints; observation
  failures retain last-good data and require a new hint or explicit refresh.
  Stable callbacks own observation interests; the shared
  React-independent async settlement utility preserves cleanup outside compiler lowering.
- Add repository explains the automatic first workspace and lack of agent launch.
  No additional shadcn component or custom primitive was needed. Existing controls
  retain their local compact variants and Base UI render contracts.

### Human labels and diagnostic identities

Use meaningful human names for navigation, headings, badges and ordinary status.
Keep internal UUIDs in keys, action targets and persistence; display them only in
explicit troubleshooting details when needed. Do not append raw IDs to labels or
show them as a fallback for missing names. Pending allocation displays “New
workspace” and states that no name/folder is reserved. Newly allocated locations
are readable; WorkspaceDetails discloses retained legacy branches/folders through
an explicit keyboard-accessible Collapsible. No new toolkit component is needed.
Connection failure replaces initial navigation skeletons with unavailable guidance;
retained observations remain readable and marked stale.

### Workspace change inspection (05a–05b)

- **WorkspaceInspection** (`blocks/workspace-changes.tsx`) coordinates the ready
  workspace's Workspace/Changes tabs and one `useWorkspaceChanges` owner above
  their content. Use it for workspace-wide uncommitted inspection; existing
  WorkspaceDetails retains creation/recovery presentation. It receives an exact
  workspace ID and connected supervisor owner. One app-lifetime XState reader coordinates summary
  refresh every two seconds while Changes, the expanded file panel or the open
  Sheet is visible; selected previews load only in Changes. Pending jobs follow accepted feed invalidations with
  observation-only reads; covered 250 ms progress polling is removed. Hidden documents/surfaces, disconnection and failures
  pause automatic reads; manual Refresh remains available after errors. Reconnect
  first retrieves observations. Query retains validated observations and last-good data; bounded dirty generations
  preserve hints during reads. Actor request identities isolate stale replies;
  the physical sequence drains before another sequence starts. App
  owns the reader's lifetime above layouts and the workspace-keyed presentation, so navigation
  also drains old IPC before reading the new target. Supply that stable reader
  when composing WorkspaceInspection; keyed view resets release interest and retain
  no extra readers. useWorkspaceChanges requires that owner. Zustand persists an allowlist of workspace file/comparison, reading position,
  panel width/collapse and remembered view defaults. Current view is a controlled `view`/`onViewChange` contract supplied by the route;
  remembered views supply navigation defaults only. Workspace or Settings-style interest release
  preserves that memory; a changed comparison rejects restored file/scroll context.
  Live preview refreshes retain native-validated file identity and reset scroll when
  the comparison changes. Storage errors remain visible above routed content even when
  inspection is unmounted; pending action records remain separate.
- **ChangedFiles** is the bounded file-selection block shared by the desktop
  right panel and the below-880-px Sheet. It receives observations and a selection
  callback. Opaque file IDs, never display labels, target previews. It renders one
  page, with First/Next navigation. Selection opens the single Changes tab and
  closes the Sheet. The desktop panel is independently collapsible and resizable.
- **UnifiedDiff** (`primitives/unified-diff.tsx`) composes Code Block and Scroll
  Area for plain, bounded Git rows. Use explicit old/new source numbers, row kinds
  and supplied text; use Code Block directly for ordinary sequential code. It
  accepts `initialPosition` for one keyed mount and an explicit `onPosition`
  callback. Key by comparison/file identity to reset stale reading position.
  Scroll reporting is coalesced to 150 ms and flushes pending position on cleanup;
  consumers capture workspace/comparison/file identity and reject late saves.
  Tab switches unmount inactive heavy content. No highlighting or HTML rendering
  is enabled.
- **Foundation additions:** Tabs, Resizable, Scroll Area and Code Block came from
  shadcn base-nova through the project runner. Existing compact Button source was
  preserved. Tabs forwards orientation to Base UI and uses explicit
  `data-[orientation=…]` selectors because this stylesheet does not define the
  registry’s shorthand orientation variants. Scroll Area uses the same explicit
  orientation selectors for track sizing and supplies both scrollbars and omits an unused React
  import. CodeBlockContent omits upstream's extra keyboard stop; Scroll Area's
  viewport owns keyboard scrolling. New diff appearance uses shared semantic
  roles in the global stylesheet.
- **CSP:** WorkspaceInspection supplies `CSPProvider disableStyleElements`, with
  Base UI 1.9.0's scrollbar suppression externalized into the global stylesheet.
  Keep the existing Tauri policy. Actual WebView scrolling, focus, resizing and
  responsiveness remain 05c qualification; controlled DOM checks are partial
  evidence only.

The Rust observation contracts live in `workspaces::inspection`, re-exported
through `workspaces`; TypeScript validation lives in `lib/changes.ts`. Failure and
last-good comparison identity stay explicit. This component documentation does
not duplicate the wire schema or authorize provider actions. Automatic inspection
is read-only. A refreshed comparison resets list paging to First; expired
continuations are explicit errors, and a removed selection stays named until the
user selects another file. Preview failures label retained content as stale even
when the retained comparison ID has not changed.

### F04 observation delivery

`App` owns one acknowledged native feed through `useObservationFeed`, independently
of feature reads. Its XState owner validates owner/subscription/batch/cursor
and typed hints, then calls the synchronous app acceptance sink before advancing
the accepted cursor and acknowledging. Repository/workspace modules own Query
freshness and visible/retained-action interests; the feed has no competing
feature-dirtiness ledger. A shared bounded generation budget also retains hidden
inspection invalidations until that feature becomes interested.
Feed failures appear in the existing connection status/details and retain last-good
feature observations. The actor serializes replacement after physical stop/join,
including late registration and Strict Mode cleanup. Configuration changes during
cleanup update the latest desired owner; failed cleanup stays visible and fences
replacement until exact native stop/join reconciliation succeeds. Failure remains
visible while cleanup is pending. Views release interest rather than terminating
app actors. Historical F04–F04e evidence is preserved; focused actual transport/fixture-desktop
qualification is recorded in [F04g evidence](../../docs/evidence/frontend-navigation/f04g.md).
F08 retains cumulative routed/accessibility qualification.

### F06 route composition

`App` constructs one Router and observation composition. Its persistent root runtime
owns connection/feed and repository/workspace action hooks above child routes.
Router supplies current repository/workspace IDs and the validated Workspace/Changes
view; Zustand retains restoration defaults only. Use `WorkspacePanel` for main
repository/workspace content and supply its controlled view callback. File selection
requests the Changes route through that callback. The sidebar's callbacks navigate
once to the complete destination; create/register completions require an unchanged
history generation. Paging and disclosure do not change the current route.

Route loaders prepare exact feature observations without configuring Query keys or
native IPC. Missing explicit targets and wrong membership show destination guidance;
Settings's F06 route placeholder is replaced by the F07 composition below. Route changes release inspection
interest while its app-lifetime owner drains work and retains validated context.

### F07 routed Settings

- **RepositorySidebar** retains the separate supervisor diagnostics and adds the
  footer gear. `onSettingsOpen: () => void` records focus provenance for ordinary
  activation; the typed Link owns navigation to `/settings/general`. Base UI Button
  composes that Link with `nativeButton={false}` and explicit link semantics,
  producing one interactive target named “Settings.” Modified clicks retain Link
  behavior without claiming focus return for the current window.
- **SettingsLayout** (`blocks/settings-layout.tsx`) owns the Settings header,
  Back control, navigation and main landmark. Use it for routed app settings;
  use WorkspacePanel for repository/workspace presentation and SupervisorConnection
  for transport diagnostics. It receives `onBack: () => void` and route content;
  the General content supplies `settings-general-heading` for the main landmark.
  At the F07 checkpoint, General is the only section and uses Empty with honest
  placeholder text. No account or provider controls were implemented there.
- The layout focuses its heading on mount. Its window bubble-phase Escape handler
  follows installed Base UI overlay dismissal; consumed, composing and repeated
  Escape events do not navigate. Navigation owns ephemeral return provenance keyed
  to the Settings history entry, including replacement redirects. Back/Escape uses
  its preceding app entry; when none is usable, replace with home, per the F07 user
  decision. Remembered destinations do not create Settings return authority.
  After routing settles, AppRuntime restores the remounted gear for a gear-origin
  return, otherwise the main landmark. The skip link follows the current layout.
- Both layouts share the persistent runtime and SidebarProvider. Settings unmounts
  WorkspaceInspection, releasing its interest without stopping the reader, feed,
  action owners or Query client. Hidden collection/detail interests are released,
  while independent retained-action interests continue. Accepted hints remain
  dirty until the main layout returns; inactive observation bodies can expire.
  F05's validated file/comparison/scroll/layout
  context survives; comparison changes reject stale restoration. Existing
  Sheet/resizing contracts remain. Actual WebView focus/layout and cumulative
  responsiveness qualification belongs to F08.
- [Original milestone 06](../../.scratch/milestone-01/issues/06-connect-codex-with-actionable-setup-diagnostics.md)
  now indexes the [connection feature sessions](../../.scratch/codex-connection/spec.md#session-sequence).
  Their [shared journey](../../.scratch/codex-connection/spec.md#connection-journey)
  owns provider presentation, login controls and progress continuity. Connection 01
  extends Settings landmark labels and return/focus/context behavior; 02 adds
  browser progress/cancellation; 02a adds runtime compatibility; 04 qualifies the
  combined desktop interaction. The current connection scope defers 03's
  CLI/terminal renderer and command-output viewer. Each slice
  uses existing UI/primitives/tokens and documents actual new block contracts
  when implemented. Supervisor diagnostics stay separate. These are planned
  successor contracts; F07 implemented no provider setup.
  The [provider-independent workspace prerequisite](../../.scratch/provider-independent-workspaces/spec.md)
  precedes connection 01. Its cleanup removes workspace-level provider ownership;
  setup links explicitly target a provider. All setup/progress consumers use the
  [shared provider observation module](../../.scratch/codex-connection/spec.md#architecture-and-extension-constraints)
  planned on the existing foundation; blocks do not own login attempts or another cache.

### Provider readiness (connection 01)

- **ProviderSetup** (`blocks/provider-setup.tsx`) presents one exact Codex/user
  configuration's current/retained evidence, contextual Check again, recovery guidance and secondary
  technical details. Use it for provider setup; SupervisorConnection remains
  transport diagnostics and WorkspaceDetails remains workspace identity/recovery.
  It receives the cohesive `useProviderSetup` view and callback, owns no IPC,
  cache, retry or process policy, and renders runtime strings as bounded text.
  Existing Tabs (line variant), Alert, Button and Collapsible retain their Base UI
  contracts. No toolkit component was added or regenerated. Official documentation actions use the scoped native opener for exactly the
  two fixed Codex installation/authentication URLs; no provider-return deep link
  or login flow is added. Opener automatic link interception is disabled.
- **ClaudeSetup** is an informational section with genuinely disabled connection
  controls. Its route mounts no provider hook and sends no Claude request.
- **SettingsLayout** accepts the current `general | providers` section for its
  landmark label. Section and provider-tab changes replace the current Settings
  history entry, retaining the initial return/focus provenance. Tab selection
  lives in Router search; Query owns setup evidence. The workspace's “Check Codex
  setup” link explicitly targets the Codex user setup without a workspace provider
  field or automatic provider read on workspace mount.
- **ProviderObservations** is constructed once by AppObservations and uses its
  shared Query client, feature freshness, accepted feed and physical scheduler.
  `useProviderSetup` acquires/releases presentation interests. A committed Codex-tab
  activation requests one fresh check with the existing targeted refresh operation
  before attaching readiness interest. Entry intent and attachment markers are scoped
  by owner/target, so a replacement cannot inherit the previous scope's presentation.
  Same-scope transport detachment retains the visible readiness interest and its
  undispatched force flag. Navigation or actual scope replacement releases it.
  Deferred setup skips disposed mounts; effect identity uses stable target IDs.
  Query remains the sole evidence cache. Initial entry pending feedback prevents
  retained healthy evidence from appearing current before the check. Preload reads
  only the default descriptor; rerenders, feed updates and reconnect do not replay
  the entry intent. Returning from another tab/route is a new activation. Rust owns
  probe admission/coalescing, last-good evidence and cleanup fencing. User setup runs from a private neutral
  directory, with captured executable/environment/Codex home; project overrides
  and account/profile management are outside this slice.

### Provider presentation refinement

- **StatusIndicator** (`primitives/status-indicator.tsx`) owns the inline decorative
  light, static success halo and polite atomic text announcement. Use for observed
  readiness, supplying `tone: success | neutral | error` and a text `label`; use
  Badge for enclosed labels/tags and Spinner for busy actions. The caller interprets
  evidence. `className` supplies placement; the primitive owns appearance through
  shared semantic tokens and has no IPC, hook or process dependency.
- **ProviderSetup** uses one status line and compact account/billing facts. Healthy
  setup has no Refresh, including in disclosures. Checking/stale evidence is Last
  known; recovery offers Check again only when connected and not checking/fenced.
  Read failures override retained runtime Checking for recovery; an actual retry
  shows Checking alongside the retained failure and withholds another Check again
  until that read settles. CodexSetup supplies existing feed availability to the
  hook; unavailable delivery forces stale/non-pending presentation and visible
  supervisor-reconnection guidance, without changing interest/intent lifetimes.
  Cleanup uncertainty still fences retry. Technical details
  and general Setup help start collapsed. Recovery, help-opening
  errors and future login controls remain outside the disclosures. The scoped
  documentation opener and all Settings return/focus contracts are unchanged.
- The [shared connection journey](../../.scratch/codex-connection/spec.md#presentation-refinement--2026-11-15)
  supersedes earlier retained-only navigation and permanent-Refresh requirements.
  02 adds the visible browser sign-in action beneath readiness and coordinates
  its attempts with checks in Rust; 04 qualifies the combined UI. The current
  [scope amendment](../../.scratch/codex-connection/spec.md#authentication-scope-amendment--2026-11-15)
  defers the CLI choice and terminal work formerly assigned to 03.

### Browser sign-in (connection 02)

- **ProviderSignIn** presents the explained ChatGPT browser action, exact-attempt
  progress, reopening, cancellation and reconciliation. Use below readiness and
  account facts, outside Setup help. It receives observations and targeted
  callbacks; it owns no authentication, URL, process or retry policy. Button,
  Alert and Spinner retain their existing Base UI contracts. CLI controls and
  ticket 03 are deferred from the current connection scope. A completed browser
  attempt and current verified readiness are
  separate facts; verification failure keeps account evidence Last known.
- **AppRuntime** presents a compact progress link above routed content while
  authoritative sign-in or uncertain delivery/cleanup is pending. Providers,
  General and workspace navigation preserve the same attempt and Settings return
  provenance. Native shutdown progress/failure remains visible here too. Window
  close hides; explicit Quit requests supervisor-owned cleanup and waits for
  owner release. A desktop crash leaves the supervisor-owned attempt running.
- **ProviderObservations** retains identity-only pending-action interests before
  dispatch, reconciles lost replies without replay and keeps Query as the only
  observation cache. Pure status reads cannot start readiness checks; committed
  Codex activation still requests its explicit bounded check. A supervisor
  snapshot advertises pending provider login so a reopened desktop can recover
  progress without probing providers on an ordinary workspace mount. The native
  owner opens and retains the returned URL; JavaScript receives no auth URL or
  native login ID. Unconfirmed cleanup fences its credential scope.

### Codex connection refinement (05)

ProviderSetup now presents the agreed status/recovery matrix and whole-section
Skeleton while a check runs. Healthy setup contains only Connected and available
identity (or Using API key); billing and general help are absent. Pending browser
sign-in and cleanup/delivery uncertainty remain actionable rather than loading
placeholders. StatusIndicator adds semantic warning alongside success/error.
Focus recovery stays within the mounted section and yields to navigation or user
focus movement. Recovery diagnostics remain collapsed and do not grant retry authority.

Already installed? composes the existing Collapsible and Button. Locate Codex…
uses the installed native file chooser through useProviderSelection, guarding late
results by owner/target/view generation. The provider feature owns identity-only
selection delivery feedback and observation interests; Rust validates, persists
and reconciles selection. No extra cache or toolkit dependency is added. A saved
location becoming unavailable never silently selects another runtime. Native
chooser cancellation changes nothing; selected path and resolved launch are distinct.
