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
        legacy: false,
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
            continuation: 1,
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
            continuation: 1,
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
            continuation: 2,
        }),
    )
    .await?;
    checks.push(json!({"case":"cross-repository-folder-constraint/settled-next-candidate", "status":"passed"}));

    let selection = uuid::Uuid::from_u128(50);
    written(
        &store,
        Write::ProviderSelection {
            scope: "fixture".into(),
            action: selection,
            path: "/synthetic/first".into(),
        },
    )
    .await?;
    written(
        &store,
        Write::ProviderSelection {
            scope: "fixture".into(),
            action: uuid::Uuid::from_u128(51),
            path: "/synthetic/second".into(),
        },
    )
    .await?;
    written(
        &store,
        Write::ProviderSelection {
            scope: "fixture".into(),
            action: selection,
            path: "/synthetic/first".into(),
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
    let attempt = uuid::Uuid::from_u128(60);
    written(
        &store,
        Write::ProviderLogin {
            scope: "fixture".into(),
            attempt,
            pending: true,
        },
    )
    .await?;
    written(
        &store,
        Write::ProviderLogin {
            scope: "fixture".into(),
            attempt,
            pending: false,
        },
    )
    .await?;
    written(
        &store,
        Write::ProviderLogin {
            scope: "fixture".into(),
            attempt,
            pending: true,
        },
    )
    .await?;
    let login = snapshot(&store, Read::ProviderLogin("fixture".into()))
        .await?
        .records::<(uuid::Uuid, i64)>()?;
    ensure(
        login[0].1 == 0,
        "Duplicate login reopened a completed fence",
    )?;
    ensure(
        store.close().await.faults.is_empty(),
        "Expected domain rejections poisoned storage",
    )?;
    checks.push(json!({"case":"provider-records-only/exact-action-history/no-duplicate-replay", "status":"passed"}));

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
    drop(lease); // All owners/drains above have physically completed their pool barriers.
    Ok(json!({
        "root":root, "sqlite_version":version,"sqlite_source_id":source,
        "checks":checks,
        "not_performed":["physical disk-full/I/O fault", "physical rollback/close-ack failure", "process kill points", "power-loss durability", "same-user path-replacement races (excluded contract)", "native host and provider qualification"],
        "limits":"SQL fixture establishes commit-failure retirement, not disk-fault durability; pool barriers are not literal OS-thread joins"
    }))
}
