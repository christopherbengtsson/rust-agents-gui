<!-- LLM's: Ignore this file unless told otherwise -->

# feat

- Possible to view feature branch as PR before publishing to Github for self review.
  - Basically, copy GitHub into our app. Then full PR workflow could be within our app
- When others requested your review on a PR our application notices and starts a review agent in background. Before submitting review, user gets a summarize and actions on how to proceed

Large vision:

- A unified inbox for assigned tickets, requested reviews, own PR feedback and failing CI.
- Native Jira context and reliable status synchronization.
- Complete GitHub review drafting and publication.
- Consistent manual and agent controls for the same operations.
- Automation that connects these stages and exposes failures or decisions requiring attention.

# Improvements

Conductor’s frontend accesses SQLite through Tauri’s SQL plugin, backed by SQLx. Our project uses rusqlite inside the Rust core, keeping persistence behind Rust-owned operations

- app.db for application data, including repositories, workspaces, sessions, settings and attachments.
- cache.db for cached data.
- Both are configured to use WAL mode.
