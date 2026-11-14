# F08 failed attempts and unperformed checks

No failed attempt was removed. Exact commands, status and elapsed wall time are
in [checks.jsonl](checks.jsonl); each named command has a same-named `.log`.
Expected red regressions are failures of the earlier implementation, not evidence
that the final gate failed. Diagnostic failures below are not product defects.

| Attempt                                               | Failure / interpretation                                                                                                  | Resolution or limit                                                                                                |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `socket-current`                                      | Staged sidecar SIGKILL at startup although its bytes matched the Cargo executable                                         | Fresh-inode publication; `sidecar-execution-fixed.json`, final 21-case socket cohort passes                        |
| `socket-reported-executable`                          | Physical subscription handlers from earlier probes still occupied admission; immediate peer closure could report ENOTCONN | Explicit heartbeat-drain precondition; expected close errors handled by errno only                                 |
| `socket-drained-precondition`                         | Rejected ninth peer could report ENOTCONN before send, not only BrokenPipe                                                | Exact expected rejected-peer errno handling; no unexpected bytes/errors accepted                                   |
| `staging-red`                                         | Existing direct inode overwrite violated new publication assertion                                                        | Eight substituted staging cases green; original provenance assertions retained                                     |
| `socket-expanded`                                     | Qualification moved source before its initial workspace was ready, producing a real creation failure                      | Move-source fault reordered after ready inspection; `socket-expanded-settled` and final native cohort pass         |
| `reload-red`                                          | New replacement regression could not compile before API existed                                                           | Implemented owning Rust replacement                                                                                |
| `reload-green`                                        | Test `unwrap` required a Debug implementation for a controlled channel payload                                            | Assert send success without changing product types; corrected focused cases and final gate pass                    |
| `package-reload-fixed`                                | Read-only generated check rejected stale source manifest                                                                  | Explicit `generated:write`; only manifest source hash changed; subsequent drift check/package pass                 |
| `hidden-collections-red`                              | Two new routed scenarios proved hidden Settings list reads                                                                | Visible interest gating; three final routing regressions plus actual zero-read/expiry proof                        |
| `verify-visibility-fixed`, `package-visibility-fixed` | New pending registration fixture omitted required `initial_workspace_id`                                                  | Fixed precise fixture; `verify-visibility-complete` and separate package pass                                      |
| `current-final-native-resources`                      | Diagnostic incorrectly expected a `--state-dir` option rather than positional state argument                              | Verified exact full native command; corrected five-sample RSS runs pass                                            |
| `fixture-cleanup`                                     | Diagnostic assumed action ID was a top-level SQL column                                                                   | Inspected private schema; action uses `id`, workspace identity is serialized record data; no process was signalled |
| `fixture-cleanup-schema-corrected`                    | Python quoting error in corrected read-only SQL                                                                           | Corrected quoted query; `fixture-cleanup-final` audited exact action/workspace and all five owned processes exited |

Additional desktop/setup attempts retained in AX/JSON evidence:

- Initial Computer Use denial: “Computer Use was not approved to use Agent
  Workspace.” User enabled access; actual qualification subsequently ran.
- Cmd-Option-I hit the production ACL's internal-toggle denial on the inspector
  build. Used its explicit Inspect Element context menu; did not weaken the ACL.
- Inspector focus can be in the native window or main document after navigation,
  despite AX clicking the empty Console prompt. Early diagnostic commands/one cold
  Settings Escape did not execute in the intended surface. Focus was verified
  explicitly; the final production focus checks use no inspector. Capture 72b
  supersedes 72 for the absent-return Escape claim.
- A long Console export exceeded AX's approximately 99-item history; strict chunk
  offsets detected truncation. Two later oversized exports exceeded CUA execution
  timeouts and reset its helper session. Partial raw chunks remain; bounded compact
  exports, contiguous offsets and JSON length checks supply the reported results.
- First physical-capacity SIGSTOP guard resumed before the probe started; its
  draining snapshot was already empty. It establishes no physical retention.
  `physical-bounds-database.json` supplies the later real blocked-operation proof.
- Minimize did not establish `document.visibilityState=hidden`; that attempt is
  excluded. Hidden-interest evidence uses actual Settings and unmounted inspection.
- Early scroll probes captured scroll 0. One precise diagnostic guessed
  `settings-trigger` rather than the source's `settings-entry`, so its click threw.
  `precise-scroll-restoration.json` is the final nonzero 220-pixel proof.
- Initial baseline matched-catalog reload restored the default Workspace tab,
  without a selected preview. Its first slow window is retained in
  `baseline-final-slow.json` but excluded. The reported comparison explicitly
  selects large.txt and uses `baseline-final-selected-slow.json`.
- The first final startup-export loop lost Console focus after reload; its records
  are excluded. Explicit focus checking and warm-up restart produced the final
  six samples. Expected old-document Tauri callback warnings can occur while
  asynchronous native operations finish across reload; the current feed is live.
- The first one-shot final-release launcher did not retain its child execution
  session, and the process was no longer running before CUA could select it.
  `launch-final-release.py` retains/waits its child; final production smoke uses
  that recorded PID and exact binary.
- A scratch Python script-construction command had an indentation error before
  execution, so no file or application effect occurred. Corrected generation ran.

## Unperformed or excluded qualification

| Check                                                       | Result and boundary                                                                                                                           |
| ----------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| Cold native process launch → IPC-ready latency              | **not-run**; warm document load/FCP and real post-reload IPC are measured separately                                                          |
| Total WebView/process-tree RSS and total JS heap            | **not-run**; native desktop+supervisor RSS and exact Query body retention are measured; inspector snapshots themselves retain diagnostic data |
| Hosted CI execution                                         | **not-run**; configured local verification, hooks, production builds and a fresh private checkout/target passed                               |
| Other OSes and assistive speech output                      | **not-run**; this cohort uses macOS native AX semantics, keyboard/focus and visible layout at two supported sizes                             |
| Native external URL scheme cold/warm delivery               | **not-run**, F09; internal routes/reloads/history qualified here                                                                              |
| Full product-owned Quit and provider descendant termination | **not-run**, separately scoped milestone; explicit private fixture termination is not that proof                                              |
| Native providers, chat/history, migrations                  | **not-run**, outside F08 allowance; zero provider submissions, usage or credential changes                                                    |

The last four rows remain milestone/successor acceptance, not inferred passes.
Material bundle/FCP/navigation/fast-IPC increases are returned in the main report;
no universal performance or memory improvement is claimed.
