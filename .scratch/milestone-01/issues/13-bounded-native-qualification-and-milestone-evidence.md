# 13: Bounded native qualification and milestone evidence

## Native core amendment — 2026-11-16

Apply the [accepted native-core/SQLx spec](../../sqlx-architecture/spec.md) and
[amendments A2/A3/A8](../../sqlx-architecture/amendments.md) before this ticket.
They replace separate-supervisor/native-crash continuation and single-active
instructions with native ownership, WebView continuity, durable native-failure
recovery and the concurrent three-conversation scenario. Preserve exact controls,
remaining milestone scope and historical evidence. Acceptance remains unchecked;
this publication authorizes no live-provider or authentication action.

**What to build:** Qualify the integrated desktop workflow against the supported official Codex runtime under a separately agreed allowance and close the milestone only against retained evidence.

**Blocked by:** 12: Verify the complete desktop workflow with fixtures

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** Native closure of A1–A10 and verification stages 3–4. This ticket’s ready-for-agent status is not a live-provider allowance.

- [ ] Execute the agreed native scenarios through the same completed React/shadcn desktop controls and state presentations qualified in ticket 12. Record native rendering/interaction issues and uncertainty visibly; preserve the agreed submission/usage bounds rather than adding unapproved scenarios or bypassing the UI to claim desktop success.
- [ ] Before any real run, obtain agreement on exact scenarios, permitted submissions, selected model/effort, time/usage limits, and stop conditions. Stop before submissions if that allowance is absent.
- [ ] Use disposable test checkouts and private application state while keeping authentication runtime-owned. Record runtime/schema versions, effective configuration and MCP startup exposure, and source hashes before/after the cohort.
- [ ] Perform only allowed submissions with no automatic scenario retries, larger-model fallback, archived allowance reuse, or assumed hard dollar cap.
- [ ] Through the actual desktop, qualify native connection/streaming, exact allow/deny operation matching and effects, interruption/owned cleanup, restart/history, and exact conversation resume with earlier-marker recall.
- [ ] Include the agreed concurrent Q6 cohort: two active conversations share one workspace while a third runs elsewhere, on the selected per-active-conversation app-server topology. Qualify exact routing, quiet-conversation approvals/interrupt under hot output/slow persistence, bounded overload with explicit failure scope/prefix/gap, native crash/recovery and final owned Quit. Confirm actual process-sharing/permission/credential behavior for the supported runtime rather than promoting fixture results to native proof. Any additional submissions/fault scenarios must fit the separately agreed allowance.
- [ ] Observe subscription-backed execution through the existing account and record non-secret account/billing evidence when available; report unknowns honestly. API-key compatibility stays separately unqualified unless included in the allowance.
- [ ] Record actual submissions/usage, screenshots, native pass/fail/not-run outcomes, failed/unperformed checks, and unresolved limits separately from fixture/desktop results.
- [ ] Mark the milestone Passed only if all required acceptance and deterministic/desktop/native checks succeed; otherwise preserve completed evidence and leave it incomplete.
- [ ] On a pass, preserve the completed milestone and add the next numbered milestone with its own narrow outcome, acceptance criteria, and evidence-informed deferred capabilities. Carry forward the [global MVP development compatibility policy](../../../docs/MVP.md#development-compatibility-policy), including fresh current-state verification and the clean-state final MVP gate; a milestone boundary adds no compatibility obligation. Do not repurpose the completed record or claim MVP completion from this milestone alone.

## Comments

2026-11-16: Reconciled with the native-core/SQLx spec and amended A8/A10/concurrency
acceptance. Existing dependency edges remain; milestone 07 now inherits SQLx 08.
Qualification stays open and no production implementation is claimed.
