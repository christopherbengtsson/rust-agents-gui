# App-specific UX/UI skill: research and blueprint

2026-11-14 · Research and proposed skill contract. Documentation only; no skill
created or installed, product change, policy amendment, or feature commitment.

For implementation status, see the [skill creation checkpoint](#skill-creation-checkpoint-2026-11-15).

## Decision and authority

The proposed skill would help design, change, and review this application's user
experience. Its starting point is the underlying goal: **treat a requested
interface as a proposed solution, understand what Christopher needs to accomplish,
and seek a simpler way before designing screens or choosing components.**

The user chose a balance of workflow design and visual craft. Clarity and
trustworthy task completion take priority over novelty. Challenge unnecessary UI
early within the authorized task; routine simplifications need no extra approval.
If an alternative materially changes agreed behavior or scope, show the original
approach, concrete alternative, and tradeoff, then ask for the focused decision
before affected implementation. Continue independent work where possible.

This is a blueprint for a future skill, not new repository policy. Follow
[AGENTS.md](../../AGENTS.md), the [MVP](../MVP.md), and the current
[frontend-navigation spec](../../.scratch/frontend-navigation/spec.md) under their
existing precedence. The [component guide](../../src/components/README.md) owns
layers, tokens, composition, and custom component decisions. The
[verification policy](../agents/verification.md) owns testing and qualification;
[observation guidance](../agents/observations.md) owns relevant native boundaries.
Existing engineering skills remain responsible for implementation concerns.

## App context and user decisions

The MVP names Christopher, coordinating coding work on his own Mac, as the first
user. Its daily loop spans repositories, isolated Git workspaces, concurrent
provider conversations, exact approvals, change inspection, and durable history.
The product contract includes switching work without stopping it, explicit resume,
and recovery that exposes uncertainty rather than duplicating work.

The current foundation spec records F07 completion and retains F08 cumulative
qualification as separate work. Its approved routes, Settings return behavior,
retained reading context, and observation ownership supply constraints, not an
invitation to redesign or requalify those increments in this research.

User-selected direction for the proposed skill:

- Ask what problem is being solved, for whom, and in what situation.
- Question whether another dialog, screen, setting, or decision is needed at all.
- Reduce whole-task effort, including monitoring, uncertainty, and recovery.
- Seek a memorable benefit at workflow level; keep ordinary controls familiar.
- Favor a calm, precise visual character, with deliberate hierarchy and spacing.
- Treat substantial UX claims as provisional until Christopher tries the task.
- Scale design effort to the change rather than requiring a ceremony each time.

Continuity is a candidate hypothesis: returning to the right work with enough
validated context might be a benefit Christopher remembers. It is not a chosen
feature, a guarantee of delight, or authority to resume execution automatically.

## Primary sources and evidence limits

Sources were consulted on 2026-11-14. Published findings, official guidance, user
decisions above, and the synthesis below have different evidential status.

| Source                                                                                                                                                                                    | Supported finding or guidance                                                                                                                                                                                                                                                               | Limit and implication                                                                                                                                                                                       |
| ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [GOV.UK Service Standard: Understand users and their needs](https://www.gov.uk/service-manual/service-standard/point-1-understand-user-needs)                                             | Official guidance starts with users, the problem, and the full context of their goal; research and prototypes examine assumptions.                                                                                                                                                          | Government service guidance, not evidence about this app's users. Supports examining the goal before accepting the requested interface.                                                                     |
| [Nielsen: Ten Usability Heuristics](https://www.nngroup.com/articles/ten-usability-heuristics/)                                                                                           | The original author's heuristics cover visible status, familiar language, consistency, recognition, prevention, control, and recovery.                                                                                                                                                      | Explicitly broad rules of thumb. Translate “intuitive” into inspectable interaction questions; an expert review cannot establish actual usability.                                                          |
| [Porter/UIE: Testing the Three-Click Rule](https://articles.centercentre.com/three_click_rule/)                                                                                           | Reports 44 users attempting 620 tasks, with more than 8,000 clicks. Their dataset found neither a three-click abandonment threshold nor a correlation between click count and finding the sought content.                                                                                   | Older observational website research reported in an article, not a controlled experiment on this app. Refutes treating that threshold as established; does not prove clicks never matter.                   |
| [NIST: Usability Testing](https://www.nist.gov/programs-projects/usability-testing)                                                                                                       | Describes representative users and tasks, considering effectiveness, efficiency, and satisfaction together; examples include completion, errors, task time, and feedback.                                                                                                                   | Official methodological guidance. Supports multiple outcome measures rather than using speed or click count as the sole proxy.                                                                              |
| [Hassenzahl, Diefenbach & Göritz: Needs, affect, and interactive products](https://doi.org/10.1016/j.intcom.2010.04.002)                                                                  | Original 2010 questionnaire study connects psychological need fulfillment with positive affect and hedonic product perceptions; attribution to the product also mattered.                                                                                                                   | Retrospective, self-reported positive experiences; association does not establish causation or durable memorability. A meaningful benefit is a hypothesis, not a recipe for delight. See access note below. |
| [Amershi et al., CHI 2019: Guidelines for Human-AI Interaction](https://www.microsoft.com/en-us/research/wp-content/uploads/2019/01/Guidelines-for-Human-AI-Interaction-camera-ready.pdf) | Developed 18 guidelines through multiple evaluation stages, including 49 practitioners examining 20 AI products. Covers contextual timing, relevant information, capabilities, limitations, dismissal, correction, uncertainty, explanations, and global controls.                          | Guideline applicability evaluation, not an experiment proving anticipatory features improve outcomes. Useful for assistance boundaries; it grants no application execution authority.                       |
| [W3C: WCAG 2.2](https://www.w3.org/TR/WCAG22/)                                                                                                                                            | Recommendation includes keyboard operation, visible focus, predictable behavior, error assistance, and redundant entry. SC 3.3.7 addresses repeated information within a process, with exceptions; SC 3.3.4 permits reversibility, checking, or confirmation for its specified submissions. | Scope individual criteria accurately. Conformance does not address every user need, and removing a meaningful review step to save clicks can damage the task.                                               |
| [W3C: Making Content Usable for People with Cognitive and Learning Disabilities](https://www.w3.org/TR/coga-usable/)                                                                      | Recommends familiar design, clear steps, short critical paths, memory support, fewer interruptions, avoiding data loss, and user-controlled changes.                                                                                                                                        | Supplemental Working Group Note, not a WCAG conformance requirement. Helps consider interrupted work and cognitive effort beyond a technical checklist.                                                     |
| [GOV.UK Design System: Type scale](https://design-system.service.gov.uk/styles/type-scale/)                                                                                               | Official system guidance starts new components with existing typography styles and aligns additions with its scale; coordinated font sizes and line heights support consistent rhythm.                                                                                                      | Evidence of one design system's craft guidance, not a universal scale for desktop tools. Reuse this app's tokens and variants; do not import GOV.UK dimensions, typography, or branding.                    |

Hassenzahl access note: the DOI/publisher abstract route failed in the browser
tool. Research read an accessible [full-text copy of the original ten-page
paper](https://assets.website-files.com/6130bc5b67f7afc3311fd697/61ecc672e7e91dceae8bb1cd_Needs%2C%20affect%2C%20and%20interactive%20products%20%281%29.pdf),
identified by its authors, journal pagination, and DOI. It reports 548 of 668
participants completing the majority of crucial measures, recruitment through a
panel and email lists, and a German questionnaire. The paper explicitly limits
generalization from positive experiences and identifies implied causality as an
assumption. The copy is externally hosted; it is not an author-owned archive.

No reviewed source establishes that one signature moment guarantees memorability,
that visual novelty guarantees delight, or that three clicks is a usability limit.
The user's “already solved that for me” aspiration suggests looking for meaningful
avoided effort, but whether any proposed interaction produces that response here
remains an app-specific question.

## Proposed skill contract

Tentative name: `ux-ui`. Purpose: improve the path from Christopher's goal to a
clear, trustworthy outcome, combining workflow judgment and visual craft.

Proposed triggers: designing or changing a workflow, screen, navigation pattern,
information hierarchy, interaction, or presentation; reviewing work that affects
usability or visual quality. A pure implementation task with no UX decision stays
with its relevant engineering skill. When both concerns apply, use this skill for
the user experience and existing skills for React, XState, shadcn, Rust, or routing.
Do not require unrelated skills merely because their libraries are installed.

The future skill would read task-relevant authoritative documents and actual
affected UI before proposing changes. It would not maintain a second component
system, product contract, state architecture, or testing policy. Candidate skill
text and installation remain a separate task after this blueprint is reviewed.

## Proportionate workflow and output

This workflow is a design synthesis informed by the sources, not an independently
validated method. It should adapt to the requested change:

| Change size                               | Design effort                                                                                                                                               | Reviewable output                                                                                                                                                      |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Small, local presentation adjustment      | Inspect context and the relevant existing token/variant; check the immediate hierarchy and accessible interaction.                                          | A short rationale and the visible change. No mandatory research brief, new artifact, or approval checkpoint.                                                           |
| Bounded interaction or requested extra UI | Clarify the goal, challenge the proposed interface, compare a simpler path, and inspect affected states.                                                    | A concise flow or before/after description, visual choices, important tradeoff, and remaining uncertainty.                                                             |
| Substantial workflow or navigation change | Establish user/task evidence, map the whole journey and state consequences, explore alternatives, and make a concrete preview proportionate to uncertainty. | Goal brief, proposed flow/interface, alternative, visual rationale, hypotheses, and evidence needed. Seek only decisions that materially affect agreed behavior/scope. |

Recommended sequence:

1. **Understand the goal.** State who is doing what, why now, what success means,
   and the current obstacle. Separate supplied facts from assumptions. Use known
   app context before asking Christopher for information already documented.
2. **Challenge the requested solution early.** Ask whether removing, combining,
   deferring, or improving an existing interaction achieves the goal more simply.
   Consider no new UI as a real alternative. Explain the concrete difference.
3. **Account for the whole task.** Include preparation, decisions, navigation,
   repeated input, waiting, monitoring, interruption, uncertainty, and recovery.
   Identify why each necessary step remains; raw click count is diagnostic only.
4. **Choose a memorable workflow benefit.** For substantial design work, finish:
   “The user should remember that this app helped them ___.” Tie the answer to a
   useful outcome and label it as a hypothesis. Small changes support the relevant
   workflow; they do not each need a signature moment.
5. **Find meaningful assistance.** Identify useful work the app can prepare,
   preserve, summarize, or suggest within existing authority. Record timing,
   uncertainty, visible consequences, and correction/recovery if the inference
   is wrong. Fewer actions that create more supervision are not assumed better.
6. **Shape the interaction and visual hierarchy.** Make the current context,
   important status, next action, and outcome readable. Reuse the component guide's
   layers, tokens, and variants. Keep ordinary controls recognizable.
7. **Review the proposal and its limits.** Explain how it serves the goal, the
   tradeoff, and what remains unproven. For substantial changes, treat usability,
   continuity, delight, and memorability claims as provisional until Christopher
   tries the task. Follow repository policy for any later implementation checks.

Visual craft should consider content hierarchy, typography, density, spacing,
alignment, grouping, contrast, icon/label consistency, and usable overflow at the
app's supported window sizes. Status and action emphasis should reflect importance
rather than decoration. Account for empty, pending, stale, failed, and unavailable
states when relevant. Preserve keyboard/focus behavior and reduced-motion support.
These are review dimensions, not an authorization to restyle unrelated surfaces.

For a substantial proposal, the compact output contract is: **goal and evidence →
simpler alternative → proposed flow and visual rationale → tradeoff and authority
boundaries → open hypotheses and relevant checks**. Omit redundant sections for
small changes; the skill should reduce work for the user as well as in the UI.

## App-specific scenario evaluation

The examples below are hypothetical applications of the proposed skill. They do
not describe newly implemented features or override accepted behavior.

| Scenario                                     | Expected skill behavior                                                                                                                                                              | Boundary or unresolved evidence                                                                                                                                                           |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Small spacing or button-emphasis request     | Inspect its surrounding hierarchy and existing compact variants; make the smallest coherent token/variant choice.                                                                    | No workflow redesign or automatic approval stop. Visual improvement remains a judgment until observed in context.                                                                         |
| Request for an additional dialog             | Identify the decision the dialog would support; compare the requested dialog with existing inline status/action or no extra step.                                                    | An optional configuration detour may be removable. A material change to an agreed operation or review requires a concrete comparison and focused decision before affected implementation. |
| Returning from Settings or another workspace | Examine whether validated reading context and clear status reduce reorientation. Consider continuity as the memorable workflow benefit.                                              | Preserve the current spec's route/return rules and comparison validation. Restoring a view does not resume a provider turn; no new continuity feature is committed here.                  |
| Approval, lost reply, or failure             | Prioritize the exact target, pending operation, authoritative state, consequence, and available recovery. Consider whether less monitoring can be achieved through clearer evidence. | Exact approval matching stays strict. Rust owns execution/reconciliation; reconnect retrieves observations rather than replaying uncertain work. Stale/unknown is not success.            |

A calm display cannot compensate for false certainty. Preserve distinctions among
local intent/durability, provider receipt, execution, and completion where relevant
to the task. Plan approval and tool approval remain distinct. The proposal must
not invent an optimistic success state or frontend retry policy to appear smoother.

## Review rubric and tradeoffs

Use short findings with rationale and evidence, not a universal numeric score:

| Dimension               | Review question                                                                                                       |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Goal and necessity      | Does this solve Christopher's actual problem, and could the same outcome need less UI?                                |
| Whole-task effort       | What effort disappears, and does any uncertainty, monitoring, or recovery burden move elsewhere?                      |
| Comprehension and trust | Are context, status, next step, consequence, and uncertainty understandable and truthful?                             |
| Control and recovery    | Are targets explicit, consequential choices meaningful, and wrong/late results recoverable within existing authority? |
| Visual craft            | Do hierarchy, type, spacing, density, grouping, and emphasis serve the task through existing conventions?             |
| Accessibility           | Are keyboard/focus, names, contrast, motion, overflow, and relevant cognitive needs addressed?                        |
| Memorable benefit       | Is the proposed distinction a useful workflow outcome while routine controls stay familiar?                           |
| Evidence and scope      | Are findings, assumptions, user choices, and hypotheses separate, with material scope changes surfaced?               |

Visible choices can aid recognition while increasing clutter; disclosure can reduce
clutter while hiding needed actions. Automation can reduce actions while increasing
monitoring and recovery. An extra review step can cost time while preventing a
consequential mistake. A distinctive visual treatment can aid identity while
weakening familiar affordances. Explain the relevant tradeoff rather than enforcing
novelty, minimalism, fewer clicks, or automation as a universal answer.

For later validation of a substantial workflow, have Christopher attempt the
relevant task in a prototype or working app. Compare the current and proposed
experience where feasible, noting task completion, errors, effort, time, and
feedback. Record the task, conditions, observations, and remaining uncertainty;
one personal trial supplies feedback for this first user, not evidence about a
broader population. Ask what benefit he remembers on a later return to the task
before prompting him with the intended signature. Treat immediate reactions and
later recall as distinct observations. This is the proposed app-specific evaluation
approach, informed by the NIST guidance above, not a completed study or validated
memorability measure. Routine edits need no mandatory user trial or extra approval.

## Completion and unperformed checks

This note records source research and a proposed contract/workflow. The installed
Intent inventory was checked by the coordinating agent; only routing-related
skills were listed and none matched this documentation task. Existing repository
documents were read for constraints. No RAT files or provider history were read.

No actual skill was created, installed, invoked, or validated. No real-user task
session with Christopher, desktop/accessibility inspection, prototype comparison,
product implementation, unit suite, packaging, native qualification, or provider
submission was performed for this note. Published research does not supply that
missing app-specific evidence. Future checks remain governed by the selected task
and the repository's existing verification/qualification policy.

## Skill creation checkpoint (2026-11-15)

Created the repository-local [ux-ui skill](../../.agents/skills/ux-ui/SKILL.md) and
its [UI metadata](../../.agents/skills/ux-ui/agents/openai.yaml), with a task pointer
under UI engineering in [AGENTS.md](../../AGENTS.md#ui-engineering). This implements
the approved blueprint; the earlier dated research and proposal remain a historical
checkpoint. Product source was not changed. The working baseline was
`261405167c5dbc473a7a2ad2d5bf0d9c9d2efe8a`.

Applied skill-creator and writing-for-agents: one concise entrypoint, conditional
access to this research, and no extra scripts, assets, or copied reference catalog.
Generated interface metadata with the bundled `generate_openai_yaml.py` helper.
Automatic invocation retains its default; no Claude-specific invocation field was
transferred from the writing skill. Installed Intent exposed only routing skills,
so none matched this authoring task.

One fresh-context agent applied the skill to four independent requests, using raw
source, specifications, and repository policy without the blueprint or prior
discussion. Its proposals were reviewed against the agreed design criteria:

| Request                                             | Observed decision                                                                                                                                           |
| --------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Compact Add repository without behavior changes     | Reused the existing smaller Button variant and proposed focused keyboard/focus/fit checks, without new primitives or a design ceremony.                     |
| Add a pre-Settings section chooser to reduce effort | Challenged the extra decision, compared existing direct entry/section navigation, recognized Providers as future scope, and surfaced the material tradeoff. |
| Reduce reorientation after workspace switching      | Inspected existing retained context before suggesting additions, proposed a bounded return-cue trial, and separated view restoration from execution.        |
| Make unknown provider outcomes less disruptive      | Proposed persistent action-bound status and observation-based recovery, preserving exact identity, visible uncertainty, and no automatic replay.            |

These were read-only decision checks, not implemented features, runtime discovery
tests, user trials, or proof that the skill improves usability. No comparison with
a no-skill baseline was run. No wording change was justified by these results.

Authoring checks: the bundled `quick_validate.py` could not import PyYAML. A bare
Node import of `js-yaml` was also unavailable under pnpm's module layout; an inline
fallback using the installed `js-yaml` 4.3.2 package passed YAML parsing, supported
frontmatter fields, name/folder agreement, description bounds, placeholder checks,
interface metadata, and implicit-invocation defaults. It did not execute the
bundled validator or establish runtime discovery. Formatting and relative-link
checks cover the publication files separately from behavioral evaluation.
