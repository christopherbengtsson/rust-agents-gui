# Disposable driver experiments

These standalone executables exercise real SQLite and OS threads. They are research experiments, separate from deterministic application units and desktop/native-provider qualification. No production storage or root dependency manifest is changed.

`common.rs` is compiled by two independent packages because the selected drivers have incompatible `libsqlite3-sys` requirements. `rusqlite/Cargo.lock` and `sqlx/Cargo.lock` freeze both graphs. SQLx uses its narrower `sqlite-bundled` and `runtime-tokio` features without macros, migrations, TLS, or the `sqlite` umbrella's optional extension/deserialization features. The identical synthetic schema has an indexed `(conversation, seq)` transcript and metadata upserts. It does not reproduce the production application's domain policy.

## Reproduce

From the repository root, create a new **mode-0700**, task-owned private directory outside the checkout. Replace `/tmp/storage-driver-06a.NEW` below with that newly created directory. Run the builds and experiments sequentially with other storage measurements idle:

```sh
mkdir -m 700 /tmp/storage-driver-06a.NEW
/usr/bin/time -l cargo build --release --locked --manifest-path docs/evidence/milestone-01-storage/driver-bench/rusqlite/Cargo.toml --target-dir /tmp/storage-driver-06a.NEW/fresh-rusqlite
/usr/bin/time -l cargo build --release --locked --manifest-path docs/evidence/milestone-01-storage/driver-bench/sqlx/Cargo.toml --target-dir /tmp/storage-driver-06a.NEW/fresh-sqlx
python3 docs/evidence/milestone-01-storage/driver-bench/run.py --private-root /tmp/storage-driver-06a.NEW --phase all --output /tmp/storage-driver-06a.NEW/results.jsonl
```

The runner refuses a direct symlink, nonprivate directory, or directory owned by another user. Every invocation creates a fresh private `data-*` child. Only that child's fixed experiment files are replaced. Databases, binaries, and Cargo targets remain outside the repository. Existing user state is never selected. Private state can be retained for inspection or removed explicitly after confirming it is the generated experiment directory.

The as-run root was `/tmp/storage-driver-06a.e28Ggc`. Stages were invoked separately with `--phase screen`, `finalists`, `scale`, and `resources`, and outputs captured in their corresponding `../driver-results-*.jsonl`/`.log` files. Initial screening used `common-screen.rs` and `manifest-screen.json`; later finalists use `common.rs` and `manifest.json`. Screening preceded the safe fresh-child runner correction; its state root was itself freshly created by `mktemp -d`. Source hashes in each JSONL provenance record identify the respective cohorts.

## Configuration and interpretation

- `manifest.json` freezes the deterministic workload and budgets. Payload index `sequence + conversation` mixes payload sizes within retained histories. All seeded/live bodies are synthetic `x` strings, with no runtime/provider/user content.
- Both adapters run through one bounded writer worker and one connection; SQLx also owns its internal SQLite worker and an explicitly owned current-thread Tokio runtime. The SQLx adapter drives each async operation with `block_on` to expose the same synchronous worker shape. This is a whole-stack comparison, not a proof about every possible optimized async design.
- Both adapters materialize history pages into `Vec<String>`. Inline reads alternate latest pages and a cursor halfway into retained history; the returned checksum ensures payloads are actually read. Page count is capped at 100.
- Initial `b16` screens supply preformed per-conversation bursts. Their queue timer excludes formation, which could add 750 ms at 20 events/second for one chat. They establish batching throughput only.
- Finalist `coalesce25ms` samples generate individual events at 20/second/chat across active chats, then form one global transaction at 16 events or 25 ms from its first event. `batch_formation` records per-event wait; `queue` begins at transaction admission; `service` covers begin/inserts/metadata/commit. Percentile sums are not an end-to-end percentile. User intents are not subject to batching in this probe or proposed application policy.
- Optional two-reader runs replace inline mixed-page reads with two independent recent-page readers polling every 10 ms. They change the offered workload and cannot isolate the benefit/cost of reader connections alone.
- No queue retries occur. A full queue is an explicit rejection. A lifecycle probe blocks the worker, fills its one-slot admission queue, rejects another command, drops the reply receiver, then releases and joins the worker; it verifies accepted persistence still completes and rejected work is absent. This does not qualify production cancellation/timeout policy.
- Each timed sample admits work for three seconds, then drains it. `elapsed_s` includes writer drain/close and stopping/joining optional readers, but excludes setup, copying/seeding, and final integrity verification. `events_per_s` uses that elapsed time. Percentile samples are operations within each short run; five-run aggregate medians/ranges describe repeatability, not a long soak.
- `sampled_main_wal_shm_bytes` samples three files after transactions. It excludes transient rollback-journal peaks and is not a peak-total-disk guarantee. Historical screens named this `peak_db_sidecar_bytes`; that name is overbroad.
- `writer_close_ms` reports the writer close while optional readers remain open. It does not include a final-reader checkpoint. Historical screens named it `close_ms`. `writer_drain_ms` likewise covers only joining the writer; historical screens called it `drain_ms`.
- Both connections request fullfsync, foreign keys, a 100 ms busy timeout, 100 cached statements, a 2 MiB page-cache budget, and a 1000-page WAL autocheckpoint. Effective engine/source IDs, compilation options, journal, sync, and fullfsync are reported. Shipping bundles differ, so this is not a pure driver-overhead result.

## Failed and intermediate attempts

Retain the original logs: SQLx's first compilation rejected dynamic borrowed SQL strings under its 0.9 `SqlSafeStr` contract; the corrected adapter audits fixed/allowlisted statements through `AssertSqlSafe` and binds all data. A later reopen-per-operation implementation moved a connection into a closure and failed ownership checking; moving explicit close/reopen into the owning worker fixed it. These are harness construction failures, not database failures.

Fresh-target logs measure the earlier complete harness before global coalescing was added; incremental final-source build logs are separate. Fresh final-source builds are recorded separately by `build_costs.py` in `driver-results-build-costs.json`; their binary hashes match runtime binaries. Initial package declarations produced inactive-feature warnings, fixed by declaring both mutually exclusive driver feature names in each standalone manifest. Runtime results do not use those failed artifacts.

The first resource counter included the mandatory `ps -M` heading. Corrected resource evidence excludes the heading and retains raw process-specific thread output. `summarize.py` excludes that superseded cohort and computes the retained medians/ranges.
