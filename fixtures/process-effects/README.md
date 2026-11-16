# Process-effect fixtures

Synthetic Python standard-library OS scenarios for ticket 05, authored 2026-11-16.
They implement no provider protocol, use no authentication, and contain no captured
runtime messages. Codex protocol fixtures remain separately sourced in
`fixtures/codex-readiness/README.md` (0.169.0 / d32063f96c2dff1dc96af839b3952abb6acfeda4).

Use only with a new disposable private root and the feature-gated
`qualify-process-effects` binary. Files under that root coordinate bounded harness
observations and voluntary escaped/orphan cleanup; recorded PIDs are observation
only and grant the product no signaling authority.
