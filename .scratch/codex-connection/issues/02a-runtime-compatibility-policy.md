# 02a: Provider runtime compatibility policy

**What to build:** Admit Codex stable releases from 0.169.0 using required protocol
checks, preserve the actual observed version, and give browser and future CLI
login the same compatibility policy.

**Blocked by:** [02 — Official browser sign-in](02-official-browser-sign-in.md)

**Status:** ready-for-agent

## Implementation

Keep version parsing, required capability checks and provider decoding in the
concrete Codex adapter. Preserve the provider-connection owner's admission,
reconciliation, stale-result rejection and physical cleanup. Future providers
follow their own documented contracts under the policy in the MVP; this increment
adds no provider framework or CLI/PTY implementation.

Accept `codex-cli <semver>` with LF/CRLF, valid build metadata, no prerelease,
and version >= 0.169.0, without an upper version limit. Reject malformed output.
Use the locked semver crate. Carry the detected version into SetupEvidence.
Share initialize/initialized and successful account/gatewayOAuth/read probing
between readiness and browser login on every new connection, before subsequent
authenticated requests. Probe success confirms support, not authentication.
Unsupported/malformed probes are incompatible protocol; other RPC/transport/time
failures remain status-read failures. Never fall back to automatic login.

Rename ProviderTarget.supported_version to minimum_supported_version; regenerate
contracts and use protocol 13 with matching peers/fresh private development state.
Technical details distinguish minimum and observed runtime; recovery guidance
allows newer stable releases without implying native qualification.

## Acceptance

- [ ] Minimum stable policy and truthful observed version work through readiness and browser verification.
- [ ] Both paths prove required capability support before dependent requests, preserving correlation, cancellation, deadlines and owned cleanup.
- [ ] Typed contracts, contextual validation and desktop guidance communicate minimum versus observed version accurately.
- [ ] Deterministic units cover version policy, probe success/failures, additive messages, blocked effects and preserved owning-module behavior.
- [ ] Synthetic fixture process and focused packaged-desktop checks cover newer readiness, probe recovery and browser completion/reverification; record native limits separately.
- [ ] MVP/spec record the provider policy; 03 is blocked by this prerequisite; prior evidence and user edits are preserved.
- [ ] Generated/static/type/lint/format/unit gates, packaging and independent Standards/Spec review are recorded with failures and unperformed limits.

## Comments

2026-11-15: Approved implementation plan from the provider compatibility research
and architecture review. Initial minimum is 0.169.0; newer stable releases are
allowed after required checks; prereleases remain unsupported. Qualification is
fixture-only: no real authentication, credential changes or native agent work.
