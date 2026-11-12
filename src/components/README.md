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
  Receives `Connection` from App and performs no IPC. Ticket 03 extends repository
  registration/navigation here; ticket 04 adds workspace entries and selection.
- **SupervisorConnection** (`blocks/supervisor-connection.tsx`): owns connection
  summary and diagnostics. Use for supervisor observations, not conversation or
  runtime status. Receives `Connection`; dialog disclosure stays local and stale
  observations remain explicitly labeled. Its sidebar parent owns external
  padding and the divider. `ConnectionDetails` remains private to this block.
- **WorkspacePanel** (`blocks/workspace-panel.tsx`): owns the main landmark,
  workspace header, refresh/retry feedback, connection alert, empty content, and
  footer. Receives `Connection` and `onRefreshStatus: () => Promise<void>`; App’s
  Hook handles errors and pending guards. Use for workspace content rather than
  navigation. Ticket 04 adds selected-workspace context; ticket 05 adds change
  inspection. Add those concrete interfaces in their tickets, without placeholder
  slots or data models now.

App owns the single connection observation Hook and composes the blocks inside
SidebarProvider with the skip link. Future selection changes must retain that
owner and bind pending work to its actual domain target. Blocks add no duplicate
subscriptions. Shared shell dimensions and typography live in the global theme;
parents control arrangement, while blocks retain internal layout.

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
