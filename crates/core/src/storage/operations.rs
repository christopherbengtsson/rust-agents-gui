use super::*;
use futures_util::TryStreamExt;

pub(super) async fn write(shared: &Shared, write: Write) -> Result<u64> {
    let mut connection = shared.writer.acquire().await?;
    if shared
        .control
        .admission
        .lock()
        .expect("storage admission poisoned")
        .fenced
    {
        return Err(Error::Fenced);
    }
    // Keep the acquired connection outside the consumed borrowed transaction scope.
    let mut committing = false;
    let result = async {
        let mut tx = connection.begin_with("BEGIN IMMEDIATE").await?;
        match apply(&mut tx, write).await {
            Err(Error::CandidateConflict) => { tx.rollback().await?; return Err(Error::CandidateConflict); }
            other => other?,
        }
        let revision: i64 = sqlx::query_scalar("UPDATE storage_revision SET revision=revision+1 WHERE id=1 AND revision<9223372036854775807 RETURNING revision").fetch_one(&mut *tx).await?;
        committing = true;
        tx.commit().await?;
        u64::try_from(revision).map_err(|_| Error::Invalid("Invalid committed revision"))
    }.await;
    match result {
        Ok(revision) => Ok(revision),
        Err(Error::CandidateConflict) => Err(Error::CandidateConflict),
        Err(original) => {
            // The transaction has dropped/enqueued cleanup. Never return the connection
            // to idle first; explicit close orders cleanup and awaits worker shutdown.
            Err(
                retire_failed(shared.control.clone(), original, committing, async {
                    connection.close().await.map_err(Error::from)
                })
                .await,
            )
        }
    }
}
fn id(id: impl std::fmt::Display) -> String {
    id.to_string()
}
fn conflict() -> Error {
    Error::Invalid("Identity/payload mismatch or immutable reservation conflict")
}
fn json<T: DeserializeOwned>(s: &str) -> Result<T> {
    serde_json::from_str(s).map_err(|_| Error::Corrupt)
}
async fn apply(tx: &mut Transaction<'_, Sqlite>, write: Write) -> Result<()> {
    match write {
        Write::RegistrationIntent(request) => {
            let payload = encode(&request, PAGE_BYTES)?;
            action_identity(tx, request.action_id.0, "registration", &payload).await?;
            let existing: Option<String> =
                sqlx::query_scalar("SELECT payload FROM registration_actions WHERE id=?")
                    .bind(id(request.action_id.0))
                    .fetch_optional(&mut **tx)
                    .await?;
            if let Some(existing) = existing {
                if existing != payload {
                    return Err(conflict());
                }
                return Ok(());
            }
            let action = Registration {
                create_initial_workspace: request.create_initial_workspace,
                initial_workspace_id: None,
                action_id: request.action_id,
                path: request.path,
                state: RegistrationState::Pending,
            };
            sqlx::query(
                "INSERT INTO registration_actions(id,payload,pending,record) VALUES(?,?,1,?)",
            )
            .bind(id(action.action_id.0))
            .bind(payload)
            .bind(encode(&action, PAGE_BYTES)?)
            .execute(&mut **tx)
            .await?;
        }
        Write::RegistrationFinish {
            action,
            repository,
            initial,
        } => {
            let saved: String =
                sqlx::query_scalar("SELECT record FROM registration_actions WHERE id=?")
                    .bind(id(action.action_id.0))
                    .fetch_one(&mut **tx)
                    .await?;
            let saved: Registration = json(&saved)?;
            if saved.path != action.path
                || saved.create_initial_workspace != action.create_initial_workspace
            {
                return Err(conflict());
            }
            if saved.state != RegistrationState::Pending {
                if saved != action {
                    return Err(conflict());
                }
                return Ok(());
            }
            if action.state == RegistrationState::Pending {
                return Err(conflict());
            }
            if let Some(repo) = &repository
                && action.state
                    != (RegistrationState::Completed {
                        repository_id: repo.id,
                    })
            {
                return Err(conflict());
            }
            if let Some(repo) = repository {
                let record = encode(&repo, PAGE_BYTES)?;
                let previous: Option<String> =
                    sqlx::query_scalar("SELECT record FROM repositories WHERE id=?")
                        .bind(id(repo.id.0))
                        .fetch_optional(&mut **tx)
                        .await?;
                match previous {
                    Some(old) if old != record => return Err(conflict()),
                    Some(_) => {}
                    None => {
                        if action.create_initial_workspace && initial.is_none() {
                            return Err(conflict());
                        }
                        sqlx::query("INSERT INTO repositories VALUES(?,?,?)")
                            .bind(id(repo.id.0))
                            .bind(&repo.common_dir)
                            .bind(record)
                            .execute(&mut **tx)
                            .await?;
                    }
                }
            }
            if let RegistrationState::Completed { repository_id } = action.state {
                let exists: bool =
                    sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM repositories WHERE id=?)")
                        .bind(id(repository_id.0))
                        .fetch_one(&mut **tx)
                        .await?;
                if !exists {
                    return Err(conflict());
                }
            }
            if let Some(workspace) = initial {
                if !action.create_initial_workspace
                    || action.initial_workspace_id != Some(workspace.id)
                {
                    return Err(conflict());
                }
                if action.state
                    != (RegistrationState::Completed {
                        repository_id: workspace.repository_id,
                    })
                {
                    return Err(conflict());
                }
                let create = WorkspaceAction::Create {
                    action_id: workspace.creation_action_id,
                    repository_id: workspace.repository_id,
                };
                save_workspace(tx, &workspace, Some(&create), false).await?;
            } else if action.initial_workspace_id.is_some() {
                return Err(conflict());
            }
            sqlx::query("UPDATE registration_actions SET pending=?,record=? WHERE id=?")
                .bind(action.state == RegistrationState::Pending)
                .bind(encode(&action, PAGE_BYTES)?)
                .bind(id(action.action_id.0))
                .execute(&mut **tx)
                .await?;
        }
        Write::WorkspaceSave { workspace, action } => {
            save_workspace(tx, &workspace, action.as_ref(), false).await?
        }
        Write::Reservation(reservation) => {
            let Reservation {
                workspace,
                folder,
                continuation,
            } = reservation;
            if workspace.name.is_none() || workspace.branch.is_none() || workspace.path.is_none() {
                return Err(conflict());
            }
            if folder.is_empty() || folder.len() > 4096 {
                return Err(Error::Oversized);
            }
            let saved_folder: Option<String> =
                sqlx::query_scalar("SELECT folder FROM repository_folders WHERE repository_id=?")
                    .bind(id(workspace.repository_id.0))
                    .fetch_optional(&mut **tx)
                    .await?;
            if saved_folder.as_ref().is_some_and(|saved| saved != &folder) {
                return Err(Error::CandidateConflict);
            }
            let folder_taken: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM repository_folders WHERE folder=? AND repository_id<>?)")
                .bind(&folder).bind(id(workspace.repository_id.0)).fetch_one(&mut **tx).await?;
            let name_taken: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM workspaces WHERE repository_id=? AND name=? AND id<>?)")
                .bind(id(workspace.repository_id.0)).bind(&workspace.name).bind(id(workspace.id.0)).fetch_one(&mut **tx).await?;
            if folder_taken || name_taken {
                return Err(Error::CandidateConflict);
            }
            sqlx::query("INSERT INTO repository_folders(repository_id,folder) VALUES(?,?) ON CONFLICT(repository_id) DO NOTHING").bind(id(workspace.repository_id.0)).bind(&folder).execute(&mut **tx).await?;
            let saved: String =
                sqlx::query_scalar("SELECT folder FROM repository_folders WHERE repository_id=?")
                    .bind(id(workspace.repository_id.0))
                    .fetch_one(&mut **tx)
                    .await?;
            if saved != folder {
                return Err(conflict());
            }
            // Claim only an existing pending creation; reservation cannot invent its intent.
            let saved: Option<String> =
                sqlx::query_scalar("SELECT record FROM workspaces WHERE id=?")
                    .bind(id(workspace.id.0))
                    .fetch_optional(&mut **tx)
                    .await?;
            if saved.is_none() {
                return Err(conflict());
            }
            save_workspace(tx, &workspace, None, true).await?;
            set_cursor(tx, workspace.id, continuation).await?;
        }
        Write::AllocationCursor {
            workspace_id,
            cursor,
        } => set_cursor(tx, workspace_id, cursor).await?,
        Write::ProviderIntent(action) => {
            let payload = encode(&action, PAGE_BYTES)?;
            let previous = provider_identity(tx, &action, &payload).await?;
            if previous.is_none() && action.operation == ProviderOperation::Login {
                sqlx::query("INSERT INTO provider_login_fences VALUES(?,?,1) ON CONFLICT(scope) DO UPDATE SET attempt=excluded.attempt,pending=1")
                    .bind(&action.scope).bind(id(action.id)).execute(&mut **tx).await?;
            }
        }
        Write::ProviderFinish { action, remember } => {
            let saved: (String, bool) =
                sqlx::query_as("SELECT payload,completed FROM provider_actions WHERE id=?")
                    .bind(id(action.id))
                    .fetch_one(&mut **tx)
                    .await?;
            if saved.0 != encode(&action, PAGE_BYTES)? {
                return Err(conflict());
            }
            if saved.1 {
                return Ok(());
            }
            match &action.operation {
                ProviderOperation::Selection(path) if remember => {
                    sqlx::query("INSERT INTO provider_executables VALUES(?,?,?) ON CONFLICT(scope) DO UPDATE SET action=excluded.action,path=excluded.path")
                        .bind(&action.scope).bind(id(action.id)).bind(path).execute(&mut **tx).await?;
                }
                ProviderOperation::Login => {
                    let changed = sqlx::query("UPDATE provider_login_fences SET pending=0 WHERE scope=? AND attempt=? AND pending=1")
                        .bind(&action.scope).bind(id(action.id)).execute(&mut **tx).await?.rows_affected();
                    if changed != 1 {
                        return Err(conflict());
                    }
                }
                _ => {}
            }
            sqlx::query("UPDATE provider_actions SET completed=1 WHERE id=?")
                .bind(id(action.id))
                .execute(&mut **tx)
                .await?;
        }
    }
    Ok(())
}
async fn action_identity(
    tx: &mut Transaction<'_, Sqlite>,
    action: uuid::Uuid,
    kind: &str,
    payload: &str,
) -> Result<()> {
    let saved: Option<(String, String)> =
        sqlx::query_as("SELECT kind,payload FROM action_identities WHERE id=?")
            .bind(id(action))
            .fetch_optional(&mut **tx)
            .await?;
    match saved {
        Some((saved_kind, saved_payload)) if saved_kind != kind || saved_payload != payload => {
            Err(conflict())
        }
        Some(_) => Ok(()),
        None => {
            sqlx::query("INSERT INTO action_identities VALUES(?,?,?)")
                .bind(id(action))
                .bind(kind)
                .bind(payload)
                .execute(&mut **tx)
                .await?;
            Ok(())
        }
    }
}
async fn provider_identity(
    tx: &mut Transaction<'_, Sqlite>,
    action: &ProviderAction,
    payload: &str,
) -> Result<Option<bool>> {
    action_identity(tx, action.id, "provider", payload).await?;
    let saved: Option<(String, bool)> =
        sqlx::query_as("SELECT payload,completed FROM provider_actions WHERE id=?")
            .bind(id(action.id))
            .fetch_optional(&mut **tx)
            .await?;
    match saved {
        Some((saved, _)) if saved != payload => Err(conflict()),
        Some((_, completed)) => Ok(Some(completed)),
        None => {
            sqlx::query("INSERT INTO provider_actions VALUES(?,?,0,?)")
                .bind(id(action.id))
                .bind(payload)
                .bind(&action.scope)
                .execute(&mut **tx)
                .await?;
            Ok(None)
        }
    }
}
async fn set_cursor(
    tx: &mut Transaction<'_, Sqlite>,
    workspace: WorkspaceId,
    cursor: CandidateCursor,
) -> Result<()> {
    let name = i64::try_from(cursor.name).map_err(|_| Error::Oversized)?;
    let folder = i64::try_from(cursor.folder).map_err(|_| Error::Oversized)?;
    let changed = sqlx::query(
        "UPDATE workspaces SET allocation_cursor=?,folder_cursor=? WHERE id=? AND allocation_cursor<=? AND folder_cursor<=?",
    )
    .bind(name)
    .bind(folder)
    .bind(id(workspace.0))
    .bind(name)
    .bind(folder)
    .execute(&mut **tx)
    .await?
    .rows_affected();
    if changed != 1 {
        return Err(conflict());
    }
    Ok(())
}
async fn save_workspace(
    tx: &mut Transaction<'_, Sqlite>,
    workspace: &Workspace,
    action: Option<&WorkspaceAction>,
    reservation: bool,
) -> Result<()> {
    if let Some(path) = &workspace.path {
        validate_workspace_path(path).map_err(|_| conflict())?;
    }
    let saved: Option<String> = sqlx::query_scalar("SELECT record FROM workspaces WHERE id=?")
        .bind(id(workspace.id.0))
        .fetch_optional(&mut **tx)
        .await?;
    if saved.is_none()
        && (workspace.name.is_some() || !matches!(action, Some(WorkspaceAction::Create { .. })))
    {
        return Err(conflict());
    }
    if let Some(saved) = saved {
        let saved: Workspace = json(&saved)?;
        if !reservation && saved.name.is_none() && workspace.name.is_some() {
            return Err(conflict());
        }
        if saved.repository_id != workspace.repository_id
            || saved.creation_action_id != workspace.creation_action_id
            || (saved.name.is_some()
                && (saved.name != workspace.name
                    || saved.branch != workspace.branch
                    || saved.path != workspace.path))
        {
            return Err(conflict());
        }
    }
    sqlx::query("INSERT INTO workspaces(id,repository_id,creation_action_id,queued,record,name,branch,path,cleanup_unconfirmed) VALUES(?,?,?,?,?,?,?,?,?) ON CONFLICT(id) DO UPDATE SET queued=excluded.queued,record=excluded.record,name=excluded.name,branch=excluded.branch,path=excluded.path,cleanup_unconfirmed=excluded.cleanup_unconfirmed")
        .bind(id(workspace.id.0)).bind(id(workspace.repository_id.0)).bind(id(workspace.creation_action_id.0)).bind(workspace.status == CreationStatus::Queued).bind(encode(workspace, PAGE_BYTES)?).bind(&workspace.name).bind(&workspace.branch).bind(&workspace.path).bind(workspace.cleanup_unconfirmed).execute(&mut **tx).await?;
    if let Some(action) = action {
        match action {
            WorkspaceAction::Create {
                action_id,
                repository_id,
            } if *action_id == workspace.creation_action_id
                && *repository_id == workspace.repository_id => {}
            WorkspaceAction::Reconcile { workspace_id, .. } if *workspace_id == workspace.id => {}
            _ => return Err(conflict()),
        }
        let record = encode(action, PAGE_BYTES)?;
        action_identity(tx, action.id().0, "workspace", &record).await?;
        let saved: Option<(String, String)> =
            sqlx::query_as("SELECT workspace_id,record FROM workspace_actions WHERE id=?")
                .bind(id(action.id().0))
                .fetch_optional(&mut **tx)
                .await?;
        if let Some((target, previous)) = saved {
            if target != id(workspace.id.0) || previous != record {
                return Err(conflict());
            }
        } else {
            sqlx::query("INSERT INTO workspace_actions VALUES(?,?,?)")
                .bind(id(action.id().0))
                .bind(id(workspace.id.0))
                .bind(record)
                .execute(&mut **tx)
                .await?;
        }
    }
    Ok(())
}

// Read projections cap bytes before the driver materializes record text. Streaming
// uses one buffered driver row; never fetch_all arbitrary TEXT and then check size.
macro_rules! records_sql {
    ($table:literal, $suffix:literal) => {
        concat!("SELECT id,length(CAST(record AS BLOB)),CASE WHEN length(CAST(record AS BLOB))<=49152 THEN record END FROM ", $table, $suffix)
    };
}

enum RecordShape {
    Registration,
    Repository,
    Workspace,
    WorkspaceAction,
    Scalar,
}

pub(super) async fn read(shared: &Shared, read: Read) -> Result<Snapshot> {
    let mut connection = shared.readers.acquire().await?;
    let result = async {
        let mut tx = connection.begin().await?;
        // This SELECT establishes the snapshot shared by every following query.
        let revision: i64 = sqlx::query_scalar("SELECT revision FROM storage_revision WHERE id=1").fetch_one(&mut *tx).await?;
        let validation = match &read {
            Read::Registration(_) | Read::PendingRegistrations { .. } => RecordShape::Registration,
            Read::Repository(_) | Read::CommonDirectory(_) | Read::Repositories { .. } => RecordShape::Repository,
            Read::Workspace(_) | Read::Workspaces { .. } | Read::PendingWorkspaces { .. } | Read::Name { .. } => RecordShape::Workspace,
            Read::WorkspaceAction(_) => RecordShape::WorkspaceAction,
            Read::Folder { .. } | Read::AllocationCursor(_) | Read::CleanupBlocked(_) | Read::ProviderSelection(_) | Read::ProviderPending(_) | Read::ProviderAction(_) => RecordShape::Scalar,
        };
        let (sql, arguments): (&'static str, Vec<String>) = match read {
            Read::Registration(a) => (records_sql!("registration_actions", " WHERE id=?"), vec![id(a.0)]),
            Read::Repository(a) => (records_sql!("repositories", " WHERE id=?"), vec![id(a.0)]),
            Read::CommonDirectory(a) => (records_sql!("repositories", " WHERE common_dir=?"), vec![a]),
            Read::Repositories { after } => (records_sql!("repositories", " WHERE id>? ORDER BY id LIMIT 33"), vec![after.map_or(String::new(), |x| id(x.0))]),
            Read::PendingRegistrations { after } => (records_sql!("registration_actions", " WHERE pending=1 AND id>? ORDER BY id LIMIT 33"), vec![after.map_or(String::new(), |x| id(x.0))]),
            Read::Workspace(a) => (records_sql!("workspaces", " WHERE id=?"), vec![id(a.0)]),
            Read::Workspaces { repository, after } => (records_sql!("workspaces", " WHERE repository_id=? AND id>? ORDER BY id LIMIT 33"), vec![id(repository.0), after.map_or(String::new(), |x| id(x.0))]),
            Read::PendingWorkspaces { after } => (records_sql!("workspaces", " WHERE queued=1 AND id>? ORDER BY id LIMIT 33"), vec![after.map_or(String::new(), |x| id(x.0))]),
            Read::WorkspaceAction(a) => ("SELECT a.id,length(CAST(a.record AS BLOB))+length(CAST(w.record AS BLOB)),CASE WHEN length(CAST(a.record AS BLOB))+length(CAST(w.record AS BLOB))<=48000 THEN json_array(json(a.record),json(w.record)) END FROM workspace_actions a JOIN workspaces w ON w.id=a.workspace_id WHERE a.id=?", vec![id(a.0)]),
            Read::Folder { repository, candidate } => ("SELECT repository_id,0,json_array(folder,repository_id) FROM repository_folders WHERE repository_id=? OR folder=? LIMIT 2", vec![id(repository.0),candidate]),
            Read::Name { repository, candidate } => (records_sql!("workspaces", " WHERE repository_id=? AND name=? LIMIT 1"), vec![id(repository.0),candidate]),
            Read::AllocationCursor(a) => ("SELECT id,0,json_object('name',allocation_cursor,'folder',folder_cursor) FROM workspaces WHERE id=?", vec![id(a.0)]),
            Read::CleanupBlocked(a) => ("SELECT '',0,CASE WHEN EXISTS(SELECT 1 FROM workspaces WHERE repository_id=? AND cleanup_unconfirmed=1) THEN 'true' ELSE 'false' END", vec![id(a.0)]),
            Read::ProviderSelection(scope) => ("SELECT action,0,json_array(action,path) FROM provider_executables WHERE scope=?", vec![scope]),
            Read::ProviderPending(scope) => ("SELECT id,length(CAST(payload AS BLOB)),CASE WHEN length(CAST(payload AS BLOB))<=48000 THEN json_array(json(payload),completed) END FROM provider_actions WHERE scope=? AND completed=0", vec![scope]),
            Read::ProviderAction(action) => ("SELECT id,length(CAST(payload AS BLOB)),CASE WHEN length(CAST(payload AS BLOB))<=48000 THEN json_array(json(payload),completed) END FROM provider_actions WHERE id=?", vec![id(action)]),
        };
        let mut query = sqlx::query(sql);
        for argument in arguments { query = query.bind(argument); }
        let mut rows = query.fetch(&mut *tx);
        let mut records = Vec::new();
        let mut bytes = 0;
        let mut next = None;
        let mut last = None;
        while let Some(row) = rows.try_next().await? {
            let row_id: String = row.try_get(0)?;
            let size: i64 = row.try_get(1)?;
            let record: Option<&str> = row.try_get(2)?;
            let Some(record) = record else { return Err(Error::Oversized); };
            if size < 0 || record.len() > PAGE_BYTES { return Err(Error::Oversized); }
            if records.len() == PAGE_ROWS || record.len() > PAGE_BYTES - bytes {
                if records.is_empty() { return Err(Error::Oversized); }
                next = last; break;
            }
            bytes += record.len(); records.push(record.to_owned()); last = Some(row_id);
        }
        drop(rows);
        for record in &records {
            match validation {
                RecordShape::Registration => { let _: Registration = json(record)?; },
                RecordShape::Repository => { let _: RepositoryRecord = json(record)?; },
                RecordShape::Workspace => { let _: Workspace = json(record)?; },
                RecordShape::WorkspaceAction => { let _: (WorkspaceAction, Workspace) = json(record)?; },
                RecordShape::Scalar => { let _: serde_json::Value = json(record)?; },
            }
        }
        tx.commit().await?;
        Ok(Snapshot { revision: u64::try_from(revision).map_err(|_| Error::Invalid("Invalid revision"))?, records, next })
    }.await;
    match result {
        Ok(snapshot) => Ok(snapshot),
        Err(original) => Err(
            retire_failed(shared.control.clone(), original, false, async {
                connection.close().await.map_err(Error::from)
            })
            .await,
        ),
    }
}

pub(super) async fn checkpoint(shared: &Shared) -> Result<Checkpoint> {
    let started = std::time::Instant::now();
    let mut connection = shared.writer.acquire().await?;
    let result = sqlx::query("PRAGMA wal_checkpoint(PASSIVE)")
        .fetch_one(&mut *connection)
        .await;
    let row = match result {
        Ok(row) => row,
        Err(original) => {
            return Err(
                retire_failed(shared.control.clone(), original.into(), false, async {
                    connection.close().await.map_err(Error::from)
                })
                .await,
            );
        }
    };
    let path = shared
        .path
        .with_file_name(format!("{}-wal", opening::DATABASE));
    let bytes = tokio::task::spawn_blocking(move || std::fs::metadata(path).map(|m| m.len()))
        .await
        .map_err(|_| Error::Panic)??;
    Ok(Checkpoint {
        busy: row.try_get(0)?,
        log_frames: row.try_get(1)?,
        checkpointed_frames: row.try_get(2)?,
        wal_bytes: bytes,
        elapsed: started.elapsed(),
    })
}
