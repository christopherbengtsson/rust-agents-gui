# Ticket 05 — workspace change inspection

Status: ready-for-agent

Accepted 2026-11-13. Preparation only; implementation and qualification remain
open. Feature branch: `feature/m01-change-inspection`. Review base:
`8aa37ab` (the completed ticket 04 baseline).

## Authority and starting a session

This plan elaborates [ticket 05](issues/05-inspect-uncommitted-workspace-changes.md)
under the [milestone spec](spec.md), [MVP](../../docs/MVP.md),
[Milestone 01](../../docs/MILESTONE-01.md) and
[verification policy](../../docs/agents/verification.md). Those owners remain
authoritative. Read repository instructions, the component guide and applicable
repository-local Rust, React and shadcn skills before implementation.

Continue on the same feature branch in the same working directory. Inspect its
current status and predecessor ticket's evidence before editing. Each session
ends with a working increment, relevant passing checks, a commit and a dated
ticket comment linking its evidence and remaining work. Preserve existing edits.
Repository verification policy governs dependency substitution and test surfaces,
including when an engineering skill recommends a different testing approach.

## Delivery sequence

| Session | Ticket                                                                                  | Deliverable                                                                                                                                            | Blocked by          |
| ------- | --------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------- |
| 1       | [05a: Inspect changes manually](issues/05a-inspect-workspace-changes-manually.md)       | Working selected-file inspection through the production Rust/supervisor/Tauri/desktop path, with the intended presentation and deterministic coverage. | Completed ticket 04 |
| 2       | [05b: Refresh inspection reliably](issues/05b-refresh-workspace-changes-reliably.md)    | Automatic refresh, complete race/failure/content coverage and retained reading context.                                                                | 05a                 |
| 3       | [05c: Qualify and review inspection](issues/05c-qualify-workspace-change-inspection.md) | Actual desktop fixture qualification, accessibility/performance evidence, packaging, review and fixes.                                                 | 05b                 |

Keep 05 as the acceptance parent. Complete only the child ticket requested in an
implementation session; do not start the next ticket implicitly. Parent 05 stays
open until its original acceptance items have evidence from the completed slices.
Tickets 06–13 and whole-milestone qualification remain separate.

## Accepted experience

- Keep repository/workspace navigation on the left. Within `WorkspacePanel`, add
  **Workspace** and **Changes** tabs; Workspace retains existing details and
  recovery controls. Both are working destinations, with no placeholder chat tab.
- Add an independently collapsible/resizable right-hand changes panel for ready
  workspaces. Show flat repository-relative paths, change-status badges and the
  selected file. Full paths remain accessible when labels truncate.
- Show the right panel initially at desktop widths. Below 880 px window width,
  expose its file selector through the existing Sheet foundation. Selecting a file
  closes the sheet and opens Changes. Workspace navigation remains reachable.
- Selecting another path updates the single Changes tab. Render one unified
  selected-file diff with hunk context, old/new line numbers, additions, removals
  and missing-final-newline markers. Distinguish empty additions/deletions and
  metadata-only changes from an absent or failed preview.
- Label the scope **Uncommitted changes**, with observed HEAD and refresh status.
  Distinguish initial loading, refreshing, clean, unsupported, limited-preview and
  failed states. Only a complete successful inspection establishes a clean state.
- Preserve selection and reading position across tab switches. Reset reading
  position when the selected file changes. If refresh removes that file, explain
  that it no longer appears rather than silently opening another.
- Keep Resizable, Scroll Area and the narrow-window Sheet. The user explicitly
  retained this presentation when simplifying the architecture; fixed-width,
  native-scroll substitutes are not the agreed implementation.

The choice of a central Changes tab supports the intended shared workspace diff
alongside future conversation tabs. A large modal would block navigation and need
its own file navigation. This is a product decision informed by
[Conductor workspace tabs](https://www.conductor.build/changelog/0.82.0-conductor-mcp),
[Apple modality guidance](https://developer.apple.com/design/human-interface-guidelines/modality)
and [split-view guidance](https://developer.apple.com/design/human-interface-guidelines/split-views),
not a claim that comparative user research proves one layout universally better.

## Rust ownership and inspection interface

One private inspection module within `workspaces` owns comparison capture,
classification, refresh coordination, revisions and bounded last-good
observations. Removing it would spread comparison/error policy into callers;
that depth and locality justify the module. Concrete internal parsing and I/O
adapters may remain small without becoming separate application owners.

Extend the existing `WorkspaceRequest`/`WorkspaceReply` interface and
`workspace_request` transport path for changed-file observations, refresh and
selected-file previews. Tauri forwards typed requests/observations. Validate reply
shape and exact workspace/comparison targets at the TypeScript seam. Update the
wire protocol and its compatibility tests; inspection results are ephemeral and
need no SQLite migration or durable mutation-action ledger.

Reuse the existing workspace worker. Coalesce duplicate inspection requests and
fairly select between existing whole jobs and pending inspection jobs, so neither
repeated refreshes nor pending workspace jobs permanently starve the other. An
in-flight whole job may finish first; show queued/refreshing observations rather
than blocking observation requests on its completion. Keep Git/filesystem work
outside storage and observation locks. Preserve the creation/reconciliation
implementation; this is not a preemptive or generic scheduler refactor. Bound
pending work, retained observations and worker shutdown ownership.

Use the shared core Git helper and supplied filesystem outcomes:

- Inspect staged and unstaged status plus nonignored untracked files. Display the
  final working-tree comparison against the resolved HEAD commit; preserve index
  versus working-tree status distinctions. Explain status entries whose staged
  and unstaged changes cancel in the net comparison. The pinned workspace base
  remains a different, deferred comparison.
- Parse NUL-delimited output as bytes. Use literal path handling, disable rename
  detection, external diff drivers and text conversion, and preserve the helper's
  Git environment isolation, optional-lock suppression and execution limits.
- Validate the saved workspace target and comparison identity. Revalidate HEAD
  and relevant file evidence around reads; an outdated selection cannot receive
  a preview silently attributed to a newer comparison. Inspection is observed
  local state, not a promise of an atomic filesystem snapshot during edits.
- Classify binary, non-UTF-8, symlink, submodule, conflict, special-file and
  oversized content honestly. Preview reads do not follow symlinks or open special
  files as ordinary text. Preserve raw path identity; ambiguous/lossy display
  labels cannot become file lookup targets.
- Retain last-good lists and selected-file previews when Git/read failures occur,
  labeling the retained comparison and its failure. An unsupported result is
  distinct from an I/O error. After supervisor restart, obtain a fresh inspection
  rather than treating absent ephemeral data as a clean workspace.

## Bounded list and preview

Keep changed-file-list paging: at most 32 entries and a 48 KiB serialized response
budget per page, below the current 64 KiB transport frame limit. Bind list
continuations to comparison identity; expire them explicitly after a changed
comparison. Never hide an incomplete inventory behind an apparent clean result.

Return one selected-file preview, limited to 200 diff rows and a 48 KiB serialized
response budget. Measure the encoded response including metadata and escaping.
Emit complete rows only. A valid diff exceeding the display budget shows the rows
that fit with **Preview limited** and a reason; a single row that cannot fit is
labeled unsupported by the preview limit. Preserve correct source line numbers
and context when the excerpt ends within a hunk.

There are no diff-page cursors, continuation requests, Show More accumulation or
diff page histories. A changed comparison replaces the selected preview as a
whole. The shared Git runner's existing byte/time limits still apply: command
failure or output-budget failure never becomes a successful partial preview.
Display truncation is applied only to valid acquired/parsed data.

## React ownership and resource lifetime

`WorkspacePanel` coordinates the tabs, right panel and one changes-observation
hook above their content. It supplies observations and explicit callbacks to the
file list and diff display. Keep frequent feature observations near these
consumers rather than adding them to `App` or duplicating subscriptions.

- Use Base UI panels with `keepMounted={false}`; inactive heavy content unmounts.
  Avoid exit animations that retain an outgoing diff. Keep file selection and
  scroll position above the panels so switching tabs does not lose reading state.
- Refresh summaries every two seconds while either changes surface is visible.
  Fetch/render the selected preview only while Changes is active. Use one read
  sequence in flight, and retrieve owner-produced pending observations while a
  queued inspection settles without resubmitting it on every observation poll.
- Pause automatic reads while both surfaces or the document are hidden, transport
  is unavailable, or inspection has failed. Refresh on return; failures retain an
  explicit manual Refresh. Reconnect retrieves observations without replaying
  workspace or provider mutations.
- Capture workspace, supervisor owner and request generation. Ignore obsolete
  replies and clean up timers/subscriptions on target changes and unmount.
- Keep DOM/data bounded by the list page and selected preview. Start with plain
  code text and diff coloring, leaving optional syntax-highlighter imports unused.
  Rely on the configured React Compiler for routine memoization; add specialized
  caching, virtualization or manual memoization only for measured concerns.
- Compose the diff primitive from shadcn Code Block. Supply Git old/new numbers;
  its automatic sequential numbering is not a substitute. Add required toolkit
  source through the project runner and preserve existing compact customizations.
- Integrate Scroll Area with the existing CSP using `CSPProvider
  disableStyleElements` and the required external scrollbar CSS in the shared
  stylesheet. Qualify actual desktop behavior and keyboard scrolling.

Future conversation observations and supervisor work remain independent of tab
visibility. This ticket introduces no hidden-tab execution owner or general tab
manager. Document new product primitives/blocks in the component guide.

## Verification and completion

Apply the repository verification policy from each slice's first tests. Rust units
substitute Git/process/filesystem/storage outcomes while retaining real policy,
parsers and command construction. Frontend units substitute IPC and controlled
time. Private owning-module test surfaces are valid. Keep these distinct from
actual desktop/process qualification in disposable repositories/private state.

05a proves manual inspection end to end with errors visible from the first
increment. 05b completes automatic refresh and race/content coverage. 05c qualifies
the accumulated feature, reviews the branch against the recorded base, fixes
findings and links retained evidence before parent acceptance is checked.

Read checked-out scripts before choosing commands. Run relevant checks for each
checkpoint and the configured complete verification gate once per complete gate;
package separately. Record exact commands, source/version/hardware identity,
counts, exit statuses and timings, separating compilation, unit execution,
packaging and desktop measurements. Follow the policy's warm-run measurements.
Fixes after a gate require the relevant checks again; unchanged passed checks do
not need ritual repetition.

The final desktop cohort covers changed/clean/unsupported/limited/refreshed/failed
states, last-good retention, switching workspaces/tabs during reads, keyboard and
focus behavior, resizing, Sheet focus return, default/minimum window sizes and
navigation feedback against 100 ms with the largest supported preview. Preserve
failed attempts and unperformed checks. Fixture success establishes no native
provider or full-MVP concurrency qualification.

Out of scope: diff continuation, individual file tabs, modal expansion,
side-by-side rendering, a full file explorer, base comparisons, inline feedback,
editor/terminal handoff, Git mutation controls, native windows and live-provider
submissions. Parent 05 completion does not complete Milestone 01.
