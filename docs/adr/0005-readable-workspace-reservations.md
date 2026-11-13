# Readable workspace reservations

Workspace names are human labels, distinct from internal workspace/action identity. Generate footballer slugs and numeric variants per registered repository; reserve every saved name permanently with its history. Reserve readable repository folders separately within application state. This trades reusable short names for stable history and recovery targets while avoiding UUIDs in ordinary UI, new branches and generated folder components.

The workspaces module owns allocation and reconciliation. Save a pending identity before read-only inspection, then reserve the final name, branch, path and repository folder in one storage transaction before mutation. A race after reservation requires explicit recovery of that exact location; it never silently chooses another. Keep UUIDs for internal identity and ownership evidence.

Schema 3 migration gives legacy workspaces stable, distinct clean labels without renaming branches or moving folders. Exact legacy locations remain accessible through explicit details. Development uses persistent worktree/protocol/application-version-specific state so incompatible owners can coexist; release state and explicit overrides retain their established meaning.

See the [accepted plan](../../.scratch/milestone-01/readable-workspaces-plan.md) and [research](../research/readable-workspace-names.md).
