# 01: Bootstrap and verified startup

**What to build:** Establish a reproducible development baseline that opens a minimal desktop window and exchanges status with a separately launched supervisor before implementing product behavior.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), and [milestone acceptance and verification requirements](../../../docs/MILESTONE-01.md). Build lasting modules under their existing ownership; keep policy in Rust and presentation thin.

**Coverage:** Bootstrap; user stories 1–2. This is the agreed setup exception to vertical product slicing.

- [x] Select the latest compatible stable development tools and dependencies at execution time; record resolved versions, platform, scaffold generator version/selections if used, and commit dependency lockfiles.
- [x] Use the agreed TypeScript presentation/pnpm and Rust/Cargo tooling. Record the chosen frontend framework without making product ownership depend on it.
- [x] Build and open a minimal Tauri window; launch the supervisor as a separate local process and demonstrate a successful connection/status exchange.
- [x] Keep the supervisor/core arrangement independent of Tauri presentation state; introduce only the code needed for startup and status.
- [x] Provide reproducible development-start, build, formatting, and static-check commands; record actual commands and results, including observed desktop startup.
- [x] Leave repository registration, persistence, chat, and recovery behavior to subsequent tickets. Make no live agent submissions or authentication changes.

**Qualification limit:** Use deterministic fixtures and private test state for execution checks. This ticket does not authorize live agent submissions; native results are recorded separately under the agreed cohort.

## Comments

2026-11-12: Implemented and startup verified. See [bootstrap evidence](../../../docs/evidence/milestone-01-bootstrap.md) for the toolchain, commands, screenshot, and qualification limits. Overall milestone qualification remains incomplete.

2026-11-12: This ticket predates `docs/agents/verification.md`. Its completion evidence remains historical evidence. Current commands, unit coverage and qualification separation are recorded in [policy-alignment evidence](../../../docs/evidence/milestone-01-verification-policy-alignment.md).
