# Conductor frontend and our observation foundation

2026-11-14 · F01 · Read-only installed-code research and architecture decisions.

## Evidence and limits

Inspected `/Applications/Conductor.app` version/build **0.93.0**, identifier
`com.conductor.app`. Its arm64 executable embeds Brotli-compressed frontend assets.
Selected assets were decoded to private temporary storage during planning. No
Conductor sessions were started, credentials/history/databases accessed, or app
files modified. The app was not exercised to qualify the inferred UI behavior.

[Source provenance](../evidence/frontend-navigation/conductor-sources.json) records
paths, decoded sizes and SHA-256. Main entry `main-B00nPG1d.js` lazy-loads
`main-CkPOABPb.js`, then `renderApp-Ec9d1ssb.js`. Bundles reference source maps, but no
maps were available. Symbol names/character offsets below locate shipped code;
original component names, folders, dependency versions and file-based/code-based
authoring cannot be recovered reliably from that evidence.

## Observed Conductor architecture

| Concern | Shipped evidence |
| --- | --- |
| Router | TanStack Router behavior: `getParentRoute`, `addChildren`, `beforeLoad`, `validateSearch`, `__TSR_key/index`, `onResolved`; router `PY` at renderApp ~12,436,652 |
| History | No explicit history override; bundled default uses browser pushState/replaceState/popstate |
| Root | Query client, theme, tooltip/highlighting and global observation/dialog infrastructure surround routed content |
| Layouts | Settings and pathless main-layout are siblings under the authenticated subtree; main shell `JLa`, settings shell `L1l` |
| Workspace URL | `/repository/$repositoryId/workspace/$localWorkspaceId`; validated search selects session/terminal/special view/git panel |
| Settings URL | Account action `YOt` always targets constant `idi="/settings/user/general"` |
| Settings presentation | Full-page sidebar/outlet; route-derived section, main scroll container keyed by pathname; Back/Escape uses history or `/` |
| Account trigger | Top navigation, not footer, in this installed version; organization selection, Settings and Log out |

Main workspace search schema is in main ~935,062–938,000. Store
`workspace-tab-state-v2` (~3,281,000) retains last workspace routes and per-workspace
active search. Router onResolved stores validated navigation. Root launch validates
the saved workspace before restoration. Direct missing workspace navigation renders
retryable missing/access/loading errors (`k8a`, renderApp ~10,543,170). Startup
restoration clears its saved route on any lookup exception or an archived workspace.
The inspected route chain does not establish repository/workspace membership
validation; our plan explicitly requires it. These are source observations, not
runtime-tested guarantees about every global Conductor guard.

Conductor's generated settings deep links include HTTPS webapp settings URLs and
native `conductor://settings/...` handling. Info.plist registers its scheme. This
establishes code/configuration, not a qualified native handoff in our app.

## Query and Zustand responsibilities

Conductor uses TanStack Query and Zustand. Main ~631,081 identifies QueryClientProvider;
main ~953,216 creates the shared Query client with retry:false, networkMode:always
and staleTime:Infinity. Individual resources override refetch policy. Examples:
DB compatibility metadata via native invoke, file stat/listing queries, and
checkpoint notifications calling setQueryData/invalidateQueries (renderApp
~12,488,813). No Query persistence provider was found in the inspected bundles;
this is not proof that Conductor has no durable derived caches elsewhere.

Main ~274,734 identifies Zustand persist middleware and vanilla store implementations.
Zustand holds presentation records and some observable domain/service projections,
not only UI flags. The `workspace-tab-state-v2` persist middleware defaults to JSON
localStorage and partializes retained fields. Workspace center-panel layout is
persisted separately. File viewport maps are session memory keyed by workspace/path;
transcript positions use session IDs in a non-persisted store. Durability remains a
separate concern: inspected session ingestion persists through DB dependencies before
notifying observers.

A sidecar JSON-RPC bridge uses Tauri `rpc_to_sidecar` and `rpc_from_sidecar` (main
~2,351,634). This is reference evidence for observation paths; our MVP explicitly
keeps execution and storage in the supervisor's Rust core.

Follow-up static inspection confirmed SQL-plugin calls in main ~956,197:
`plugin:sql|load`, `plugin:sql|select`, and `plugin:sql|execute`. The shipped default
application URI is **`sqlite:conductor.db`**, alongside `sqlite:cache.db` (~956,617),
rather than `app.db` in this build. Application initialization obtains its URI from
`get_db_uri`; therefore the default constant does not establish the actual user-data
path. Application initialization `Hnt` and cache initialization `qnt` both request
`PRAGMA journal_mode = WAL` (~956,755 onward). This establishes configuration code,
not the active mode or contents of an opened database. No user databases were read.
The official [SQL plugin documentation](https://v2.tauri.app/plugin/sql/) identifies
SQLx as its backing implementation. Driver, WAL and layout evaluation remains
milestone 06a, after 06 and before durable chat.

### Comparison with our process architecture

Our process path is React presentation → Tauri desktop bridge → separate supervisor
hosting the Rust core → official runtimes. The core is a library hosted by the
supervisor, not another process upstream of it.

| Concern | Conductor shipped-code evidence | Our accepted contract |
| --- | --- | --- |
| Presentation | Router, Query and Zustand with frontend services | Same library roles, with feature observation interfaces |
| SQL | Frontend select/execute through Tauri SQL plugin; default application/cache URIs and WAL setup | Storage operations remain inside supervisor-hosted Rust core |
| Runtime plumbing | JSON-RPC forwarded through Tauri to a sidecar and notifications back to frontend | Typed local supervisor connection; runtimes execute under core ownership |
| Failure/lifetime guarantees | Equivalent supervisor exclusivity and desktop-crash recovery were not qualified by bundle inspection | One storage/execution owner; desktop reconnect reads observations without replay |

We adopt the navigation/layout patterns and preserve our explicit process and
storage ownership. The additional local process seam has startup/IPC/recovery cost;
this review establishes no measured reliability or performance advantage over
Conductor's private implementation.

## Our chosen architecture and tradeoffs

Router owns the current destination; Query owns validated asynchronous observation
caches; Zustand retains selected presentation context. Rust owns authoritative
records and execution/reconciliation. The [ADR](../adr/0006-routed-observation-presentation.md)
and [full plan](../../.scratch/frontend-navigation/spec.md) own the decision.

Zustand's keyed records/actions fit per-workspace context directly. Jotai is a valid
alternative when independently derived presentation state becomes a substantial atom
graph. Neither is an established performance winner here. Both need explicit store
lifetime, stable identities, validation and bounded retained data. Their official
sources describe [Zustand scoped stores](https://zustand.docs.pmnd.rs/learn/guides/initialize-state-with-props),
[Zustand persistence](https://zustand.docs.pmnd.rs/reference/integrations/persisting-store-data),
[Jotai stores](https://jotai.org/docs/core/store) and
[Jotai derived atoms](https://jotai.org/docs/core/atom).

Query accepts asynchronous functions, so local IPC benefits from shared results,
loading/error state, deduplication and targeted invalidation. Future GitHub reads
also fit. Costs include cache retention, observer/timer work, structural comparison,
and potentially redundant IPC if invalidation is broad. Configure local connectivity
and freshness deliberately. See [Query defaults](https://tanstack.com/query/latest/docs/framework/react/guides/important-defaults),
[network mode](https://tanstack.com/query/latest/docs/framework/react/guides/network-mode),
[query keys](https://tanstack.com/query/latest/docs/framework/react/guides/query-keys),
[cancellation](https://tanstack.com/query/latest/docs/framework/react/guides/query-cancellation),
and [render behavior](https://tanstack.com/query/latest/docs/framework/react/guides/render-optimizations).

Tauri invoke cancellation does not establish physical work cancellation. Retain the
existing deep inspection reader and move its lifetime above routed pages. Keep
current-target selection separate from pending action outcomes. A late registration
currently uses selection equality, while workspace creation uses a navigation
generation; Router integration must unify that protection, including A→B→A and
Settings transitions. These are static source findings, not newly executed failures.

## Cross-stack seams

- Core's global owner/revision is coarse; existing frontend Hooks receive owner
  alone. Target-aware push must cover admissions/fault overlays as well as existing
  revision sites. Cache invalidation must not refetch every resource on every change.
- Tauri currently holds one mutex across independent socket exchanges. Bounded
  native lanes with short state locks are planned; no measured latency defect is
  claimed by this review.
- Feature requests currently connect the active state socket without caller-owner
  binding. Query keys alone cannot stop replacement-owner replies contaminating
  an older scope. F02 adds request/reply owner validation before dispatch/cache.
- Repository/workspace replies have runtime guards; supervisor snapshots currently
  trust a generic invoke type. F02 adds consistent decoding and a serializer corpus.
- File-based routing introduces generated inputs. Generation/drift checking must
  precede tsc through one preparation owner, rather than relying on a prior Vite run.

Core reference: `crates/core/src/lib.rs`, `connection.rs`, and workspaces modules.
Frontend reference: `App.tsx`, use-supervisor-connection/use-repositories/use-workspaces,
WorkspacePanel and WorkspaceChangesReader. These are the pre-foundation baseline.

Supported Tauri 2.12.5 asset resolution falls back to root index.html after exact,
.html and directory-index lookups (`src/manager/mod.rs`); this supports the choice of
browser history. It does not persist route selection on app relaunch or qualify
WebKit reload. See [Router history](https://tanstack.com/router/latest/docs/guide/history-types),
[Vite integration](https://tanstack.com/router/latest/docs/installation/with-vite),
[Tauri channels](https://v2.tauri.app/develop/calling-frontend/#channels), and
[native deep links](https://v2.tauri.app/plugin/deep-linking/).

## F01 preparation boundary

Pinned packages and reviewed skill hashes are in the F01 evidence directory.
No app providers, route generation wiring, Rust protocol, journal, scheduling or UI
behavior is implemented in F01. Intent's installed Query package ships no standalone
skill; the allowlisted Router/Query composition plus official Query sources cover
that gap. Skills are conditional guidance under repository policy, not runtime
requirements. See [local task mapping](../agents/tanstack.md) and
[Intent consumer model](https://tanstack.com/intent/latest/docs/overview).

The production baseline records actual emitted bytes, not npm package-size badges.
F08 must measure initial/lazy/total bundles, packaged size, latency, IPC volume and
retention under the same workload. No performance improvement, new desktop
qualification, native deep links or provider compatibility is claimed by F01.
