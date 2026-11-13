# Ticket 04 — isolated workspaces and recoverable retries

2026-11-13. Ticket 04 complete; whole Milestone 01 remains incomplete. No provider
submissions were performed or authorized.

## Implemented behavior and source

Adding a new repository requests its first workspace. Repository + immediately
creates another. Generated city/UUID names, branches and private-state folders
retain their identity across recovery. The user explicitly deferred the composer
and base picker. Use usable local origin/HEAD, otherwise exactly one usable local
main/master; unavailable or ambiguous defaults fail visibly. No fetch, project
setup or Codex launch occurs. See [research](../research/ticket-04-conductor-workspace-flow.md).

Registration remains a separate operation without Git/project-file mutation.
Successful new registration atomically saves its initial-creation continuation.
Duplicate registration and reopening do not infer additional creation. Rust
`workspaces` owns admission, pinning, staged effects and explicit reconciliation;
SQLite owns committed metadata. Tauri forwards protocol-3 requests. Schema 2
preserves schema-1 repository/action identities.

Intent and pinned base precede mutation. Branch creation uses non-replacing Git
reference creation with an action-specific ownership ref. A private wrapper owns
its marker, execution lease and checkout. Existing conflicts remain intact.
Git effects inherit the kernel lease, so a replacement supervisor can observe
surviving work without signaling saved PIDs. Reconciliation inspects artifacts
before completing or continuing the same operation. Normal capacity admission
cannot strand existing workspaces' recovery.

One React observation owner retains workspace selection, action identities and
bounded pages. View changes/reconnect retrieve status. Delayed observations cannot
retarget selection or clear a newer request. Typed definite admission failures
clear local pending state while retaining their error; storage/transport uncertainty
retains the action ID. Existing shadcn controls suffice.

Implementation starts at `5c16386f3d77c8665d9af342c17bcfe06cf13550`; implementation
commit `28f21da`, review fixes `230bf75`. The final layout-only change reserves
space for both repository actions. Exact source hashes, environment, staged binary
provenance, and qualification bundle hashes are retained beside this report.
[Source/environment](milestone-01-workspaces/environment-source.json),
[build provenance](milestone-01-workspaces/build-provenance.json).

## Deterministic verification

macOS 26.2 / Apple M4 Pro, 48 GiB, arm64. Rust 1.99.1, Node 26.13.1,
pnpm 12.18.0, Git 2.50.0. Used the worktree-default Cargo target and warm caches,
serial Rust/tooling execution and one Vitest worker. Unit execution uses no real
Git/worktrees, SQLite, supervisor/provider processes or production waits.

Focused red/green checks covered automatic-base policy, intent-before-effect,
failed writes blocking subsequent mutation, branch/worktree lost replies, pinned
recovery, restored uncertainty, action replay/conflicts, atomic first creation,
readable pending status during effects, byte/page boundaries, recovery capacity,
and UI identity/rejection/navigation races. Existing bounded Git process tests
cover timeout/output-limit and unconfirmed cleanup using controlled outcomes.

| Gate                     | Result                                                                |
| ------------------------ | --------------------------------------------------------------------- |
| Final complete gate      | Pass: 68 Rust, 19 tooling, 39 UI units; exit 0; 5.26 s wall           |
| Final warm measurements  | Pass: one warm-up plus five warm executions per harness; 22.97 s wall |
| Final production package | Pass, default production configuration; exit 0; 42.60 s wall          |
| Native-provider cohort   | Not run; not authorized by this ticket                                |

Commands:

```sh
/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm verify
/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm test:measure
/usr/bin/time -p fnm exec --using 26.13.1 corepack pnpm build
```

The final complete gate separately measured unit compilation at 0.126603 s and
execution at Rust 0.134531 s, tooling 0.902023 s, UI 2.970936 s.
Final measurement compilation was 0.085260 s. Warm execution medians/ranges:

| Harness           | Median     | Range               |
| ----------------- | ---------- | ------------------- |
| Rust, 68 units    | 0.131466 s | 0.128597–0.137346 s |
| Tooling, 19 units | 0.786527 s | 0.775598–0.799995 s |
| UI, 39 units      | 2.910418 s | 2.786165–2.954423 s |

[Final gate](milestone-01-workspaces/verify-final.log),
[measurements](milestone-01-workspaces/unit-measurements-final.log),
[production package](milestone-01-workspaces/production-packaging-layout.log).
Fresh-private-target compilation was not measured. The earlier passing gate and
measurements are retained. Review findings justified the subsequent regression
checks and repeated final gates. The last repository-button padding correction
was separately typechecked and packaged; it changes no stateful behavior.

Focused static checks initially found incomplete test construction/TypeScript
narrowing, test-module placement and collapsible conditionals; these were corrected.
Compiler diagnostics still report unsupported finally lowering in observation/
picker code, preserving lifecycle cleanup as in the existing repository Hook.
No dependency or compiler change was made.

## Actual desktop qualification

Used the actual packaged Tauri/WebKit application, production Rust core and
supervisor, disposable repositories and private application state under
`/private/tmp/rag04-desktop-iu1f_8u7`. No mocked IPC was used for these observations.
Metadata-isolated, ad-hoc-signed copies use `dev.rag.workspace04`, with explicit
private state and separately recorded binary hashes. This preserves normal user
state and the original packaged application. Qualification copies and state remain
private and retained.

| Scenario             | Observed result                                                                                                                                                                                                                                                                                |
| -------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Add repository       | First workspace created and selected automatically; branch/path/base/provider visible; original dirty tracked/untracked files unchanged. [01](milestone-01-workspaces/01-first-workspace.jpg)                                                                                                  |
| Repository +         | Another workspace created immediately without a dialog; Creating worktree stage was readable before Ready.                                                                                                                                                                                     |
| Missing default      | Feature-only repository remained registered with a selected failed workspace and explicit recovery. [02](milestone-01-workspaces/02-default-failure.jpg)                                                                                                                                       |
| Default recovery     | After adding local main externally, Reconcile completed that same saved workspace ID and folder. [03](milestone-01-workspaces/03-recovered-same-workspace.jpg)                                                                                                                                 |
| Partial Git failure  | Denying writes to disposable `.git/worktrees` left an owned branch and reserved folder visibly uncertain. Restoring mode 700 and explicit reconciliation reused those artifacts. [04](milestone-01-workspaces/04-partial-creation.jpg), [05](milestone-01-workspaces/05-partial-recovered.jpg) |
| Supervisor restart   | New supervisor restored the selected workspace and four workspaces without inferred creation. [07](milestone-01-workspaces/07-final-restored.jpg)                                                                                                                                              |
| Final minimum layout | At 600×480 logical pixels, controls/footer remained usable, labels reserved both action slots, and long paths/recovery content scrolled. [08](milestone-01-workspaces/08-final-minimum-recovery.jpg)                                                                                           |
| Keyboard recovery    | Tab traversal reached Reconcile with visible focus; Return recovered the same workspace at minimum size. [09](milestone-01-workspaces/09-final-keyboard-recovery.jpg), [10](milestone-01-workspaces/10-final-ready.jpg)                                                                        |
| Desktop-only crash   | SIGKILL/reopen restored selection and reconnected to the same supervisor; repository/action/workspace counts stayed unchanged. [11](milestone-01-workspaces/11-crash-reconnected.jpg)                                                                                                          |

The final desktop run retained two repositories, five workspaces and eight workspace
actions. Original fixture contents match the initial known dirty/untracked bytes;
Git metadata permissions were restored. [Reconnect/count/hash evidence](milestone-01-workspaces/desktop-reconnect.json).
AX captures accompany key screenshots. Source/bundle contexts identify the early,
review-fixed and final-layout builds separately.

An early staged debug-supervisor launch exited 137 despite matching Cargo/staged
hashes and successful on-disk signature validation. The Cargo-reported executable
launched successfully; final packaged qualification copies also launched. The cause
was not established and is not reported as a product recovery failure.
[Retained failed attempt](milestone-01-workspaces/staging-launch-failure.json).

## Actual process and migration qualification

The opt-in evidence harness uses real Git, SQLite, supervisors and a synthetic Git
wrapper that pauses worktree addition. It is outside the unit gate and establishes
application recovery/OS-lease behavior, not native-provider behavior.

```sh
python3 docs/evidence/milestone-01-workspaces/qualify_creation.py \
  target/release/workspace-supervisor \
  docs/evidence/milestone-01-workspaces/process-qualification.json
```

Pass in 2.525380 s: kill the harness-owned supervisor during the paused mutation;
replacement owner reports uncertainty; explicit reconciliation observes the still
held execution lease; conflicting new creation is rejected. Release the fixture,
then reconcile again: the exact workspace/branch/folder becomes Ready. Replayed
creation returns its saved result, conflicting action ID is rejected, and only
one workspace branch/two worktrees including the original exist. Original checkout
hashes match, and the new checkout contains the pinned committed version.
[Results and identities](milestone-01-workspaces/process-qualification.json).

Separate disposable schema-1 metadata migrated to schema 2, retaining repository
and registration-action IDs and defaulting old requests to no initial creation.
No workspace was inferred. [Migration evidence](milestone-01-workspaces/migration-qualification.json).
The synthetic fixture follows the retained ticket-03 SQL/JSON schema; no prior
milestone state or evidence was modified.

## Review and limits

Independent Standards and Spec reviews compared the implementation with the pinned
starting revision and repository requirements. Initially Standards reported two
findings and Spec two, with a shared selection finding. Corrections covered
capacity-blocked recovery, late initial selection, and definitively rejected
pending actions, including rejection after repository navigation. Each regression
failed before its correction and passed afterward. Both axes confirmed resolution
with no remaining concrete findings.

Unit, actual-desktop and actual-process results are separate above. No live Codex
or Claude work was submitted. Provider compatibility, chat/streaming, ticket 09's
product Quit workflow, escaped processes that deliberately discard inherited
ownership, power-loss durability, and full-milestone qualification remain outside
this evidence. Navigation's 100 ms/recent-history's one-second targets were not
measured here. Existing process-exclusivity qualification was retained rather than
rerun; this ticket's process cohort focuses on workspace recovery.

All qualification processes launched here were stopped after checking their current
private-state executable identities. Disposable state and worktrees remain retained.
[Cleanup](milestone-01-workspaces/cleanup.json). Existing Git hook configuration was
preserved; no new hook or duplicate hook gate was installed.
