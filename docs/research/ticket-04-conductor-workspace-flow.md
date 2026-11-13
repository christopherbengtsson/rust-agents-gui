# Ticket 04: Conductor workspace creation and component research

2026-11-13. Primary-source research, repository inspection, and user decisions.

## Accepted flow

Adding a repository automatically creates its first workspace. The repository's
**+** creates another immediately. Defer the full composer dialog and base picker
for Milestone 01. Generate names, branches, and managed folders; creation never
starts Codex. These user decisions replace the earlier manual-base-selection
requirement. They preserve Rust execution/recovery ownership.

Conductor explicitly documents first-workspace creation after adding a repository.
[First workspace](https://www.conductor.build/docs/first-workspace#create-the-workspace)
Its Dispatcher combines workspace creation with submitting a task and supports
“create more.” [Dispatcher](https://www.conductor.build/changelog/0.63.0-cursor-support-dispatcher)
Version 0.65.3 added an experimental preference to skip the modal.
[Skip-modal setting](https://www.conductor.build/changelog/0.65.0-codex-goal-support)
This supports deferring that presentation while implementing usable workspaces.

The user's screenshots/observation show creation after dismissing a composer in
that build. Official sources do not establish dismissal as the creation trigger:
the deep-link composer requires clicking Create, and the changelog distinguishes
closing/reopening before submission from closing after creation.
[Deep links](https://www.conductor.build/docs/reference/deep-links#workspace),
[0.65](https://www.conductor.build/changelog/0.65.0-codex-goal-support),
[0.68](https://www.conductor.build/changelog/0.68.0-markdown-frontmatter).
Our explicit triggers are Add repository and repository +. Navigating away after
admission changes presentation; it does not cancel creation.

## Defaults and ownership

Conductor normally creates from the configured base and fetches origin first.
[Workspace model](https://www.conductor.build/docs/concepts/workspaces-and-branches#branches-and-review)
Milestone 01 deliberately performs no fetch. The accepted local policy is usable
`refs/remotes/origin/HEAD`, otherwise exactly one usable local `main`/`master`.
Missing or ambiguous defaults fail actionably; no feature-branch HEAD fallback.
Resolve/persist the commit before mutation and display reference and pinned SHA.

Conductor documents city names as stable directory identities and worktrees
usually under `~/conductor/workspaces/<repo>/<workspace>`.
[FAQ](https://www.conductor.build/docs/faq#what-are-the-city-names),
[Git worktrees](https://www.conductor.build/docs/concepts/git-worktrees#how-conductor-uses-worktrees).
Our city list, UUID suffixes, `workspace/` branch prefix, and private-state managed
root are application decisions, not reverse-engineered Conductor algorithms.

Registration remains a separately recorded operation without Git/project-file
mutation. Successful new registration atomically records a first-creation
continuation. Reopening reads it; an empty workspace list is never execution
intent. Creation/reconciliation belongs to Rust `workspaces`; storage owns writes.
No provider adapter or provider history is involved.

## Components

CLI project inspection found Vite, base-nova, Base UI 1.9.0, Tailwind v4, Lucide,
and existing components for every working state. No additions are needed.
Sidebar submenu exports support workspace children; use a native button through
Base UI render composition for selection. Existing Button, Spinner, Badge,
Alert and Empty cover actions, observed stages, failures and empty navigation.
[Sidebar](https://ui.shadcn.com/docs/components/base/sidebar),
[Button](https://ui.shadcn.com/docs/components/base/button),
[Spinner](https://ui.shadcn.com/docs/components/base/spinner),
[Badge](https://ui.shadcn.com/docs/components/base/badge),
[Alert](https://ui.shadcn.com/docs/components/base/alert),
[Empty](https://ui.shadcn.com/docs/components/base/empty).

A Combobox would serve a searchable ref selector, but the accepted creation flow
has no picker. Progress percentages would imply unavailable measurements; present
core-observed stages instead. The composer’s task/model/effort/cloud/issue controls
belong to later working capabilities rather than placeholders in ticket 04.
