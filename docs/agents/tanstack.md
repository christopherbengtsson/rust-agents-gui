# TanStack guidance

Use the pinned local CLI: `corepack pnpm exec intent list`, then
`corepack pnpm exec intent load <package>#<skill>`. Repository policy has precedence.
Package upgrades also upgrade guidance: review skill content and the exact allowlist
in package.json with the dependency diff. Keep Intent and route generation tooling
in development dependencies.

## Task mapping

| Task                                           | Installed guidance                                                          |
| ---------------------------------------------- | --------------------------------------------------------------------------- |
| Router concepts and route tree                 | `@tanstack/router-core#router-core`, then the relevant allowed sub-skill    |
| React Router bindings                          | `@tanstack/react-router#react-router`                                       |
| File generation and splitting                  | `@tanstack/router-plugin#router-plugin` and core `code-splitting`           |
| Paths, search, links/history                   | Core `path-params`, `search-params`, or `navigation`                        |
| Route errors/type inference                    | Core `not-found-and-errors` or `type-safety`                                |
| Query/Router coordination                      | `@tanstack/react-router#compositions/router-query` plus core `data-loading` |
| Query-only observations, caching, invalidation | Installed Query declarations/source and the official references below       |

`@tanstack/react-query` and `@tanstack/query-core` 5.107.1 ship no standalone Intent
skills. The reviewed composition skill supplies Query integration guidance from
the Router package. Do not substitute an invented skill ID or claim a missing
Query skill was loaded. Recheck discovery on upgrades. SSR, auth, React Router
migration, and virtual-route guidance is outside the current allowlist.

## Application constraints

The [agreed foundation](../../.scratch/frontend-navigation/spec.md) owns requirements.
F01 pinned dependencies/guidance; F02–F04 implemented owner-bound contracts, admission
and protocol-7 push without Query adoption. F04a publishes the amended plan only.
F04e/F04f implement complete feature observation/freshness/polling replacement;
F05 finishes presentation persistence/integration and F06 introduces routes. Follow
[observation engineering](observations.md) and predecessor evidence before assuming
planned commands/modules exist. The target ownership is:

- Query for validated owner/target-scoped observations; Router for current navigation;
  Zustand for selected presentation memory; Rust for execution, recovery and storage.
- Local IPC uses `networkMode: "always"`, disabled automatic retries/focus/reconnect
  refetches, explicit freshness and bounded retention. Query success is a client
  request result, not provider receipt or execution completion.
- Effects, route loaders and preloading retrieve observations. Execution remains an
  explicit targeted action. An abandoned promise does not cancel physical IPC.
- Keep one app-lifetime physical inspection owner and feed above routes. XState owns
  their coordination; Query owns observations and Zustand owns presentation memory.
- Feature observation modules hide keys, decoding, semantic assertions, dirty
  generations and interests. Route loaders use those interfaces and express no
  execution intent. Keep Query as freshness authority; set Router's
  defaultPreloadStaleTime to 0 and avoid a second long-lived loader result cache.
- Query cancellation/refetch does not prove IPC completion. The shared p-queue
  module/physical registry drains actual work independently; running abort/timeout
  must not free capacity early. Feature freshness satisfies captured generations,
  leaving hints received during a read pending.
- Preserve repository verification/dependency substitution and component contracts.

The shipped composition skill discusses SSR and generic loader/preloading patterns.
Apply only the relevant client-side guidance under these contracts. Its
`QueryClient.query` examples exist in the pinned Query 5.107.1 source; older Query
examples may instead use now-deprecated `fetchQuery`/`ensureQueryData`. Check installed
exports and types before implementation.

## Primary references

- [Intent overview](https://tanstack.com/intent/latest/docs/overview)
- [Intent consumer setup](https://tanstack.com/intent/latest/docs/getting-started/quick-start-consumers)
- [Query defaults](https://tanstack.com/query/latest/docs/framework/react/guides/important-defaults)
- [Query network mode](https://tanstack.com/query/latest/docs/framework/react/guides/network-mode)
- [Query keys](https://tanstack.com/query/latest/docs/framework/react/guides/query-keys)
- [Query cancellation](https://tanstack.com/query/latest/docs/framework/react/guides/query-cancellation)
- [Query render behavior](https://tanstack.com/query/latest/docs/framework/react/guides/render-optimizations)

Packaged skill hashes and versions are retained in
[the F01 evidence](../evidence/frontend-navigation/intent-skills.json).
