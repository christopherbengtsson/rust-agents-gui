# Native core and direct SQLx

Accepted 2026-11-16; implementation and qualification remain pending. Host the
UI-independent Rust core in native Tauri and use SQLx directly inside Rust. This
deletes the application supervisor, discovery/socket/peer protocol and staging
lifetime while retaining Rust domain authority and one private-state lease.

The selected lifetime preserves work through navigation, hidden windows and WebView
replacement; native failure requires durable interrupted/uncertain recovery rather
than uninterrupted execution. Explicit Quit completes owned cleanup/persistence.
Uncapped product cardinality uses bounded resource admission and exact conflict
coordination, including deliberately shared workspaces. SQLx opening uses the accepted
private-directory validation contract with its same-user replacement-race limitation;
it does not promise atomic SQLite NOFOLLOW equivalence. Read-only split pools and
storage-owned failed-transaction retirement keep ordering and failure facts local.

The separate host's crash-survival advantage no longer earns its coordination cost
under this selected lifetime. Frontend SQL/plugin ownership would spread durable
action policy and add another lifecycle; a retained rusqlite bridge would preserve
opening flags but is superseded by the explicit SQLx/private-opening choice. Existing
benchmarks establish no performance winner. A later Claude bridge remains a provider
adapter, never a replacement application storage/execution owner.

The [final spec](../../.scratch/sqlx-architecture/spec.md) and
[amendments](../../.scratch/sqlx-architecture/amendments.md) scope supersession of
MVP/milestone, frontend precedence, ADRs 0005–0007, provider setup and storage
recommendations. Earlier decision/results remain historical evidence; ADRs 0003/0004
retain their identity and durable-continuation decisions. Future PR/review/automation
authority and schemas remain feature work.
