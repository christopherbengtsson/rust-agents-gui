# Component guide

Read this guide before choosing, creating, styling, moving, updating, or reviewing
frontend components. It owns the repository's component conventions. Use the
[React skill](../../.agents/skills/react-engineering/SKILL.md) for state and lifecycle
ownership and the [shadcn skill](../../.agents/skills/shadcn/SKILL.md) for toolkit
APIs, composition rules, and CLI documentation lookup. Product responsibilities
remain governed by [MVP.md](../../docs/MVP.md).

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
  selected-workspace context; ticket 05 adds change
  inspection through concrete interfaces in those tickets.

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
  existing connection Hook. It coordinates bounded pages, selected details and
  retained action-status lookup. Desktop preferences retain selection/action IDs;
  SQLite remains authoritative. Mount/reconnect reads status and never resubmits.
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
  explicit reconciliation callback. Use it for workspace creation/result/recovery;
  use WorkspacePanel for the screen and RepositorySidebar for navigation. It has
  no IPC or execution policy. Shared appearance uses semantic tokens; long names,
  branches and paths wrap without hiding recovery controls.
- RepositorySidebar now includes independent native-button workspace children and
  an accessible per-repository +. Existing selection/disclosure remains separate.
  WorkspacePanel composes WorkspaceDetails and retained transport/preference errors.
- App calls `useWorkspaces` once. It owns bounded pages, workspace selection and
  pending action status across navigation. Reconnect only reads. Creation and
  recovery callbacks retain explicit target IDs; late replies cannot steal changed
  selection. Action settlement uses owner and explicit-navigation generations,
  independent of repository-driven observation resubscription. Admission errors
  remain visible while unrelated workspace progress continues polling; observation
  failures pause polling. Stable callbacks own observation subscriptions; the shared
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
  workspace ID and connected supervisor owner. One reader coordinates summary
  refresh every two seconds while Changes, the expanded file panel or the open
  Sheet is visible; selected previews load only in Changes. Pending jobs use
  observation-only polling. Hidden documents/surfaces, disconnection and failures
  pause automatic reads; manual Refresh remains available after errors. Reconnect
  first retrieves observations. Generations isolate targets and stale replies;
  the physical read lane drains before another sequence starts. WorkspacePanel
  owns the reader's lifetime above the workspace-keyed presentation, so navigation
  also drains old IPC before reading the new target. Supply that stable reader
  when composing WorkspaceInspection; keyed view resets retain no extra readers.
- **ChangedFiles** is the bounded file-selection block shared by the desktop
  right panel and the below-880-px Sheet. It receives observations and a selection
  callback. Opaque file IDs, never display labels, target previews. It renders one
  page, with First/Next navigation. Selection opens the single Changes tab and
  closes the Sheet. The desktop panel is independently collapsible and resizable.
- **UnifiedDiff** (`primitives/unified-diff.tsx`) composes Code Block and Scroll
  Area for plain, bounded Git rows. Use explicit old/new source numbers, row kinds
  and supplied text; use Code Block directly for ordinary sequential code. It
  accepts a reading-position ref owned above the tabs and an explicit scroll
  callback. Tab switches unmount inactive heavy content; changing files resets
  position. No highlighting or HTML rendering is enabled.
- **Foundation additions:** Tabs, Resizable, Scroll Area and Code Block came from
  shadcn base-nova through the project runner. Existing compact Button source was
  preserved. Scroll Area supplies both scrollbars and omits an unused React
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
