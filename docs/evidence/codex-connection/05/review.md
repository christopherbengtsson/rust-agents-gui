# Connection 05 independent review

2026-11-15 · Fixed starting point `612ffef82de675fd9261a40fec4a2f60a1c49b2f`;
reviewed the staged implementation diff, excluding pre-existing user edits.
Repository guidance and the accepted 05 amendment supplied standards/spec authority.

## Standards

Two P2 findings were resolved:

1. Chooser generation invalidation retained busy state across supervisor replacement
   or same-owner reconnect. Interaction state now follows owner/target scope and
   clears during invalidation; late results remain rejected. Parameterized failing-
   then-passing regressions cover both paths. Re-review: resolved.
2. The qualification helper could leave detached processes on startup, EOF or command
   failure. One try/finally now owns all exits, bounded cleanup and retained-process
   reporting. Actual EOF and deliberate-fault cohorts independently establish exit0
   supervisor cleanup/no remaining processes. Re-review: resolved.

Final narrow review found no new concrete issue in shell `+m` startup or healthy
helper-error hiding. **Zero unresolved Standards findings.** Reviews were read-only;
reviewers did not rerun checks or claim native UI qualification.

## Spec

One P2 finding was resolved: a retained failed login made a new status-read failure
red. Current verification failure now takes precedence and remains yellow
“Couldn’t verify Codex”; the regression failed before the fix and passed after.
Re-review also confirmed chooser corrections and the final shell/harness/healthy
presentation additions. **Zero unresolved Spec findings.**

Native chooser, Finder/Dock launch, focus and layout acceptance remains open under
Christopher's explicit choice to record UI checks as unperformed.
