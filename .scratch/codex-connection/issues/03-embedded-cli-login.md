# 03: Embedded official CLI login

**What to build:** Run official Codex login in an embedded terminal, keep the same
terminal available after navigating away and back, and finish or cancel with an
authoritative authentication result and observed process cleanup.

**Blocked by:** [02a — Provider runtime compatibility policy](02a-runtime-compatibility-policy.md) (after 02)

**Status:** wontfix

**Disposition — 2026-11-15:** Deferred from the current connection feature by
user decision, not implemented or qualified. Keep the single runtime-managed
browser sign-in action and automatically recognize existing Codex authentication.
CLI login, PTY/xterm, terminal styling/CSP work and a read-only command-output
replacement are all deferred. Ticket 03 no longer blocks
[04 — Qualification](04-qualify-connection-journey.md), which follows 02a directly.
The `wontfix` triage label removes this proposal from the actionable queue; it does
not rule out a separately selected future terminal feature.

The proposal and unchecked acceptance below are retained as planning history.
The current [scope amendment](../spec.md#authentication-scope-amendment--2026-11-15)
supersedes instructions to implement them.

## Start here

The [archived implementation plan and API references](../03-implementation-plan.md)
retain the former replay/CSP decisions and extension research. Consult them only
if a future task explicitly selects this deferred capability. The next selected
connection session is 04 under the current scope amendment.

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

## Terminal implementation

Use these selected libraries, adding dependencies with their first use:

- **React terminal:** [`@xterm/xterm`](https://github.com/xtermjs/xterm.js)
  for terminal rendering and input, with
  [`@xterm/addon-fit`](https://github.com/xtermjs/xterm.js/tree/master/addons/addon-fit)
  for container sizing. A small React adapter owns renderer attachment, focus,
  keyboard handling and resize notifications.
- **Rust PTY:** [`portable-pty`](https://docs.rs/portable-pty/0.9.0/portable_pty/)
  directly inside a small private module in supervisor-hosted `runtimes`. That
  module owns PTY creation, command launch, bounded input/output, resize and worker
  completion behind an internal execution seam. Reuse the captured executable,
  effective configuration, environment and directory for CLI login and its status
  recheck. Run the selected Codex executable with `login` directly in the PTY. Keep
  blocking I/O and waits in owned workers, with handles retained through physical
  completion, including setup that finishes after cancellation.
- **Application integration:** reuse the production typed supervisor/Tauri path
  and provider-observation module. The supervisor owns PTY sessions; Tauri forwards
  targeted actions and observations. Extend 02's login-attempt owner for admission,
  authentication verification, cancellation and cleanup fencing. Factor the common
  intent-recording, cancellation, cleanup, verification and fence-release sequence
  once within that owner as CLI execution is added. Keep browser reopen and CLI
  terminal controls in method-specific attempt resources. Keep renderer and PTY
  mechanics separate from Codex authentication policy so future development
  terminals can reuse them.

These are focused implementation changes within this ticket, introduced with the
CLI consumer. Limit frontend changes to the required attachment/progress behavior
inside the existing provider-observation module; the xterm adapter owns display
resources, focus and keyboard handling. A future workspace terminal will own its
shell/session policy while reusing the renderer and PTY mechanics. Broader hook
refactoring, a general terminal framework and workspace terminal management remain
deferred.

Select and pin compatible library versions against official release/API sources
and record them in the implementation handoff. The links identify the selected
libraries; they do not establish compatibility with our packaged Tauri WebView.
Qualify that through the focused fixture-desktop checks below.

Bound transport and renderer-write queues as well as scrollback, following
[xterm's flow-control guidance](https://xtermjs.org/docs/guides/flowcontrol/).
Reattachment must preserve or reconstruct terminal screen state; a truncated raw
output tail alone does not guarantee a valid screen. PTY kill requests, child exit
and renderer disposal do not establish complete owned-process/worker cleanup;
the existing lifecycle acceptance below still applies.

## Acceptance

- [ ] Implement the selected terminal libraries and ownership boundaries above; record pinned versions and actual fixture-desktop compatibility evidence.
- [ ] Add the explained CLI choice alongside ChatGPT, retaining browser sign-in's greater emphasis. Selecting it changes presentation only; an explicit Run codex login action launches the selected installed official executable directly with its login argument and effective runtime configuration in a PTY.
- [ ] Provide bounded input/output, resize, focus, starting/running/error/exited states and Done through the production supervisor/typed desktop path. The terminal ends with the login command and leaves no general shell. Use small renderer/process seams a later development terminal can extend; workspace terminal management remains deferred.
- [ ] Extend 02's shared Rust login-attempt owner, progress observations and duplicate/cancellation rules instead of introducing a second authority. Browser and CLI attempts cannot compete within the same effective credential/configuration scope; late input, resize, output and completion remain bound to the exact provider/configuration, terminal/attempt and owner. Preserve independent unrelated targets within resource budgets.
- [ ] Navigation releases the terminal view without stopping the login process. Returning reattaches the same bounded transient terminal presentation; the compact progress link remains discoverable. Explicit Cancel sign-in or Cancel sign-in and close terminal requests owned cleanup. Done after exit dismisses the completed presentation without implying authentication success.
- [ ] Recheck authoritative authentication after command exit, including successful exit codes. Present authentication failure separately from status-read failure and stale account evidence. Completion starts no agent work and does not navigate away. Preserve terminal keyboard precedence, accessible control names, focus return and Settings/workspace context.
- [ ] Cancellation, late PTY setup, process failure, desktop loss/reconnect and explicit Quit retain owned handles until physical cleanup is observed. Show failed/unknown cleanup and fence replacement. Keep transcript presentation transient; runtime-owned credential data does not become application history/logs.
- [ ] Extend the existing provider-observation feature module and retained pending interests for terminal attachment/progress, using its Query/feed/scheduling mechanisms. Keep process/PTY mechanics behind small internal seams; extract shared mechanisms only for exercised consumers, without a general terminal framework or provider-wide CLI requirement.
- [ ] Passing deterministic units exercise the real owning-module lifecycle with controlled execution/storage/process/PTY/time outcomes, covering command identity/construction, input/resize targeting, bounded output, exit without authentication, launch/transport/owner errors, duplicate/competing actions, late setup, cancellation and cleanup uncertainty.
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

2026-11-15: The user-approved [presentation refinement](../spec.md#presentation-refinement--2026-11-15)
supersedes the original permanent Refresh presentation. Add the visible CLI choice alongside ChatGPT beneath readiness, outside collapsed Setup help. Preserve automatic entry verification and verification after command exit. Extend the same Rust admission so browser/CLI login and readiness checks cannot compete within a configuration scope.

2026-11-15: User selected `@xterm/xterm`, `@xterm/addon-fit` and direct Rust
`portable-pty` integration following read-only library and installed Conductor
research. The Terminal implementation section records the decision for session 03;
dependency installation, version pinning and implementation/qualification remain
pending.

2026-11-15: User approved the scoped architecture refinement in Terminal
implementation. Shared login lifecycle and privately owned PTY mechanics belong
to 03 alongside its required frontend attachment work. Implementation acceptance
and qualification remain unchecked.

2026-11-15: Planning is ready for a fresh implementation session. The user selected
complete transient replay capped at 2 MiB, with visible failure and owned cleanup
on exhaustion, and a pinned xterm nonce patch preserving the strict CSP. The
[implementation plan](../03-implementation-plan.md) retains official API/source
references, current interfaces, library versions selected for first use and the
user's bounded verification instruction: one full gate, targeted reruns for
concrete failures or review fixes, and separately recorded fixture qualification.
Product code, dependency installation and acceptance remain unperformed here.

2026-11-15: The user approved reducing the connection journey to existing-auth
detection plus one official-runtime browser sign-in action. The normal direct
`codex login` path also performs browser authentication; its separate command
viewer did not establish a required recovery capability for this local-only app.
Deferred this ticket and its terminal plan, including the proposed read-only
shadcn alternative. No acceptance was completed. 04 now follows 02a; completed
01/02/02a evidence and the parent/06a/07 gates remain intact.
