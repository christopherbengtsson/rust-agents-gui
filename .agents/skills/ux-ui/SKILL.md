---
name: ux-ui
description: Design, refine, or review this app's UX/UI, including workflows, interactions, information hierarchy, and visual presentation. Use for user-facing design decisions; use engineering skills for pure implementation mechanics.
---

# UX/UI

Treat the requested interface as a proposed solution. Understand the underlying
goal, then seek a simpler way to achieve it before choosing screens or components.
Clarity and trustworthy task completion take priority over novelty.

Apply this skill under [repository precedence](../../../AGENTS.md#instruction-precedence).
Read the [MVP](../../../docs/MVP.md) for the user and product goals, the task's
active spec for agreed behavior, and the
[component guide](../../../src/components/README.md) before component decisions.
Inspect the affected UI and existing journey before proposing replacements.
Use the relevant [engineering guidance](../../../AGENTS.md#ui-engineering) for
implementation; this skill supplies UX judgment rather than a second architecture
or component system.

## Challenge the solution

Establish who is doing what, why now, what success means, and where the current
experience gets in the way. Use repository context and available evidence first;
ask only for missing intent that materially affects the design.

Before adding a screen, dialog, setting, or manual step, consider whether removing,
combining, or improving an existing interaction achieves the goal more simply.
Keeping the current interface or adding no UI are valid alternatives. Explain a
stronger alternative early, before investing in the requested design.

Stay within the authorized task. If the alternative materially changes agreed
behavior or scope, show the original approach, the concrete alternative, and its
tradeoff; resolve that decision before implementing the affected part. Continue
independent work. Routine improvements within agreed behavior proceed without an
added approval checkpoint.

## Reduce whole-task effort

Account for preparation, decisions, navigation, repeated input, waiting,
interruptions, monitoring, and recovery. Identify what effort disappears and what
moves elsewhere. Judge the path by successful completion and understandable
progress; click count alone cannot establish an improvement. Keep steps that
support comprehension, meaningful review, or reliable recovery.

Look for useful work the app can preserve, prepare, connect, summarize, or suggest
using available information. Make timing, uncertainty, visible consequences, and
correction clear. Evaluate what happens when the inference is wrong. Ground
proposed assistance in the MVP's execution and approval boundaries: restoring a
view is distinct from resuming work, and uncertainty needs truthful presentation.

## Make the benefit memorable

For substantial workflow design, state one useful outcome the user should remember:
“This app helped me ___.” Treat that benefit as a hypothesis to validate through
use. Continuity across workspaces, conversations, and changes is a candidate, not
a required feature. Let routine controls remain familiar and support the workflow.

Aim for a calm, precise interface. Use hierarchy, typography, density, spacing,
grouping, contrast, language, and purposeful feedback to make context, important
status, and the next action legible. Express character through the existing tokens
and components, with custom interactions justified by a concrete user need.

Consider relevant empty, pending, stale, failed, and unavailable states alongside
the normal path. Preserve accessible names, keyboard operation, visible focus,
focus return, contrast, and reduced-motion support. Inspect changed layouts at
relevant supported window sizes when possible; report visual or accessibility
checks that remain unperformed.

## Scale the work and show the evidence

For a small local edit, a brief rationale and focused checks are sufficient. For a
bounded interaction, explain the goal, simpler alternative, affected flow, and
important tradeoff. For substantial work, make the proposed journey and visual
direction concrete with a preview or prototype proportionate to uncertainty.
Avoid turning every UI edit into a separate design artifact or approval stage.

For UX review, report specific findings with task impact, evidence, and the
smallest coherent improvement. Separate observed behavior, user requirements,
assumptions, and design hypotheses. A heuristic review is not a user trial.

Keep important usability and memorability claims provisional until Christopher
tries the relevant task. Where feasible, compare the existing and proposed
experience through completion, errors, effort, time, and feedback. Ask about the
remembered benefit on a later return before suggesting the intended answer.
Record limits; one user's experience does not establish general usability.

Follow the repository's verification policy for implementation checks and keep
unperformed user trials explicit without blocking routine edits. Consult the
[research and blueprint](../../../docs/research/ux-ui-skill-blueprint.md) when
revisiting evidence, design tradeoffs, or scenario evaluation; it retains source
limits and the rationale behind this guidance.
