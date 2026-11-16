# Readiness review corrections

2026-11-15 · Scoped follow-up to [the first UI checkpoint](01-ui.md). The user
authorized correcting findings that belong to ticket 01 and its approved UI
refinement, while leaving browser/CLI/combined-journey work in tickets 02–04.
The [shared clarification](../../../.scratch/codex-connection/spec.md#readiness-correction-clarification--2026-11-15)
records the boundary. Earlier evidence remains historical.

## Corrections and scope

- Visible readiness interest remains bound to its retained supervisor scope through
  transport detachment. An undispatched force flag survives unavailable baseline
  admission; navigation still releases interest. Already admitted reads retain
  physical identity. Same-scope reconnect does not replay verification.
- Entry intent and attachment markers use exact owner/target identity. A replacement
  scope receives its own read-only verification and cannot inherit another scope's
  attached/current presentation. Stable target IDs determine effect lifetime.
- The hook distinguishes current fetch activity from a failed retained Checking
  phase. Errors and disconnects expose recovery. Actual retries show Checking,
  retain the prior failure explanation and withhold another Check again until the
  current read settles. Cleanup uncertainty continues to fence replacement.
- Providers consumes the existing observation-feed availability. Stopped delivery
  makes retained setup unavailable/stale, with guidance to the existing workspace
  supervisor recovery, instead of an ineffective recheck or indefinite Checking.
  This adds a presentation input to `useProviderSetup`, not a feed lifecycle owner.

Changes are confined to the readiness hook, its existing route/block composition,
controlled tests and documentation. Rust, native IPC, generated contracts, provider
adapters/fixtures, credentials and admission/retry authority are unchanged. No
login-attempt state, browser sign-in, terminal, new cache/scheduler or successor
implementation was added. Healthy setup still has no Refresh action.

## Red/green verification and review

Used the existing hook/feature and presentation seams with controlled IPC outcomes.
Repository verification policy overrides generic skill requests for integration
tests or another seam-approval checkpoint. The two original causes were already
minimized and independently traced; regressions were promoted before corrections.

Retained red cases: [baseline interruption](01-ui-fixes/entry-red.log),
[recovery precedence](01-ui-fixes/recovery-red.log),
[replacement owner](01-ui-fixes/replacement-red.log),
[retry exclusion](01-ui-fixes/retry-pending-red.log),
[retry feedback](01-ui-fixes/retry-feedback-red.log), and
[stopped delivery](01-ui-fixes/feed-red.log).
Additional regressions cover view release before admission, a real hook-to-block
read failure and successful recovery, cleanup fencing during read errors, and no
entry replay when delivery availability changes.

[Final focused checks](01-ui-fixes/focused-final.log) passed **66** provider,
presentation and routed tests; `tsc --noEmit` also passed. An intermediate
[feed-green attempt](01-ui-fixes/feed-green.log) failed because recovery data was
placed in the label branch; [corrected focused output](01-ui-fixes/feed-green-final.log)
passed. Failed output is preserved rather than counted as qualification.

Independent [Standards and Spec re-reviews](01-ui-fixes/review.md) ended with
**0 unresolved findings per axis**, including replacement-scope, retry-progress
and stopped-delivery follow-ups. Reviewers inspected read-only and ran no gates;
root-owned verification remains separate. No new `any`, suppression, assertion
escape, declaration shim, relaxed checking or debug instrumentation was added.

## Final automated gates

Source is baseline **30cf4d1a8d00181f65549a4a59dc5c5602e05f5e** plus the cumulative
session worktree. [Final source hashes](01-ui-fixes/source-hashes.json) match the
packaged source; [environment](01-ui-fixes/environment.json) and the preceding
[cohort](01-ui/environment.json) retain the Apple M4 Pro/Mac16,8, macOS 26.2,
Node 26.13.1, pnpm 12.18.0 and Rust 1.99.1 setup. User staging and the unrelated
`docs/IDEAS.md` edit were preserved.

[Final commands, status and wall times](01-ui-fixes/gates.json):

| Gate                         | Result                                      | Wall time |
| ---------------------------- | ------------------------------------------- | --------: |
| `corepack pnpm format:check` | Passed                                      |   0.254 s |
| `corepack pnpm lint`         | Passed                                      |   0.101 s |
| `corepack pnpm check`        | Generated drift/types/Clippy passed         |   2.593 s |
| `corepack pnpm test:measure` | 174 Rust + 51 tooling + 246 frontend passed |  82.273 s |
| `corepack pnpm build`        | Packaged app passed, 17.33 MiB              |  44.624 s |

Raw logs: [format](01-ui-fixes/format.log), [lint](01-ui-fixes/lint.log),
[check](01-ui-fixes/check.log), [units](01-ui-fixes/units.log),
[build](01-ui-fixes/build.log). Existing ts-rs and Vite size notices remained.
The before-feed gate/source logs remain separately named checkpoints.

Existing private worktree target, warm caches, test profile, one Rust harness
thread and one frontend worker. One warm-up plus five executions: Rust median
**0.408676 s** (0.406270–0.413146), tooling **0.901268 s**
(0.889875–0.911136), frontend **12.359681 s** (12.322206–12.455037).
Unit preparation/compilation was separately **0.112729 s**. Fresh-target
compilation and product responsiveness were not measured.

## Actual fixture-desktop outage and recovery

Used the production packaged Tauri → supervisor/core → synthetic readiness peer,
private state and the existing `fixtures/codex-readiness/qualify.py` host. No frontend
state injection or browser mock supplied desktop results. Reproduce the host:

```sh
python3 fixtures/codex-readiness/qualify.py target/release/workspace-supervisor docs/evidence/codex-connection/01-ui-fixes/desktop-final.json 'target/release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop'
```

Open Settings → Providers, switch the host to `scenario deadline`, and reenter
Codex. Pause only the exact host-owned supervisor child with SIGSTOP while the UI
has retained Checking; reenter Providers and allow the production IPC/feed deadline.
Switch the host to `scenario ready`, resume that exact child with SIGCONT, return
to the workspace, use its existing Refresh status, then reopen Providers. Finish
the host with `stop`. [Signals/owned PID](01-ui-fixes/signals-final.json) and
[host provenance/cleanup](01-ui-fixes/desktop-final.json) retain the actual cohort.

The first [outage attempt](01-ui-fixes/outage-attempt1.json) exposed stopped feed
delivery not reaching Providers: native setup was observed at revision 6 while the
UI retained Checking. [Failed screenshot](01-ui-fixes/stopped-feed-attempt1.jpg)
and [AX](01-ui-fixes/stopped-feed-attempt1.ax.txt) remain. The same attempt had
already demonstrated [visible IPC recovery](01-ui-fixes/transport-error-minimum.jpg)
and [retry progress without duplicate retry](01-ui-fixes/retry-checking-minimum.jpg).
Its [host](01-ui-fixes/desktop-attempt1.json) and signal log are separate from the
corrected final build.

Corrected final cohort passed:

- Healthy setup at [900×680](01-ui-fixes/final-ready-ordinary.jpg) and
  [600×480](01-ui-fixes/final-ready-minimum.jpg), with no Refresh/recheck.
- Actual stopped delivery exposes [Status unavailable and reconnection guidance](01-ui-fixes/final-delivery-unavailable-minimum.jpg),
  [AX](01-ui-fixes/final-delivery-unavailable-minimum.ax.txt), retained stale facts
  and no ineffective Check again.
- Escape returns to the workspace and restores Settings focus. Existing Refresh
  status changes Observation delivery stopped back to Connected. Reopening
  Providers performs verification and restores green Connected, observed in CUA.
- [Final native snapshot](01-ui-fixes/final-native-snapshot.json) confirms phase
  observed/problem null. Its event audit contains only version/app-server commands
  and initialize/initialized/account-read/config-read methods.

[Check summary](01-ui-fixes/desktop-checks.json) records the observed outcome.
JPEG captures are Retina 1800×1360 / 1200×960. Final desktop SHA-256 is
`40bea7d05c10286b0e24c00717b74c41b5f3a7b90a0b4c354993cc1664cafac0`.
The unchanged supervisor remains
`b7df607529e71267a1ac54fe7c17fc0119ca8f55271be4a321741a8256d0fd32`.
Both actual hosts shut down/joined their owned children and reported zero remaining
fixture PIDs; unrelated supervisors were preserved.

Baseline interruption and replacement-owner schedules are controlled-unit evidence,
not actual multi-desktop qualification. Screen-reader speech, a hands-on user trial,
live authentication/native-agent compatibility and full product Quit remain
unperformed or separately scoped. This does not complete tickets 02–04 or the
parent milestone; their existing work and acceptance remain open.
