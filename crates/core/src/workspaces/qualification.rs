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
    let owner = Workspaces::open(state, storage.clone());
    let result = cohort(&owner, &storage, &root, &a, &b).await;
    let completion = owner.finish().await;
    let close = storage.close().await;
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
        json!({"root":root,"checks":["concurrent-registration-and-first-continuations","global-app-folder-collision","same-repository-distinct-permanent-names","concurrent-disposable-git-worktrees","pinned-base-and-original-checkout-protection","concurrent-sqlx-claim-winner-and-settled-conflict","explicit-reconcile-retains-pinned-location-and-original-action","nil-action-identity-creation"],"workspaces":[wa,wb,first,second,nil],"desktop":"not run: combined provider conversion remains ticket 04","providers":"not run"}),
    )
}
