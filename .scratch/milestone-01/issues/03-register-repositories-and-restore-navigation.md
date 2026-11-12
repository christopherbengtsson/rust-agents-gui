# 03: Register repositories and restore navigation

**What to build:** Register an existing Git repository from the desktop, retain its identity, and restore repository navigation after reopening without modifying project files or starting an agent.

**Blocked by:** 02a: Establish the incremental UI foundation

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A1; durable identity/storage foundation used by subsequent behavior.

- [ ] Validate an existing Git repository before saving a usable registration; invalid paths and unavailable repositories give actionable errors without creating broken saved entries.
- [ ] Persist repository identity in supervisor-owned application storage outside project files and restore that identity across restart.
- [ ] Display registered repositories in usable navigation and surface an unavailable previously registered repository honestly.
- [ ] Extend the established React/shadcn application shell and compact dark visual baseline with registration, selected repository, validation, loading and restored-navigation states; retain shared tokens and controls.
- [ ] Keep registration/identity policy in workspaces and authoritative writes in storage; Tauri submits actions and displays observations.
- [ ] Introduce only the durable action/observation behavior registration needs, including repeat-action status and visible storage failures.
- [ ] Unit-test validation, canonical identity, restored records, repeated/conflicting action IDs, unavailable paths and storage failures with mocked dependency outcomes. Assert no project mutation or provider start. Separately demonstrate desktop registration, restored navigation and error states using disposable repositories/private state.
- [ ] Establish small internal storage/Git/path substitutions as the behavior needs them, keeping production effects in their owning modules. Cover navigation admission and bounded snapshot growth with supplied records, not bulk repository creation.
- [x] Establish an explicit unit-only default behavior gate without sidecar staging, excluding inherited process fixtures. Cover inherited connection/discovery decisions with controlled outcomes where needed; retain actual OS qualification separately. Keep useful static checks single-run and report unit execution separately from compilation, complete verification and packaging.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

2026-11-12: The user requested current-repository alignment with `docs/agents/verification.md`, which was introduced after tickets 01–02. Brought forward only this ticket's verification gate: controlled Rust unit replacements, explicit process qualification, separate compilation/execution measurements and target safeguards. See [policy-alignment evidence](../../../docs/evidence/milestone-01-verification-policy-alignment.md). Repository registration and the remaining acceptance items are still unimplemented; this ticket is not complete.
