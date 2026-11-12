# 02a: Establish the incremental UI foundation

**What to build:** Replace the centered supervisor diagnostics page with the first lasting desktop application shell, preserving the working supervisor status connection. Establish a compact dark Conductor-inspired visual baseline that repository navigation, workspace chat and change inspection can extend incrementally.

**Blocked by:** 02: Reliable supervisor discovery and connection

**Status:** ready-for-agent

**Context:** Read the [parent spec](../spec.md), [product contract](../../../docs/MVP.md), [milestone](../../../docs/MILESTONE-01.md), [testing policy](../../../docs/agents/verification.md), [React decision](../../../docs/adr/0001-react-presentation-with-vite.md), and [component foundation decision](../../../docs/adr/0002-shadcn-base-ui-presentation-components.md). Use the repository-local [shadcn skill](../../../.agents/skills/shadcn/SKILL.md) under repository policy. The [Conductor UI research](../../../docs/research/conductor-incremental-ui-foundation.md) provides official visual/workflow references.

**Coverage:** Presentation foundation for A1 and subsequent A3–A10 views; preserves the completed ticket 02 connection behavior. This inserted ticket runs before ticket 03 without renumbering existing tickets.

- [ ] Adopt React with TypeScript in the existing Vite/pnpm/Tauri setup. Add official shadcn/ui components backed by Base UI, Tailwind and shared theme variables only as this working shell needs them; record resolved stable versions and generated-source provenance and retain lockfiles.
- [ ] Establish compact dark typography, spacing, surfaces, borders, focus and semantic status styling using Conductor's documented desktop workspace as the visual reference. Keep common styles and repeated controls consistent without a general component-framework or docking-system project.
- [ ] Provide a lasting application shell with repository navigation and main-content regions plus a compact supervisor connection indicator. Structure the composition so later tickets can add workspace identity, chat/composer and change inspection in their intended locations.
- [ ] Show an honest initial empty state and only working actions. Preserve supervisor status retrieval, refresh/retry feedback, error details and reconnection behavior through existing typed actions/observations; keep process/version/owner diagnostics available without making them the main product composition.
- [ ] Keep execution, storage ownership, retries and reconciliation in their existing Rust modules. React owns presentation concerns such as selection, layout and focus; mounting or remounting views must not gain authority to repeat uncertain work.
- [ ] Create and retain three project-specific visual-reference artifacts (screenshots or static mockups) showing this application's empty shell, selected-workspace arrangement, and conversation with change inspection. Each artifact must show the proposed application composition and shared visual treatment; Conductor source screenshots alone do not satisfy this criterion. Record repository-relative artifact paths in the ticket's completion evidence.
- [ ] Label each visual-reference artifact as implemented or proposed, and identify planned content within mixed views. Future workspace/chat/diff content is design-only in this ticket; repository registration remains ticket 03 and later functional capabilities remain in their owning tickets.
- [ ] Check keyboard focus, readable error/connection states and layout at the application's configured minimum and ordinary window sizes in the actual Tauri desktop under its existing CSP. Record screenshots and observed results separately from unit checks; do not assume browser/library claims qualify the webview composition.
- [ ] Keep automated checks proportional and follow the unit-only testing policy. Where presentation behavior needs regression coverage, supply mocked IPC/dependency observations; introduce no automated integration suite or real-process unit dependencies. Preserve the existing verification commands and avoid unnecessary reruns.
- [ ] Record build/static/unit results and desktop qualification evidence with source/version/profile context. Do not claim streaming/history performance targets pass before representative workloads exist, and do not make live agent submissions.

**Out of scope:** Repository registration/storage, workspace creation, functional chat/diff/approval controls, light-theme qualification, a full component catalog, terminal/PR/cloud features, and general-purpose UI infrastructure. These do not prevent the shell and tokens from being extended later.

## Comments

The user confirmed a compact dark Conductor-inspired baseline and a separate UI-foundation ticket before repository registration. Tickets 01–02 remain completed evidence; ticket 03's already completed verification-gate item is preserved.
