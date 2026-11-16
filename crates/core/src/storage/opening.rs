use super::*;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous};
use std::{
    fs::{self, OpenOptions},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
};

pub(super) const DATABASE: &str = "metadata.sqlite3";
const SCHEMA_VERSION: i64 = 1;
const APPLICATION_ID: i64 = 0x41575332;
const SCHEMA: &str = "
CREATE TABLE storage_revision(id INTEGER PRIMARY KEY CHECK(id=1), revision INTEGER NOT NULL CHECK(revision>=0));
INSERT INTO storage_revision VALUES(1,0);
CREATE TABLE repositories(id TEXT PRIMARY KEY NOT NULL, common_dir TEXT UNIQUE NOT NULL CHECK(length(CAST(common_dir AS BLOB))<=4096), record TEXT NOT NULL CHECK(length(CAST(record AS BLOB))<=49152));
CREATE TABLE registration_actions(id TEXT PRIMARY KEY NOT NULL, payload TEXT NOT NULL CHECK(length(CAST(payload AS BLOB))<=49152), pending INTEGER NOT NULL CHECK(pending IN(0,1)), record TEXT NOT NULL CHECK(length(CAST(record AS BLOB))<=49152));
CREATE INDEX pending_registrations ON registration_actions(pending,id);
CREATE TABLE repository_folders(repository_id TEXT PRIMARY KEY NOT NULL REFERENCES repositories(id), folder TEXT UNIQUE NOT NULL CHECK(length(CAST(folder AS BLOB))<=4096));
CREATE TABLE workspaces(id TEXT PRIMARY KEY NOT NULL, repository_id TEXT NOT NULL REFERENCES repositories(id), creation_action_id TEXT UNIQUE NOT NULL, queued INTEGER NOT NULL CHECK(queued IN(0,1)), record TEXT NOT NULL CHECK(length(CAST(record AS BLOB))<=49152), name TEXT, branch TEXT, path TEXT, allocation_cursor INTEGER NOT NULL DEFAULT 0 CHECK(allocation_cursor>=0), CHECK((name IS NULL AND branch IS NULL AND path IS NULL) OR (name IS NOT NULL AND branch IS NOT NULL AND path IS NOT NULL)));
CREATE INDEX workspace_navigation ON workspaces(repository_id,id);
CREATE INDEX queued_workspaces ON workspaces(queued,id);
CREATE UNIQUE INDEX reserved_workspace_names ON workspaces(repository_id,name);
CREATE UNIQUE INDEX reserved_workspace_branches ON workspaces(repository_id,branch);
CREATE UNIQUE INDEX reserved_workspace_paths ON workspaces(path);
CREATE TABLE workspace_actions(id TEXT PRIMARY KEY NOT NULL, workspace_id TEXT NOT NULL REFERENCES workspaces(id), record TEXT NOT NULL CHECK(length(CAST(record AS BLOB))<=49152));
CREATE TABLE provider_executables(scope TEXT PRIMARY KEY NOT NULL CHECK(length(CAST(scope AS BLOB))<=4096), action TEXT NOT NULL, path TEXT NOT NULL CHECK(length(CAST(path AS BLOB))<=4096));
CREATE TABLE provider_login_fences(scope TEXT PRIMARY KEY NOT NULL CHECK(length(CAST(scope AS BLOB))<=4096), attempt TEXT NOT NULL, pending INTEGER NOT NULL CHECK(pending IN(0,1)));
CREATE TABLE provider_actions(id TEXT PRIMARY KEY NOT NULL, payload TEXT NOT NULL CHECK(length(CAST(payload AS BLOB))<=49152), completed INTEGER NOT NULL CHECK(completed IN(0,1)));
CREATE TABLE action_identities(id TEXT PRIMARY KEY NOT NULL, kind TEXT NOT NULL, payload TEXT NOT NULL CHECK(length(CAST(payload AS BLOB))<=49152));
PRAGMA application_id=1096241970;
PRAGMA user_version=1;";

pub(super) fn inspect(state: &Path, create: bool) -> Result<PathBuf> {
    let canonical = fs::canonicalize(state)?;
    if canonical != state {
        return Err(Error::Invalid(
            "State root must be canonical; aliases are rejected",
        ));
    }
    let metadata = fs::symlink_metadata(state)?;
    // SAFETY: geteuid has no preconditions and only reads the process credential.
    let uid = unsafe { libc::geteuid() };
    if !metadata.is_dir() || metadata.uid() != uid || metadata.mode() & 0o077 != 0 {
        return Err(Error::Invalid(
            "State root must be an owner-private directory",
        ));
    }
    let path = state.join(DATABASE);
    for suffix in ["", "-journal", "-wal", "-shm"] {
        match fs::symlink_metadata(state.join(format!("{DATABASE}{suffix}"))) {
            Ok(m) => validate_file(m.is_file(), m.uid() == uid, m.nlink(), m.mode())?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    if create {
        if state.join("workspace.sqlite3").try_exists()? {
            return Err(Error::Invalid(
                "Retired metadata found. Stop its owner and deliberately select fresh private development state; preserve repositories, worktrees and runtime credentials. No files were reset",
            ));
        }
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)?;
    } else if !path.try_exists()? {
        return Err(Error::Invalid(
            "Metadata disappeared; replacement must not create a new database",
        ));
    }
    Ok(path)
}
fn validate_file(regular: bool, owner: bool, links: u64, mode: u32) -> Result<()> {
    if !regular || !owner || links != 1 || mode & 0o077 != 0 {
        return Err(Error::Invalid(
            "Metadata must be owner-private regular single-link files",
        ));
    }
    Ok(())
}
async fn validate_path(state: PathBuf) -> Result<()> {
    tokio::task::spawn_blocking(move || inspect(&state, false))
        .await
        .map_err(|_| Error::Panic)??;
    Ok(())
}
fn options(path: &Path, reader: bool) -> SqliteConnectOptions {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false)
        .read_only(reader)
        .foreign_keys(true)
        .synchronous(SqliteSynchronous::Full)
        .pragma("fullfsync", "ON")
        .pragma("wal_autocheckpoint", "1000")
        .busy_timeout(Duration::from_millis(100))
        .row_buffer_size(1)
        .command_buffer_size(8)
        .statement_cache_capacity(16);
    if reader {
        options
    } else {
        options.journal_mode(SqliteJournalMode::Wal)
    }
}
pub(super) async fn effective(connection: &mut SqliteConnection) -> Result<()> {
    let journal: String = sqlx::query_scalar("PRAGMA journal_mode")
        .fetch_one(&mut *connection)
        .await?;
    if !journal.eq_ignore_ascii_case("wal") {
        return Err(Error::Invalid("WAL is required"));
    }
    for (sql, expected) in [
        ("PRAGMA synchronous", 2_i64),
        ("PRAGMA fullfsync", 1),
        ("PRAGMA foreign_keys", 1),
        ("PRAGMA busy_timeout", 100),
        ("PRAGMA wal_autocheckpoint", 1000),
    ] {
        let value: i64 = sqlx::query_scalar(sql).fetch_one(&mut *connection).await?;
        if value != expected {
            return Err(Error::Invalid("Required effective SQLite setting rejected"));
        }
    }
    Ok(())
}
fn pool(state: &Path, max: u32, control: Arc<Control>) -> SqlitePoolOptions {
    let establish_state = state.to_owned();
    let acquire_state = state.to_owned();
    let establish_control = control.clone();
    let acquire_control = control.clone();
    SqlitePoolOptions::new()
        .max_connections(max)
        .min_connections(0)
        .acquire_timeout(Duration::from_secs(1))
        .idle_timeout(None)
        .max_lifetime(None)
        .after_connect(move |connection, _| {
            let state = establish_state.clone();
            let control = establish_control.clone();
            Box::pin(async move {
                control.ready_connection().await.map_err(hook_error)?;
                let result = async {
                    validate_path(state).await?;
                    effective(connection).await
                }
                .await;
                observe_hook(&control.admission, result)
            })
        })
        .before_acquire(move |connection, _| {
            let state = acquire_state.clone();
            let control = acquire_control.clone();
            Box::pin(async move {
                control.ready_connection().await.map_err(hook_error)?;
                let result = async {
                    validate_path(state).await?;
                    effective(connection).await
                }
                .await;
                observe_hook(&control.admission, result)?;
                Ok(true)
            })
        })
        .after_release(move |connection, _| {
            let control = control.clone();
            Box::pin(async move {
                // Ping orders the worker's queued rollback before inspecting tracked depth.
                match reusable_after_cleanup(connection).await {
                    Ok(true) => Ok(true),
                    Ok(false) => {
                        let _ = observe_hook(
                            &control.admission,
                            Err(Error::Invalid(
                                "Return guard rejected unresolved transaction",
                            )),
                        );
                        Ok(false)
                    }
                    Err(error) => {
                        let _ = observe_hook(&control.admission, Err(error));
                        Ok(false)
                    }
                }
            })
        })
}
fn observe_hook(
    admission: &Mutex<Admission>,
    result: Result<()>,
) -> std::result::Result<(), sqlx::Error> {
    if let Err(error) = &result {
        let mut state = admission.lock().expect("storage admission poisoned");
        if !state.retirement_unknown {
            state.faults.push(Fault {
                action: None,
                error: error.clone(),
            });
        }
        state.fenced = true;
        // Hook-driven rejection uses SQLx's unreported close path: do not authorize replacement.
        state.retirement_unknown = true;
    }
    result.map_err(hook_error)
}
fn hook_error(e: Error) -> sqlx::Error {
    sqlx::Error::Configuration(Box::new(e))
}
async fn schema(writer: &SqlitePool, control: Arc<Control>) -> Result<()> {
    let mut connection = writer.acquire().await?;
    let result = async {
        let mut tx = connection.begin().await?;
        let version: i64 = sqlx::query_scalar("PRAGMA user_version").fetch_one(&mut *tx).await?;
        let application: i64 = sqlx::query_scalar("PRAGMA application_id").fetch_one(&mut *tx).await?;
        if version == 0 && application == 0 {
            let tables: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'").fetch_one(&mut *tx).await?;
            if tables != 0 { return Err(Error::Invalid("Unrecognized metadata. Select fresh private development state; no migration or reset was performed")); }
            sqlx::raw_sql(SCHEMA).execute(&mut *tx).await?;
        } else if version != SCHEMA_VERSION || application != APPLICATION_ID {
            return Err(Error::Invalid("Incompatible metadata. Stop the owner and deliberately select fresh private development state; no migration or reset was performed"));
        }
        tx.commit().await?;
        Ok(())
    }.await;
    match result {
        Ok(()) => Ok(()),
        Err(original) => Err(retire_failed(control, original, true, async {
            connection.close().await.map_err(Error::from)
        })
        .await),
    }
}
pub(super) async fn open(
    state: &Path,
    control: Arc<Control>,
) -> Result<(SqlitePool, SqlitePool, PathBuf)> {
    let state_path = state.to_owned();
    let path = tokio::task::spawn_blocking(move || inspect(&state_path, true))
        .await
        .map_err(|_| Error::Panic)??;
    let writer = pool(state, 1, control.clone()).connect_lazy_with(options(&path, false));
    if let Err(error) = schema(&writer, control.clone()).await {
        writer.close().await;
        return Err(error);
    }
    let readers = pool(state, 2, control).connect_lazy_with(options(&path, true));
    let opened = readers.acquire().await;
    if let Err(error) = opened {
        readers.close().await;
        writer.close().await;
        return Err(error.into());
    }
    drop(opened);
    Ok((writer, readers, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn substituted_file_metadata_rejects_aliases_wrong_owner_and_public_modes() {
        assert!(validate_file(true, true, 1, 0o100600).is_ok());
        for (regular, owner, links, mode) in [
            (false, true, 1, 0o600),
            (true, false, 1, 0o600),
            (true, true, 2, 0o600),
            (true, true, 1, 0o644),
        ] {
            assert!(validate_file(regular, owner, links, mode).is_err());
        }
    }
}

// Substituted worker outcomes retain application ordering without exercising SQLite in units.
trait ReturnConnection {
    async fn order_cleanup(&mut self) -> Result<()>;
    fn tracked_transaction(&self) -> bool;
}
impl ReturnConnection for SqliteConnection {
    async fn order_cleanup(&mut self) -> Result<()> {
        self.ping().await.map_err(Error::from)
    }
    fn tracked_transaction(&self) -> bool {
        self.is_in_transaction()
    }
}
async fn reusable_after_cleanup(connection: &mut impl ReturnConnection) -> Result<bool> {
    connection.order_cleanup().await?;
    Ok(policy::reusable(true, connection.tracked_transaction()))
}
#[cfg(test)]
mod return_tests {
    use super::*;
    struct Worker {
        ordered: bool,
        rollback_failed: bool,
        ping_ok: bool,
    }
    impl ReturnConnection for Worker {
        async fn order_cleanup(&mut self) -> Result<()> {
            self.ordered = true;
            if self.ping_ok {
                Ok(())
            } else {
                Err(Error::Invalid("ping failure"))
            }
        }
        fn tracked_transaction(&self) -> bool {
            assert!(self.ordered, "depth sampled before queued cleanup");
            self.rollback_failed
        }
    }
    #[tokio::test]
    async fn queued_rollback_is_ordered_before_the_reuse_decision() {
        for (rollback_failed, ping_ok, expected) in [
            (false, true, true),
            (true, true, false),
            (false, false, false),
        ] {
            let mut worker = Worker {
                ordered: false,
                rollback_failed,
                ping_ok,
            };
            let result = reusable_after_cleanup(&mut worker).await;
            if ping_ok {
                assert_eq!(result.unwrap(), expected);
            } else {
                assert!(matches!(result, Err(Error::Invalid("ping failure"))));
            }
        }
    }
}
