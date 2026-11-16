//! Actual disposable SQLx/Git effects. Excluded from application units.
use super::*;
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    process::Command,
    time::Duration,
};

fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(RepositoryError::storage(message))
    }
}
fn git(root: &Path, args: &[&str]) -> Result<String> {
    let result = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(RepositoryError::storage)?;
    ensure(
        result.status.success(),
        &String::from_utf8_lossy(&result.stderr),
    )?;
    Ok(String::from_utf8_lossy(&result.stdout).trim().into())
}
fn directory(path: &Path) -> Result<()> {
    fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(RepositoryError::storage)
}
fn fixture(root: &Path) -> Result<(PathBuf, String)> {
    directory(root)?;
    git(root, &["init", "--template=", "-b", "main"])?;
    directory(&root.join("empty-hooks"))?;
    git(
        root,
        &[
            "config",
            "core.hooksPath",
            root.join("empty-hooks").to_str().unwrap(),
        ],
    )?;
    fs::write(root.join("sentinel.txt"), "original checkout\n")
        .map_err(RepositoryError::storage)?;
    git(root, &["add", "sentinel.txt"])?;
    git(
        root,
        &[
            "-c",
            "user.name=Disposable qualification",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-m",
            "fixture",
        ],
    )?;
    Ok((
        fs::canonicalize(root).map_err(RepositoryError::storage)?,
        git(root, &["rev-parse", "HEAD"])?,
    ))
}
async fn registered(owner: &Workspaces, id: ActionId) -> Result<Registration> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(action) = owner.action(id).await?
            && action.state != RegistrationState::Pending
        {
            return Ok(action);
        }
        ensure(
            tokio::time::Instant::now() < deadline,
            "Registration qualification deadline expired",
        )?;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
async fn ready(owner: &Workspaces, id: WorkspaceId) -> Result<Workspace> {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        if let WorkspaceReply::Workspace {
            workspace: Some(workspace),
        } = owner
            .workspace_request(WorkspaceRequest::Get { id })
            .await?
        {
            if workspace.status == CreationStatus::Ready {
                return Ok(workspace);
            }
            if matches!(
                workspace.status,
                CreationStatus::Failed { .. } | CreationStatus::Uncertain { .. }
            ) {
                return Err(RepositoryError::storage(format!(
                    "Workspace failed: {workspace:?}"
                )));
            }
        }
        ensure(
            tokio::time::Instant::now() < deadline,
            "Workspace qualification deadline expired",
        )?;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
pub(crate) async fn run(root: &Path) -> Result<Value> {
    // create() rejects every existing root. No cleanup or existing-user-state access.
    fs::DirBuilder::new()
        .mode(0o700)
        .create(root)
        .map_err(RepositoryError::storage)?;
    let root = fs::canonicalize(root).map_err(RepositoryError::storage)?;
    let lease = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(root.join("owner.lock"))
        .map_err(RepositoryError::storage)?;
    lease.try_lock().map_err(RepositoryError::storage)?;
    let a = fixture(&root.join("fixtures/a/app"))?;
    let b = fixture(&root.join("fixtures/b/app"))?;
    let state = root.join("state");
    directory(&state)?;
    let storage = crate::storage::Storage::start(state.clone())
        .wait()
        .await
        .map_err(RepositoryError::storage)?;
    let processes = crate::process_effects::Processes::new();
    let owner = Workspaces::open(state, storage.clone(), processes.clone());
    let result = cohort(&owner, &storage, &root, &a, &b).await;
    let completion = owner.finish().await;
    let physical = processes.finish().await;
    let close = storage.close().await;
    physical.map_err(RepositoryError::storage)?;
    completion?;
    ensure(
        close.faults.is_empty(),
        &format!("Storage close faults: {:?}", close.faults),
    )?;
    result
}
async fn cohort(
    owner: &Workspaces,
    storage: &crate::storage::Storage,
    root: &Path,
    a: &(PathBuf, String),
    b: &(PathBuf, String),
) -> Result<Value> {
    let requests = [a, b]
        .into_iter()
        .map(|(path, _)| RegisterRepository {
            create_initial_workspace: true,
            action_id: ActionId(Uuid::new_v4()),
            path: path.to_string_lossy().into_owned(),
        })
        .collect::<Vec<_>>();
    let (ra, rb) = tokio::join!(
        owner.register(requests[0].clone()),
        owner.register(requests[1].clone())
    );
    ra?;
    rb?;
    let (ra, rb) = tokio::join!(
        registered(owner, requests[0].action_id),
        registered(owner, requests[1].action_id)
    );
    let (ra, rb) = (ra?, rb?);
    let (wa, wb) = tokio::join!(
        ready(
            owner,
            ra.initial_workspace_id
                .ok_or_else(|| RepositoryError::storage("Missing first creation"))?
        ),
        ready(
            owner,
            rb.initial_workspace_id
                .ok_or_else(|| RepositoryError::storage("Missing first creation"))?
        )
    );
    let (wa, wb) = (wa?, wb?);
    let folder = |w: &Workspace| {
        Path::new(w.path.as_deref().unwrap())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf()
    };
    ensure(
        folder(&wa) != folder(&wb),
        "Unrelated app repositories shared a folder",
    )?;
    let create = || {
        owner.workspace_request(WorkspaceRequest::Create {
            action_id: ActionId(Uuid::new_v4()),
            repository_id: wa.repository_id,
        })
    };
    let (first, second) = tokio::join!(create(), create());
    let extract = |reply: WorkspaceReply| match reply {
        WorkspaceReply::Action {
            workspace: Some(w), ..
        } => Ok(w.id),
        _ => Err(RepositoryError::storage("Missing creation action")),
    };
    let (first, second) = tokio::join!(
        ready(owner, extract(first?)?),
        ready(owner, extract(second?)?)
    );
    let (first, second) = (first?, second?);
    ensure(
        first.name != second.name && first.name != wa.name && second.name != wa.name,
        "Same repository reused a reserved name",
    )?;
    ensure(
        folder(&first) == folder(&wa) && folder(&second) == folder(&wa),
        "Same repository changed its saved folder",
    )?;
    // Reconcile an already owned partial-effect record under a new action,
    // preserving its original creation identity, pinned base and physical target.
    let mut interrupted = first.clone();
    interrupted.status = CreationStatus::Uncertain {
        error: RepositoryError::new(ErrorCode::Git, "qualification lost reply"),
    };
    save(
        storage,
        Write::WorkspaceSave {
            workspace: interrupted,
            action: None,
        },
    )
    .await?;
    owner
        .workspace_request(WorkspaceRequest::Reconcile {
            action_id: ActionId(Uuid::new_v4()),
            workspace_id: first.id,
        })
        .await?;
    let reconciled = ready(owner, first.id).await?;
    ensure(
        reconciled.path == first.path
            && reconciled.base == first.base
            && reconciled.creation_action_id == first.creation_action_id,
        "Reconciliation retargeted saved work",
    )?;
    let nil = owner
        .workspace_request(WorkspaceRequest::Create {
            action_id: ActionId(Uuid::nil()),
            repository_id: wa.repository_id,
        })
        .await?;
    let nil = ready(owner, extract(nil)?).await?;
    for w in [&wa, &wb, &first, &second, &nil] {
        ensure(
            git(Path::new(w.path()?), &["rev-parse", "HEAD"])? == w.base.as_ref().unwrap().commit,
            "Created worktree differs from its pinned base",
        )?;
    }
    for (path, head) in [a, b] {
        ensure(
            git(path, &["rev-parse", "HEAD"])? == *head,
            "Original checkout HEAD changed",
        )?;
        ensure(
            git(path, &["status", "--porcelain"])?.is_empty(),
            "Original checkout was modified",
        )?;
        ensure(
            fs::read_to_string(path.join("sentinel.txt")).map_err(RepositoryError::storage)?
                == "original checkout\n",
            "Original file changed",
        )?;
    }
    let allocation = allocation_continuation(owner, storage, &wa, root).await?;
    let pressure = pressure(owner, storage, &wa, &wb).await?;
    // Exercise two concurrent raw typed claims on one candidate, separately from
    // the domain decision gate. One must lose without poisoning writer admission.
    owner.fence().await;
    let repo = wa.repository_id;
    let (x, y) = (
        allocate(repo, ActionId(Uuid::new_v4()), root)?,
        allocate(repo, ActionId(Uuid::new_v4()), root)?,
    );
    for w in [&x, &y] {
        save(
            storage,
            Write::WorkspaceSave {
                workspace: w.clone(),
                action: Some(WorkspaceAction::Create {
                    action_id: w.creation_action_id,
                    repository_id: repo,
                }),
            },
        )
        .await?;
    }
    let existing_folder = folder(&wa)
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let claim = |mut w: Workspace| {
        w.name = Some("qualification-claim".into());
        w.branch = Some("workspace/qualification-claim".into());
        w.path = Some(
            folder(&wa)
                .join("qualification-claim/checkout")
                .to_string_lossy()
                .into_owned(),
        );
        Write::Reservation(crate::storage::Reservation {
            workspace: w,
            folder: existing_folder.clone(),
            continuation: crate::storage::CandidateCursor { name: 0, folder: 0 },
        })
    };
    let (rx, ry) = (
        storage.write(claim(x)).map_err(RepositoryError::storage)?,
        storage.write(claim(y)).map_err(RepositoryError::storage)?,
    );
    let (rx, ry) = tokio::join!(rx, ry);
    let (rx, ry) = (
        rx.map_err(RepositoryError::storage)?.revision(),
        ry.map_err(RepositoryError::storage)?.revision(),
    );
    ensure(
        rx.is_ok() != ry.is_ok(),
        "Concurrent SQLx claims did not choose exactly one winner",
    )?;
    ensure(
        matches!(rx, Err(crate::storage::Error::CandidateConflict))
            || matches!(ry, Err(crate::storage::Error::CandidateConflict)),
        "Losing claim did not report a settled candidate conflict",
    )?;
    ensure(
        storage.faults().is_empty(),
        "Normal claim conflict fenced storage",
    )?;
    // This is synthetic claim qualification only; no Git effect is requested for
    // either pending record. Fencing prevents their later dispatch during close.
    owner.fence().await;
    Ok(
        json!({"root":root,"checks":["concurrent-registration-and-first-continuations","global-app-folder-collision","same-repository-distinct-permanent-names","concurrent-disposable-git-worktrees","pinned-base-and-original-checkout-protection","concurrent-sqlx-claim-winner-and-settled-conflict","explicit-reconcile-retains-pinned-location-and-original-action","nil-action-identity-creation"],"workspaces":[wa,wb,first,second,nil],"allocation":allocation,"pressure":pressure,"desktop":"separate native fixture cohort","providers":"synthetic connection evidence separate; no live provider"}),
    )
}

async fn inspected(owner: &Workspaces, workspace: WorkspaceId) -> Result<()> {
    owner
        .workspace_request(WorkspaceRequest::Changes {
            workspace_id: workspace,
            refresh: true,
            comparison_id: None,
            after: None,
        })
        .await?;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        let reply = owner
            .workspace_request(WorkspaceRequest::Changes {
                workspace_id: workspace,
                refresh: false,
                comparison_id: None,
                after: None,
            })
            .await?;
        if let WorkspaceReply::Changes { observation } = reply {
            ensure(
                observation.phase != InspectionPhase::Failed,
                "Read-only inspection failed",
            )?;
            if observation.phase == InspectionPhase::Ready {
                return Ok(());
            }
        }
        ensure(
            tokio::time::Instant::now() < deadline,
            "Inspection deadline expired",
        )?;
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

// Fixed offered fixture trace: 512 large metadata commits and 16 read-only Git
// inspections in A, while one independent creation in B must finish before both
// noisy producers finish. This is foundation progress, never chat stream fairness.
async fn pressure(
    owner: &Workspaces,
    storage: &crate::storage::Storage,
    noisy: &Workspace,
    quiet: &Workspace,
) -> Result<Value> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let writes = AtomicUsize::new(0);
    let inspections = AtomicUsize::new(0);
    let started = std::time::Instant::now();
    let noise_actions: Vec<_> = (0..512).map(|_| Uuid::new_v4()).collect();
    let writer = async {
        for (number, id) in noise_actions.iter().enumerate() {
            save(
                storage,
                Write::ProviderIntent(crate::storage::ProviderAction {
                    id: *id,
                    scope: format!("synthetic-foundation-pressure-{number}"),
                    provider: "codex".into(),
                    target: Uuid::nil(),
                    revision: 0,
                    executable: None,
                    operation: crate::storage::ProviderOperation::Selection(format!(
                        "/synthetic/{}",
                        "p".repeat(3_400)
                    )),
                }),
            )
            .await?;
            writes.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        Ok::<_, RepositoryError>(())
    };
    let effects = async {
        for _ in 0..16 {
            inspected(owner, noisy.id).await?;
            inspections.fetch_add(1, Ordering::SeqCst);
        }
        Ok::<_, RepositoryError>(())
    };
    let quiet_work = async {
        // Start after both producers have made observable progress.
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        while writes.load(Ordering::SeqCst) == 0 || inspections.load(Ordering::SeqCst) == 0 {
            ensure(
                tokio::time::Instant::now() < deadline,
                "Noisy fixture failed to start",
            )?;
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        let reply = owner
            .workspace_request(WorkspaceRequest::Create {
                action_id: ActionId(Uuid::new_v4()),
                repository_id: quiet.repository_id,
            })
            .await?;
        let WorkspaceReply::Action {
            workspace: Some(workspace),
            ..
        } = reply
        else {
            return Err(RepositoryError::storage("Missing quiet creation"));
        };
        let completed = ready(owner, workspace.id).await?;
        let progress = (
            writes.load(Ordering::SeqCst),
            inspections.load(Ordering::SeqCst),
        );
        ensure(
            progress.0 < 512 && progress.1 < 16,
            "Quiet creation only completed after noisy producers finished",
        )?;
        Ok::<_, RepositoryError>((
            completed,
            progress,
            started.elapsed().as_secs_f64() * 1000.0,
        ))
    };
    let (writer, effects, quiet_result) = tokio::join!(writer, effects, quiet_work);
    writer?;
    effects?;
    let (completed, progress, quiet_ms) = quiet_result?;
    let mut retained = 0;
    for id in noise_actions {
        let saved =
            one::<(crate::storage::ProviderAction, i64)>(storage, Read::ProviderAction(id)).await?;
        ensure(
            saved.is_some_and(|(action, completed)| action.id == id && completed == 0),
            "Pressure lost exact retained synthetic metadata",
        )?;
        retained += 1;
    }
    Ok(
        json!({"case":"quiet-creation-during-large-storage-and-read-only-effect-pressure",
        "status":"passed", "offered_commits":512,"settled_commits":writes.load(Ordering::SeqCst),
        "retained_synthetic_action_records":retained, "completed_inspections":inspections.load(Ordering::SeqCst),
        "quiet_workspace":completed.id, "quiet_completion_ms":quiet_ms,
        "commits_at_quiet_completion":progress.0,"inspections_at_quiet_completion":progress.1,
        "elapsed_ms":started.elapsed().as_secs_f64()*1000.0,
        "limits":"No chat producer, sustained-load guarantee or provider execution is qualified"}),
    )
}

async fn allocation_continuation(
    owner: &Workspaces,
    storage: &crate::storage::Storage,
    existing: &Workspace,
    root: &Path,
) -> Result<Value> {
    let workspace = allocate(existing.repository_id, ActionId(Uuid::new_v4()), root)?;
    let folder = Path::new(existing.path()?)
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| RepositoryError::storage("Missing managed folder"))?;
    let mut occupied = Vec::new();
    for cursor in 0..40 {
        let name = naming::candidate(cursor, workspace.id.0.as_u128())?;
        let path = folder.join(&name);
        // Existing real workspaces already occupy their names. New fixture folders
        // have a sentinel and are never adopted, overwritten or removed.
        if !path.exists() {
            directory(&path)?;
        }
        let sentinel = path.join("qualification-sentinel");
        fs::write(&sentinel, "external collision\n").map_err(RepositoryError::storage)?;
        occupied.push(sentinel);
    }
    save(
        storage,
        Write::WorkspaceSave {
            workspace: workspace.clone(),
            action: Some(WorkspaceAction::Create {
                action_id: workspace.creation_action_id,
                repository_id: workspace.repository_id,
            }),
        },
    )
    .await?;
    owner.notify();
    let completed = ready(owner, workspace.id).await?;
    let cursor =
        one::<crate::storage::CandidateCursor>(storage, Read::AllocationCursor(workspace.id))
            .await?
            .ok_or_else(|| RepositoryError::storage("Missing durable allocation continuation"))?;
    ensure(
        cursor.name >= 32,
        "Allocation did not persist a bounded turn",
    )?;
    ensure(
        cursor.name >= 40
            && completed.name.as_deref()
                == Some(naming::candidate(cursor.name, workspace.id.0.as_u128())?.as_str()),
        "Allocation did not resume past exact external collisions",
    )?;
    for sentinel in occupied {
        ensure(
            fs::read_to_string(sentinel).map_err(RepositoryError::storage)?
                == "external collision\n",
            "Allocation modified an external collision",
        )?;
    }
    Ok(
        json!({"case":"forty-actual-folder-collisions/durable-32-candidate-continuation",
        "status":"passed","workspace":completed.id,"cursor":cursor,"external_collisions":40}),
    )
}
