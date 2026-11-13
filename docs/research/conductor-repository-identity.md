# Conductor repository identity

Researched 2026-11-13 using official public documentation. No installed Conductor
application or private implementation was inspected.

Conductor's Open project imports an existing local repository. Its documented
onboarding then creates a workspace. Our milestone separates registration and
workspace creation into tickets 03 and 04. [First workspace](https://www.conductor.build/docs/first-workspace)

Its model has one repository per project and multiple workspaces per repository.
Workspaces carry their own branches and working trees. Conductor-created linked
worktrees share Git data with the original checkout, which remains the project
root. [Core model](https://www.conductor.build/docs/concepts/workspaces-and-branches#the-core-model),
[Git worktrees](https://www.conductor.build/docs/concepts/git-worktrees)

Local deep links select already registered repositories by absolute root path,
ignoring trailing slashes. This documents lookup, not registration deduplication.
[Deep links](https://www.conductor.build/docs/reference/deep-links#path)

These sources do not establish whether initial linked-worktree imports are
accepted, rejected, or normalized; how separate clones are deduplicated; or how
moves and symlink aliases affect Conductor identity. Our user explicitly chose
shared Git-directory grouping, separate application IDs, first-location retention,
and separate registrations for independent clones. [Decision](../adr/0003-shared-local-repository-identity.md)

Git's supported local version is 2.50.0. Read-only `rev-parse` supplies checkout
root and shared Git directory; application policy canonicalizes both. This does
not discover or import workspace history. [Git rev-parse](https://git-scm.com/docs/git-rev-parse),
[Git worktree](https://git-scm.com/docs/git-worktree)
