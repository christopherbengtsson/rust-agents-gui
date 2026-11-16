# Current application format

2026-11-16. Code: `9bb70a9`, with review fixes in `388cd6c`. Baseline:
`a00a243259ff238bbb73e8dd9c0682572b72a9d1`.
[Accepted scope](../../../.scratch/current-format-cleanup/spec.md).

## Result

One current schema, default namespace and presentation revision: **1**. Production
accepts complete current pending-action envelopes and registration records. There
is no missing-field reconstruction, old database filename handling, selection
migration, migration ladder, or alternate application host/driver. Unreadable
pending evidence preserves bytes, exposes the earlier outcome as unknown, and
fences new submissions through both hooks and controls. Storage initializes an
empty root (apart from its lease) or opens valid current metadata; missing metadata
in a nonempty root is an actionable error.

The repository-wide source sweep traced defaults, aliases, readers, schema opening,
IPC contracts, tests and active guidance. Retained optional omissions/refresh
inputs, error recovery and provider capability/version handling serve current
behavior. Syntax aliases and generated TypeScript aliases are not retired-format
support. The old unselected supervisor fixture driver, numbered observation
corpora, source schemas and completed checkpoint reports remain explicitly
historical; current entry points do not select them.

## Coverage and checks

| Check                                         | Result                                                                                                            | Evidence                                                  |
| --------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| Complete static/unit gate, after review fixes | Passed: 172 Rust, 48 tooling, 263 frontend tests                                                                  | [log](verify.log), [result](verify-result.json)           |
| Actual current SQLx/storage qualification     | Passed: 17 cases, including missing-metadata rejection and same-build reopen                                      | [report](storage.json), [log](storage.log)                |
| Fixture-only native package                   | Built                                                                                                             | [build](native-build.log)                                 |
| Actual native lifecycle qualification         | Passed: registration/creations, same-build recovery, WebView replacement, ownership and Quit/failed-cleanup cases | [report](native.json), [log](native.log)                  |
| Normal production package                     | See recorded exit status; no qualification environment/features                                                   | [build](package-build.log), [result](package-result.json) |
| Independent Standards and Spec reviews        | Zero remaining findings after two Standards fixes                                                                 | [review](review.md)                                       |
| Normal fresh start                            | Recorded separately from disposable fixture qualification                                                         | [state record](fresh-start.json)                          |

The existing registration recovery tests now use complete current envelopes; they
retain exact lookup/no-replay and oversized-rejection assertions. The restored
workspace test now proves exact lookup and settlement, not merely absence of a
create call. The old registration reconstruction test now checks current explicit
serialization and rejection of a corrupted required field. Useful generic Serde
serialization assertions remain. New corruption tests verify preserved bytes,
visible guidance and no submission. The App regression proves both creation
controls disable on unreadable workspace evidence.

Focused frontend verification passed 62 tests before review; the new corruption
checks first produced four expected red failures. The review control regression
also failed against the earlier control and passed after the fix; its
[red](review-regression-red.log) and [green](review-regression-green.log) logs are
retained. The final complete gate includes the uncertainty-message assertions.

Failed preparation/verification attempts remain evidence: the first all-feature
compile caught a type error in the new qualification report entry, corrected to
its existing JSON representation ([log](verify-initial.log)). A subsequent format
gate caught unformatted new reset evidence ([log](verify-format-failure.log)). The
passing pre-review gate and first fixture build remain separate from final results.
The schema/presentation counter was normalized to 1 following the user's explicit
clean-baseline clarification; no counter transition is implemented in production.

## Fresh state and preservation

No native application was running before the deliberate preference reset. Only
the three approved app-owned keys were targeted in the stopped application's
owner-specific WebKit local storage; one was present. No database file, old root,
worktree, branch, provider home or credential was removed. Earlier supervisor
processes at separate retired roots were inventoried and left untouched; this
operation neither took over nor replaced their metadata. The newly selected
native-schema-1 root has its own current owner lease.

The normal package was opened with its default native-schema-1 root and showed
Connected, No repositories yet, and No repository selected. Application-menu Quit
released its lease. A read-only inspection while stopped confirmed schema 1 and
zero repository, registration, workspace, workspace-action and provider-action
rows. Reopening showed the same empty connected UI and retained the lease inode.
The application is left open on that fresh state.

The development default selects its absent native-schema-1 root on next startup;
explicit overrides retain their exact meaning and must deliberately name fresh
state. Old root contents remain intact. [Development instructions](../../development-state.md)
record exact roots, stop/restart, three-key clearing and preservation requirements.

## Correction to earlier completeness claims and limits

[Ticket 08's coverage record](../sqlx-architecture/08/coverage.md) remains an
unaltered checkpoint. Its broad claim that no compatibility reader remained did
not cover the pending missing-field defaults and registration deserialization
identified by this audit. This report records their removal and repaired coverage;
it does not revise the earlier results.

[Environment](environment.json) records toolchain, Mac hardware and source
baseline. Gates used the worktree's private default target and warm caches, with
single-threaded unit harnesses. Command-level wall times and Cargo versus unit
execution times remain separately identifiable; this cleanup makes no performance
claim or fresh-target compilation comparison. ts-rs reports ignored Serde
attributes (including the required-nullable deserializer); current generated
schemas/declarations and codec assertions pass. Normal package chunk-size
advisories are retained in the build log.

Unit, actual storage and fixture-native results establish their named boundaries.
No live-provider/authentication action, native chat qualification, power-loss test,
new performance/heap qualification, or final MVP clean-state completion gate was
performed or claimed. The fixture uses synthetic provider traffic and disposable
repositories; normal user state remains separate.
