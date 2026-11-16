# Version/revision documentation follow-through

2026-11-16. Baseline: `4c58b37690972697c2c4b75acd34db3df9afac1e`.
Scope follows the [research snapshot](../../research/version-revision-schema-sweep.md)
and the user's request to verify and review the edits already in the working tree.
The sidebar indicator and its diagnostics control remain unchanged.

## Changes

- Root README links the current development-state owner instead of repeating a
  stale schema number.
- Contract documentation presents current native generation guidance separately
  from historical protocol checkpoints and removes the obsolete preference rule.
- Fixture documentation identifies both retired supervisor drivers as historical;
  their source files remain unchanged.
- Current native qualification uses neutral lifecycle artifact names for future
  runs; no old artifact, report, corpus, provider schema or resource is renamed.

## Verification

`corepack pnpm verify` passed (exit 0): 172 Rust, 48 tooling and 263 frontend tests.
The [complete gate log](verify.log) identifies generated drift checking, format,
lint, TypeScript, all-feature Clippy, compilation and separate unit execution.
The worktree's default target and warm caches were used; unit harnesses ran with
one thread/worker. This makes no new performance or fresh-target measurement claim.
The platform/toolchain cohort is unchanged from the previous
[environment record](../current-format-cleanup/environment.json).

Python AST parsing passed for `scripts/qualify-native.py` without executing the
qualification driver. Local link target checks passed for the three edited
README files and the research note. `git diff --check` passed. These low-impact
text/name changes add no tests.

No native process/provider qualification, package build, live-provider call or
application-state change was performed or claimed. Existing native/storage
qualification remains evidence of its own source checkpoint.

## Standards

Independent fresh-context read-only review: **0 findings**. The diff follows
repository compatibility and preservation policies, separates current guidance
from historical material and changes only future artifact names. No actionable
code-smell finding or additional CONTRIBUTING/coding-standard source was found.

## Spec

Independent fresh-context read-only review: **0 findings**. Both stale claims,
current/historical separation, both fixture-driver labels and neutral future
qualifier names match the authorized scope. Runtime counters, external versions,
application identity and retained evidence remain unchanged. Sidebar work was
explicitly excluded from this continuation.
