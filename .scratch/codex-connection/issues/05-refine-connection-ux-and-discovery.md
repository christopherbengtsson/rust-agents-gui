# 05: Refine Codex connection UX and executable discovery

Status: ready-for-agent

**Blocked by:** [04 — Qualify the complete connection journey](04-qualify-connection-journey.md), completed for its recorded fixture scope in 627f1af and 612ffef.

## Outcome

Implement the [accepted UX/discovery amendment](../spec.md#connection-ux-and-discovery-amendment--2026-11-15).
Christopher should see only verified identity when connected, understand uncertainty,
and recover a missed or updated Codex installation without restarting the supervisor.
Read AGENTS.md and its required product, verification, observation and component
guidance. Preserve existing trial/research edits and historical evidence.

## Implementation contract

Rust owns fresh discovery, logical selected path versus pinned resolved launch,
validation, action reconciliation, persistence and process fencing. Reuse the
provider observation module and Query cache. Native chooser results target the
captured owner/configuration/revision; cancellation has no effect. Persist a
validated selection after capability checks and cleanup, even when auth is absent.
Unavailable saved selections do not fall back. No credentials are stored.

Use inherited PATH, bounded fresh login/interactive-shell PATH, then conventional
Homebrew/local locations. Keep credential configuration fixed. Use current contracts
and fresh private state; advance protocol 13 to 14 with no legacy support.
Reuse installed UI controls, semantic indicator tones and native dialog. Preserve
pending sign-in, focus, navigation, uncertainty and cleanup fences.

## Acceptance

- [x] Implement the agreed presentation matrix, including full-body skeletons, minimal healthy identity/API-key status, actionable pending sign-in, yellow verification uncertainty, and red confirmed failures/not-found recovery.
- [x] Check again performs fresh discovery including shell PATH and symlink changes without replacing an active attempt's launch.
- [x] Already installed? exposes a native Locate Codex… chooser; validate selected executables/capabilities and check authentication automatically.
- [x] Remember validated selections across restart; cancellation/invalid selections preserve preferences; unavailable selections never silently fall back and persistence errors stay visible.
- [x] Deterministically cover changed decisions, stale/duplicate/lost-reply actions, navigation/focus and cleanup exclusion with substituted effects.
- [ ] Qualify affected packaged fixture and controlled native discovery paths, including unavailable/missed runtime, installation/update rediscovery, remembered and unavailable selection.
- [ ] Qualify a normal Finder/Dock launch with private state and separately label read-only real-authentication observations; preserve installation/configuration/credentials.
- [ ] Record keyboard/focus, ordinary/minimum-layout and cleanup observations with explicit accessibility and native limits.
- [x] Run affected checks, one final full gate/build and requested review; publish dated source/cohort/results evidence, reusing unchanged evidence and checking only verified acceptance.

## Limits

No sign-out, credential replacement, API-key setup, account switching, logout,
Remove connection/Change account controls, installer, terminal, Claude integration,
agent thread/turn, storage evaluation 06a or chat 07. Real browser sign-in requires
a separately agreed exact scenario. Distinguish fixture, controlled native discovery,
real authentication and participant feedback; never claim unperformed checks pass.

## Comments

2026-11-15: Published from the accepted implementation plan. Historical 04/trial
results and the user's existing edits remain intact. Implementation/qualification
acceptance starts unchecked.


### 2026-11-15 — implementation and bounded qualification

[Evidence](../../../docs/evidence/codex-connection/05.md) records protocol 14,
204 Rust / 51 tooling / 260 frontend cases passing across the full gate and focused
corrections, final packaging, 14 controlled native discovery assertions and separate
real Codex 0.169.0 existing-authentication readiness. Native process cohorts preserve
credentials and qualify unavailable/missed executable, fresh install/update lookup,
manual validation/persistence/restart and unavailable-selection fencing. Both
independent review axes have zero unresolved findings after scoped corrections.
Earlier failures and unchanged predecessor evidence are retained.

Computer Use rejected Agent Workspace access; Christopher explicitly selected
“Record UI checks as unperformed.” Therefore packaged fixture UI, native chooser,
normal Finder/Dock launch, focus, minimum-layout/accessibility and UI Quit checks
remain unchecked above. The real-authentication portion of the combined desktop
criterion was performed through the packaged supervisor; it is not a Finder/UI pass.
Implementation checks do not complete this ticket's remaining UI qualification or
whole-milestone/native-agent acceptance. Stop here; 06a/07 were not implemented.
