# Frontend foundation: skill writing review

2026-11-14 · Documentation review at product baseline
`7dc4eac26ab51a2146805b0aa07bcfb84d03417c`, including the existing staged foundation
amendments. Applies `skill-creator`, `writing-for-agents` and `research`.

## Authority and assessment

The [frontend spec](../../.scratch/frontend-navigation/spec.md) and its tickets own
the foundation requirements and delivery order under
[repository precedence](../../AGENTS.md#instruction-precedence). This review covers
the local React and Rust skills, their supporting guidance, and integration of the
upstream shadcn skill. The earlier [coverage review](frontend-navigation-skill-coverage.md)
retains its original results and source dispositions.

React and Rust are substantially aligned: task-selected references, contextual
validation after generated structural checks, explicit execution authority, and
physical completion distinct from logical cancellation. The assessed guidance
already supports Query observations, XState coordination and Zustand presentation
memory. No additional Rust edit or per-library skill was needed. These conclusions
concern instruction alignment, not implementation or qualification of the planned
foundation. Current versus planned behavior remains with the spec and
[observation guidance](../agents/observations.md).

Two actionable findings were resolved:

| Finding                                                                                                                      | Consequence                                                                     | Resolution                                                                                                                 |
| ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| P2: The React state-ownership example presented component-owned repository selection as correct.                             | Copying it for navigation would compete with Router, contrary to F06.           | Use an unsent dialog draft to demonstrate lifted local state; explain route ownership and restoration defaults separately. |
| P2: The upstream shadcn entry point assumes project JSON was injected, while a plain file read supplies a shell placeholder. | An agent could assume configuration or component availability without evidence. | The component guide directs context acquisition and cross-checking against local configuration and installed source.       |

The [React entry point](../../.agents/skills/react-engineering/SKILL.md) now routes
routing/cache tasks directly to [TanStack guidance](../agents/tanstack.md).
The [state rule](../../.agents/skills/react-engineering/rules/state-ownership.md)
keeps the example local and points to existing observation owners. Its generated
full guide was regenerated from the maintained sources.

## Authoring and upstream maintenance decisions

Skills disclose names/descriptions before their full instructions. Keep discovery
specific and load substantial branch-specific detail on demand; the local React
index and Rust task table already provide that structure.
[OpenAI skill documentation](https://learn.chatgpt.com/docs/build-skills)

Include non-obvious conventions and failure modes, choose coherent task boundaries,
and match procedural rigidity to actual fragility. This supports a small correction
to the misleading example and a conditional routing pointer rather than a general
rewrite or additional entry points.
[Agent Skills authoring practices](https://agentskills.io/skill-creation/best-practices)

The user chose to preserve shadcn upstream-owned for easier updates. Its files,
metadata and invocation settings remain unchanged. The
[component guide](../../src/components/README.md#upstream-shadcn-guidance) supplies
project integration; AGENTS.md retains precedence and task routing. Toolkit API
instructions remain upstream, while project scope, layers and styling conventions
remain in their existing owners. Review upstream updates against the actual stack;
keep any unavoidable local patch small, attributable and suitable for an upstream
correction. This is the chosen maintenance policy, not a universal prohibition on
adapting an upstream skill.

Runtime-specific mechanics need separate treatment. Claude Code documents shell
preprocessing for dynamic skill context and frontmatter invocation controls such
as `disable-model-invocation` and `user-invocable`. Their presence in a file does
not establish that another runtime performed the preprocessing.
[Claude Code skill configuration](https://code.claude.com/docs/en/skills)

Codex documents `policy.allow_implicit_invocation` in `agents/openai.yaml`, with
implicit invocation enabled by default. Preserve current settings rather than
transferring the Claude-specific invocation advice in `writing-for-agents` into
Codex metadata. No runtime policy or global authoring skill was changed.
[OpenAI optional skill metadata](https://learn.chatgpt.com/docs/build-skills#optional-metadata)

## Independent decision checks

Two agents started with fresh contexts, the current skills, repository pointers and
two realistic requests each. They received neither the review findings nor the
expected answers. They only read files and returned proposed decisions; they made
no product edits or native-provider calls.

| Request                                                                                                             | Observed decision and assessment                                                                                                                                                                                                             |
| ------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| After the foundation lands, coordinate sidebar/workspace selection, Settings, Back/Forward and startup restoration. | Selected Router as current destination authority, Zustand for restoration/reading memory, Query for observations, and local state for drafts. Preserved explicit-route precedence and navigation generations. Pass for the requested design. |
| Share supervisor observations while slow inspection remains outstanding during a Settings switch.                   | Retained app-scoped owners, released view interest, drained actual IPC before releasing capacity, and separated controlled checks from physical qualification. Pass for the requested design.                                                |
| Establish shadcn configuration and a control API without supplied project-info JSON.                                | Identified the literal placeholder, proposed the project-runner info command, checked local config/source, and required compatible official API documentation. Pass for context acquisition.                                                 |
| Make the existing Add repository button compact while preserving behavior.                                          | Inspected the installed Button and consumer, proposed the existing smaller size at the call site, preserved Base UI composition, and avoided regeneration or new variants. Pass for scope and component selection.                           |

These are read-only decision checks, not executed implementation, automatic skill
selection tests, or proof of reliable behavior across all tasks. No old-skill or
no-skill baseline was run; no comparative quality, token-use or latency improvement
is claimed. Comparative evaluation requires equivalent cases against a baseline
and inspection of actual outputs and execution traces.
[Agent Skills evaluation guidance](https://agentskills.io/skill-creation/evaluating-skills)

## Validation and limits

- `corepack pnpm exec intent list` succeeded using the installed CLI. It exposed
  routing/composition guidance but no skill-authoring workflow; this change adds
  routing pointers rather than TanStack API implementation.
- `node .agents/skills/react-engineering/scripts/build-guide.mjs --check` passed
  all 44 rules, index coverage, relative links/anchors and generated-guide equality.
- `corepack pnpm exec oxfmt --check` passed on all four edited source Markdown files.
  The first check
  found formatting in the edited state example; formatting and guide regeneration
  resolved it.
- The bundled `quick_validate.py` failed at import because Python lacks PyYAML.
  No dependency was installed. An inline check using installed `js-yaml` 4.3.2
  passed YAML parsing, folder/name agreement, naming/description bounds and
  nonempty descriptions for all three skills, plus interface YAML parsing. This
  fallback does not validate runtime invocation semantics or behavior.
- An inline standard-library audit passed 247 relative file targets, including 44
  anchors, across 70 Markdown files: skill Markdown, the component guide and this
  report. No targets left the repository. `git diff --check` passed.
- A pre-edit hash snapshot and staged-diff hash confirmed that only this report,
  the React entry point/state rule/generated guide, and the component guide changed
  during implementation. All 934 other pre-existing files, existing staging and
  upstream shadcn files were unchanged.

Application static/unit gates, bundling, packaging, desktop/accessibility and
native-provider qualification were not run for this documentation-only change.
No application behavior, dependency, ticket acceptance, historical evidence or
RAT files were changed. Primary web sources were checked on 2026-11-14; the research
pass and subsequent decision checks used independent contexts.

## Follow-up — official XState guidance

2026-11-14 · Guidance-only implementation of the subsequently approved review plan.
The earlier assessment and validation results above remain their original checkpoint.
The user selected the official Stately skill for the additional XState API/modeling
workflow rather than another locally authored reference.

Vendored [xstate-v5](../../.agents/skills/xstate-v5/SKILL.md) from
[statelyai/skills at e83214e](https://github.com/statelyai/skills/tree/e83214e458667b302dca5b4091a038f8bb89ee0d/skills/xstate-v5).
Its entry point, five referenced documents and repository MIT license are unchanged
upstream bytes. The skill-installer helper fetched that exact revision into a
temporary directory; only those seven files were copied into the repository.
Maintainer evaluation scripts/data have no invocation from the skill and were
omitted. Normal automatic discovery remains enabled; no runtime metadata or
external-tool connection was added. Review updates against the selected XState
version and repository contracts before replacing the vendored files.

The skill supplies v5 machine modeling, actor-kind selection, typed setup/actions,
React adapters and review/migration guidance. This supplements the earlier
ownership-focused assessment: the React rules cover integration, while the official
skill now supplies the distinct library workflow. Repository and React entry points
route XState work to it. Project-specific compatibility guidance stays in
[observation engineering](../agents/observations.md#cache-actor-and-presentation-ownership),
including ephemeral actor state, allowlisted presentation persistence and required
verification. The upstream skill's generic store/persistence/test suggestions do
not change the agreed architecture.

The existing draining requirement now explains its XState mechanism: actions,
including entry/exit actions, do not await promises; use invoked work and explicit
completion/failure transitions before replacement. Actor stop and callback disposal
alone establish no asynchronous physical completion.
[Official actions](https://stately.ai/docs/xstate/v5/actions),
[invocation](https://stately.ai/docs/invoke),
[callback actors](https://stately.ai/docs/callback-actors)

| Library/toolkit                      | Follow-up disposition                                                                                                    |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| React, Rust, shadcn and TanStack     | Retain the existing skills, upstream ownership and task mapping.                                                         |
| Tokio and tokio-util                 | Existing async lifecycle/framing references cover the selected mechanisms.                                               |
| ts-rs, Schemars, Ajv and ajv-formats | Existing Serde/contract-preparation guidance covers the selected generation boundary.                                    |
| p-queue                              | Existing physical-completion guidance covers pending-only abort and actual-work draining.                                |
| Zustand                              | Existing snapshot/presentation guidance is sufficient; repair two obsolete official URLs in the active library research. |
| XState                               | Add the official v5 skill and the narrow application compatibility/cleanup guidance above.                               |

### Decision walkthroughs

These are documentation walkthroughs by the implementing agent, not independent
behavioral evaluations or executed actor tests:

- **Stop during pending setup:** retain the setup promise/resource owner, complete
  late registration and cleanup in the draining workflow, and observe settlement
  before replacement. An async exit action cannot establish that ordering.
- **Navigate during outstanding IPC:** release view interest, retain app-owned actors
  and the physical registry, and hold queue capacity until actual IPC settles.
  Actor or Query cancellation cannot complete the physical operation.
- **Add simple presentation state:** keep an ephemeral draft local or use the
  existing presentation store for retained allowlisted context. The upstream
  suggestion of `@xstate/store` does not expand the selected stack; Query and Router
  retain observation and destination authority.

No additional library skills, dependency installations or general guidance rewrites
are warranted. Product implementation and native qualification remain unperformed
by this documentation change.

### Follow-up validation and limits

- `corepack pnpm exec intent list` succeeded with the installed CLI; no listed
  skill matched this authoring/vendor-integration change.
- The installer completed at the pinned revision. A final comparison against
  commit-addressed upstream sources passed for all seven vendored files; both
  replacement Zustand documentation URLs returned HTTP 200.
- `node .agents/skills/react-engineering/scripts/build-guide.mjs` regenerated the
  full guide; its `--check` mode passed all 44 rules and guide consistency.
- `corepack pnpm exec oxfmt --check` passed on the five edited local source
  Markdown files. Upstream skill files were excluded from formatting to preserve
  their original bytes. `git diff --check` passed.
- `quick_validate.py` could not run because its Python environment lacks PyYAML.
  An inline fallback using installed `js-yaml` 4.3.2 passed frontmatter, naming,
  description and scaffold checks for React, Rust and XState. The first fallback
  also attempted the unchanged shadcn skill and rejected its upstream-specific
  `user-invocable` key under the creator validator's narrower field allowlist;
  subsequent checking parsed that YAML separately without altering it. These
  structural checks do not establish runtime invocation semantics.
- An inline local-link audit passed 175 targets, including 19 anchors, across
  12 Markdown files. Bare upstream reference pointers also resolved.
- A pre-edit SHA-256 snapshot confirmed that only the six intended existing
  documentation/guide files changed; all 933 other recorded files and the existing
  Git index were preserved. The only added files are the seven vendored skill files.

The three walkthroughs above were performed by the implementing agent without
subagents or an old/no-skill comparison. They establish documented decisions, not
automatic discovery, compiled examples against the planned XState dependencies,
or behavioral reliability. Application static/unit gates, bundling, packaging and
desktop/native qualification were not run. No runtime package, external service
connection, product behavior, historical evidence or milestone acceptance changed.
The new skill is available for discovery on the next turn.
