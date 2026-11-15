# 03: Embedded official CLI login

**What to build:** Run official Codex login in an embedded terminal, keep the same
terminal available after navigating away and back, and finish or cancel with an
authoritative authentication result and observed process cleanup.

**Blocked by:** [02 — Official browser sign-in](02-official-browser-sign-in.md)

**Status:** ready-for-agent

## Start here

Follow the [fresh-session procedure](../spec.md#authority-and-fresh-sessions).
Read 02's checked acceptance and dated attempt-owner/progress handoff, then the
[connection journey](../spec.md#connection-journey) for CLI and navigation behavior,
[ownership/protocol](../spec.md#ownership-and-protocol) for process/cleanup rules,
and [verification policy](../../../docs/agents/verification.md). Use the
[Conductor research](../../../docs/research/conductor-provider-connection.md)
for the reference interaction and intentional differences, not as a qualified
terminal dependency/version or native-provider contract.
Apply [architecture and extension constraints](../spec.md#architecture-and-extension-constraints)
and reuse 01/02's actual provider-connection and observation interfaces.

## Acceptance

- [ ] Add the explained CLI choice alongside ChatGPT, retaining browser sign-in's greater emphasis. Selecting it changes presentation only; an explicit Run codex login action launches the selected installed official executable directly with its login argument and effective runtime configuration in a PTY.
- [ ] Provide bounded input/output, resize, focus, starting/running/error/exited states and Done through the production supervisor/typed desktop path. The terminal ends with the login command and leaves no general shell. Use small renderer/process seams a later development terminal can extend; workspace terminal management remains deferred.
- [ ] Extend 02's shared Rust login-attempt owner, progress observations and duplicate/cancellation rules instead of introducing a second authority. Browser and CLI attempts cannot compete within the same effective credential/configuration scope; late input, resize, output and completion remain bound to the exact provider/configuration, terminal/attempt and owner. Preserve independent unrelated targets within resource budgets.
- [ ] Navigation releases the terminal view without stopping the login process. Returning reattaches the same bounded transient terminal presentation; the compact progress link remains discoverable. Explicit Cancel sign-in or Cancel sign-in and close terminal requests owned cleanup. Done after exit dismisses the completed presentation without implying authentication success.
- [ ] Recheck authoritative authentication after command exit, including successful exit codes. Present authentication failure separately from status-read failure and stale account evidence. Completion starts no agent work and does not navigate away. Preserve terminal keyboard precedence, accessible control names, focus return and Settings/workspace context.
- [ ] Cancellation, late PTY setup, process failure, desktop loss/reconnect and explicit Quit retain owned handles until physical cleanup is observed. Show failed/unknown cleanup and fence replacement. Keep transcript presentation transient; runtime-owned credential data does not become application history/logs.
- [ ] Extend the existing provider-observation feature module and retained pending interests for terminal attachment/progress, using its Query/feed/scheduling mechanisms. Keep process/PTY mechanics behind small internal seams; extract shared mechanisms only for exercised consumers, without a general terminal framework or provider-wide CLI requirement.
- [ ] Passing deterministic units cover command identity/construction, input/resize targeting, bounded output, exit without authentication, launch/transport/owner errors, duplicate/competing actions, late setup, cancellation and cleanup uncertainty with controlled process/PTY/time outcomes.
- [ ] Prove same-scope browser/CLI exclusion and cross-provider/configuration terminal/result isolation using controlled synthetic targets. A cancelled or failed-cleanup target retains its handles and fences conflicting replacement while unrelated targets remain independent. These checks introduce no Claude/OpenCode processes or compatibility claim.
- [ ] Focused actual fixture-desktop evidence demonstrates terminal input/output/resize, focus/keyboard use, navigation/reattachment, completion/recheck and owned process cleanup using controlled fixture commands/private state. Record exact results, failures and unperformed limits with a dated handoff for 04; separately record any agreed real login check without agent threads/turns.

## Handoff boundary

The working increment is both authentication methods with the same lifecycle and
progress authority. [04](04-qualify-connection-journey.md) qualifies their combined
desktop journey; it does not replace this session's focused checks. General shells,
workspace terminals, run/test controls and native-agent submissions remain outside
this ticket. Stop after this ticket.

## Comments

2026-11-15: Published from the approved four-session split. Implementation and
qualification acceptance remain unchecked.

2026-11-15: CLI login extends the same provider-scoped owner/observations as the
browser path. Small internal process seams support this working slice; general
terminals and new provider implementations remain deferred.
