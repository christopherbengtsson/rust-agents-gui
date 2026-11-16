//! Supervisor-owned SQLite metadata. No project files or provider history are used.
use crate::workspaces::*;
use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

pub(crate) struct SqliteStore {
    connection: Connection,
    path: PathBuf,
}

/// Only non-secret intent/cleanup evidence. Never a provider URL, token or PID.
pub(crate) trait ProviderLoginPersistence: Send + Sync {
    fn record(&self, attempt: uuid::Uuid, pending: bool) -> Result<()>;
}
pub(crate) struct ProviderLoginStore {
    path: PathBuf,
    scope: String,
}
impl ProviderLoginStore {
    pub(crate) fn open(state: &Path, scope: String) -> Result<Self> {
        let store = SqliteStore::open(state)?;
        store.connection.execute_batch("CREATE TABLE IF NOT EXISTS provider_login_fences (scope TEXT PRIMARY KEY NOT NULL, attempt TEXT NOT NULL, pending INTEGER NOT NULL)").map_err(storage)?;
        Ok(Self {
            path: store.path,
            scope,
        })
    }
    pub(crate) fn pending(&self) -> Result<Option<uuid::Uuid>> {
        let connection = SqliteStore::connect(&self.path)?;
        let value: Option<String> = connection
            .query_row(
                "SELECT attempt FROM provider_login_fences WHERE scope=?1 AND pending=1",
                [&self.scope],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)?;
        value
            .map(|value| uuid::Uuid::parse_str(&value).map_err(storage))
            .transpose()
    }
}
impl ProviderLoginPersistence for ProviderLoginStore {
    fn record(&self, attempt: uuid::Uuid, pending: bool) -> Result<()> {
        let connection = SqliteStore::connect(&self.path)?;
        connection.execute("INSERT INTO provider_login_fences(scope,attempt,pending) VALUES(?1,?2,?3) ON CONFLICT(scope) DO UPDATE SET attempt=excluded.attempt,pending=excluded.pending", params![self.scope, attempt.to_string(), pending]).map_err(storage)?;
        Ok(())
    }
}
/// Application-owned non-secret executable preference; runtime credentials stay external.
pub(crate) trait ProviderSelectionPersistence: Send + Sync {
    fn remember(&self, id: uuid::Uuid, path: &str) -> Result<()>;
}
pub(crate) struct ProviderSelectionStore {
    path: PathBuf,
    scope: String,
}
impl ProviderSelectionStore {
    pub(crate) fn open(state: &Path, scope: String) -> Result<Self> {
        let store = SqliteStore::open(state)?;
        store.connection.execute_batch("CREATE TABLE IF NOT EXISTS provider_executables (scope TEXT PRIMARY KEY NOT NULL, action TEXT NOT NULL, path TEXT NOT NULL)").map_err(storage)?;
        Ok(Self {
            path: store.path,
            scope,
        })
    }
    pub(crate) fn selected(&self) -> Result<Option<(uuid::Uuid, String)>> {
        let connection = SqliteStore::connect(&self.path)?;
        let row: Option<(String, String)> = connection
            .query_row(
                "SELECT action,path FROM provider_executables WHERE scope=?1",
                [&self.scope],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage)?;
        row.map(|(id, path)| Ok((uuid::Uuid::parse_str(&id).map_err(storage)?, path)))
            .transpose()
    }
}
impl ProviderSelectionPersistence for ProviderSelectionStore {
    fn remember(&self, id: uuid::Uuid, path: &str) -> Result<()> {
        let connection = SqliteStore::connect(&self.path)?;
        connection.execute("INSERT INTO provider_executables(scope,action,path) VALUES(?1,?2,?3) ON CONFLICT(scope) DO UPDATE SET action=excluded.action,path=excluded.path", params![self.scope, id.to_string(), path]).map_err(storage)?;
        Ok(())
    }
}
fn storage(error: impl std::fmt::Display) -> RepositoryError {
    RepositoryError::storage(error)
}
fn encode(value: &impl Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(storage)
}
fn decode<T: DeserializeOwned>(value: String) -> Result<T> {
    serde_json::from_str(&value).map_err(storage)
}

impl SqliteStore {
    pub(crate) fn open(state: &Path) -> Result<Self> {
        let path = state.join("workspace.sqlite3");
        // Refuse aliases before SQLite opens its database and journal files.
        for candidate in [
            path.clone(),
            state.join("workspace.sqlite3-journal"),
            state.join("workspace.sqlite3-wal"),
            state.join("workspace.sqlite3-shm"),
        ] {
            match fs::symlink_metadata(&candidate) {
                Ok(metadata)
                    if !metadata.is_file()
                        || metadata.nlink() != 1
                        || metadata.permissions().mode() & 0o077 != 0 =>
                {
                    return Err(storage(
                        "Metadata files must be private regular files, not aliases. Preserve the state directory and restore its permissions.",
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(storage(error)),
            }
        }
        // Creation permissions do not depend solely on a caller's process-wide mask.
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)
            .map_err(storage)?;
        let connection = Self::connect(&path)?;
        let version: u32 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(storage)?;
        if version > 3 {
            return Err(storage(
                "This database belongs to a newer application. Open it with a compatible build; do not delete it.",
            ));
        }
        if version == 0 {
            connection.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE repositories (id TEXT PRIMARY KEY NOT NULL, common_dir TEXT UNIQUE NOT NULL, record TEXT NOT NULL);
                CREATE TABLE registration_actions (id TEXT PRIMARY KEY NOT NULL, pending INTEGER NOT NULL, record TEXT NOT NULL);
                CREATE INDEX pending_registrations ON registration_actions(pending, id);
                PRAGMA user_version=1;
                COMMIT;").map_err(storage)?;
        }
        if version < 2 {
            connection.execute_batch("BEGIN IMMEDIATE;
                CREATE TABLE workspaces (id TEXT PRIMARY KEY NOT NULL, repository_id TEXT NOT NULL REFERENCES repositories(id), queued INTEGER NOT NULL, record TEXT NOT NULL);
                CREATE INDEX workspace_navigation ON workspaces(repository_id,id);
                CREATE INDEX queued_workspaces ON workspaces(queued,id);
                CREATE TABLE workspace_actions (id TEXT PRIMARY KEY NOT NULL, workspace_id TEXT NOT NULL REFERENCES workspaces(id), record TEXT NOT NULL);
                PRAGMA user_version=2; COMMIT;").map_err(storage)?;
        }
        if version < 3 {
            transaction(&SqliteTransaction(&connection), || {
                connection.execute_batch("ALTER TABLE workspaces ADD COLUMN name TEXT; ALTER TABLE workspaces ADD COLUMN branch TEXT; ALTER TABLE workspaces ADD COLUMN path TEXT;
                    CREATE TABLE repository_folders(repository_id TEXT PRIMARY KEY NOT NULL REFERENCES repositories(id), folder TEXT UNIQUE NOT NULL);") .map_err(storage)?;
                let mut query = connection
                    .prepare("SELECT record FROM workspaces ORDER BY repository_id,id")
                    .map_err(storage)?;
                let records: Vec<Workspace> = query
                    .query_map([], |r| r.get::<_, String>(0))
                    .map_err(storage)?
                    .map(|r| decode(r.map_err(storage)?))
                    .collect::<Result<_>>()?;
                for workspace in crate::workspaces::naming::migrate_labels(records)? {
                    connection
                        .execute(
                            "UPDATE workspaces SET record=?2,name=?3,branch=?4,path=?5 WHERE id=?1",
                            params![
                                workspace.id.0.to_string(),
                                encode(&workspace)?,
                                workspace.name,
                                workspace.branch,
                                workspace.path
                            ],
                        )
                        .map_err(storage)?;
                }
                connection.execute_batch("CREATE UNIQUE INDEX reserved_workspace_names ON workspaces(repository_id,name);
                    CREATE UNIQUE INDEX reserved_workspace_branches ON workspaces(repository_id,branch);
                    CREATE UNIQUE INDEX reserved_workspace_paths ON workspaces(path);
                    PRAGMA user_version=3;").map_err(storage)
            })?;
        }
        Ok(Self { connection, path })
    }
    fn connect(path: &Path) -> Result<Connection> {
        let connection = Connection::open_with_flags(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(storage)?;
        connection
            .busy_timeout(std::time::Duration::from_millis(100))
            .map_err(storage)?;
        connection.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA fullfsync=ON; PRAGMA foreign_keys=ON;").map_err(storage)?;
        Ok(connection)
    }
    fn read<T: DeserializeOwned>(&self, sql: &str, value: &str) -> Result<Option<T>> {
        self.connection
            .query_row(sql, [value], |row| row.get::<_, String>(0))
            .optional()
            .map_err(storage)?
            .map(decode)
            .transpose()
    }
    fn write<T>(&mut self, body: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        transaction(&SqliteTransaction(&self.connection), || {
            body(&self.connection)
        })
    }
    fn ensure_committed(&mut self) -> Result<()> {
        if !self.connection.is_autocommit() {
            self.connection = Self::connect(&self.path)?;
        }
        Ok(())
    }
}

// This seam supplies SQLite transaction outcomes, not application registration policy.
trait TransactionControl {
    fn begin(&self) -> Result<()>;
    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
    fn active(&self) -> bool;
}
struct SqliteTransaction<'a>(&'a Connection);
impl TransactionControl for SqliteTransaction<'_> {
    fn begin(&self) -> Result<()> {
        self.0.execute_batch("BEGIN IMMEDIATE").map_err(storage)
    }
    fn commit(&self) -> Result<()> {
        self.0.execute_batch("COMMIT").map_err(storage)
    }
    fn rollback(&self) -> Result<()> {
        self.0.execute_batch("ROLLBACK").map_err(storage)
    }
    fn active(&self) -> bool {
        !self.0.is_autocommit()
    }
}
fn transaction<T>(
    control: &impl TransactionControl,
    body: impl FnOnce() -> Result<T>,
) -> Result<T> {
    control.begin()?;
    let result = body().and_then(|value| {
        control.commit()?;
        Ok(value)
    });
    if result.is_err()
        && control.active()
        && let Err(rollback) = control.rollback()
    {
        return Err(storage(format!(
            "Transaction cleanup failed: {rollback}. Restart the supervisor before further registration."
        )));
    }
    result
}

impl RepositoryStore for SqliteStore {
    fn close(self: Box<Self>) -> crate::workspaces::StoreClose {
        let Self { connection, path } = *self;
        match connection.close() {
            Ok(()) => Ok(()),
            Err((connection, error)) => Err((Box::new(Self { connection, path }), storage(error))),
        }
    }
    fn action(&mut self, id: ActionId) -> Result<Option<Registration>> {
        self.ensure_committed()?;
        self.read(
            "SELECT record FROM registration_actions WHERE id=?1",
            &id.0.to_string(),
        )
    }
    fn intent(&mut self, req: &RegisterRepository) -> Result<Registration> {
        let action = Registration {
            create_initial_workspace: req.create_initial_workspace,
            initial_workspace_id: None,
            action_id: req.action_id,
            path: req.path.clone(),
            state: RegistrationState::Pending,
        };
        let record = encode(&action)?;
        self.ensure_committed()?;
        self.write(|db| {
            db.execute(
                "INSERT INTO registration_actions (id,pending,record) VALUES (?1,1,?2)",
                params![req.action_id.0.to_string(), record],
            )
            .map_err(storage)?;
            Ok(action)
        })
    }
    fn pending(&mut self) -> Result<Option<Registration>> {
        self.ensure_committed()?;
        self.connection
            .query_row(
                "SELECT record FROM registration_actions WHERE pending=1 ORDER BY id LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(storage)?
            .map(decode)
            .transpose()
    }
    fn pending_count(&mut self) -> Result<usize> {
        self.ensure_committed()?;
        let count: i64 = self
            .connection
            .query_row(
                "SELECT COUNT(*) FROM registration_actions WHERE pending=1",
                [],
                |r| r.get(0),
            )
            .map_err(storage)?;
        usize::try_from(count).map_err(storage)
    }
    fn find_common(&mut self, common: &str) -> Result<Option<RepositoryRecord>> {
        self.ensure_committed()?;
        self.read(
            "SELECT record FROM repositories WHERE common_dir=?1",
            common,
        )
    }
    fn finish(
        &mut self,
        action: &Registration,
        repository: Option<&RepositoryRecord>,
        status: &RegistrationState,
        initial: Option<&Workspace>,
    ) -> Result<Registration> {
        let action = Registration {
            state: status.clone(),
            ..action.clone()
        };
        let encoded = encode(&action)?;
        let repository = repository.map(|r| Ok((r, encode(r)?))).transpose()?;
        self.ensure_committed()?;
        self.write(|db| {
            if let Some((repo, json)) = repository {
                db.execute("INSERT INTO repositories (id,common_dir,record) VALUES (?1,?2,?3) ON CONFLICT(id) DO NOTHING", params![repo.id.0.to_string(), repo.common_dir, json]).map_err(storage)?;
            }
            if let Some(workspace) = initial {
                write_workspace(db, workspace, Some(&WorkspaceAction::Create { action_id: workspace.creation_action_id, repository_id: workspace.repository_id }))?;
            }
            let changed = db.execute("UPDATE registration_actions SET pending=0,record=?2 WHERE id=?1 AND pending=1", params![action.action_id.0.to_string(), encoded]).map_err(storage)?;
            if changed != 1 { return Err(storage("Registration no longer has pending intent")); }
            Ok(action)
        })
    }
    fn repository(&mut self, id: RepositoryId) -> Result<Option<RepositoryRecord>> {
        self.ensure_committed()?;
        self.read(
            "SELECT record FROM repositories WHERE id=?1",
            &id.0.to_string(),
        )
    }
    fn list(&mut self, after: Option<RepositoryId>, limit: usize) -> Result<Vec<RepositoryRecord>> {
        self.ensure_committed()?;
        let mut query = self
            .connection
            .prepare("SELECT record FROM repositories WHERE id>?1 ORDER BY id LIMIT ?2")
            .map_err(storage)?;
        let rows = query
            .query_map(
                params![
                    after.map(|id| id.0.to_string()).unwrap_or_default(),
                    i64::try_from(limit).map_err(storage)?
                ],
                |r| r.get::<_, String>(0),
            )
            .map_err(storage)?;
        rows.map(|r| decode(r.map_err(storage)?)).collect()
    }
}

fn write_workspace(
    db: &Connection,
    workspace: &Workspace,
    action: Option<&WorkspaceAction>,
) -> Result<()> {
    // A final location is immutable, including on recovery or failed creation.
    let saved: Option<String> = db
        .query_row(
            "SELECT record FROM workspaces WHERE id=?1",
            [workspace.id.0.to_string()],
            |r| r.get(0),
        )
        .optional()
        .map_err(storage)?;
    if let Some(saved) = saved {
        let saved: Workspace = decode(saved)?;
        if saved.name.is_some()
            && (saved.name != workspace.name
                || saved.branch != workspace.branch
                || saved.path != workspace.path)
        {
            return Err(storage("A reserved workspace location cannot be changed"));
        }
    }
    db.execute("INSERT INTO workspaces(id,repository_id,queued,record,name,branch,path) VALUES (?1,?2,?3,?4,?5,?6,?7) ON CONFLICT(id) DO UPDATE SET queued=excluded.queued,record=excluded.record,name=excluded.name,branch=excluded.branch,path=excluded.path", params![workspace.id.0.to_string(), workspace.repository_id.0.to_string(), i64::from(workspace.status == CreationStatus::Queued), encode(workspace)?,workspace.name,workspace.branch,workspace.path]).map_err(storage)?;
    if let Some(action) = action {
        db.execute(
            "INSERT INTO workspace_actions(id,workspace_id,record) VALUES (?1,?2,?3)",
            params![
                action.id().0.to_string(),
                workspace.id.0.to_string(),
                encode(action)?
            ],
        )
        .map_err(storage)?;
    }
    Ok(())
}
impl CreationStore for SqliteStore {
    fn naming(&mut self, repository: RepositoryId) -> Result<crate::workspaces::naming::Naming> {
        self.ensure_committed()?;
        let folder = self
            .connection
            .query_row(
                "SELECT folder FROM repository_folders WHERE repository_id=?1",
                [repository.0.to_string()],
                |r| r.get(0),
            )
            .optional()
            .map_err(storage)?;
        let mut query = self
            .connection
            .prepare("SELECT folder FROM repository_folders")
            .map_err(storage)?;
        let folders = query
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(storage)?
            .map(|r| r.map_err(storage))
            .collect::<Result<_>>()?;
        let mut query = self
            .connection
            .prepare("SELECT name FROM workspaces WHERE repository_id=?1 AND name IS NOT NULL")
            .map_err(storage)?;
        let names = query
            .query_map([repository.0.to_string()], |r| r.get::<_, String>(0))
            .map_err(storage)?
            .map(|r| r.map_err(storage))
            .collect::<Result<_>>()?;
        Ok(crate::workspaces::naming::Naming {
            folder,
            folders,
            names,
        })
    }
    fn reserve_name(&mut self, workspace: &Workspace, folder: &str) -> Result<()> {
        self.ensure_committed()?;
        self.write(|db| {
            db.execute("INSERT INTO repository_folders(repository_id,folder) VALUES (?1,?2) ON CONFLICT(repository_id) DO NOTHING", params![workspace.repository_id.0.to_string(), folder]).map_err(storage)?;
            let saved: String = db.query_row("SELECT folder FROM repository_folders WHERE repository_id=?1", [workspace.repository_id.0.to_string()], |r| r.get(0)).map_err(storage)?;
            if saved != folder { return Err(storage("Repository folder reservation conflicts with saved identity")); }
            write_workspace(db, workspace, None)
        })
    }
    fn workspace(&mut self, id: WorkspaceId) -> Result<Option<Workspace>> {
        self.ensure_committed()?;
        self.read(
            "SELECT record FROM workspaces WHERE id=?1",
            &id.0.to_string(),
        )
    }
    fn workspace_action(&mut self, id: ActionId) -> Result<Option<(WorkspaceAction, WorkspaceId)>> {
        self.ensure_committed()?;
        self.connection
            .query_row(
                "SELECT record,workspace_id FROM workspace_actions WHERE id=?1",
                [id.0.to_string()],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(storage)?
            .map(|(a, w)| {
                Ok((
                    decode(a)?,
                    WorkspaceId(uuid::Uuid::parse_str(&w).map_err(storage)?),
                ))
            })
            .transpose()
    }
    fn save_workspace(
        &mut self,
        workspace: &Workspace,
        action: Option<&WorkspaceAction>,
    ) -> Result<()> {
        self.ensure_committed()?;
        self.write(|db| write_workspace(db, workspace, action))
    }
    fn workspace_pending(&mut self, after: Option<WorkspaceId>) -> Result<Option<Workspace>> {
        self.ensure_committed()?;
        self.connection
            .query_row(
                "SELECT record FROM workspaces WHERE queued=1 AND id>?1 ORDER BY id LIMIT 1",
                [after.map(|id| id.0.to_string()).unwrap_or_default()],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(storage)?
            .map(decode)
            .transpose()
    }
    fn workspace_cleanup_blocked(&mut self, repository: RepositoryId) -> Result<bool> {
        self.ensure_committed()?;
        self.connection.query_row("SELECT EXISTS(SELECT 1 FROM workspaces WHERE repository_id=?1 AND json_extract(record,'$.cleanup_unconfirmed')=1)", [repository.0.to_string()], |r| r.get(0)).map_err(storage)
    }
    fn workspace_count_pending(&mut self) -> Result<usize> {
        self.ensure_committed()?;
        let count: i64 = self.connection.query_row("SELECT COUNT(*) FROM workspaces WHERE queued=1 OR json_extract(record,'$.status.state')='running'", [], |r| r.get(0)).map_err(storage)?;
        usize::try_from(count).map_err(storage)
    }
    fn workspace_list(
        &mut self,
        repository: RepositoryId,
        after: Option<WorkspaceId>,
        limit: usize,
    ) -> Result<Vec<Workspace>> {
        self.ensure_committed()?;
        let mut query = self.connection.prepare("SELECT record FROM workspaces WHERE repository_id=?1 AND id>?2 ORDER BY id LIMIT ?3").map_err(storage)?;
        query
            .query_map(
                params![
                    repository.0.to_string(),
                    after.map(|id| id.0.to_string()).unwrap_or_default(),
                    i64::try_from(limit).map_err(storage)?
                ],
                |r| r.get::<_, String>(0),
            )
            .map_err(storage)?
            .map(|r| decode(r.map_err(storage)?))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    struct Control {
        calls: RefCell<Vec<&'static str>>,
        fail_begin: bool,
        fail_commit: bool,
        fail_rollback: bool,
        active: Cell<bool>,
    }
    impl TransactionControl for Control {
        fn begin(&self) -> Result<()> {
            self.calls.borrow_mut().push("begin");
            if self.fail_begin {
                Err(storage("read-only database"))
            } else {
                self.active.set(true);
                Ok(())
            }
        }
        fn commit(&self) -> Result<()> {
            self.calls.borrow_mut().push("commit");
            if self.fail_commit {
                Err(storage("commit outcome unavailable"))
            } else {
                self.active.set(false);
                Ok(())
            }
        }
        fn rollback(&self) -> Result<()> {
            self.calls.borrow_mut().push("rollback");
            if self.fail_rollback {
                Err(storage("rollback failed"))
            } else {
                self.active.set(false);
                Ok(())
            }
        }
        fn active(&self) -> bool {
            self.active.get()
        }
    }
    #[test]
    fn success_requires_commit_and_begin_failure_blocks_the_write() {
        for fail_begin in [true, false] {
            let control = Control {
                calls: RefCell::new(vec![]),
                fail_begin,
                fail_commit: false,
                fail_rollback: false,
                active: Cell::new(false),
            };
            let result = transaction(&control, || {
                control.calls.borrow_mut().push("write");
                Ok(42)
            });
            if fail_begin {
                assert!(result.is_err());
                assert_eq!(*control.calls.borrow(), ["begin"]);
            } else {
                assert_eq!(result.unwrap(), 42);
                assert_eq!(*control.calls.borrow(), ["begin", "write", "commit"]);
            }
        }
    }
    #[test]
    fn failed_commit_attempts_rollback_and_keeps_uncertain_cleanup_visible() {
        for fail_rollback in [true, false] {
            let control = Control {
                calls: RefCell::new(vec![]),
                fail_begin: false,
                fail_commit: true,
                fail_rollback,
                active: Cell::new(false),
            };
            let error = transaction(&control, || Ok(42)).unwrap_err();
            assert_eq!(*control.calls.borrow(), ["begin", "commit", "rollback"]);
            assert_eq!(error.code, ErrorCode::Storage);
            assert_eq!(control.active.get(), fail_rollback);
            assert!(error.message.contains(if fail_rollback {
                "cleanup failed"
            } else {
                "commit outcome"
            }));
        }
    }
    #[test]
    fn current_workspace_records_reopen_without_provider_and_retain_recovery_facts() {
        use serde_json::json;
        let pending = json!({
            "id": "00000000-0000-0000-0000-000000000001",
            "repository_id": "00000000-0000-0000-0000-000000000002",
            "creation_action_id": "00000000-0000-0000-0000-000000000003",
            "name": null, "branch": null, "path": null, "legacy": false,
            "base": null,
            "artifacts": { "branch": "unknown", "folder": "unknown" },
            "status": { "state": "queued" }, "cleanup_unconfirmed": false
        });
        let allocated = json!({
            "name": "kyoto", "branch": "workspace/kyoto",
            "path": "/private/state/workspaces/repo/kyoto/checkout",
            "base": { "reference": "refs/heads/main", "commit": "a".repeat(40) },
            "artifacts": { "branch": "owned", "folder": "worktree" }
        });
        let error = json!({ "code": "git", "message": "Worktree creation failed; files retained" });
        for status in [
            json!({ "state": "queued" }),
            json!({ "state": "running", "stage": "creating_worktree" }),
            json!({ "state": "ready" }),
            json!({ "state": "failed", "error": error }),
            json!({ "state": "uncertain", "error": error }),
        ] {
            let mut record = pending.clone();
            if status["state"] != "queued" {
                for (key, value) in allocated.as_object().unwrap() {
                    record[key] = value.clone();
                }
            }
            if status["state"] == "failed" || status["state"] == "uncertain" {
                record["artifacts"]["folder"] = json!("partial");
                record["cleanup_unconfirmed"] = json!(true);
            }
            record["status"] = status;
            let restored: Workspace = decode(record.to_string()).unwrap();
            let written: serde_json::Value =
                serde_json::from_str(&encode(&restored).unwrap()).unwrap();
            assert_eq!(written, record);
            assert!(written.get("provider").is_none());
        }
    }

    #[test]
    fn corrupt_saved_metadata_is_a_storage_failure_instead_of_an_empty_result() {
        assert_eq!(
            decode::<RepositoryRecord>("{\"id\":\"wrong\"}".into())
                .unwrap_err()
                .code,
            ErrorCode::Storage
        );
    }
}
