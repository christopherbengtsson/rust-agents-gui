# Conductor provider connection reference

2026-11-15 · Documentation research for Milestone 01 ticket 06. This records
shipped-source findings and the limits of that evidence. The
[connection journey](../../.scratch/codex-connection/spec.md#connection-journey)
and [ownership/protocol plan](../../.scratch/codex-connection/spec.md#ownership-and-protocol)
own our future implementation requirements; this note does not claim they are
implemented or qualified.

## Sources and method

Read-only inspection used installed `/Applications/Conductor.app`, whose
`Info.plist` reports version/build **0.93.0** and identifier `com.conductor.app`.
The executable and both decoded bundles were rehashed for this note; every byte
count and SHA-256 matched the retained
[source manifest](../evidence/frontend-navigation/conductor-sources.json).
The earlier [frontend architecture research](conductor-frontend-architecture.md)
records the extraction context. No Conductor application files or user data were
changed, and no credentials, provider history or databases were read.

The table below locates evidence in the manifest's decoded
`renderApp-Ec9d1ssb.js` (**R**) and `main-B00nPG1d.js` (**M**). Every offset is a
**zero-based UTF-8 byte offset**, measured against those exact files, not a
JavaScript string index or an offset copied from the earlier note. Read a byte
slice and decode it as UTF-8 to reproduce a location. Small excerpts identify
relevant branches without retaining complete bundles in the repository.
[Bundle identity and paths](../evidence/frontend-navigation/conductor-sources.json).

| Evidence | Location and concise finding |
| --- | --- |
| Local provider navigation | **R 11,381,923**, `eGa`: settings route search supplies `providerTab`; renders Claude Code, Codex, Cursor and additional agent tabs. The absent/invalid-tab fallback is `"claude"`. This is a source-derived reference, not our chosen default. |
| Codex connection choices | **R 11,303,780**, `EVa`, labels CLI / API key / ChatGPT, with ChatGPT recommended; **R 11,371,046**, `qWa`, orders `chatgpt`, `cli`, `apikey`; **R 11,376,257**, `JWa`, selects the associated connection UI and settings behavior. |
| Status and refresh | **R 11,308,036**, `Iar`: checking, connected/not-connected presentation, account information, refresh and optional detail. **R 11,376,257**, `JWa`, supplies mode-specific unauthenticated messages and can retain last-known account information with a verification error. |
| CLI helper | **R 11,314,814**, `MZe`: idle → running presentation, obtains a path prefix, builds the command and renders `xZi`. **R 11,376,257**, `JWa`, passes `loginCommand:"login"`, `loginCommandLabel:"codex login"`, `cliName:"codex"`, `runFromBundledBin:!0`. The button combines `"Run "` with that label, yielding “Run codex login.” |
| Embedded interactive terminal | **R 9,644,808**, `xZi`: creates a terminal session with `kind:"interactive"`, `mode:"interactive_bootstrap"`, user-shell environment and `persistence:"none"`; attaches input/display and resize behavior through `u5e` and the terminal session service. |
| Completion and continued shell | **R 9,643,980**, `U0a`, appends a command-completion marker when needed. **R 9,644,508**, `q0a`, says the shell remains usable after the command finishes. `MZe` refreshes connection state when Done is selected (**R 11,314,814**). Completion presentation alone is not an independent authentication check. |
| Default unmount behavior | **R 9,644,808**, `xZi`, defaults `keepAliveOnUnmount` to false. Persistence requires both that flag and an explicit terminal ID; effect cleanup otherwise calls `Gje`. **R 9,644,436**, `Gje`, requests session destruction with transcript clearing and disposes renderer state. `MZe` supplies neither keep-alive nor a terminal ID. This establishes the cleanup request in code, not observed OS process termination. |
| Terminal renderer family | **R 4,563,758** contains the `xterm.js:` log prefix. **R 4,726,748**, `oso`, constructs the terminal, loads Unicode/fit/link addons and opens a DOM container; **R 4,733,210**, `u5e`, connects that renderer lifetime to its consumer. This supports identifying the xterm family. Exact npm package names/versions are unproven by these minified bundles. |

All table locations refer to the
[hash-verified source manifest](../evidence/frontend-navigation/conductor-sources.json),
not source-map-recovered original files. Both bundles contain source-map references
(**M 3,601,758**, **R 12,556,934**), but the decoded research directory contains no
`.map` files. Original component names, source paths and dependency versions remain
unavailable from this inspection.

## Hosted authentication is a separate reference

Conductor's ChatGPT choice is not evidence that its browser button simply runs
the local Codex CLI login. `kar` opens an authorization URL obtained through
Conductor cloud settings and polls hosted sign-in status, with explicit cancel
handling (**R 11,297,235**; `pVa`/`gVa`, **R 11,293,296** and **11,293,770**).
The configured-state selector `XQi` reads a `codex_chatgpt_oauth` provider record
(**R 9,473,492**). These are source observations about Conductor's path, not a
qualified end-to-end account flow.
[Source identity](../evidence/frontend-navigation/conductor-sources.json).

A separate device-code path starts and finishes Codex cloud authentication;
`_Va` requires an `authCache` result (**R 11,295,041**). `Q1i` serializes
that cache into a credential mutation named `codex_chatgpt_auth`
(**R 11,299,535**). The shipped schema `Ggt` describes the cache as exported
files with relative paths and base64 content (**M 1,668,554**). Thus shipped
hosted-connection code can collect and submit an authentication cache for storage.
The inspected code does not establish its production backend storage, security
properties, or any provider agreement.
[Source identity](../evidence/frontend-navigation/conductor-sources.json).

Our [runtime-owned authentication contract](../MVP.md#starting-architecture)
does not adopt that cache collection or hosted credential path. The application
must not extract, copy or store runtime tokens/authentication caches. Connection
presentation can follow the reference while credential ownership stays with Codex.

## Current official Codex capabilities

The current official app-server documentation distinguishes managed ChatGPT
authentication, where Codex owns OAuth, persistence and refresh, from externally
managed tokens. For the managed browser flow, `account/login/start` with
`type:"chatgpt"` returns a `loginId` and `authUrl`; the client opens that URL and
app-server hosts the local callback. `account/login/completed` reports completion,
and `account/updated` reports the resulting authentication mode and available plan
metadata. These establish a documented runtime-managed browser path without
host-owned token handling.
[Official auth endpoints](https://learn.chatgpt.com/docs/app-server#auth-endpoints)
(retrieved 2026-11-15).

The same source documents cancellation of a pending managed login by `loginId`
through `account/login/cancel`, and `account/read` for current account state with
an optional managed-token refresh. It documents API-key accounts as well as
ChatGPT accounts; unavailable account/plan details must not be invented. An
authoritative account read after a login attempt is our planning choice, rather
than inferring readiness from terminal output or process exit alone.
[Official auth endpoints](https://learn.chatgpt.com/docs/app-server#auth-endpoints);
[our ownership/protocol plan](../../.scratch/codex-connection/spec.md#ownership-and-protocol).

Codex's credential-storage configuration supports file, OS credential-store,
automatic and ephemeral modes. File storage uses `auth.json` beneath `CODEX_HOME`;
the runtime configuration controls the choice. This is a reason to leave
credential access and persistence with the official runtime rather than assuming
one credential filename is the source of truth.
[Official credential storage](https://learn.chatgpt.com/docs/auth#credential-storage)
(retrieved 2026-11-15).

The auth-endpoints page also distinguishes the continuation allowance for local
or open-source applications from commercial/hosted use and recommends Sign in
with ChatGPT. This research does not resolve distribution policy beyond the
existing [MVP evidence and limits](../MVP.md#evidence-keep-four-categories-separate).
[Official auth guidance](https://learn.chatgpt.com/docs/app-server#auth-endpoints).

These are current documentation capabilities, **not qualification of our installed
supported runtime/schema cohort**. This note selects no runtime version and does
not prove that a selected installed build accepts these exact methods or fields.
The adapter still needs supported-version official schema and separately recorded
qualification under [Milestone 01](../MILESTONE-01.md#verification-order).

## Agreed adaptation for our connection journey

**Historical proposal:** the [2026-11-15 scope amendment](../../.scratch/codex-connection/spec.md#authentication-scope-amendment--2026-11-15)
supersedes the CLI/terminal requirements below. Preserve them as the original
research-to-design handoff; current requirements live in the connection spec.

The decisions below belong to the future
[connection feature journey](../../.scratch/codex-connection/spec.md#connection-journey),
not claims about Conductor or completed implementation:

- Use shared Providers settings with Codex selected by default. Reuse existing
  runtime authentication/configuration first; an already configured API-key route
  remains recognizable without adding an API-key setup UI in Milestone 01.
- Offer runtime-managed ChatGPT browser sign-in as the primary setup action and
  a CLI login helper. Keep Claude's simple actions visibly disabled, without
  probing or starting Claude processes. Use installed Codex; add no installer.
- Run `codex login` directly in a PTY. End the helper when that command ends,
  without retaining a post-login shell. Keep renderer and supervisor process
  seams reusable for a future development terminal; a full terminal is later scope.
- Preserve an active login across navigation and provide a compact link back to
  it. Navigation does not cancel login; expose explicit Cancel. This intentionally
  differs from the inspected helper's default unmount destruction.
- Recheck authoritative authentication after completion and present its outcome;
  command exit or a success-looking message alone does not establish connection.
  The supervisor-hosted core owns process/login lifetime and reconciliation, with
  Tauri and React presenting observations.

The [MVP process architecture](../MVP.md#starting-architecture) and
[ticket ownership/protocol plan](../../.scratch/codex-connection/spec.md#ownership-and-protocol)
govern these differences. Reusing the reference interaction does not move
credentials or execution ownership into the frontend.

## Verification limits

This work performed source inspection, hash/byte-count verification and official
documentation retrieval only. No successful sign-in, provider submission, native
agent run, runtime installation, credential mutation or application implementation
was performed. No visual, accessibility, responsiveness or user-trial result is
claimed. Static shipped-code branches establish available code paths, not their
reachability for every account or successful execution. Source-map and dependency
version limits remain as recorded above; native-provider and desktop qualification
remain separate future work under [Milestone 01](../MILESTONE-01.md#verification-order).

## Authentication scope correction — 2026-11-15

Further read-only review of the same pinned Codex source found that normal
[`codex login` dispatch](https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/cli/src/main.rs)
invokes the [browser login flow](https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/cli/src/login.rs):
start a callback server, print the URL and await browser completion. The inspected
normal path requires no terminal input; secret-on-stdin options are separate.
Managed remote-configuration authorization is a separate branch, so this is not
a universal equivalence claim for every configuration or newer release. No native
login was run. The ordinary ChatGPT path also clears/revokes existing authentication
before replacement sign-in; a future CLI helper must not promise cancellation
restores the previous credentials.

Conductor's [official announcement](https://www.conductor.build/blog/sign-in-with-chatgpt)
describes bringing ChatGPT identity and subscription access to a Conductor account
without separately running `codex login`. Its
[provider guide](https://www.conductor.build/docs/guides/providers#codex) separately
describes CLI auth for an existing login. This explains why Conductor's choices
need not map to distinct user capabilities in our local-only, runtime-owned design;
it does not establish Conductor's exact backend credential storage or refresh.

The [shadcn Terminal pattern](https://ui.shadcn.com/docs/ai-elements/base/terminal)
is a read-only Trace + CodeBlock composition; Stack Trace displays error frames.
A command viewer could avoid terminal emulation if a future CLI use case is
selected. The user instead approved deferring the separate CLI helper and both
terminal/output-viewer variants. The current connection scope retains one browser
sign-in action and detection of existing authentication. Earlier terminal design
and dependency research remain history, not an active implementation requirement.
