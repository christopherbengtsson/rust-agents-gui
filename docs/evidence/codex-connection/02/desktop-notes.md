# Ticket 02 fixture-desktop observations

Observed 2026-11-15 through the packaged Tauri desktop, production supervisor,
and the synthetic runtime. No frontend state/IPC injection, real authentication,
agent thread/turn, or Claude runtime supplied these screens.

## Journey observations

| Scenario                 | Observed result                                                                                                                                                                                                                                                                                                                                                                     |
| ------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Initial Providers        | Existing readiness appears before the visible ChatGPT action. Missing authentication shows recovery; help/details start collapsed. [Ordinary capture](desktop-no-auth.png).                                                                                                                                                                                                         |
| Explicit browser handoff | Clicking Sign in through browser starts one pending attempt and opens the harness's loopback page in Dia. The page explicitly identifies synthetic authentication and collects no credentials. [Browser capture](browser-handoff.png).                                                                                                                                              |
| Navigation               | Pending progress remains above Claude, General and the main workspace layout. Settings Back restores the gear's focus. The compact link returns to the same pending attempt. [Workspace capture](workspace-progress.png).                                                                                                                                                           |
| Desktop crash/reconnect  | The harness kills only its exact desktop child and reopens it against the same supervisor. The reopened main layout immediately shows pending progress. The retained attempt was `f361bb01-ba09-48de-bf9d-05e5331cb017`, target `9ca6902a-fbd1-46cd-b8e5-4ad185a5bf9e`; no second login was submitted.                                                                              |
| Cancellation             | Cancel requests exact runtime cancellation. After physical completion, the UI says Browser sign-in cancelled and offers an explicit new action.                                                                                                                                                                                                                                     |
| Keyboard focus           | Initial cancellation removed the focused control and lost focus. Corrected builds focus the status paragraph when their action removes its control. Final keyboard Tab/Return cancellation focused AX container 71 containing Browser sign-in cancelled. Reopen's pending-disable cycle initially lost focus too; its final Tab/Return check retained AX button 76, Reopen browser. |
| Completion               | The browser's Complete fixture sign-in link reaches `/complete`. The desktop subsequently displays Connected, the fixture account/plan/billing and Browser sign-in completed without navigation or agent work. [Completion capture](completed-current.png).                                                                                                                         |
| Browser-open failure     | An intentionally invalid synthetic URL leaves the original attempt pending and displays Browser could not be opened. Reopen retains that attempt; cancellation remains available.                                                                                                                                                                                                   |
| Verification failure     | Synthetic browser success followed by a failed account read shows Browser sign-in completed separately from Setup needs attention and Last known evidence. [Failure capture](verification-failure.png).                                                                                                                                                                             |
| Login failure            | The controlled failed completion shows Browser sign-in failed, followed by authoritative unauthenticated readiness. It does not navigate or start work.                                                                                                                                                                                                                             |
| Minimum size             | The 600×480-point window renders at 1200×960 pixels. Main content scrolls, long text wraps, and sign-in, result and disclosures remain reachable. An initial drag selected text; the corrected corner drag established the minimum size. [Minimum capture](minimum-controls.png).                                                                                                   |

The journey cohort in `desktop-second.json` includes several desktop restarts
while focus fixes were built. Its initial executable hash is not a hash for every
reopened executable. The screenshots establish their recorded UI checkpoints;
the final steady owned-Quit cohort below has its own exact hashes. Existing
workspace/diff restoration and overlay precedence also retain their routed unit
coverage and predecessor desktop evidence; this cohort used an empty disposable
workspace layout, not a new full diff-restoration qualification.

## Native lifecycle correction and final result

Window close hides instead of cancelling. The first hidden-window attempt exposed
a missing macOS Reopen handler. The corrected build reopened the hidden window
with the same pending sign-in; both desktop and supervisor remained alive while
hidden. The tool returned the pending Providers tree after reopening; the saved
`.ax.txt` file is only a subsequent no-change delta, as explained below.

Actual Cmd-Q initially exited the desktop while leaving the supervisor/login
alive. Pinned Muda 0.20.0 maps its predefined macOS Quit directly to NSApp
`terminate:`, bypassing Tauri's cancellable ExitRequested callback. The custom Quit
menu item now requests Tauri exit and waits for owned shutdown. Other default menu
items remain intact. These failed observations are retained in
[native-lifecycle.json](native-lifecycle.json); harness-owned cleanup subsequently
stopped the failed cohorts.

The final steady [owned-Quit cohort](desktop-owned-quit.json) used desktop
SHA-256 `3edeb076b991f4ca023b57d8473d520978f76cfaa84414a1e311155b0696504c`
and supervisor SHA-256
`7c757a859092571f39567d15f68521c633fce71f447d1101ad76f058434100c7`.
Cmd-Q during an active browser attempt produced exact runtime cancellation and
runtime exit. The independent harness then observed both desktop and supervisor
exited, supervisor status 0, and no remaining fixture PIDs. It did not signal the
supervisor to produce this successful result. This is the `owned_quit` entry in
`native-lifecycle.json`.

## Evidence limits

The `.ax.txt` captures use the tool's delta mode; several contain only a no-change
notice or focus delta. They are not complete standalone accessibility snapshots.
The table above records the actual returned observations and screenshots; it does
not establish screen-reader speech. No hands-on user trial, real authentication,
native provider compatibility, or escaped-descendant qualification was performed.
The initial `desktop.json` host lacked persistent stdin and exited before manual
checks; it is retained as an unperformed attempt, not a pass. Dia's ambiguous
bundle lookup was resolved using its full application path. Disposable browser
tabs were closed, and control returned to Christopher before headless final gates.
