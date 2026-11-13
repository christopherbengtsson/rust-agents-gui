# 06: Connect Codex with actionable setup diagnostics

**What to build:** Provide Codex connection/setup behavior through the desktop and establish the real and deterministic execution adapters against a recorded supported protocol cohort.

**Blocked by:** 02a: Establish the incremental UI foundation

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**UI:** Follow the [incremental UI foundation](../spec.md#incremental-ui-foundation). For shadcn/ui component work, use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy.

**Coverage:** A3 deterministic implementation; protocol/fixture prerequisite for A4–A5 and A7–A9.

**Execution:** Follow the [accepted Codex setup plan](../codex-setup-plan.md),
in order: [06a: Settings and retained context](06a-settings-and-retained-workspace-context.md)
→ [06b: setup end to end](06b-check-codex-setup-end-to-end.md)
→ [06c: protocol/process hardening](06c-harden-codex-protocol-and-process-recovery.md)
→ [06d: qualification and review](06d-qualify-codex-setup.md).
Complete only the child requested in each implementation session. This remains
the acceptance parent; child completion alone does not check its original items.

- [ ] Present runtime setup and connection/configuration status in its own product composition within the established shell, distinct from `SupervisorConnection` transport diagnostics, using shared fields, labels, controls and semantic status styling. Make missing-executable, incompatible-protocol and unavailable-authentication guidance readable and actionable without adding app-owned login or model-selection flows.
- [ ] Before adapter or fixture changes, select/record the supported Codex runtime version, inspect its official protocol/schema and retained evidence, and record schema/source revision and captured-versus-synthetic provenance beside fixtures.
- [ ] Ground observer/approval-validation assumptions in the selected official schema and retained local evidence, preserving exact operation/request/turn matching. Qualify the selected cohort explicitly; archived repository inspection is not required.
- [ ] Implement the concrete Codex stdio app-server adapter and deterministic fixture adapter at the same execution seam, using supported-version protocol behavior rather than guessed payloads.
- [ ] Expose connection/configuration observations and actionable missing-executable, incompatible-protocol, and unavailable-authentication errors in the desktop through core actions/observations.
- [ ] Preserve runtime-owned authentication/configuration; extract no tokens, change no login state, and silently substitute no billing route.
- [ ] Use metadata/schema inspection without agent sessions. Unit-test setup with schema-grounded frames and mocked process outcomes: version/exit failure, authentication/configuration, unrelated IDs, malformed/oversized frames, unexpected requests, EOF, timeout and cleanup uncertainty. Verify permitted metadata requests without starting threads/turns or changing login. Separately demonstrate desktop setup/error states; native compatibility remains unqualified until its authorized cohort.
- [ ] Record inspection sources, versions, checks, and results. Starting a real agent session is outside this ticket’s allowance.
- [ ] Separate protocol interpretation from process/stdio mechanics with small internal substitutions. Test deadline decisions using controlled time/events or immediate timeout outcomes; keep production deadlines intact and unit execution free of runtime processes or waits.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

### 2026-11-13 — accepted plan and documentation-only session split

The user accepted a full Settings screen with Agent runtimes/Codex, a small
application navigation module without a router, and one deep Rust runtimes module.
Keep workspace context across Settings; keep existing selection owners and fence
late action-driven selection. Codex retains authentication across app restarts;
setup diagnostics never become a prerequisite for normal later Send/Resume.

The [plan](../codex-setup-plan.md) records all agreed scope and skill-validation
corrections, including uncertain Save lookup, newer-draft retention, explicit
bounded worker/process cleanup, scoped documentation opening and visible copy
failures. [Navigation research](../../../docs/research/settings-navigation.md)
records official sources and the full-MVP router comparison. Shared supervisor
exchange consolidation is documented as a deferred opportunity for later tickets.
[ADR 0006](../../../docs/adr/0006-application-owned-desktop-navigation.md)
records the accepted navigation architecture under repository documentation policy.

Preparation on `main` from `b3f8cf00db6965786db6ace294a1a85fd7d01817`
creates 06a–06d and their handoffs only. No product code, runtime adapter, fixture,
credential, provider history or completed milestone evidence is changed. All
original parent acceptance items and all child implementation items remain
unchecked; no A3/native qualification or live submission is claimed.

Preparation verification: all relative links in the eight changed Markdown files
resolve, whitespace checks pass, and the original nine parent acceptance items
are unchanged and unchecked. Independent Standards and Spec review follow-ups
report zero open findings; the Standards review's ADR gap was resolved by ADR 0006.
Product units, the complete product verification gate, packaging and actual
desktop/process/native qualification were not run for this documentation-only
change. Existing commit hooks retain their formatting/lint checks.
