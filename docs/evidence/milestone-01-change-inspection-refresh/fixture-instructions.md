# Disposable change-inspection scenarios for 05c

Prepared 2026-11-13 for 05b. **Setup instructions, not executed desktop evidence.**
These synthetic edits/faults exercise the production Rust/Git/filesystem →
supervisor → Tauri → React inspection path. They use no mock IPC or fixture
provider and submit no agent work. Run as the ordinary desktop user, not root.
Preserve screenshots, source identity, logs and failed attempts during 05c.

## Start in private state

From this application's checkout, in one terminal:

```sh
fixture_root="$(mktemp -d /tmp/agent-workspace-05c.XXXXXX)"
export fixture_root
export AGENT_WORKSPACE_STATE_DIR="$fixture_root/state"
mkdir "$fixture_root/repository"
git -c init.defaultBranch=main init "$fixture_root/repository"
printf 'original text\n' > "$fixture_root/repository/text.txt"
printf 'delete this\n' > "$fixture_root/repository/delete.txt"
: > "$fixture_root/repository/empty.txt"
git -C "$fixture_root/repository" add -- text.txt delete.txt empty.txt
git -C "$fixture_root/repository" -c user.name=Fixture -c user.email=fixture@example.invalid \
  -c core.hooksPath=/dev/null commit --no-gpg-sign -m 'Disposable inspection baseline'
printf 'Private fixture root: %s\n' "$fixture_root"
fnm exec --using 26.13.1 corepack pnpm dev
```

The existing development launcher prepares a fresh sidecar and honors the explicit
private state directory. Record its sidecar provenance, source revision/dirty
state and reported executable. Add **only** the disposable repository in the app;
registration requests its first isolated workspace. Wait for ready, and copy its
actual checkout path from Workspace details. It must be inside the private state
root, not the original repository or this application's checkout.

In a second terminal, substitute the printed root and copied workspace path:

```sh
fixture_root='/tmp/agent-workspace-05c.REPLACE'
checkout='/tmp/agent-workspace-05c.REPLACE/state/workspaces/REPLACE/REPLACE/checkout'
case "$checkout" in "$fixture_root"/state/workspaces/*) ;; *) exit 1 ;; esac
test -f "$checkout/.git" || exit 1
test -f "$checkout/text.txt" || exit 1
```

The example folder is illustrative: use the actual app-created path. Keep these
variables in that terminal for subsequent commands. Never point them at a retained
workspace. Keep unrelated application instances on their existing state.

## Clean, changed, refreshed and removed selection

1. With the desktop file panel visible, wait for automatic inspection (up to two
   seconds plus queued work), or use Refresh changes. Record the complete clean
   comparison, observed HEAD and working manual Refresh.
2. Create tracked modification/deletion and a nonignored untracked file:

   ```sh
   printf 'first edit\n' > "$checkout/text.txt"
   rm -- "$checkout/delete.txt"
   printf 'new file\n' > "$checkout/new.txt"
   ```

   Select text.txt and retain its preview. Replace it with `second edit` using the
   same printf command. Its status remains modified; the visible preview must
   refresh without reselection. Confirm source numbering/comparison identity.

3. Scroll the preview, switch Workspace → Changes, and confirm retained reading
   position and one mounted diff. Select another path and confirm position reset.
4. Restore text.txt while it remains selected:

   ```sh
   git -C "$checkout" restore --source=HEAD --staged --worktree -- text.txt
   ```

   Confirm the selection remains named, its disappearance is explained and no
   other file opens automatically. Select an available path to continue.

5. Hide the right panel while Workspace is active, or close the Sheet below
   880 px. Edit a file; inspection should pause until a surface returns. Repeat
   with the window/document hidden and with a disconnected desktop. Returning
   reads authoritative observations; it starts no provider/workspace action.
   Record actual WebView visibility behavior, not a browser inference.

## Unsupported and limited previews

```sh
python3 - "$checkout" <<'PY'
import os
import pathlib
import sys
root = pathlib.Path(sys.argv[1])
(root / 'binary.bin').write_bytes(b'\x00binary\xff\n')
(root / 'non-utf8.txt').write_bytes(b'invalid \xff\n')
(root / 'crlf.txt').write_bytes(b'one\r\ntwo\r\n')
(root / 'no-newline.txt').write_bytes(b'no final newline')
(root / 'empty-new.txt').write_bytes(b'')
(root / 'limited.txt').write_text(''.join(f'line {n}\n' for n in range(1, 401)))
(root / 'oversized-row.txt').write_text('x' * (50 * 1024) + '\n')
(root / 'oversized-file.txt').write_bytes(b'x' * (256 * 1024 + 1))
os.symlink('text.txt', root / 'link.txt')
os.mkfifo(root / 'special.fifo')
PY
```

Select each file explicitly. Binary/non-UTF-8/symlink/special-file/read-limit labels
are distinct from errors and clean comparisons. limited.txt shows at most 200
complete diff rows and a Preview limited reason. oversized-row.txt reports its
individual-row limit. The symlink and FIFO are never read as ordinary text.
Inspect CRLF and missing-final-newline presentation. Supplement with conflicts,
submodules, paging and unusual paths if qualifying those native mechanics;
controlled units cover supplied outcomes but do not establish OS behavior.

## Read failure with last-good results through the production path

Make text.txt modified again and select it to obtain a successful preview:

```sh
printf 'read-failure baseline\n' > "$checkout/text.txt"
```

After the preview is visible, apply a **synthetic permission fault**:

```sh
chmod 000 "$checkout/text.txt"
```

Wait for automatic refresh. The production metadata/open path must fail visibly,
retain the last-good inventory and preview with staleness, and pause automatic
retry. Refresh changes remains usable. If permissions do not deny reads for the
current user/environment, record that failed fault setup; do not claim success.
Restore access and explicitly refresh:

```sh
chmod 644 "$checkout/text.txt"
```

Confirm a successful new comparison/preview clears the failure. This permission
fault is synthetic; its visible response is real desktop evidence only when run.

## Git failure through the production path

After another successful comparison, temporarily hide the disposable workspace's
Git indirection file. This affects only this private test workspace:

```sh
test ! -e "$checkout/.git.fixture-hidden" || exit 1
mv -- "$checkout/.git" "$checkout/.git.fixture-hidden"
```

Wait for refresh or use manual Refresh. Saved-target validation/Git inspection
must fail visibly while retaining last-good content. Restore the exact file:

```sh
mv -- "$checkout/.git.fixture-hidden" "$checkout/.git"
```

Refresh explicitly and confirm recovery. This is a **synthetic Git-identity
fault**, not a provider fault or a mocked observation.

## Interaction and qualification record

Check arrow-key focus plus Enter/Space tab activation, keyboard file selection,
separator arrow-key resizing, collapse/expand button, keyboard diff scrolling,
Sheet selection/dismissal/focus return, long paths, default/minimum sizes and
crossing the 880 px breakpoint. Repeat navigation during slow/pending reads and
owner reconnect; retain exact targets and last-good labels. Measure navigation
feedback against 100 ms with the maximum supported preview using 05c's actual
WebView measurement method.

Use the app's explicit Quit and record shutdown. Leave disposable state available
until evidence is captured. Packaging, actual desktop/OS/latency qualification and
native-provider qualification were not performed by preparing these instructions.
