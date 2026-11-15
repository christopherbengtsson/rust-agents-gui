# Native desktop action sequence — 2026-11-15

All UI actions used native `cua_repl` App methods (`click`, `pressKey`, `scroll`,
`drag`, `typeText`). Captures used `getAXState({disableDiffing:true})` and
`getScreenshot`; no WebView evaluation or state/IPC injection was used.

From the workspace root, the initial/main/loss hosts used:

```sh
python3 -u fixtures/codex-readiness/qualify.py \
  'target/release/bundle/macos/Agent Workspace.app/Contents/Resources/workspace-supervisor' \
  docs/evidence/codex-connection/04/desktop.json \
  'target/release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop' \
  --browser --journey
```

Subsequent fresh-state reports were `desktop-main.json` and `supervisor-loss.json`.
For diagnosis/affected checks, the same command used `reading-debug.json`,
`reading-debug-second.json` and `reading-final.json`, with:

```sh
--resume-from docs/evidence/codex-connection/04/desktop-main.json
```

These hosts ran in retained PTYs. Their `commands` arrays record every control
command and timestamp, plus disposable Git preparation argv/results. `capture`
files retain full supervisor observation/event evidence at each checkpoint.

1. Initial cohort: `scenario ready`; Settings → Providers; capture immediate
   Checking then settled Connected. `scenario unknown_billing`; Claude tab →
   Left/Return to Codex. Capture unknown billing/account and disabled Claude.
2. `runtime-missing`; General → Providers; capture installation recovery.
   `runtime-restore`; `scenario gateway_unsupported`; Check again; capture
   capability recovery. `scenario no_auth`; Check again; capture sign-in guidance.
   Escape returns to the Settings gear. Register the prepared disposable repository.
   `dirty-workspace` failed due the helper's missing owner; host cleaned up.
3. Corrected main host: Add repository → enter its recorded private repository
   path → Register. `dirty-workspace` now succeeds. Select reading.txt in Changed
   files. Native screenshot coordinates are pixels: the successful scroll used
   `scroll([900,950], 'down', 2)` in the 1800×1360 capture. The earlier [450,470]
   attempt hit the surrounding layout. Captures 09–11 distinguish these states.
4. Settings → Providers → explicit Sign in through browser. Observe the loopback
   fixture page in Dia, named Fixture Codex browser sign-in. `compete`; capture
   pending state. Claude → General → Escape to workspace. Capture progress and
   the lost reading position (product failure, capture 15).
5. Compact progress link → Reopen browser. `capture before-desktop-crash`;
   `kill-desktop`; `restart-desktop`; capture reopened progress and unchanged
   owner/attempt/event count. Compact link → Reopen → Tab → Return cancels the
   same attempt and focuses its result.
6. `drag([1797,1357],[1197,957])` establishes a 1200×960 screenshot / 600×480-point
   minimum window. Start a new explicit attempt. Scroll the main area with
   `scroll([800,700], 'down', 2)` and capture reachable wrapped controls. In Dia,
   activate Complete fixture sign-in; capture verified completion. Restore ordinary
   size with the reverse corner drag.
7. `scenario browser_pending`; explicit sign-in; `scenario browser_verify_failure`;
   capture completed browser outcome alongside failed/stale readiness.
   `scenario gateway_delay`; explicit sign-in then Cancel during Starting;
   `scenario browser_pending`; capture settlement, including the production
   15-second verification timeout. Start a final pending attempt, capture, Cmd-Q.
   `processes` observed both children exited before `stop`.
8. Fresh loss host: Settings → Providers → explicit sign-in; `capture before-loss`;
   `kill-supervisor`; capture disconnected stale evidence. `restart-supervisor`;
   Escape → workspace Refresh status → Codex sign-in needs attention. Capture
   durable cleanup fence and unavailable replacement controls. Cmd-Q displays owned
   shutdown uncertainty. `capture fenced-quit`; `stop` performs separately labeled
   harness termination of its exact children without clearing the durable fence.
9. Diagnostic builds: resume main private state after its owner exits, select
   reading.txt, scroll the same viewport. Read only that fixture bundle's WebKit
   local-storage database to capture temporary attachment/scroll/save/identity
   probes. The first native assertion fails; the second probe locates automatic
   comparison-ID replacement. Diagnostic UI source was removed before final build.
10. Final corrected source: resume main private state. Add repository → Browse
    opens an inner native folder chooser. Escape leaves the app dialog open;
    capture 26. Escape closes that dialog, restores its trigger and retains route.
    No unrelated chooser listing is saved. Select reading.txt, scroll, capture 27
    and allowlisted presentation record (top=1360). Settings → Providers → sign-in
    → Claude → General → Escape. Capture 28 and identical presentation record:
    file, comparison, position and route are preserved, with progress still active.
11. Close eight currently observed fixture pages with Dia Cmd-W, stopping when
    the active content ceases to be the fixture page. Cmd-Q in the final desktop.
    `processes` independently confirms both desktop/supervisor exited; `stop` then
    joins them. Final report records desktop_exit=0, supervisor_exit=0 and
    desktop_terminated_by_harness=false. Final process/original-checkout audit
    finds no remaining owned processes or fixture PIDs.

The capture helper initially retained the first App binding while the second host
was running. Its observer-induced relaunch/capture 08 is recorded in
`observer-error.json`; it is not a reading-context pass. Later capture helpers
accept the App target explicitly. One attempted click after a full AX capture
used an obsolete element index and was rejected before action; the next call
used the new General index. No product defect is inferred from observer errors.
