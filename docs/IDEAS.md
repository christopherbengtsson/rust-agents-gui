<!-- LLM's: Ignore this file unless told otherwise -->

# Milestone 1 fix

- No fucking migration code or backwards compatibility code. E.g. crates/core/src/workspaces/creation.rs `pub legacy`

# MVP scope ideas

- Provider CLI setup - should we use https://ui.shadcn.com/docs/ai-elements/base/terminal instead for the UI and not full terminal, letting out program do the cli command in the background and just display it with the shadcn terminal ui?
- Claude support. Make code provider agnostic
- Auto discovered local repos when user explicitly adding new repo
- Auto discovered remote repos when user explicitly adding new repo (auto clone)

# MVP fix

- profile scrolling with many remembered workspaces and a large diff. If it becomes costly, the first changes would be isolating scroll updates from broader rendering and batching storage writes further. Existing checks prove restoration works; they don’t quantify that performance cost.

# Improvements

- tsconfig build: exclude stuff like test-support

# Big future ideas

- Possible to view feature branch as PR before publishing to Github for self review.
  - Basically, copy GitHub into our app. Then full PR workflow could be within our app
- When others requested your review on a PR our application notices and starts a review agent in background. Before submitting review, user gets a summarize and actions on how to proceed
- A feature where users can setup agent workflows for autonomous feature implementations, e.g. when one session is complete with implementation and review on it's ticket it starts a new session by itself with configured prompt and so on. Have a service storing context in a db table to be shared etc, maybe one or many coordinators.

Large vision:

- A unified inbox for assigned tickets, requested reviews, own PR feedback and failing CI.
- Native Jira context and reliable status synchronization.
- Complete GitHub review drafting and publication.
- Consistent manual and agent controls for the same operations.
- Automation that connects these stages and exposes failures or decisions requiring attention.
