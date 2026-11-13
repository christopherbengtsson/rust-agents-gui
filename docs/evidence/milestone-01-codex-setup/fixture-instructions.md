# 06b private-state fixture instructions

These are reproducible **qualification instructions**, not a claim that the actual
desktop/process cohort ran in 06b. 06d owns that cohort. The adapter supplies synthetic
stdio outcomes through the production runtime owner, storage, supervisor, transport,
Tauri and TypeScript validation path. It starts no native Codex process.

From the implementation checkout on `feature/m01-codex-setup`, in a fresh terminal:

```sh
fixture_state=$(mktemp -d /tmp/agent-workspace-codex-06b.XXXXXX)
chmod 700 "$fixture_state"
AGENT_WORKSPACE_STATE_DIR="$fixture_state" \
AGENT_WORKSPACE_CODEX_FIXTURE=success corepack pnpm dev
```

Keep the printed private state path and supervisor source/provenance. Open Settings.
Only an observation should occur; no check result is fabricated. Explicitly choose
Check setup. Expect discovery/version/metadata/cleanup progress and “Setup check
passed”, version 0.167.2, synthetic ChatGPT account/OpenAI provider evidence and
confirmed synthetic cleanup. This proves neither native compatibility nor billing.

Edit the executable to `/fixture/custom-codex` and Save. Save must invalidate the
diagnostic and start no fixture/native process. Type another path while Save is
pending: the newer draft must remain, with Check disabled until it matches saved
configuration. Leave Settings and return: only the unsaved draft is discarded.
Reset saved path persists automatic discovery. Restart the desktop against the same
supervisor/state to observe saved preference and action status without replay.

For each other scenario use a **fresh private directory and freshly started owner**;
the fixture mode is captured by that supervisor at startup. Replace `success` with:

| Value           | Expected result                                                         |
| --------------- | ----------------------------------------------------------------------- |
| `missing`       | Missing executable and install/reset guidance                           |
| `version`       | Incompatible version, observed 0.0.0 and supported 0.167.2 guidance     |
| `auth`          | Existing authentication unavailable; external official sign-in guidance |
| `configuration` | Configuration-read failure                                              |
| `timeout`       | Controlled synthetic deadline, confirmed cleanup                        |
| `malformed`     | Invalid protocol frame                                                  |
| `oversized`     | Provider frame budget exceeded                                          |
| `exit`          | Process exit failure                                                    |
| `cleanup`       | Unconfirmed cleanup, retained fence and blocked Save/Check              |

The fixture intentionally replaces discovery/process outcomes, so arbitrary saved
paths need not exist. Native discovery precedence and command construction retain
controlled unit coverage. Unknown scenario names reject; they never fall back to
native execution. A separate private supervisor instance is required to change mode.

The desktop launcher does not own supervisor lifetime. Closing the desktop alone
can leave this disposable supervisor running. Record its actual PID/command/private
state at launch and stop that verified fixture owner explicitly after qualification;
do not signal a PID recovered from an unverified saved record. Do not remove state
while its owner is running. Preserve failed observations and captured evidence.

Desktop keyboard/focus/layout/CSP, actual SQLite migration/durability, kernel leases,
ownership exclusion and descendant termination remain separate real-process checks.
No native provider submission, login/logout or runtime configuration change is
authorized by these instructions.
