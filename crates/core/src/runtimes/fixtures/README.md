# Codex setup protocol cohort

Runtime cohort: **Codex 0.167.2**. Official source revision:
`ce21072c90d32c40af7a55b60b2fb35c84a43a6a`.

`schema/` retains selected official generated TypeScript schema files from
`codex-rs/app-server-protocol/schema/typescript/` at that revision. `rpc.rs.txt`
retains `codex-rs/app-server-protocol/src/rpc.rs`. These are reference artifacts,
not application-generated types. `SHA256SUMS` records their retained bytes and
the synthetic success transcript. The upstream schema JSON subtree is
`6f1d5728dcc0197cfa772ebf2f78d1116906c7be`; the TypeScript subtree is
`7b4fbbf3a6ddf652fd07db8384e405fceb175c5c`.

Source: [pinned official protocol](https://github.com/openai/codex/tree/ce21072c90d32c40af7a55b60b2fb35c84a43a6a/codex-rs/app-server-protocol).
Files were retrieved through the GitHub connector before adapter implementation.
No local Codex executable, schema-generator process, login or model run was used.

`success.jsonl` is **synthetic**, derived from those schemas. Its account/config
values are invented. All fixture-mode faults are synthetic application-recovery
inputs. They establish no native provider outcome. Schema reference fields not
needed by setup are deliberately absent from this minimal transcript.

The retained RAT standalone proof was inspected read-only following its AGENTS.md
and required documents. It describes Codex **0.167.1**, has incomplete approval
payload evidence and does not establish 0.167.2 compatibility or escaped-descendant
cleanup. No archived source, state, credentials, provider history or ledger changed.

Provider request identity is `String(String) | Integer(i64)`; `"1"` and `1` are
different. Approval assumptions retain exact request/thread/turn/item identity
and optional distinct `approvalId`, with no string/number coercion or permissive
matching. Setup never grants approvals: any server request fails closed. Later
conversation tickets must implement exact operation matching independently.

The concrete fixture adapter is selected only by an explicit supervisor environment
variable; see the [private-state instructions](../../../../../docs/evidence/milestone-01-codex-setup/fixture-instructions.md).

06b recovery correction adds `lease-version` and `lease-metadata`: synthetic lease
acquisition failures before the version or app-server launch, respectively. They
supply the same `CleanupUnconfirmed` startup outcome as native lease contention;
production check policy must retain uncertainty without a quarantined handle.
`timeout-cleanup` combines a synthetic metadata timeout with unconfirmed cleanup;
its primary timeout must survive while owned handles remain quarantined.
The pinned schema/source and success transcript are unchanged. These faults prove
application recovery decisions, not native lock or descendant-cleanup behavior.
