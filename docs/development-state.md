# Native development state

The global [MVP development compatibility policy](MVP.md#development-compatibility-policy) applies. Run current builds from fresh private state. Retired application metadata, history and preferences need no migration/import or preserved reader/build. This document performs no reset.

## Roots and exact overrides

`corepack pnpm dev` prints its state root and uses:

```text
~/.agent-workspace/dev/<SHA-256(canonical checkout path), first 12 hex>/native-schema-4
```

The schema namespace is independent of application binary hashes and retired peer protocols. `corepack pnpm start` starts the normal package built by `corepack pnpm build`; its default root is the macOS data directory plus `AgentWorkspace/native-schema-4` (normally `~/Library/Application Support/AgentWorkspace/native-schema-4`).

`AGENT_WORKSPACE_STATE_DIR` overrides either root exactly, without a suffix. Relative overrides resolve against the launcher checkout working directory; use an absolute path for deliberate handover. `CARGO_TARGET_DIR` selects a real private Cargo target; symlinked or other-worktree-owned targets are rejected. Vite binds strictly to `127.0.0.1:1420`.

Provider homes remain runtime-owned (`CODEX_HOME`/normal runtime configuration). Do not delete or reset them. Qualification additionally sets `AGENT_WORKSPACE_CODEX_FIXTURE` to a copied synthetic fixture and `CODEX_HOME` to a new disposable directory; these are not real authentication instructions.

## Deliberate fresh-state start

1. Inventory the printed old root, selected build, current native owner, registered repositories and every managed worktree/branch. Current managed workspaces live beneath `<state root>/workspaces`; preserve the entire old root rather than deleting it. Older layouts can differ; preserve them too.
2. Quit the matching app and wait for completed shutdown. Hide/close-to-hide leaves the native owner running. Pending/failed Quit retains ownership: keep that app open to inspect/reconcile. A deliberate force termination is crash recovery, never successful drain. Saved PIDs do not authorize signaling.
3. Choose a new, absent private metadata location. For a disposable rehearsal:

   ```sh
   rehearsal_root=$(mktemp -d /private/tmp/agent-workspace-fresh.XXXXXX)
   chmod 700 "$rehearsal_root"
   AGENT_WORKSPACE_STATE_DIR="$rehearsal_root/native-schema-4" corepack pnpm dev
   ```

4. Confirm the printed override and ready core diagnostics. Register repositories explicitly. New managed locations must be fresh; do not adopt old worktrees. Native Quit must finish before restarting after Rust edits. Frontend edits retain Vite HMR.
5. Preserve old evidence, checkouts, all worktrees/branches and provider credentials/native history. An optional investigation copy of `metadata.sqlite3`, `metadata.sqlite3-wal` and `metadata.sqlite3-shm` must be made only after the old owner has stopped, as one set. Preserving the whole stopped root also retains managed worktrees.

For intentional reuse of a current-schema root, keep `owner.lock` at its existing inode. A competing owner must fail before opening storage/effects. Never unlink, replace or rename an active lease to bypass exclusion. A stopped-owner handover reacquires that same file. Incompatible or unrecognized metadata fails visibly; the app never silently initializes over it. Selecting a new root is the reset mechanism, not an automatic fallback.

WebView preferences use the `agent-workspace.presentation` key. They are disposable under the policy; when deliberately resetting presentation, remove that key through the app's WebView storage inspector. Do not clear provider browser profiles, credentials or unrelated local storage. Pending action IDs from old metadata must not authorize replay in fresh state.

## Separate qualification commands

`pnpm test` selects substituted Rust/tooling/UI units and needs no staged executable. `pnpm verify` is the complete static/unit gate; `pnpm build` packages normally. Neither selects actual processes or providers.

```sh
corepack pnpm qualify:storage -- /absolute/new-storage-report.json
corepack pnpm qualify:effects -- /absolute/new-effects-report.json
```

Both preserve disposable roots and refuse report overwrite. Storage uses the source-linked current SQLx implementation; effects use synthetic local descendants. They are not alternate production hosts/drivers.

To prepare a fixture-only native package, first use `node scripts/cargo.mjs metadata --locked --no-deps --format-version=1` to validate/claim the selected target, then:

```sh
VITE_NATIVE_QUALIFICATION=1 corepack pnpm tauri build --features qualification --bundles app -- --locked
corepack pnpm qualify:native -- /absolute/qualified.app /absolute/new-native-report.json
```

Copy that qualified app aside before the separate normal `pnpm build` overwrites its output. Qualification injects only synthetic readiness/login and a loopback landing page, never real authentication or chat. Its deliberately killed owners are recovery results. Retain every failed attempt. Build provenance and package inventory belong beside the report. No fixture result qualifies a live provider or ticket 08's cumulative foundation handoff.
