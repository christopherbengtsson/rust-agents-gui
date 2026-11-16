# Conductor SQLite private opening

2026-11-16 · Read-only installed-code inspection and tagged primary sources.

**Finding:** Conductor supplies concrete production evidence for SQLx through the
Tauri SQL plugin, and a native terminal-history helper requests mode 0600. This
inspection did not establish an equivalent to our SQLite `NOFOLLOW` opening
behavior, nor a complete alternative private-opening contract. It does not establish
that Conductor lacks protection or is unsafe.

## Provenance and local evidence

Inspected `/Applications/Conductor.app`, version/build **0.93.0**, identifier
`com.conductor.app`. Executable SHA-256 before and after inspection was
`062937acd521353151df361604d08af8d16b6ba687b6a6e74c67ff8a1e6d05e4`.
The decoded main bundle's hash matches the prior
[frontend provenance](../evidence/frontend-navigation/conductor-sources.json).
[Opening evidence](../evidence/conductor-private-storage-opening/sources.json)
retains hashes, offsets, symbols, instructions, inspection commands and limits.
Offsets below are zero-based; native addresses are executable virtual addresses.

| Observed evidence | What it establishes |
| --- | --- |
| Main bundle `plugin:sql\|load`, `execute`, `select` at character offsets 956197, 956310, 956440 (bytes 956213, 956326, 956456) | Frontend submits SQL-plugin operations. |
| `sqlite:conductor.db` at character 956591; `sqlite:cache.db` at 956617; `get_db_uri` at 956680 | Default constants exist, but application initialization obtains its URI from native code. The constants do not establish the effective data path. |
| `PRAGMA journal_mode = WAL` at characters 956748 and 957483 | Application/cache initialization requests WAL; active database state was not inspected. |
| Executable paths `tauri-plugin-sql-2.4.0/src/wrapper.rs` at byte 46775221 and `commands.rs` at 46776806; `sqlx-sqlite-0.8.6/src/connection/worker.rs` at 45370677; `sqlx-core-0.8.6/src/rt/mod.rs` at 46776709 | Compiled dependency version labels identify a useful upstream comparison cohort. They do not prove an unmodified upstream build. |
| `conductor_lib::terminal_history_storage::set_private_file_permissions`, address `0x1007b5d84`: `mov w2, #0x180` at `0x1007b5da0`, followed by a call to `std::sys::fs::set_permissions` | A native helper requests 0600 for its supplied path. Its coverage of the main database, journal/WAL/SHM, and pool reopens was not established. |
| Terminal-history `resolve_db_path` at `0x1007b18a4` accepts an absolute SQLite path or joins `app_data_dir`; that helper at `0x1007b15a8` uses Tauri's path resolver | Terminal-history path handling exists. These inspected helpers do not recover the main `get_db_uri` implementation or establish directory permissions. |

Conductor was not launched. No databases, user support/configuration directories,
credentials, provider history or RAT were accessed. No app files were modified.
This extends the earlier [frontend architecture research](conductor-frontend-architecture.md#query-and-zustand-responsibilities).

## What the matching upstream sources do

In stock **tauri-plugin-sql 2.4.0**, `load` passes the supplied URI to
`DbPool::connect`. Its SQLite branch resolves `app_config_dir`, calls ordinary
`create_dir_all`, maps the URI, checks/creates the database and connects a SQLx
pool. `path_mapper` pushes the text after `sqlite:` onto the application path.
This path contains no explicit owner check, permission mode, canonical-parent
validation or symlink rejection. Application-directory placement alone therefore
does not document those guarantees. This describes the tagged plugin, not all
Conductor startup code. [Load command](https://github.com/tauri-apps/plugins-workspace/blob/sql-v2.4.0/plugins/sql/src/commands.rs#L11),
[connection and path mapping](https://github.com/tauri-apps/plugins-workspace/blob/sql-v2.4.0/plugins/sql/src/wrapper.rs#L61).
The [plugin manifest](https://github.com/tauri-apps/plugins-workspace/blob/sql-v2.4.0/plugins/sql/Cargo.toml#L29)
requests SQLx 0.8; the installed executable supplies the more specific 0.8.6 labels.

In stock **SQLx 0.8.6**, SQLite opening constructs URI, mutex, read/write/create,
memory and cache flags, then calls `sqlite3_open_v2`. It does not add
`SQLITE_OPEN_NOFOLLOW`. Its public options expose no NOFOLLOW/raw-opening-flags
override; its URL parser accepts `mode`, `cache`, `immutable` and `vfs`, with no
NOFOLLOW parameter. [Flag construction](https://github.com/launchbadge/sqlx/blob/v0.8.6/sqlx-sqlite/src/connection/establish.rs#L54),
[options](https://github.com/launchbadge/sqlx/blob/v0.8.6/sqlx-sqlite/src/options/mod.rs),
[URL parser](https://github.com/launchbadge/sqlx/blob/v0.8.6/sqlx-sqlite/src/options/parse.rs#L33).

SQLite documents NOFOLLOW as rejecting a database filename containing a symbolic
link. `cache=private` concerns SQLite cache sharing, not filesystem permissions.
A VFS selector chooses an implementation; selecting one alone proves no alias
policy. SQLx's raw handle is available after opening and cannot retroactively
change that opening. [SQLite opening contract](https://www.sqlite.org/c3ref/open.html),
[SQLx handle access](https://github.com/launchbadge/sqlx/blob/v0.8.6/sqlx-sqlite/src/connection/mod.rs#L166).
The separately evaluated SQLx 0.9.0 has the same unresolved supported-opening gap;
see the [async comparison](storage-async-architecture-comparison.md#private-opening-contract-versus-implementation).

## Unknowns and implication for this app

The main native URI implementation, complete startup/open/reopen path, effective
directory/file modes, owner checks, alias checks, dependency patches and any custom
VFS remain unestablished. No literal NOFOLLOW marker was found in the searched
resources, but compiled flags can be numeric: that absence is not proof of absence.
The 0600 terminal-history helper is positive evidence of a privacy mechanism;
permissions and symlink rejection address different properties.

**Assessment:** Conductor does not resolve our SQLx opening gate on the available
evidence. Our [MVP](../MVP.md#starting-architecture) requires private local state and
supervisor-owned persistence; it does not mandate matching another app or invent
an arbitrary same-user attacker model. The existing
[private-opening assessment](storage-async-architecture-comparison.md#private-opening-contract-versus-implementation)
records the deliberate alias rejection and flags to preserve or explicitly reassess.
A driver choice needs an explicit application contract and evidence for the chosen
opening path, including reopens. This research changes no ticket, implementation
or driver decision and qualifies no security or performance result. No tests,
benchmarks, desktop qualification or native-provider checks were performed.
