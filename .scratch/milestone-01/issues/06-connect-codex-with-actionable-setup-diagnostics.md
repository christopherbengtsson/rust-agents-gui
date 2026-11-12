# 06: Connect Codex with actionable setup diagnostics

**What to build:** Provide Codex connection/setup behavior through the desktop and establish the real and deterministic execution adapters against a recorded supported protocol cohort.

**Blocked by:** 02: Reliable supervisor discovery and connection

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Follow [testing and verification](../../../docs/agents/verification.md) for automated coverage and skill constraints. Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** A3 deterministic implementation; protocol/fixture prerequisite for A4–A5 and A7–A9.

- [ ] Before adapter or fixture changes, select/record the supported Codex runtime version, inspect its official protocol/schema and retained evidence, and record schema/source revision and captured-versus-synthetic provenance beside fixtures.
- [ ] Ground observer/approval-validation assumptions in the selected official schema and retained local evidence, preserving exact operation/request/turn matching. Qualify the selected cohort explicitly; archived repository inspection is not required.
- [ ] Implement the concrete Codex stdio app-server adapter and deterministic fixture adapter at the same execution seam, using supported-version protocol behavior rather than guessed payloads.
- [ ] Expose connection/configuration observations and actionable missing-executable, incompatible-protocol, and unavailable-authentication errors in the desktop through core actions/observations.
- [ ] Preserve runtime-owned authentication/configuration; extract no tokens, change no login state, and silently substitute no billing route.
- [ ] Use metadata/schema inspection without agent sessions. Unit-test setup with schema-grounded frames and mocked process outcomes: version/exit failure, authentication/configuration, unrelated IDs, malformed/oversized frames, unexpected requests, EOF, timeout and cleanup uncertainty. Verify permitted metadata requests without starting threads/turns or changing login. Separately demonstrate desktop setup/error states; native compatibility remains unqualified until its authorized cohort.
- [ ] Record inspection sources, versions, checks, and results. Starting a real agent session is outside this ticket’s allowance.
- [ ] Separate protocol interpretation from process/stdio mechanics with small internal substitutions. Test deadline decisions using controlled time/events or immediate timeout outcomes; keep production deadlines intact and unit execution free of runtime processes or waits.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.
