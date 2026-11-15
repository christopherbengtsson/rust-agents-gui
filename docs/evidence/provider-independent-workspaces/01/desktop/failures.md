# Qualification failures and corrections — 2026-11-15

These are retained unsuccessful qualification attempts, not passing acceptance or
proven product defects. Product source remained `f0885d6` throughout.

1. Two initial documentation reads selected nonexistent evidence paths
   `docs/evidence/milestone-01/02a.md` and `docs/evidence/workspace-foundation.md`;
   `cat` returned 1. The actual 02a evidence is
   `docs/evidence/milestone-01-ui-foundation.md` and its linked sidebar follow-up.
   A later attempted source read of `crates/core/src/workspaces/requests.rs`
   returned 1; the request type is in `workspaces/creation.rs`.
2. Initial `python3 .../desktop/fixture.py` used unsupported
   `workspace-supervisor --protocol-metadata`; the supervisor interpreted that
   positional argument as a state directory and waited. No desktop was launched.
   Exact copied executable/PID **36650** was checked with `ps`, terminated with
   SIGTERM, and the waiting `check_output` reaped it (CalledProcessError, -15).
   [Record](initial-harness-failure.json). Its newly created `--protocol-metadata`
   directory was moved to the initial private root as `initial-argument-state`;
   it was not deleted. Correct supported flag: `--protocol-version`, returning 9.
3. Next host launch lacked a retained PTY; stdin EOF ended the host after creating
   private root `/private/tmp/aw-piw-f27fn44b`, supervisor **36758** and desktop
   **36759**. The attempted targeted cleanup first ran `ps -p 36759 -o command=`;
   it returned 1 because that desktop had already exited. No signal was sent by
   that failed cleanup. Neither process remained on subsequent inspection.
   Host-join is not claimed for these initial children. The qualified cohorts
   used retained PTYs and child handles throughout.
4. The first `Command-Q` was immediately followed by `app.getAXState()`. CUA
   automatically reopened the copied app without the fixture environment, PID
   **38993**, rather than observing the exited child. The fixture host independently
   confirmed original desktop **36804** exit 0 and retained supervisor **36803**.
   The unique copied executable of 38993 was verified and terminated; no product
   action was submitted there. The default supervisor **85891** was pre-existing
   and untouched. [Record](quit-observation-relaunch.json). All subsequent Quit
   checks used host `wait()`/native state before explicitly launching the desktop
   with its fixture environment, and no post-exit AX lookup.
5. A full AX/screenshot capture renumbered elements. Clicking stale ID 118 for
   `new-file.txt` returned CUA invalid-element error -10005. The fresh tree's ID
   64 worked; capture 06 records the actual successful new-file diff.
6. Initial resize used logical-looking coordinates `[898,678] → [598,478]`,
   selecting text while attempting the resize. Physical screenshot coordinates
   `[1798,1358] → [1198,958]` reached the supported minimum; actual JPEG dimensions
   verify 1200×960 at 2×. Capture 11 retains selected text from the earlier attempt;
   later keyboard captures show the unobstructed recovery controls/focus.
7. First post-supervisor-replacement capture raced the stale socket pathname:
   `capture 16-after-supervisor-restart` returned `ConnectionRefusedError(61)`.
   This was not a desktop failure. The separately named successful `16b` capture
   records live replacement PID 39473 and identical durable tables. The later
   recovered-state restart separated launch and observation and passed.
8. Initial `python3 .../desktop/audit.py` assumed CUA screenshot bytes were PNG,
   causing its image-dimension assertion to fail after durable/native checks.
   `file .../*.png` showed JPEG bytes. Files were renamed `.jpg` without modifying
   image bytes; the corrected read-only audit used actual format/dimensions and
   passed. UI ledger capture labels are unchanged. Earlier capture 28's filename
   says diff but shows the honest cold-owner file-selection prompt; capture 29
   records the explicitly reselected new-file diff.
9. The first final document-check command validated local links before writing
   its own `document-checks.log`, so that report link failed an existence assertion.
   The log was initialized before the successful final link/JSON/source pass;
   this affected evidence generation only.
10. Staging the new raw AX/build output made `git diff --cached --check` report
    their emitted trailing spaces and the build log's blank final line (exit 2).
    The earlier unstaged check had not included untracked captures. Raw output
    bytes were preserved. Final checks use ordinary whitespace validation for
    documents/JSON/Python and ignore only end-of-line/end-of-file whitespace when
    checking raw `.txt`/`.log` captures, through a command-local Git setting.

Plain Tab did not traverse every WebKit control under the existing Mac preference.
Option-Tab did, with recorded focus/activation. No OS setting was modified and no
claim of plain-Tab traversal or spoken screen-reader qualification is made.
