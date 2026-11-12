# 06: Connect Codex with actionable setup diagnostics

**What to build:** Provide Codex connection/setup behavior through the desktop and establish the real and deterministic execution adapters against a recorded supported protocol cohort.

**Blocked by:** 02: Reliable supervisor discovery and connection

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A3 deterministic implementation; protocol/fixture prerequisite for A4–A5 and A7–A9.

- [ ] Before adapter or fixture changes, select/record the supported Codex runtime version, inspect its official protocol/schema and retained evidence, and record schema/source revision and captured-versus-synthetic provenance beside fixtures.
- [ ] Follow the archived repository guidance before inspecting its proof evidence; preserve its files, Git state, credentials, ledgers, and native conversations. Repair retained observer/approval-validation assumptions without weakening exact operation matching.
- [ ] Implement the concrete Codex stdio app-server adapter and deterministic fixture adapter at the same execution seam, using supported-version protocol behavior rather than guessed payloads.
- [ ] Expose connection/configuration observations and actionable missing-executable, incompatible-protocol, and unavailable-authentication errors in the desktop through core actions/observations.
- [ ] Preserve runtime-owned authentication/configuration; extract no tokens, change no login state, and silently substitute no billing route.
- [ ] Use metadata/schema inspection without agent sessions. Verify connection/setup behavior with protocol fixtures, including the actual supervisor/fixture-process seam; label native compatibility as unqualified until the authorized native cohort.
- [ ] Record inspection sources, versions, checks, and results. Starting a real agent session is outside this ticket’s allowance.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
