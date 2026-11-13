# Frontend connection-test mock correction

2026-11-13. Follow-up to `89b2af452b73ff0d9ea4c259b5921a754fc4debe` on
`feat/milestone-01-repository-registration`. Only `src/App.test.tsx` changes test
behavior. [Source hash](frontend-connection-test-mocks/source-manifest.json)
identifies the verified dirty source before committing.

## Diagnosis and correction

`pnpm test:ui` reproduced three failures: both obsolete-completion cases and the
desktop-identity retry case. The focused desktop-identity case reproduced twice
in approximately 0.5 seconds each during diagnosis. Running the whole App test
file before editing reproduced three failed / nine passed, exit 1, 0.91 s wall.
The error was `The supervisor returned an invalid repository observation`.

The failing mocks returned a supervisor snapshot for `repository_request` after
the App connected. The production validator correctly rejected that payload.
Their global IPC-count assertions also included legitimate repository reads.
Other connection tests already used the command-aware `mockConnection()` helper
and the `connectionCalls()` filter.

Three hypotheses guided diagnosis: incorrect mock payload routing, connection
counts polluted by repository reads, and an assertion racing a transient error.
A read-only probe evaluated the actual failing mock callbacks for
`repository_request`: both returned keys `status`, `cursor`, `reset`, with no
`kind`; a repository list requires `kind: "repositories"`. The first two
hypotheses explain the failures; the command-aware correction passed without
timing changes.

The obsolete-completion setup and both desktop-identity setups now reuse
`mockConnection()`. Their call assertions use `connectionCalls()`. Existing
no-alert, current-owner, late-success/failure, explicit-retry and pending-guard
assertions remain. No tests or assertions were removed, and production code,
tooling configuration and dependencies are unchanged.

## Verification

Same qualification host as the retained repository evidence: macOS 26.2 on
Apple M4 Pro, Rust 1.99.1, pinned Node 26.13.1 and pnpm 12.18.0. Warm caches and
worktree-default Cargo `target`, test profile, serial Rust/tooling harnesses and
one Vitest worker. Commands below were each prefixed with `/usr/bin/time -p`.

| Check                                              | Result                                                                                                  |
| -------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| App file before correction                         | 3 failed / 9 passed; exit 1; 0.91 s wall                                                                |
| Focused obsolete-completion cases after correction | 2 passed / 10 intentionally filtered; exit 0; 0.80 s wall                                               |
| Complete App file after correction                 | 12 passed; exit 0; 1.10 s wall                                                                          |
| TypeScript                                         | Pass; exit 0; 0.31 s wall                                                                               |
| Complete verification                              | Formatting, lint, TypeScript, Clippy and 54 Rust + 19 tooling + 31 UI units passed; exit 0; 5.69 s wall |
| Warm execution measurement                         | One warm-up and five warm runs per harness; all passed; exit 0; 18.59 s wall                            |

```sh
fnm exec --using 26.13.1 corepack pnpm test:ui src/App.test.tsx
fnm exec --using 26.13.1 corepack pnpm test:ui src/App.test.tsx -t 'an obsolete'
fnm exec --using 26.13.1 corepack pnpm exec tsc --noEmit
fnm exec --using 26.13.1 corepack pnpm verify
fnm exec --using 26.13.1 corepack pnpm test:measure
```

The App-file command was run both red and green. Complete-gate unit
compilation/location took 0.122987 s, with execution Rust 0.025433 s, tooling
0.813706 s and UI 2.338902 s. Measurement-only compilation took 0.073857 s.
Warm execution medians/ranges: Rust 0.018407 s / 0.018010–0.019002 s; tooling
0.771913 s / 0.765612–0.779274 s; UI 2.280435 s / 2.256847–2.312444 s.
Fresh-private-target compilation was not measured.

Retained logs: [red](frontend-connection-test-mocks/red.log),
[focused green](frontend-connection-test-mocks/obsolete-green.log),
[App green](frontend-connection-test-mocks/app-green.log),
[TypeScript](frontend-connection-test-mocks/types.log),
[complete gate](frontend-connection-test-mocks/verify.log),
[measurement](frontend-connection-test-mocks/measure.log).

No debug instrumentation or prototypes were added. Tests use the existing mocked
IPC seam; actual desktop/process qualification, production packaging, VoiceOver
and live-provider checks were not rerun for this test-only correction. No provider
was started and RAT was not inspected. Earlier qualification remains historical.

Independent Standards and Spec reviewers inspected the five-line test diff
against `89b2af4`, the helpers and the surrounding production IPC/lifecycle path.
Standards: zero findings. Spec: zero findings. Both reviews were read-only and
ran no verification commands or providers; the results above were recorded by
the implementer.
