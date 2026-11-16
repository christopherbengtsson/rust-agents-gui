# Codex executable discovery and recovery

2026-11-15 · Primary-source research for the Codex connection journey. This note
records comparable desktop practices and a proposal for the unresolved “Locate
Codex” interview question (Q8). It accepts no product decision and changes no
[connection requirements](../../.scratch/codex-connection/spec.md).

## Documented practices

**VS Code distinguishes lookup failure from installation status.** Its Git
troubleshooting guide defines “Git not found” as inability to locate Git in the
current environment. When Git works in a terminal but not the app, it recommends
checking the paths tried and setting `git.path` to the full executable path.
This is direct precedent for automatic discovery plus an explicit override.
[Git troubleshooting](https://code.visualstudio.com/docs/sourcecontrol/troubleshooting#_git-not-found).

VS Code also documents why terminal success need not imply GUI success on macOS:
a Dock launch lacks the shell environment. The app obtains it through a shell
subprocess, with a default ten-second timeout; if resolution fails, it can launch
without that environment. Its Git source validates candidates with `--version`,
tries explicit hints before platform discovery, and uses platform-specific lookup
(including `which` and an Xcode availability check on macOS). These are bounded
recovery mechanisms, not a guarantee of finding every installed executable.
[Shell environment FAQ](https://code.visualstudio.com/docs/supporting/faq#_resolving-shell-environment-fails);
[Git discovery source](https://github.com/microsoft/vscode/blob/main/extensions/git/src/git.ts#L81-L174).

**JetBrains combines detection, installation help and manual verification.**
IntelliJ IDEA automatically detects Git when cloning or enabling version control;
if it cannot locate the executable, it suggests downloading Git. Its Git settings
also permit typing an executable path or choosing it with Browse, followed by a
Test button that verifies the path. The documentation describes additional WSL
lookup on Windows. It does not establish that failed detection proves absence
from every possible installation location. The consulted pages identify IntelliJ
IDEA 2026.3 and were updated 2026-11-10.
[Repository setup](https://www.jetbrains.com/help/idea/set-up-a-git-repository.html);
[Git settings](https://www.jetbrains.com/help/idea/settings-version-control-git.html).

**Zed demonstrates an alternative ownership model.** Its current external-agent
guide makes installation through the ACP Registry the primary path, including
Codex. It separately supports custom ACP agents configured with a command,
arguments and environment. Registry agents accept environment overrides rather
than custom command/argument overrides. This supports managed installation as an
alternative to discovering an arbitrary user installation; it is not evidence
that Zed's current Codex registry entry offers a Browse executable control.
[External agents](https://zed.dev/docs/ai/external-agents);
[Agent server settings](https://zed.dev/docs/reference/all-settings#agent-servers).

## Inference and proposed adaptation

An unsuccessful bounded search cannot conclusively distinguish “not installed”
from “installed somewhere we did not search or cannot access.” A successful
candidate check establishes a usable executable at that location; unsuccessful
lookup establishes only the scope of that lookup. The VS Code environment example
shows a concrete way an installed tool can remain undiscoverable. Therefore
“Codex not found” is more defensible than asserting “Codex is not installed.”
This is an inference from the discovery limitations above, not a quoted vendor
guarantee.

The proposed recovery sequence is:

1. Improve bounded automatic discovery using the relevant user environment and
   supported installation locations. Honor an explicit saved override, validate
   candidates, and retain actionable failure evidence.
2. If discovery fails, show “Codex not found” with installation instructions,
   **Locate Codex** and **Check again**. Installation instructions help users who
   need Codex; Locate helps those whose installation was missed. Rechecking
   serves either path without requiring users to diagnose the cause first.
3. Validate a selected executable's identity/version and required protocol before
   reporting readiness. A selected file or successful process launch alone does
   not establish compatible Codex service or authentication. This validation is a
   project proposal governed by the existing
   [runtime compatibility contract](../MVP.md#mvp-scope-and-observable-acceptance).

This proposal keeps manual path selection contextual to failure. Whether to add
Locate Codex remains Q8's decision. Managed downloads or bundling would introduce
installation/update ownership and remain outside the
[current MVP installation scope](../MVP.md#mvp-scope-and-observable-acceptance).

## Evidence limits

All linked first-party documentation/source was consulted on 2026-11-15. VS Code's
source link tracks `main`; these findings are a dated reading, not a pinned-build
qualification. No real Codex discovery, installation, authentication, credentials,
provider execution, desktop flow or user trial was inspected or exercised. This
documentation-only research supplies no native-compatibility or usability result.
