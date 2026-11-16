//! Actual SQLx/disk probes. Never included in application unit discovery.
use super::*;
use serde_json::{Value, json};
use std::{
    fs::{self, DirBuilder, OpenOptions},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
};

fn directory(root: &Path, name: &str) -> Result<PathBuf> {
    let path = root.join(name);
    DirBuilder::new().mode(0o700).create(&path)?;
    Ok(path)
}
async fn owner(path: &Path) -> Result<Storage> {
    Storage::start(path.to_owned()).wait().await
}
async fn written(store: &Storage, write: Write) -> Result<u64> {
    match store
        .write(write)?
        .await
        .map_err(|_| Error::Panic)?
        .outcome?
    {
        Outcome::Written { revision } => Ok(revision),
        _ => Err(Error::Invalid("Expected write receipt")),
    }
}
async fn snapshot(store: &Storage, read: Read) -> Result<Snapshot> {
    match store.read(read)?.await.map_err(|_| Error::Panic)?.outcome? {
        Outcome::Snapshot(snapshot) => Ok(snapshot),
        _ => Err(Error::Invalid("Expected snapshot")),
    }
}
fn request(number: u128) -> RegisterRepository {
    RegisterRepository {
        action_id: ActionId(uuid::Uuid::from_u128(number)),
        path: format!("/disposable/repository-{number}"),
        create_initial_workspace: false,
    }
}
fn ensure(value: bool, message: &'static str) -> Result<()> {
    if value {
        Ok(())
    } else {
        Err(Error::Invalid(message))
    }
}

pub(crate) async fn run(root: &Path) -> Result<Value> {
    // Reject an existing root: this entry point never opens user metadata or resets it.
    DirBuilder::new().mode(0o700).create(root)?;
    let root = fs::canonicalize(root)?;
    let lease = OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(root.join("qualification.lease"))?;
    use std::os::fd::AsRawFd;
    // SAFETY: valid owned descriptor; flock retains exclusion until this file is dropped.
    if unsafe { libc::flock(lease.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let mut checks = Vec::new();
    let incomplete = directory(&root, "missing-metadata")?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(incomplete.join("unrecognized-state"))?;
    ensure(
        owner(&incomplete).await.is_err(),
        "Nonempty root without metadata was initialized",
    )?;
    ensure(
        !incomplete.join(opening::DATABASE).exists(),
        "Missing metadata was silently recreated",
    )?;
    checks.push(
        json!({"case":"nonempty-state-without-metadata-rejects-initialization", "status":"passed"}),
    );
    let path = directory(&root, "fresh")?;
    let store = owner(&path).await?;
    let mut connection = store.shared.writer.acquire().await?;
    let version: String = sqlx::query_scalar("SELECT sqlite_version()")
        .fetch_one(&mut *connection)
        .await?;
    let source: String = sqlx::query_scalar("SELECT sqlite_source_id()")
        .fetch_one(&mut *connection)
        .await?;
    opening::effective(&mut connection).await?;
    connection.close().await?;
    // A retired writer is replaced via the same opening/settings hooks.
    let first = request(1);
    let revision = written(&store, Write::RegistrationIntent(first.clone())).await?;
    ensure(revision == 1, "Fresh revision did not start at one")?;
    let page = snapshot(&store, Read::Registration(first.action_id)).await?;
    ensure(
        page.revision == 1 && page.records::<Registration>()?.len() == 1,
        "Replacement/settled read failed",
    )?;
    checks.push(
        json!({"case":"fresh-open/replacement/effective-settings/settled-read", "status":"passed"}),
    );

    let mut reader = store.shared.readers.acquire().await?;
    opening::effective(&mut reader).await?;
    let rejected = sqlx::query("UPDATE storage_revision SET revision=0")
        .execute(&mut *reader)
        .await;
    ensure(rejected.is_err(), "Read-only connection accepted a write")?;
    reader.close().await?;
    checks.push(json!({"case":"read-only-engine-rejection", "status":"passed", "error":format!("{:?}",rejected.unwrap_err())}));

    let lost = request(2);
    drop(store.write(Write::RegistrationIntent(lost.clone()))?);
    let page = snapshot(&store, Read::Registration(lost.action_id)).await?;
    ensure(
        page.records::<Registration>()?[0].action_id == lost.action_id,
        "Lost requester abandoned write",
    )?;
    checks.push(
        json!({"case":"requester-cancellation-keeps-write-and-fresh-status", "status":"passed"}),
    );

    let mut pinned = store.shared.readers.acquire().await?;
    let mut pin = pinned.begin().await?;
    let before: i64 = sqlx::query_scalar("SELECT revision FROM storage_revision")
        .fetch_one(&mut *pin)
        .await?;
    written(&store, Write::RegistrationIntent(request(3))).await?;
    let after: i64 = sqlx::query_scalar("SELECT revision FROM storage_revision")
        .fetch_one(&mut *pin)
        .await?;
    ensure(before == after, "Pinned reader changed snapshot")?;
    let checkpoint = operations::checkpoint(&store.shared).await?;
    ensure(
        checkpoint.checkpointed_frames < checkpoint.log_frames,
        "Pinned reader did not expose incomplete checkpoint",
    )?;
    checks.push(json!({"case":"pinned-reader/checkpoint-pressure", "status":"passed", "busy":checkpoint.busy,"log_frames":checkpoint.log_frames,"checkpointed_frames":checkpoint.checkpointed_frames,"wal_bytes":checkpoint.wal_bytes}));
    pin.rollback().await?;
    drop(pinned);
    let report = store.close().await;
    ensure(
        report.faults.is_empty(),
        "Normal close retained unexpected faults",
    )?;
    ensure(
        store.close().await.faults.is_empty(),
        "Repeated close changed result",
    )?;
    ensure(
        matches!(
            store.write(Write::RegistrationIntent(request(4))),
            Err(Error::Closed)
        ),
        "Closed storage reopened",
    )?;
    let reopened = owner(&path).await?;
    ensure(
        snapshot(&reopened, Read::Registration(first.action_id))
            .await?
            .records
            .len()
            == 1,
        "Current-build reopen lost durable record",
    )?;
    ensure(
        reopened.close().await.faults.is_empty(),
        "Reopened storage close failed",
    )?;
    checks.push(json!({"case":"close/repeated-close/current-build-reopen", "status":"passed"}));

    let path = directory(&root, "acquisition")?;
    let store = owner(&path).await?;
    let held1 = store.shared.readers.acquire().await?;
    let held2 = store.shared.readers.acquire().await?;
    let outcome = store
        .read(Read::Repositories { after: None })?
        .await
        .map_err(|_| Error::Panic)?
        .outcome;
    ensure(
        matches!(outcome, Err(Error::Sql(ref e)) if matches!(e.as_ref(), sqlx::Error::PoolTimedOut)),
        "Read acquisition was not bounded",
    )?;
    drop((held1, held2));
    ensure(
        !store.close().await.faults.is_empty(),
        "Close erased acquisition failure",
    )?;
    checks.push(json!({"case":"acquisition-timeout/close-retains-failure", "status":"passed"}));

    let path = directory(&root, "commit-fault")?;
    let store = owner(&path).await?;
    let mut connection = store.shared.writer.acquire().await?;
    // Controlled SQL fixture creates a genuine deferred FK COMMIT error; it is
    // not a disk/full or native I/O failure and is reported only as such.
    sqlx::raw_sql("CREATE TABLE commit_fault_probe(ref TEXT REFERENCES repositories(id) DEFERRABLE INITIALLY DEFERRED); CREATE TRIGGER commit_fault AFTER INSERT ON registration_actions BEGIN INSERT INTO commit_fault_probe VALUES('missing'); END;").execute(&mut *connection).await?;
    drop(connection);
    let action = request(5);
    let outcome = store
        .write(Write::RegistrationIntent(action.clone()))?
        .await
        .map_err(|_| Error::Panic)?
        .outcome;
    ensure(
        matches!(
            outcome,
            Err(Error::Write {
                retirement: None,
                uncertain: true,
                ..
            })
        ),
        "Failed commit did not retire explicitly",
    )?;
    let page = snapshot(&store, Read::Registration(action.action_id)).await?;
    ensure(
        page.records.is_empty() && page.revision == 0,
        "Fresh reconciliation used stale or uncommitted data",
    )?;
    let report = store.close().await;
    ensure(
        report
            .faults
            .iter()
            .any(|f| f.action == Some(action.action_id.0)),
        "Close lost original action failure",
    )?;
    checks.push(json!({"case":"real-deferred-foreign-key-commit-fault/retirement/fresh-absence", "status":"passed"}));

    let path = directory(&root, "atomic")?;
    let store = owner(&path).await?;
    let mut registration = request(10);
    registration.create_initial_workspace = true;
    written(&store, Write::RegistrationIntent(registration.clone())).await?;
    let repository = RepositoryRecord {
        id: RepositoryId(uuid::Uuid::from_u128(20)),
        name: "app".into(),
        path: "/disposable/app".into(),
        common_dir: "/disposable/app/.git".into(),
    };
    let mut workspace = Workspace {
        id: WorkspaceId(uuid::Uuid::from_u128(30)),
        repository_id: repository.id,
        creation_action_id: ActionId(uuid::Uuid::from_u128(40)),
        name: None,
        branch: None,
        path: None,

        base: None,
        artifacts: Artifacts {
            branch: BranchArtifact::Unknown,
            folder: FolderArtifact::Unknown,
        },
        status: CreationStatus::Queued,
        cleanup_unconfirmed: false,
    };
    let action = Registration {
        create_initial_workspace: true,
        initial_workspace_id: Some(workspace.id),
        action_id: registration.action_id,
        path: registration.path,
        state: RegistrationState::Completed {
            repository_id: repository.id,
        },
    };
    let mut wrong = workspace.clone();
    wrong.repository_id = RepositoryId(uuid::Uuid::from_u128(99));
    let rejected = store
        .write(Write::RegistrationFinish {
            action: action.clone(),
            repository: Some(repository.clone()),
            initial: Some(wrong),
        })?
        .await
        .map_err(|_| Error::Panic)?
        .outcome;
    ensure(rejected.is_err(), "Mismatched initial creation accepted")?;
    ensure(
        snapshot(&store, Read::Repository(repository.id))
            .await?
            .records
            .is_empty(),
        "Failed atomic continuation left a repository",
    )?;
    written(
        &store,
        Write::RegistrationFinish {
            action,
            repository: Some(repository.clone()),
            initial: Some(workspace.clone()),
        },
    )
    .await?;
    workspace.name = Some("marta".into());
    workspace.branch = Some("workspace/marta".into());
    workspace.path = Some(
        path.join("workspaces/app/marta/checkout")
            .to_string_lossy()
            .into_owned(),
    );
    written(
        &store,
        Write::Reservation(Reservation {
            workspace: workspace.clone(),
            folder: "app".into(),
            continuation: crate::storage::CandidateCursor { name: 1, folder: 0 },
        }),
    )
    .await?;
    let saved = snapshot(&store, Read::Workspace(workspace.id)).await?;
    ensure(
        saved.records::<Workspace>()?[0].path == workspace.path,
        "Atomic reservation did not persist exact location",
    )?;
    let mut changed = workspace.clone();
    changed.name = Some("pele".into());
    ensure(
        store
            .write(Write::WorkspaceSave {
                workspace: changed,
                action: None,
            })?
            .await
            .map_err(|_| Error::Panic)?
            .outcome
            .is_err(),
        "Reserved location changed",
    )?;
    let mismatch = RegisterRepository {
        action_id: registration.action_id,
        path: "/changed".into(),
        create_initial_workspace: false,
    };
    ensure(
        store
            .write(Write::RegistrationIntent(mismatch))?
            .await
            .map_err(|_| Error::Panic)?
            .outcome
            .is_err(),
        "Changed action payload accepted",
    )?;
    checks.push(json!({"case":"atomic-registration/initial-continuation/reservation/immutable-location/identity", "status":"passed"}));
    // A second registration of the already saved repository observes that repository;
    // only a newly registered repository requests the first-creation continuation.
    let mut duplicate = request(11);
    duplicate.create_initial_workspace = true;
    written(&store, Write::RegistrationIntent(duplicate.clone())).await?;
    let action = Registration {
        create_initial_workspace: true,
        initial_workspace_id: None,
        action_id: duplicate.action_id,
        path: duplicate.path,
        state: RegistrationState::Completed {
            repository_id: repository.id,
        },
    };
    written(
        &store,
        Write::RegistrationFinish {
            action,
            repository: Some(repository.clone()),
            initial: None,
        },
    )
    .await?;

    let mut other = repository.clone();
    other.id = RepositoryId(uuid::Uuid::from_u128(21));
    other.common_dir = "/disposable/other/.git".into();
    let registration = request(12);
    written(&store, Write::RegistrationIntent(registration.clone())).await?;
    let action = Registration {
        create_initial_workspace: false,
        initial_workspace_id: None,
        action_id: registration.action_id,
        path: registration.path,
        state: RegistrationState::Completed {
            repository_id: other.id,
        },
    };
    written(
        &store,
        Write::RegistrationFinish {
            action,
            repository: Some(other.clone()),
            initial: None,
        },
    )
    .await?;
    let mut other_workspace = workspace.clone();
    other_workspace.id = WorkspaceId(uuid::Uuid::from_u128(31));
    other_workspace.repository_id = other.id;
    other_workspace.creation_action_id = ActionId(uuid::Uuid::from_u128(41));
    other_workspace.name = None;
    other_workspace.branch = None;
    other_workspace.path = None;
    let create = WorkspaceAction::Create {
        action_id: other_workspace.creation_action_id,
        repository_id: other.id,
    };
    written(
        &store,
        Write::WorkspaceSave {
            workspace: other_workspace.clone(),
            action: Some(create),
        },
    )
    .await?;
    other_workspace.name = Some("pele".into());
    other_workspace.branch = Some("workspace/pele".into());
    other_workspace.path = Some(
        path.join("workspaces/app-2/pele/checkout")
            .to_string_lossy()
            .into_owned(),
    );
    let collision = store
        .write(Write::Reservation(Reservation {
            workspace: other_workspace.clone(),
            folder: "app".into(),
            continuation: crate::storage::CandidateCursor { name: 1, folder: 0 },
        }))?
        .await
        .map_err(|_| Error::Panic)?
        .outcome;
    ensure(
        matches!(collision, Err(Error::CandidateConflict)),
        "Global folder collision lost typed settled rejection",
    )?;
    written(
        &store,
        Write::Reservation(Reservation {
            workspace: other_workspace,
            folder: "app-2".into(),
            continuation: crate::storage::CandidateCursor { name: 2, folder: 0 },
        }),
    )
    .await?;
    checks.push(json!({"case":"cross-repository-folder-constraint/settled-next-candidate", "status":"passed"}));

    let provider_action = |number, operation| ProviderAction {
        id: uuid::Uuid::from_u128(number),
        scope: "fixture".into(),
        provider: "codex".into(),
        target: uuid::Uuid::from_u128(49),
        revision: 0,
        executable: Some("/synthetic/codex".into()),
        operation,
    };
    let selection = provider_action(50, ProviderOperation::Selection("/synthetic/first".into()));
    let second = provider_action(51, ProviderOperation::Selection("/synthetic/second".into()));
    for action in [&selection, &second] {
        written(&store, Write::ProviderIntent(action.clone())).await?;
        written(
            &store,
            Write::ProviderFinish {
                action: action.clone(),
                remember: true,
            },
        )
        .await?;
    }
    written(&store, Write::ProviderIntent(selection.clone())).await?;
    written(
        &store,
        Write::ProviderFinish {
            action: selection.clone(),
            remember: true,
        },
    )
    .await?;
    let selected = snapshot(&store, Read::ProviderSelection("fixture".into()))
        .await?
        .records::<(uuid::Uuid, String)>()?;
    ensure(
        selected[0].1 == "/synthetic/second",
        "Duplicate selection replayed an old preference",
    )?;
    let attempt = provider_action(60, ProviderOperation::Login);
    written(&store, Write::ProviderIntent(attempt.clone())).await?;
    written(
        &store,
        Write::ProviderFinish {
            action: attempt.clone(),
            remember: false,
        },
    )
    .await?;
    written(&store, Write::ProviderIntent(attempt.clone())).await?;
    let login = snapshot(&store, Read::ProviderPending("fixture".into()))
        .await?
        .records::<(ProviderAction, i64)>()?;
    ensure(
        login.is_empty(),
        "Duplicate login reopened a completed fence",
    )?;
    // Same ID with a different configuration is rejected without changing the saved action.
    let mut changed = attempt.clone();
    changed.revision = 1;
    ensure(
        written(&store, Write::ProviderIntent(changed))
            .await
            .is_err(),
        "Changed provider configuration accepted",
    )?;
    let pending = provider_action(61, ProviderOperation::Login);
    written(&store, Write::ProviderIntent(pending.clone())).await?;
    let conflicting = provider_action(
        62,
        ProviderOperation::Selection("/synthetic/blocked".into()),
    );
    ensure(
        written(&store, Write::ProviderIntent(conflicting))
            .await
            .is_err(),
        "Pending credential scope was replaced",
    )?;
    let saved = snapshot(&store, Read::ProviderPending("fixture".into()))
        .await?
        .records::<(ProviderAction, i64)>()?;
    ensure(
        saved.len() == 1 && saved[0].0 == pending,
        "Pending scope lost exact provider intent",
    )?;
    written(
        &store,
        Write::ProviderFinish {
            action: pending,
            remember: false,
        },
    )
    .await?;
    ensure(
        store.close().await.faults.is_empty(),
        "Expected domain rejections poisoned storage",
    )?;
    checks.push(json!({"case":"provider-records-only/exact-action-history/scope-exclusion/no-duplicate-replay", "status":"passed"}));

    // Controlled engine SQLITE_FULL, not a claim of a physically full filesystem.
    // SQLite documents max_page_count as returning SQLITE_FULL when growth exceeds it.
    let path = directory(&root, "engine-full")?;
    let store = owner(&path).await?;
    let mut connection = store.shared.writer.acquire().await?;
    let pages: i64 = sqlx::query_scalar("PRAGMA page_count")
        .fetch_one(&mut *connection)
        .await?;
    let maximum: i64 = sqlx::query_scalar("PRAGMA max_page_count=1")
        .fetch_one(&mut *connection)
        .await?;
    ensure(maximum == pages, "Engine page cap did not take effect")?;
    drop(connection);
    let mut large = request(70);
    large.path = format!("/{}", "x".repeat(4000));
    let full = written(&store, Write::RegistrationIntent(large))
        .await
        .expect_err("engine page budget must fail");
    let original = match &full {
        Error::Write {
            original,
            retirement: None,
            ..
        } => original,
        _ => return Err(Error::Invalid("SQLITE_FULL retirement failed")),
    };
    ensure(
        matches!(original.as_ref(), Error::Sql(error) if matches!(error.as_ref(), sqlx::Error::Database(db) if db.code().as_deref() == Some("13"))),
        "Expected actual SQLITE_FULL",
    )?;
    let report = store.close().await;
    ensure(
        report
            .faults
            .iter()
            .any(|fault| fault.action == Some(uuid::Uuid::from_u128(70))),
        "Close erased engine-full error",
    )?;
    checks.push(json!({"case":"controlled-engine-SQLITE_FULL/failed-write-retirement/retained-close-fault", "status":"passed", "error":full.to_string()}));

    let path = directory(&root, "bounded-pages")?;
    let store = owner(&path).await?;
    for number in 100..136 {
        let mut request = request(number);
        request.path = format!("/{}", "a".repeat(3400));
        written(&store, Write::RegistrationIntent(request)).await?;
    }
    let mut after = None;
    let mut count = 0;
    loop {
        let page = snapshot(&store, Read::PendingRegistrations { after }).await?;
        ensure(
            page.records.len() <= 32
                && page.records.iter().map(String::len).sum::<usize>() <= 48 * 1024,
            "Page budget exceeded",
        )?;
        count += page.records.len();
        after = page
            .next
            .map(|s| uuid::Uuid::parse_str(&s).map(ActionId))
            .transpose()
            .map_err(|_| Error::Corrupt)?;
        if after.is_none() {
            break;
        }
    }
    ensure(count == 36, "Keyset continuation lost records")?;
    ensure(
        store.close().await.faults.is_empty(),
        "Page fixture close failed",
    )?;
    checks
        .push(json!({"case":"row-and-byte-keyset-pagination", "status":"passed", "records":count}));

    for kind in ["symlink", "hardlink", "public-mode"] {
        let path = directory(&root, kind)?;
        let file = path.join(opening::DATABASE);
        let target = root.join(format!("target-{kind}"));
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&target)?;
        match kind {
            "symlink" => std::os::unix::fs::symlink(&target, &file)?,
            "hardlink" => fs::hard_link(&target, &file)?,
            _ => {
                OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&file)?;
                fs::set_permissions(&file, fs::Permissions::from_mode(0o644))?;
            }
        }
        ensure(
            owner(&path).await.is_err(),
            "Unsafe metadata file was accepted",
        )?;
        checks.push(json!({"case":kind,"status":"passed"}));
    }
    checks.push(pressure(&root).await?);
    drop(lease); // All owners/drains above have physically completed their pool barriers.
    Ok(json!({
        "root":root, "sqlite_version":version,"sqlite_source_id":source,
        "checks":checks,
        "not_performed":["physical disk-full/I/O fault", "physical rollback/close-ack failure", "process kill points", "power-loss durability", "same-user path-replacement races (excluded contract)", "native host and provider qualification"],
        "limits":"SQL fixture establishes commit-failure retirement, not disk-fault durability; pool barriers are not literal OS-thread joins"
    }))
}

// Actual queued work and undelivered results, through the production storage owner.
async fn pressure(root: &Path) -> Result<Value> {
    let path = directory(root, "pressure")?;
    let store = owner(&path).await?;
    let started = std::time::Instant::now();
    let mut writes = Vec::new();
    for number in 1_000..1_064 {
        let mut write = request(number);
        write.path = format!("/{}", "p".repeat(3_400));
        writes.push(store.write(Write::RegistrationIntent(write))?);
    }
    ensure(
        matches!(
            store.write(Write::RegistrationIntent(request(1_064))),
            Err(Error::Capacity)
        ),
        "Writer saturation created an unbounded waiter",
    )?;
    let mut retained = Vec::new();
    for write in writes {
        let delivery = write.await.map_err(|_| Error::Panic)?;
        delivery.revision()?;
        retained.push(delivery);
    }
    ensure(
        matches!(
            store.write(Write::RegistrationIntent(request(1_064))),
            Err(Error::Capacity)
        ),
        "Undelivered writes released physical/result capacity",
    )?;
    // Completed writer results remain retained while the independent read reserve works.
    let mut reads = Vec::new();
    for _ in 0..8 {
        reads.push(store.read(Read::Registration(ActionId(uuid::Uuid::from_u128(1_000))))?);
    }
    ensure(
        matches!(
            store.read(Read::Repositories { after: None }),
            Err(Error::Capacity)
        ),
        "Read saturation created an unbounded waiter",
    )?;
    let mut read_results = Vec::new();
    for read in reads {
        let delivery = read.await.map_err(|_| Error::Panic)?;
        delivery.revision()?;
        read_results.push(delivery);
    }
    ensure(
        matches!(
            store.read(Read::Repositories { after: None }),
            Err(Error::Capacity)
        ),
        "Undelivered reads released their capacity",
    )?;
    drop(read_results);
    drop(retained);
    written(&store, Write::RegistrationIntent(request(1_064))).await?;
    let mut oversized = request(1_065);
    oversized.path = "x".repeat(65_536);
    ensure(
        matches!(
            store.write(Write::RegistrationIntent(oversized)),
            Err(Error::Oversized)
        ),
        "Oversized input was admitted",
    )?;
    ensure(
        matches!(
            store.read(Read::CommonDirectory("x".repeat(4_097))),
            Err(Error::Oversized)
        ),
        "Oversized read was admitted",
    )?;
    let mut after = None;
    let (mut count, mut pages, mut bytes) = (0, 0, 0);
    loop {
        let page = snapshot(&store, Read::PendingRegistrations { after }).await?;
        let retained_bytes = page.records.iter().map(String::len).sum::<usize>();
        ensure(
            page.records.len() <= 32 && retained_bytes <= 48 * 1024,
            "Pressure page exceeded row/byte budget",
        )?;
        count += page.records.len();
        pages += 1;
        bytes += retained_bytes;
        after = page
            .next
            .map(|next| uuid::Uuid::parse_str(&next).map(ActionId))
            .transpose()
            .map_err(|_| Error::Corrupt)?;
        if after.is_none() {
            break;
        }
    }
    ensure(
        count == 65 && pages > 2,
        "Large retained metadata lost continuation",
    )?;
    let close = store.close().await;
    ensure(close.faults.is_empty(), "Pressure close retained faults")?;
    Ok(
        json!({"case":"ordinary-saturation/read-reserve/retained-results/large-byte-pages",
        "status":"passed", "admitted_writes":65, "retained_write_results":64,
        "retained_read_results":8, "records":count, "pages":pages, "record_bytes":bytes,
        "elapsed_ms":started.elapsed().as_secs_f64()*1000.0, "checkpoint":close.checkpoint,
        "limits":"Actual SQLx fixture; no streamed output, end-to-end status latency or JS heap guarantee"}),
    )
}
