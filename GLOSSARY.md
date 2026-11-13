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
