# Frontend foundation: local skill coverage

2026-11-14 · Guidance-only follow-up at product baseline
`7dc4eac26ab51a2146805b0aa07bcfb84d03417c`, including the amended working-tree
[frontend spec](../../.scratch/frontend-navigation/spec.md).

## Decision and ownership

Extend the existing [Rust](../../.agents/skills/rust-engineering/SKILL.md) and
[React](../../.agents/skills/react-engineering/SKILL.md) entry points. Add one
task-specific Rust Serde reference; retain the React rule inventory and regenerate
its full guide. Separate skills per library would duplicate discovery and shared
policy without adding an independent workflow.

[AGENTS.md](../../AGENTS.md#instruction-precedence) records the user's current
foundation precedence and observation-reading trigger. The spec owns foundation
requirements and sequence; [observation guidance](../agents/observations.md) owns
shared engineering pitfalls; [verification policy](../agents/verification.md) owns
test mechanisms and gate classification. Skills supply task routing and examples.
The earlier [Vercel comparison](vercel-react-best-practices-fit.md) retains the full
70-rule disposition for its baseline, with a dated cache-scope clarification.

## Sources and selection

Read the upstream indexes, relevant async/Serde/tooling/cache/subscription rules,
local skills and references, earlier coverage research, and the amended foundation.
GitHub commit API reads during planning and again during implementation reported
the same revisions already pinned locally:

- [rust-skills at fd2a861](https://github.com/leonardomso/rust-skills/tree/fd2a861ab0406a4ac536a55274d14ea6fd1ca9c9).
- [Vercel React guidance at 1a2c79d](https://github.com/vercel-labs/agent-skills/tree/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices).

This is a foundation-fit review, not a new exhaustive audit of every Rust rule.
No upstream installation or dependency update was needed. Maintain the selected
rules against primary sources rather than copying upstream examples unchanged.

| Source rules or concern                                                                                       | Disposition and maintained destination                                                                                                                                                      |
| ------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Rust `async-tokio-runtime`, `async-spawn-blocking`, `async-tokio-fs`                                          | Adapt in the existing async reference: supplied runtime context, bounded blocking dispatch, physical permit/completion ownership. No blanket filesystem-driver or runtime-flavor migration. |
| Rust `async-join-parallel`, `async-try-join`, `async-select-racing`                                           | Adapt failure/cancellation reasoning in that reference; preserve effects and cleanup independently of the requesting future.                                                                |
| Rust bounded channels, broadcast, cancellation safety/tokens and JoinSet                                      | Retain existing guidance. Channels supply mechanisms; the application's replay journal, acknowledgement and reconciliation policy stays with its owner.                                     |
| Rust Serde defaults, skip-empty, flatten, enum representation, unknown fields, custom conversion and try-from | Add the Serde wire-contract reference, grounded in actual input/output shapes and contextual authority rather than blanket strictness/omission recipes.                                     |
| Rust `proj-feature-additive`, `proj-build-rs-minimal`                                                         | Adapt export-feature additivity in the Serde reference. Keep actual generation with the existing planned preparation owner; no new build-script framework.                                  |
| Rust `test-tokio-async`                                                                                       | Correct runtime assumptions through primary docs; clarify controlled async mechanisms in verification policy. Do not import mockall, real I/O fixtures or additional test categories.       |
| Vercel `client-swr-dedup`, listener deduplication and render subscriptions                                    | Extend shared-owner/external-store rules for approved Query/actor/store bindings. Share the underlying owner; multiple subscribers are valid. No SWR adoption.                              |
| Vercel async scheduling and Activity guidance                                                                 | Extend existing scheduling/view-lifetime rules with physical draining and app-owned actor lifetimes; point to shared observation policy.                                                    |
| Vercel `js-cache-function-results` and `js-cache-storage`                                                     | Clarify the local cache rule: correctness applies to shared observation caches too; measurement gates optional computation-cache optimizations and performance claims.                      |
| Generated frontend IPC validation                                                                             | Extend the existing typed-IPC rule with planned structural validation followed by local semantic assertions. The product generator remains unimplemented here.                              |
| Next.js SSR/RSC/hydration, generic test frameworks and speculative performance recipes                        | Retain existing architecture/policy exclusions and conditional applicability; new foundation dependencies do not make these recipes applicable.                                             |

## Primary-source corrections

**Tokio.** Upstream's test example labels `#[tokio::test]` multithreaded by default;
Tokio 1.54.1 specifies a separate current-thread runtime per test and `test-util`
for paused time. An executor does not itself make effectful tests units or their
scheduling deterministic. [Test configuration](https://docs.rs/tokio/latest/tokio/attr.test.html)

Current-thread execution supports concurrent I/O when driven. Blocking work can
stall it, and the blocking pool can queue independently of application admission.
Started blocking tasks survive abort; shutdown timeout stops waiting rather than
stopping the work. These qualify upstream runtime/filesystem/fail-fast examples.
[Runtime configuration](https://docs.rs/tokio/latest/tokio/runtime/index.html),
[blocking dispatch](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html)

**Serde.** Defaults govern input; output omission is independent. Flattening is
unsupported with `deny_unknown_fields` on either participating struct. Preserve
actual wire contracts instead of applying blanket DTO strictness or compactness.
[Field attributes](https://serde.rs/field-attrs.html),
[flattening](https://serde.rs/attr-flatten.html),
[enum representations](https://serde.rs/enum-representations.html),
[Cargo additive features](https://doc.rust-lang.org/cargo/reference/features.html#feature-unification)

**React integrations.** Query cancellation describes logical request behavior,
not proof of native completion. XState's actor-creating React Hooks bind actors
to component lifetime; `useSelector` can observe an existing owner. Zustand's
5.1.0 vanilla-store binding selects through `useSyncExternalStore`. These support
feature-local library bindings without mirrored records or per-screen actor owners.
[Query cancellation](https://tanstack.com/query/v5/docs/framework/react/guides/query-cancellation),
[XState React](https://stately.ai/docs/xstate-react),
[Zustand 5.1.0 source](https://github.com/pmndrs/zustand/blob/v5.1.0/src/react.ts)

**Cache scope.** Vercel's originating rule recommends caching repeated function
results in a module Map. The former measurement-only applicability was a local
adaptation, not an upstream mandate. Keep its correction in the local cache rule
and distinguish the approved Query cache from an additional computation cache.
[Pinned upstream rule](https://github.com/vercel-labs/agent-skills/blob/1a2c79d4e6c4a2283dc2c614b6538cbe1ed5ca62/skills/react-best-practices/rules/js-cache-function-results.md)

## Implementation and evidence limits

Only guidance, the generated React guide and research change. The milestone P2
summary now marks tickets 01–05 complete for their recorded fixture scope, with
06–13 and whole-milestone qualification incomplete. Historical results are retained.
Query/Zustand are installed but foundation adoption remains planned; XState,
generated contracts, the new transport and their qualification are not implemented
by this guidance change. Existing evidence remains the product checkpoint.

The installed Intent CLI and Router/Query composition prerequisites were loaded
under repository precedence. Guidance validation results:

- `corepack pnpm exec oxfmt --check` passed on the 17 edited source Markdown files.
- `node .agents/skills/react-engineering/scripts/build-guide.mjs --check` passed:
  all 44 rules, the index, local links/anchors and regenerated full guide agree.
- A temporary standard-library link audit passed 448 relative file/anchor checks
  across all 42 changed files. One out-of-repository archive target was skipped
  without reading it.
- Both skill entry points passed YAML/frontmatter, name, description and scaffold
  checks using installed `js-yaml` 4.3.2. The standard `quick_validate.py` attempt
  failed before validation because its Python environment lacks PyYAML; no package
  was installed to repair that tooling environment.
- `git diff --check` passed. Before/after SHA-256 comparison found no changes to
  the 786 recorded tracked non-Markdown source/configuration files and retained
  evidence files during this guidance update.

No application static/unit gates, bundling, packaging or desktop/native qualification
were run for these documentation changes. No application execution, provider
submissions or RAT inspection occurred. These results qualify guidance consistency,
not implementation of the planned foundation.

## Follow-up — delivery-independent skill guidance

The user clarified that skills should express durable engineering practices rather
than depend on one delivery's research or implementation status. Removed local
research pointers from both skill entry points and milestone references from the
React examples. Repository policy pointers and upstream/primary provenance remain.
Research links to maintained guidance; the skills no longer require this note.

Added the Rust [framed-I/O reference](../../.agents/skills/rust-engineering/references/framed-io.md)
and task routing for tokio-util codec/buffer ownership, frame bounds, incomplete
EOF, deadlines and partial-progress cancellation. It links to the existing async
lifecycle reference for cooperative stop and completion. Protocol versions,
specific limits and delivery sequencing stay with the owning project documents.
Earlier validation counts above describe the preceding guidance checkpoint.

Follow-up checks passed: formatting for eight edited source Markdown files,
44-rule generated-guide consistency, 449 relative link/anchor checks across the
43 changed files, and `git diff --check`. One outside archive target was skipped
without inspection. All 787 recorded source/configuration/evidence hashes stayed
unchanged. The skill scan found no local research links, delivery identifiers or
implementation-status references; no application checks were run.
