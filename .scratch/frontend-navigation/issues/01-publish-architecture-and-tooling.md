# 01: Publish architecture and prepare dependencies

Status: ready-for-agent

Blocked by: Completed milestone ticket 05

Read the [complete agreed plan](../spec.md), [MVP](../../../docs/MVP.md), [milestone](../../../docs/MILESTONE-01.md), [verification policy](../../../docs/agents/verification.md), and applicable repository engineering skills. Preserve unrelated edits and completed evidence.

- [x] Save the complete agreed plan before product edits.
- [x] Retain Conductor research provenance and architecture ownership decisions.
- [x] Record production baseline and exact dependency versions.
- [x] Configure reviewed installed Router and Router/Query integration Intent skills under repository precedence; record unavailable standalone Query skills and official-source fallback.

Qualification: deterministic units replace dependencies; actual desktop/process evidence is separate. No live-provider submissions.

## Comments

### 2026-11-14 — published

Acceptance remains open until linked implementation and qualification evidence exists.

### 2026-11-14 — F01 complete

The full plan, tickets, research/ADR, pinned dependencies, reviewed installed Intent
configuration and baseline are published. See [F01 evidence](../../../docs/evidence/frontend-navigation/f01.md) for 110 Rust + 25 tooling + 71 UI passing checks, separate measured runs, unchanged production assets, and limitations. Standalone Query skills are not shipped by the installed package; reviewed composition guidance and official Query sources are configured. No product behavior or F02–F08 acceptance is claimed. Stop at F01 as clarified by the user.

### 2026-11-14 — Intent and fresh-session audit

Rechecked against the consumer quick start using the pinned local CLI: all 11 exact
allowlisted skills resolve/load and the managed block matches installer output.
Guidance-based setup is active; optional hook enforcement was not installed. The
lockfile remains staged, not committed, so the quick start's commit step is still
pending. Standalone Query skills remain unavailable in the installed package.

The plan now has [fresh-session instructions](../spec.md#starting-a-fresh-session),
and F02–F08 include source footholds, scope boundaries and required failure cases.
The process diagram now states that the supervisor hosts the Rust core; the
[research comparison](../../../docs/research/conductor-frontend-architecture.md#comparison-with-our-process-architecture)
separates observed Conductor plumbing from our accepted ownership/lifetime contract.
See [audit evidence](../../../docs/evidence/frontend-navigation/f01-handoff-audit.json).
These are F01 documentation corrections; successor acceptance remains unchecked.
