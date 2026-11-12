# Working in this repository

## Read by task

- Before changing product behavior, architecture, or execution ownership, read [MVP.md](docs/MVP.md). It is authoritative for scope, module responsibilities, and the supervisor lifecycle.
- Before implementing or verifying Milestone 01, read [MILESTONE-01.md](docs/MILESTONE-01.md). Its acceptance criteria, verification order, and completion gate govern that slice.

## Implementation discipline

- Keep reconciliation and retry decisions inside the owning Rust core module. Tauri submits explicitly targeted actions and presents observations; reconnect retrieves authoritative status rather than deciding to repeat uncertain work. Test this through the core interface and actual supervisor-process seam.
- Before changing a provider adapter or its fixtures, inspect the supported runtime version's official protocol/schema and retained evidence. Record fixture provenance beside the fixtures: runtime version, schema/source revision, and whether messages are captured or synthetic. Synthetic faults exercise application recovery; they do not establish native provider behavior. Keep exact approval matching strict.
- Build each milestone on the intended MVP architecture. Temporary capability limits belong in admission or presentation, not permanent identity/storage restrictions. Add concrete implementations when needed; refine them when evidence warrants it.

## Verification and evidence

Use temporary repositories and private application state for deterministic tests. Add regression coverage for changed ownership or recovery behavior through the same interface as callers. Follow the milestone's separately agreed allowance before live-provider qualification; report fixture and native results separately, including failed or unperformed checks. Keep completed milestone results as evidence.

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
