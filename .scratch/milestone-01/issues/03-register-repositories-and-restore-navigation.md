# 03: Register repositories and restore navigation

**What to build:** Register an existing Git repository from the desktop, retain its identity, and restore repository navigation after reopening without modifying project files or starting an agent.

**Blocked by:** 02: Reliable supervisor discovery and connection

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A1; durable identity/storage foundation used by subsequent behavior.

- [ ] Validate an existing Git repository before saving a usable registration; invalid paths and unavailable repositories give actionable errors without creating broken saved entries.
- [ ] Persist repository identity in supervisor-owned application storage outside project files and restore that identity across restart.
- [ ] Display registered repositories in usable navigation and surface an unavailable previously registered repository honestly.
- [ ] Keep registration/identity policy in workspaces and authoritative writes in storage; Tauri submits actions and displays observations.
- [ ] Introduce only the durable action/observation behavior registration needs, including repeat-action status and visible storage failures.
- [ ] Use temporary real repositories and private application state to verify the caller-visible behavior through the core interface and desktop; prove original files remain unchanged and no Codex process starts.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
