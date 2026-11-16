# Ticket 03 implementation plan

**Archived — 2026-11-15:** The user deferred ticket 03 after reviewing the actual
authentication need. This plan is retained research, not an implementation
instruction. The [current scope amendment](spec.md#authentication-scope-amendment--2026-11-15)
keeps existing-auth detection and one runtime-managed browser sign-in action.
CLI, PTY/xterm, the nonce patch, terminal presentation and a read-only command
viewer are outside the current feature. Start the next session with
`$implement .scratch/codex-connection/issues/04-qualify-connection-journey.md`
and its applicable repository skills. The historical plan below establishes no
implementation, dependency installation, protocol change or qualification result.

2026-11-15 · Planning handoff for [03: Embedded official CLI login](issues/03-embedded-cli-login.md).
This document records decisions and read-only research. Product code, dependencies,
tests and packaged terminal compatibility have not been implemented or qualified.
The ticket and [connection spec](spec.md) retain their acceptance and scope authority.

## Start a fresh implementation session

Use `$implement .scratch/codex-connection/issues/03-embedded-cli-login.md
$react-engineering $shadcn $ux-ui $rust-engineering` from this repository.
Work only ticket 03; stop before 04. Follow AGENTS.md's required reading and local
Intent discovery/loading. Use the installed skills, including TDD at the owning
seams below, and the requested final Standards/Spec code review and scoped commit.

Both blockers have checked acceptance and fixture-only evidence:
[02 handoff](../../docs/evidence/codex-connection/02.md) and
[02a handoff](../../docs/evidence/codex-connection/02a.md). Read both; 02a supersedes
02's exact-version policy and protocol number. Current source uses protocol 13,
stable Codex >= 0.169.0 and per-connection required capability checks.

Planning inspected HEAD `f5a6e4c5ae445b0c7fd9ccce2536292caee22455`.
Capture the actual implementation starting revision for review; this hash is
context, not an instruction to reset. The ticket's library/architecture amendments
and `docs/IDEAS.md` edits already existed. They were staged by the time this plan
was saved. Preserve the current tree and index; inspect new changes before staging.

## Decisions and implementation boundaries

The user explicitly selected:

- **Complete bounded replay:** retain the complete transient terminal session up
  to 2 MiB per terminal; exceeding the bound visibly fails the attempt and requests
  owned cleanup. Preserve screen reconstruction rather than retaining a raw tail.
- **Nonce patch:** preserve strict CSP through a small pinned xterm package patch
  assigning the document's style nonce to xterm-created style elements.
- **Bounded verification:** avoid repeated full verification cycles; use focused
  checks, one full gate and targeted corrections. See the verification section.
- **Official sources:** retain exact API/lifecycle guidance so implementation
  follows supported contracts rather than guessed terminal behavior.

The plan defaults to an inline login terminal below readiness, ChatGPT selected
initially and browser sign-in visually emphasized. ChatGPT/CLI selection changes
presentation only. Use the existing controls and configured shadcn registry; add
ToggleGroup for the two-option choice through its documented CLI workflow.
Already-configured users retain readiness/account status before login choices.

Use fixture commands and fresh private application state for qualification. This
plan authorizes no real authentication, credential changes or native agent
submissions. General shells, workspace terminals, run/test controls, broader
observation refactors and new provider implementations remain deferred.

## Build through the existing owners

### Shared login lifecycle and native PTY

`crates/core/src/runtimes/provider_connection/login.rs` currently owns
`admit_login`, `execute_browser`, `persist_attempt`, `finish_login`, cancellation,
reconciliation and shutdown. Its private `BrowserEffects` seam substitutes effects
while retaining real policy. Factor one shared lifecycle as CLI is added:
admission -> durable intent -> method execution -> physical cleanup -> authoritative
verification -> durable fence release. Keep browser reopening and CLI controls in
method-specific attempt resources. A failed cleanup retains both ownership and
the affected scope's fence; unrelated targets remain independently admissible.

Add a small private `portable-pty` module within supervisor-hosted `runtimes`.
It owns PTY creation, reader/writer/master/child resources, bounded control/output
delivery, resize and worker completion behind an internal effect seam. Retain
handles through every setup/error/cancellation exit, including setup finishing
after cancellation. Blocking setup/I/O/waits belong to owned workers whose
physical completion is observed; dropping or aborting an async waiter is insufficient.

`crates/core/src/runtimes/codex/process.rs` contains captured `Launch`, private
neutral-directory setup, version checks, probe and process-group cleanup. Reuse
the captured executable, environment, effective configuration and directory for
preflight, direct `login` launch and verification. Add only terminal-specific
environment values. Run the executable with `login` directly, without a shell.
Use the existing version/capability probe before launching CLI login and the
existing authoritative setup probe after command exit, including exit code zero.
The preflight read may establish unavailable authentication without blocking an
otherwise compatible explicit login.

Adapt the existing owned-group cleanup mechanism for PTY resources. Portable-pty's
Unix spawn establishes a new session; its kill helpers are not group/worker cleanup
evidence and can reap the direct child. Preserve the unreaped child's identity
until descendant cleanup is established, then reap and join all owned workers.
Keep cancellation/status responsive while data delivery is full. Failed or unknown
cleanup remains visible through reconnect and Quit. Retain the existing durable
fence after unclean supervisor loss; a saved PID cannot prove cleanup.

### Contracts, transport and replay

Extend Rust-owned `ProviderRequest`, `ProviderReply` and attempt observations with
login method, terminal identity/status, explicit CLI start, cursor-based terminal
reads, targeted input, resize and presentation dismissal. Exact supervisor owner,
provider/configuration, attempt and terminal identities bind every operation and
late result. Duplicate identity across different methods must not bypass exclusion.

Advance protocol 13 to 14 with matching peers and fresh private state. Regenerate
types, schemas, validators and the current serialization corpus; preserve historical
corpora. Add any newly split contract-defining files to preparation provenance:
`scripts/prepare-generated.mjs` currently lists the parent provider-connection file
but omits `login.rs`. Extend native and frontend contextual validation alongside
generated shapes.

Store a complete ordered replay journal in supervisor memory: initial dimensions,
output bytes and applied resizes. The 2 MiB limit includes event overhead, so resize
events cannot create unbounded retention. Bound message size, input queues, output
pages and renderer writes independently. Existing native frames allow 65,532 bytes
and provider reply validation allows 24 KiB; size terminal pages under those bounds
after JSON encoding instead of enlarging every observation limit.

Reattach by replaying from the beginning with original byte/resize order and only
then fitting the visible container. Suppress generated terminal input during replay.
Raw byte delivery preserves split UTF-8 and escape sequences. Reject stale traffic;
use bounded sequence acknowledgement for input/resize deduplication. Reconcile a
lost acknowledgement without automatically resending uncertain input.

Keep the transcript transient, excluding application history/logs/preferences.
Done, explicit replacement or owner disposal releases presentation data without
discarding unresolved resource handles or fences. Exit status, authentication
evidence and verification-read failure remain separate observable facts.

### Observations and React adapter

Extend `src/lib/provider-observations.ts`, `useProviderLogin`, `useProviderProgress`
and the existing Providers/ProviderSignIn composition. Query remains the sole
observation cache; reuse feed invalidation and the physical scheduler. Terminal
attachment may retain only identities/cursors/resource handles outside Query, not
a second execution-state or transcript cache. Bound cursor-page Query retention
as well as worker/renderer queues.

Pending-login status interest remains above routes. Navigation releases view,
renderer subscriptions, resize observation and display resources without cancelling
login. Returning requests the same terminal's replay. Desktop reconnect retrieves
authoritative status and terminal data without replaying authentication effects.

The small xterm adapter owns open/attachment, focus, keyboard behavior, fit/resize,
bounded writes and disposal. It receives data and explicit callbacks, independent
of Codex authentication policy. Give it a clear accessible terminal label and
screen-reader support; preserve a keyboard route out of the terminal. Escape in the
terminal takes precedence over Settings navigation. Use existing semantic theme
values and retain Settings return/focus/context behavior.

Show starting/running/error/exited presentation plus separate verification feedback.
Provide **Run codex login**, **Cancel sign-in**, **Cancel sign-in and close terminal**
and **Done** after exit. Done dismisses presentation without claiming authentication
success or clearing cleanup uncertainty. Completion starts no agent work and does
not navigate. Compact progress stays discoverable while authoritative work or
cleanup uncertainty remains.

## Official references and selected versions

Research selected the exact versions below for first-use installation. npm metadata
confirmed xterm 6.0.0 and FitAddon 0.11.0; their release-tag manifests agree.
These selections establish API sources, not packaged WebView compatibility.

| Concern | Official source and implementation consequence |
| --- | --- |
| xterm **6.0.0**, FitAddon **0.11.0** | [xterm manifest](https://github.com/xtermjs/xterm.js/blob/6.0.0/package.json), [FitAddon manifest](https://github.com/xtermjs/xterm.js/blob/6.0.0/addons/addon-fit/package.json). Pin exact versions and lockfiles. |
| Attachment and sizing | [Addon guide](https://xtermjs.org/docs/guides/using-addons/), [pinned FitAddon API](https://github.com/xtermjs/xterm.js/blob/6.0.0/addons/addon-fit/typings/addon-fit.d.ts): load the addon, open a measurable container, then fit. Release ResizeObserver and addon/event disposables on unmount. |
| Output flow | [Flow-control guide](https://xtermjs.org/docs/guides/flowcontrol/), [pinned terminal API](https://github.com/xtermjs/xterm.js/blob/6.0.0/typings/xterm.d.ts): `write` is asynchronous; use `write(Uint8Array, callback)` and advance the parsed cursor/request another page only after the callback. Keep at most one page awaiting parsing and cap scrollback at 1,000 lines. |
| Terminal content | [Security guide](https://xtermjs.org/docs/guides/security/): bundle assets locally; keep terminal data and keyboard input out of application logs. Enable only integrations required for login; terminal output does not authorize app/native actions. |
| portable-pty **0.9.0** | [Crate API](https://docs.rs/portable-pty/0.9.0/portable_pty/), [MasterPty](https://docs.rs/portable-pty/0.9.0/portable_pty/trait.MasterPty.html), [Child](https://docs.rs/portable-pty/0.9.0/portable_pty/trait.Child.html): `openpty`, `spawn_command`, one `take_writer`, `try_clone_reader`, `resize` and observed wait/worker completion. `wait` blocks. |
| PTY process cleanup | [Pinned Unix spawn](https://docs.rs/portable-pty/0.9.0/src/portable_pty/unix.rs.html), [pinned kill/wait implementation](https://docs.rs/portable-pty/0.9.0/src/portable_pty/lib.rs.html): `setsid` establishes the session; `kill`/`try_wait` may reap or signal only the direct child. Adapt the existing owning cleanup seam rather than treating helper success as quiescence. |
| Codex login | [Pinned CLI dispatch](https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/cli/src/main.rs), [login implementation](https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/cli/src/login.rs): plain `login` invokes Codex's managed login using runtime configuration. Credentials and Codex's own logs stay runtime-owned. Exit zero is followed by our independent authoritative probe. |
| Existing status contract | [Official pinned app-server README](https://github.com/openai/codex/blob/d32063f96c2dff1dc96af839b3952abb6acfeda4/codex-rs/app-server/README.md) plus [02a evidence](../../docs/evidence/codex-connection/02a.md): preserve initialize/initialized, required gateway capability probing, correlation and existing account/config reads. |
| UI composition | [Base ToggleGroup](https://ui.shadcn.com/docs/components/base/toggle-group), [Button](https://ui.shadcn.com/docs/components/base/button), [Alert](https://ui.shadcn.com/docs/components/base/alert). CLI lookup confirmed Vite, Base UI, base-nova and the `@/` aliases; ToggleGroup was not installed at planning time. |

### Strict CSP patch

[Pinned DomRenderer](https://github.com/xtermjs/xterm.js/blob/6.0.0/src/browser/renderer/dom/DomRenderer.ts)
creates theme/dimension style elements; the
[shared DOM helper](https://github.com/xtermjs/xterm.js/blob/6.0.0/src/vs/base/browser/dom.ts)
creates scrollbar styles. Set each nonce before insertion, patching the shipped
ESM/CommonJS entry points actually used by the build. Use
[pnpm patch/patch-commit](https://pnpm.io/cli/patch) with an exact-version patch and
retain upstream provenance. Avoid global DOM monkey-patching or private terminal
instance access.

The [Tauri CSP guide](https://v2.tauri.app/security/csp/) describes asset nonce
injection. Read-only inspection also confirmed installed `tauri-2.12.5`'s
`src/manager/mod.rs` (`set_csp`/`replace_csp_nonce`) and `tauri-utils-2.10.2`'s
`src/html.rs`/`html2.rs`/`assets.rs`: bundled style nodes receive a nonce token,
replaced at asset serving and added to style-src. Use a dedicated bundled nonce
anchor and copy its `.nonce` into xterm-generated styles. Verify the anchor survives
Vite's production HTML transformation; the installed exported style token is
`__TAURI_STYLE_NONCE__`. Keep all existing script/network restrictions and avoid
an inline-style CSP allowance. This mechanism requires actual packaged-WebView
proof; source inspection alone does not establish successful rendering or resizing.

## Verification, review and completion

The ticket and repository verification policy already establish the owning-module
test seams. Use real admission/recovery/parsers/command construction with controlled
storage, execution, PTY, process and time outcomes. Repository policy permits
private owning-module test surfaces; upstream TDD examples do not prohibit them
or require another approval checkpoint. Add one failing behavior test per slice,
then implement it and run that focused selection.

Required unit coverage groups:

- Shared browser/CLI lifecycle: intent ordering, duplicate/competing actions,
  same-scope exclusion, stale owner/configuration/attempt traffic and independent
  synthetic targets; preserve existing browser behavior while factoring ownership.
- PTY failure/cancellation: late setup, launch/I/O/resize failures, full queues,
  output-limit exhaustion, cancelled/unknown cleanup, worker completion, shutdown
  and replacement fencing. Keep real processes/filesystem/storage out of units.
- Completion: exit zero without authentication, failed login with existing account
  evidence, verification-read failure, cleanup-before-fence release and Done that
  does not establish readiness.
- Presentation: explicit launch, retained progress, navigation/replay, split UTF-8
  and ANSI sequences, resize ordering, replay without generated input, bounded
  renderer delivery, stale callbacks/disposal and keyboard/focus behavior.

Run focused tests and typechecking during development. Run **one**
`corepack pnpm verify` attempt for generated/static/type/Clippy/full unit gates.
If it stops early, retain the failure and run corrected subsets plus previously
unperformed gates individually. Full-suite success inside verify satisfies
implement's final full-suite requirement. The user's instruction supersedes
repeated warm measurement cohorts or a second full gate for reassurance.

Run one production package build after corrections, then one bounded fixture
process cohort and one focused actual packaged desktop pass. Extend the existing
`fixtures/codex-readiness` command seam with synthetic CLI behavior and provenance.
Cover input/output/resize, strict CSP, keyboard/focus, ordinary/minimum-window
layout, navigation/reconstruction, exit/recheck, desktop reconnect, cancellation
and owned Quit; audit actual fixture processes/workers. Test source and matching
sidecar provenance must describe the packaged binary used. Units and DOM mocks
cannot replace those desktop/OS claims. After a concrete fix, repeat only affected
checks/scenarios; rebuild only when the fix changes the package.

Apply `$code-review` at the end with independent Standards and Spec reviewers
against the captured implementation baseline. Supply repository policies, the
ticket/spec and this plan; reviewers should reuse recorded verification results
rather than rerunning full gates. Correct findings with focused regression checks.

Record dated evidence for 04 with exact versions/source/binary provenance, commands,
counts/durations, failures, unit/fixture-desktop distinctions and unperformed
native/user-trial/accessibility limits. Update component documentation for new
contracts and ticket acceptance only where demonstrated. Commit scoped changes on
the current branch, preserving unrelated work, and stop after ticket 03.
