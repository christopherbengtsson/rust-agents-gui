# Working in this repository

## Instruction precedence

Repository requirements and policies take precedence over conflicting global skill guidance. Apply this rule to direct skill use, nested skill calls, and delegated work. Use the compatible parts of a skill to carry out the requested task.

## Read by task

- Before changing product behavior, architecture, or execution ownership, read [MVP.md](docs/MVP.md). It is authoritative for scope, module responsibilities, and the supervisor lifecycle.
- Before implementing or verifying Milestone 01, read [MILESTONE-01.md](docs/MILESTONE-01.md). Its acceptance criteria, verification order, and completion gate govern that slice.
- Before designing tests, invoking engineering skills, or running verification, read [testing and verification](docs/agents/verification.md). It owns engineering-skill constraints and testing policy.

## Implementation discipline

- Keep reconciliation and retry decisions inside the owning Rust core module. Tauri submits explicitly targeted actions and presents observations; reconnect retrieves authoritative status rather than deciding to repeat uncertain work.
- Before changing a provider adapter or its fixtures, inspect the supported runtime version's official protocol/schema and retained evidence. Record fixture provenance beside the fixtures: runtime version, schema/source revision, and whether messages are captured or synthetic. Synthetic faults exercise application recovery; they do not establish native provider behavior. Keep exact approval matching strict.
- Build each milestone on the intended MVP architecture. Temporary capability limits belong in admission or presentation, not permanent identity/storage restrictions. Add concrete implementations when needed; refine them when evidence warrants it.

## Verification and evidence

Follow the milestone's separately agreed allowance before live-provider qualification; report unit, fixture-desktop, and native results separately, including failed or unperformed checks. Keep completed milestone results as evidence.

RAT is archived and read-only. Follow its AGENTS.md and required documentation before inspecting it; preserve its files, Git state, credentials, and provider history.

## Agent skills

### Issue tracker

Specs and tickets use local Markdown under `.scratch/`.
See `docs/agents/issue-tracker.md`.

### Triage labels

Use the default five triage labels.
See `docs/agents/triage-labels.md`.

### Domain docs

Use one root glossary and repository-wide ADRs.
See `docs/agents/domain.md`.
