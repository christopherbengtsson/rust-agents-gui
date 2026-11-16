# Connection 04 — Christopher's existing-authentication trial

Conducted 2026-11-15 at source `612ffef`, using the unchanged production binaries
qualified in `627f1af`. Existing real authentication was detected. Christopher
reported that the journey seemed clear and identified a confusing sign-in button
while already connected. At his direction, record that finding for a separate
session; no product correction was implemented. Normal product Quit completed.

## Allowance and cohort

The user authorized read-only detection of existing real Codex authentication and
configuration through the packaged desktop, with private application state and
preserved runtime credentials. Christopher selected exploration before guidance.
Browser sign-in requires a separately agreed scenario. Sign-out, credential
replacement and agent threads/turns are excluded; 06a and 07 are excluded.

[Cohort](04-trial/cohort.json) records source, hardware/OS, private state and bundle
identity, production executable hashes, protocol, runtime/configuration observations
and disposable repository preparation commands. Both copied executable hashes
match [04's final source/binary record](04/final-source.json). Only copied bundle
identity/display metadata changed to isolate preferences. No rebuild was needed.

The private root is `/private/tmp/aw-codex-trial-9vnkfxax`, with mode `0700` on
the root and `state` directory. The desktop was launched directly with Python
`subprocess.Popen([<bundle>/Contents/MacOS/workspace-desktop], env=env)`, preserving
the inherited environment except removing `AGENT_WORKSPACE_CODEX_FIXTURE`,
`AW_FIXTURE_CONTROL`, `AW_FIXTURE_SCENARIO` and `AGENT_WORKSPACE_SUPERVISOR`, and
setting `AGENT_WORKSPACE_STATE_DIR` to the private state directory. The bundled
supervisor was resolved by the production desktop; no helper sent provider IPC.
Logs remain private. The preparation helper did not read credential files.

Official schema provenance remains Codex 0.169.0, revision
`d32063f96c2dff1dc96af839b3952abb6acfeda4`, as retained in [02a](02a.md).
The admission policy requires stable versions >= 0.169.0 and the per-connection
initialize/gateway capability checks. This trial's actual version is recorded
separately below; synthetic newer-version evidence is not native qualification.

## Facilitator's real-runtime observation

Native accessibility controls: Settings → Providers → expand Technical details.
The packaged desktop first displayed **Checking…**, then **Connected**. The
[sanitized accessibility snapshot](04-trial/01-facilitator-readiness.ax.txt)
records existing ChatGPT authentication, Pro, ChatGPT billing, runtime 0.169.0,
model `gpt-6.1-sol`, model provider Runtime default and no setup error. Codex home
is the existing user-owned `.codex`; the adapter used its neutral setup directory.
Technical details explicitly limits this to user-level configuration, excluding
project overrides. The account email is redacted from the published snapshot.

The inspected readiness path sends initialize/initialized,
`account/gatewayOAuth/read`, `account/read` with `refreshToken:false`, and
`config/read` with `includeLayers:false`. No login/thread/turn action was invoked.
Neither the facilitator nor preparation helper extracted, edited or copied
credential files or requested a billing-route change. Runtime-owned credential
contents were not audited before/after.
This establishes observed readiness for this cohort, not successful agent execution.

## Participant journey and feedback

The task handed to Christopher was to bring the isolated app forward,
return to the workspace, find whether Codex is ready, explain the status and return
again, without clicking Sign in through browser. A subsequent
[native observation](04-trial/02-handoff-observation.ax.txt) shows the workspace
route with no repository selected. It does not reconstruct every intermediate
action or establish that the sidebar's transport Connected means Codex readiness.

Christopher reported: “Seems clear. Only thing is maybe why the sign in buttin is
there even tho I'm already connected”. The general clarity assessment is his
feedback, not a measured completion or accessibility score. His concrete concern
is that the healthy Providers page still prominently offers browser sign-in.
The facilitator explained that it starts a fresh sign-in and proposed hiding
the prompt while readiness is verified, retaining recovery and existing attempts.
Christopher then directed: “We fix it in a separate session, here we should just
record the findings”. The concern remains open; no replacement wording or hiding
rule has been accepted or implemented. A later session should resolve the
presentation against the current connection requirements while preserving
missing-auth recovery and active-attempt progress.

This was an informed exploration: the facilitator disclosed the successful
readiness/account-route result before handing over the task. It was not a blind
discovery study. Exact participant clicks, task duration and screen-reader speech
were not recorded. No general usability claim follows from this one-person trial.

## Observer problems and qualification limits

Saving a full accessibility snapshot renumbered elements. A subsequent stale
element action selected the window's minimize control instead of Technical details.
Further attempted disclosure/Back/Escape/Raise actions did not change the observed
Providers view; a screenshot confirmed that view, and a coordinate Back action
also produced no observed change. These are observer/control failures, not a
reproduced product-navigation defect. Christopher was asked to bring the app
forward and perform the actual journey. The account-containing screenshot was
viewed for diagnosis but was not saved as a repository artifact.

The observer's later Cmd-Q initially returned an inventory still showing the app
running. The launcher subsequently observed desktop exit 0, and an independent
process audit confirmed cleanup; the immediate inventory was not used as proof
of exit or failure.

No application code, contracts, adapters or fixture behavior changed. Passing
unit/static/fixture-desktop evidence from [04](04.md) is reused; those gates were
not repeated. Browser sign-in, forced missing/incompatible setup, screen-reader
speech, API-key authentication, native chat and multi-provider execution have not
been performed. A single participant cannot establish general usability.

## Cleanup, checks and acceptance

[Pre-handoff processes](04-trial/pre-handoff-processes.txt) identifies the trial
desktop and its bundled supervisor. [Cleanup audit](04-trial/cleanup.json) records
product Cmd-Q, the desktop's exit 0 and no remaining private-bundle processes or
matching-runtime process. The supervisor's independent exit code was not captured.
No harness signal/termination was used. Private state, logs and the unused
disposable repository remain under the trial's private root.

This session adds a real-runtime readiness observation and Christopher's limited
hands-on feedback to the prior fixture evidence. It verifies neither browser
authentication nor the complete native A3 journey. Detailed provider/General tab
coverage, reading-context preservation, minimum-window layout and recovery were
not separately repeated with Christopher. No defects beyond the reported
presentation concern were established by his feedback.

No acceptance checkbox changes are necessary: ticket 04's existing fixture scope
remains checked with its historical limits, while this report supplies the formerly
unperformed hands-on trial and real existing-authentication observation. Native
agent execution and whole-milestone qualification remain open. Storage evaluation
06a and chat 07 were not started.

Documentation verification: changed evidence Markdown/JSON formatting, local
links/artifact parsing, redaction checks and `git diff --check` passed. Application
units, static gates and builds were not rerun because there is no product change.
