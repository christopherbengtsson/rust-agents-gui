# PR 4 — review remediation, 2026-11-13

Implemented the accepted plan on `feat/milestone-01-isolated-workspaces`, HEAD
`cf1bc198e97bc1d6a794b2d924039d7f339d60bd` plus working changes. The
[source manifest](pr-4-remediation/source.json) binds file hashes, aggregate source
identity, environment and release-sidecar provenance. Earlier ticket-04 and
readable-workspace evidence remains historical and unchanged. No provider work
was submitted.

## Changes and controlled regressions

- Resolve the private state root before storage or worker startup and allocate
  absolute canonical managed paths. Core initialization propagates resolution
  errors. Existing private-directory and metadata-file protections remain.
- Reject saved relative/non-normalized workspace paths on decoding and before
  effects. This is the user-approved early-development compatibility break: no
  migration, repair, relocation or deletion. Valid absolute records remain valid;
  schema 3 and protocol 4 have unchanged representations.
- Retain write faults by workspace identity. Unrelated reconciliation/completion
  cannot release a fence. Cursor-based pending selection skips faulted records;
  successful explicit reconciliation releases only its target after durable
  admission. Tests cover noncommitting status writes, failed recovery intents,
  replay, multiple faults and continued unrelated work.
- Separate action selection from repository-driven read lifetimes. Direct replies
  and action lookups share settlement guarded by owner and explicit navigation.
  Three response orderings, old-owner replies, lost replies and navigation-away
  cases have controlled IPC coverage. New regressions initially reproduced two
  missing-selection failures and an old-owner pending-identity clear.
- Fresh frontend review found another P2: an admission error stopped unrelated
  progress polling. A controlled timer regression failed (`running` instead of
  `ready`), then passed after polling was gated on observation errors. The
  admission error stays visible.

Automatic behavior units substitute application filesystem, storage, Git and
process effects. State initialization tests use the owning state boundary without
opening SQLite or starting a worker. The initial focused Rust run reproduced both
unsafe saved-path acceptance and unrelated reconciliation releasing a write fence.

## Verification and packaging

Apple M4 Pro, 12 cores, 48 GiB, macOS 26.2 ARM64; Rust 1.99.1, Apple Git 2.50.1,
Node 26.13.1 and pnpm 12.18.0. Commands use this worktree's default warm `target`,
sequential builds/checks, one Rust harness thread and one UI worker. No fresh
private-target cold-compilation measurement was performed.

- Final `fnm exec --using 26.13.1 corepack pnpm verify`: exit 0, **6.486 s** wall.
  Format, lint, TypeScript, workspace Clippy and **80 Rust + 25 tooling + 49 UI**
  checks passed. [Final log](pr-4-remediation/verify.log).
- `fnm exec --using 26.13.1 corepack pnpm test:measure`: exit 0, **26.833 s** wall;
  compilation **0.076 s** separately, one warm-up and five warm executions.
  Median/range in seconds: Rust **0.142 / 0.141–0.143**, tooling
  **0.848 / 0.828–0.851**, UI **3.462 / 3.429–3.467**.
  [Log](pr-4-remediation/unit-measurements.log).
- Final `fnm exec --using 26.13.1 corepack pnpm build`: exit 0, **36.459 s** wall,
  **14.02 MiB** macOS app. Sidecar and desktop compilation remain separately
  reported in the [packaging log](pr-4-remediation/packaging.log). This final
  packaged desktop was used below.
- Before the polling finding, the complete gate passed **80 + 25 + 48** checks in
  **7.662 s**. Its verification, measurements and packaging logs are retained with
  `-before-polling` suffixes; they do not represent final UI source.
- One final-gate attempt stopped at formatting in the new regression and generated
  evidence JSON. [Failed log](pr-4-remediation/verify-format-failed.log). Formatting
  was corrected before the final gate.

## Private native and actual desktop evidence

`python3 docs/evidence/pr-4-remediation/qualify_paths.py target/release/workspace-supervisor docs/evidence/pr-4-remediation/native-paths.json`:
exit 0, **1.989 s** wall. The opt-in harness uses disposable private repositories,
SQLite and directly owned supervisor handles, outside the unit gate.
[Result](pr-4-remediation/native-paths.json) records binary hash, arguments, paths,
original file hashes and statuses. Both relative `state` and absolute
`state/../state` inputs produced canonical paths beneath the intended managed root.
Ambiguous defaults failed actionably; explicit recovery retained the same identity,
name and path, then completed. Restart restored the same record. Original dirty
and untracked files/status were unchanged, with no stray original-checkout
`state/` directory. Intentionally malformed saved relative paths were rejected
without moving or deleting the valid worktree. All owned supervisors exited.

The original review's failing [relative-path](pr-4-remediation/native-relative-before.json)
and [dotdot-path](pr-4-remediation/native-dotdot-before.json) native results are
retained separately, bound to their earlier debug binary hash.

The final signed packaged supervisor and desktop used private state
`/private/tmp/pr4-ui-uf6w9qcb/state`. Alpha/Brave repositories were registered by
fixture requests; Alpha had an initial ready workspace and Brave had an ambiguous
main/master default. With Alpha's workspace selected, clicking Brave's sidebar
`+` immediately selected Brave's new workspace and displayed allocation progress,
then an actionable default-branch failure. Removing the disposable `master` branch
and clicking **Reconcile workspace** showed progress then ready for the same
`lukasz-piszczek` workspace. This qualifies the actual unselected-repository
selection/recovery seam; it is not a repeat of the full registration workflow.

[Failure AX](pr-4-remediation/desktop-failed-ax.txt) ·
[failure screenshot](pr-4-remediation/desktop-failed.png) ·
[ready AX](pr-4-remediation/desktop-ready-ax.txt) ·
[ready screenshot](pr-4-remediation/desktop-ready.png) ·
[launch context/hashes](pr-4-remediation/desktop-context.json) ·
[final records/cleanup](pr-4-remediation/desktop-result.json).
Only the recorded qualification PIDs were stopped. The old default owner `85891`
remained alive; unrelated inherited owners were left alone.

Initial desktop setup attempted the intermediate staged sidecar, which macOS
killed before readiness (exit `-9`, no output). A bounded metadata probe repeated
that result, while the Cargo-reported executable and signed bundle resource both
returned protocol `4`; all three files had the same SHA-256. The cause of this
intermediate execution difference was not established. The qualified desktop used
the successful final bundle resource; the failed attempt remains separate evidence.

## Reviews and limits

[Four independent full-PR reviews](pr-4-remediation/review.md) checked the original
base plus remediation. The three original findings and additional polling finding
are resolved; no actionable findings remain. Reviewers inspected source and
regression coverage; root ran verification and qualification.

No native-provider requests, approval/billing checks or live-provider work were
performed. Actual disk-full/status-write fault injection was not performed; those
policy outcomes have controlled unit coverage. Whole-milestone chat, diff,
runtime cleanup, concurrency, broader accessibility and latency qualification
remain open. Prior completed milestone evidence is preserved.
