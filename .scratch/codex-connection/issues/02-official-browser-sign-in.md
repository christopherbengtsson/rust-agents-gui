# 02: Official Codex browser sign-in

**What to build:** Explicitly sign in through the browser using official Codex,
return to progress after navigating elsewhere, cancel safely, and see the
authoritative authentication result without starting agent work.

**Blocked by:** [01 — Provider readiness and diagnostics](01-provider-readiness-and-diagnostics.md)

**Status:** ready-for-agent

## Start here

Follow the [fresh-session procedure](../spec.md#authority-and-fresh-sessions) and
read 01's checked acceptance and dated adapter/cohort handoff. Read the
[connection journey](../spec.md#connection-journey) for browser/progress/recovery
behavior, [ownership/protocol](../spec.md#ownership-and-protocol) for login
authority/lifetimes, and [qualification rules](../spec.md#verification-and-qualification).
Follow [verification policy](../../../docs/agents/verification.md); inspect the
selected cohort's official authentication schema before extending its adapter.
Apply [architecture and extension constraints](../spec.md#architecture-and-extension-constraints):
extend 01's provider-connection owner and observation feature module rather than
creating another lifecycle owner or frontend execution state.

## Acceptance

- [ ] Add the explained ChatGPT choice with browser sign-in emphasized, preserving the existing-authentication-first presentation from 01. Selection changes presentation only. Leave the CLI login action for 03 and keep API-key setup omitted and Claude actions disabled.
- [ ] Only an explicit action starts official Codex-managed browser authentication. Open the returned URL, present starting/pending/completed/failed/cancelled outcomes, and reopen the same browser attempt when requested without automatically starting another. Keep credentials and effective configuration runtime-owned.
- [ ] Establish the Rust-owned login-attempt owner that 03 will extend. Bind actions/results to the expected supervisor and exact attempt; reject duplicate/competing starts and stale results. Lost start/cancel replies and reconnect retrieve/reconcile status rather than repeating effects. A selected method or pending UI flag is not evidence of an active login or changed billing route.
- [ ] Add attempt admission/reconciliation inside 01's provider-connection module. Bind every result to its provider/effective-configuration target as well as owner/attempt; exclude competing authentication within the same credential scope, preserving unrelated targets within resource budgets. Keep supported browser authentication mechanics local to Codex rather than making them mandatory for future providers.
- [ ] Pending login survives provider-tab, Settings-section and workspace navigation. A compact sign-in-progress status links back to Providers. Navigation/preload has no login effects; explicit Cancel requests runtime cancellation and retains failed/unknown cleanup visibly until reconciled. Cover late setup, desktop reconnect and explicit Quit without competing/replacement work before physical completion.
- [ ] Retain pending-attempt observation interest above routed content using 01's existing observation composition. Implement and qualify owned shutdown for the browser-login resources introduced here; desktop observation cleanup alone is insufficient. Failed cleanup fences the affected authentication scope while remaining visible.
- [ ] Recheck authoritative authentication after browser completion; distinguish login success/failure from failed status verification and retained/stale account evidence. Show the result without automatic navigation or agent launch. Preserve Settings return/focus, overlay keyboard precedence and workspace context.
- [ ] Passing deterministic units cover successful/failed/cancelled attempts, browser-open failure/reopen, unrelated/late notifications, duplicate actions, lost replies, owner changes, timeout and cleanup uncertainty. Verify cancellation and process/time effects through substitutions, and that no tokens are copied into application state/history/logs.
- [ ] With controlled synthetic targets, prove a late completion for one provider/configuration cannot update another, lost replies do not start another attempt, and uncertain cleanup does not block unrelated targets absent a shared resource conflict. Keep Claude/OpenCode integration and native concurrency qualification out of this slice.
- [ ] Focused actual fixture-desktop evidence demonstrates handoff, progress return, cancellation, error recovery, status recheck and keyboard/focus behavior through the production supervisor path. Record exact results and the attempt/progress interfaces 03 will reuse in a dated handoff. Record any agreed real-authentication scenario separately, or explicitly leave it unperformed; no agent thread/turn is permitted.

## Handoff boundary

The working increment is browser sign-in and its observation/recovery path.
The terminal renderer/PTY is [03](03-embedded-cli-login.md); cumulative
qualification is [04](04-qualify-connection-journey.md). Share the attempt owner
with the next method rather than adding a separate retry/lifetime authority.
Real-authentication and native-agent results remain separate from fixtures; this
ticket grants no live-agent submission allowance. Stop after this ticket.

## Comments

2026-11-15: Published from the approved four-session split. Implementation and
qualification acceptance remain unchecked.

2026-11-15: Browser authentication now explicitly extends 01's cohesive owner and
shared observations with provider-scoped identity, admission and cleanup. The
working browser journey and separately recorded qualification scope are retained.

2026-11-15: The user-approved [presentation refinement](../spec.md#presentation-refinement--2026-11-15)
supersedes the original permanent Refresh presentation. Add the visible ChatGPT choice beneath readiness, outside collapsed Setup help. Preserve automatic readiness verification on Codex-tab activation and after browser completion. Extend Rust admission so checks cannot compete with active authentication or uncertain cleanup; reconnect retrieves the existing attempt without replay.
