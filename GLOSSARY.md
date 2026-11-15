# Agent Workspace

Local repository and isolated-workspace navigation for agent-assisted coding.

## Language

**Repository**:
A registered local Git codebase shared by its main checkout and linked worktrees.
Independent clones are separate repositories.
_Avoid_: Project when referring to a registration.

**Registered location**:
The checkout folder first selected when registering a repository. Its availability
can change without erasing the saved repository identity.
_Avoid_: Repository identity when referring only to a folder path.

**Workspace**:
An isolated checkout and branch for a stream of work within a repository.
Several conversations using different providers can share one workspace.
_Avoid_: Repository when referring to an individual workspace.

**Registration action**:
An explicitly requested attempt to register a repository, with its own retained
identity and observable pending, completed, or failed outcome.
_Avoid_: Connection retry when referring to submitting registration.

**Creation action**:
An explicitly requested attempt to create an isolated workspace, with a retained
identity and observable outcome. Adding a new repository also requests its first
creation action.
_Avoid_: Registration when referring to branch or workspace creation.

**Pinned base**:
The exact commit a workspace starts from, retained as its comparison origin even
when the workspace branch changes.
_Avoid_: Current default branch when referring to the saved starting commit.

**Workspace name**:
A readable label that distinguishes a workspace within its registered repository.
It remains associated with that workspace's saved history.
_Avoid_: Workspace identity when referring only to its readable name.

**Provider**:
A named agent-runtime integration, such as Codex or Claude Code. Several providers
can be available together, independently of which one the user is inspecting.
_Avoid_: Selected tab when referring to provider availability.

**Provider connection**:
The observed setup and availability of a provider under its effective runtime
configuration. It does not imply an active conversation or a continuously running process.
_Avoid_: Supervisor connection when referring to agent-provider setup.

**Login attempt**:
An explicitly requested sign-in with its own identity and outcome, tied to a
provider and effective runtime configuration independently of the visible page.
_Avoid_: Authentication choice when referring to a started sign-in.

**Conversation**:
A provider-bound stream of work and retained history within a workspace. Changing
providers starts a separate conversation without changing the workspace's identity.
_Avoid_: Workspace when referring to a provider conversation.
