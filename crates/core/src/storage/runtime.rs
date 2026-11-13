//! Narrow supervisor-owned storage seam. Runtime policy does not depend on workspaces.
use crate::runtimes::{Action, Configuration, ErrorCode, Result, RuntimeError, RuntimeStore};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(crate) struct RuntimeSqlite {
    connection: Option<Connection>,
    path: PathBuf,
    boot: String,
    dirty: bool,
}
pub(crate) fn open(directory: &Path, boot: &str) -> Result<RuntimeSqlite> {
    // The common metadata opener owns privacy checks and migrations. Its historical
    // repository error is contained here; no repository contract crosses this seam.
    let database = super::SqliteStore::open(directory).map_err(|_| RuntimeError::storage())?;
    Ok(RuntimeSqlite {
        connection: Some(database.connection),
        path: database.path,
        boot: boot.into(),
        dirty: false,
    })
}
fn decode<T: serde::de::DeserializeOwned>(text: String) -> Result<T> {
    serde_json::from_str(&text).map_err(|_| RuntimeError::storage())
}
fn encode(value: &impl serde::Serialize) -> Result<String> {
    serde_json::to_string(value).map_err(|_| RuntimeError::storage())
}

// Outcomes, including COMMIT's resulting transaction state, are substituted in units.
trait Transaction {
    fn begin(&self) -> bool;
    fn commit(&self) -> bool;
    fn rollback(&self) -> bool;
    fn active(&self) -> bool;
}
impl Transaction for Connection {
    fn begin(&self) -> bool {
        self.execute_batch("BEGIN IMMEDIATE").is_ok()
    }
    fn commit(&self) -> bool {
        self.execute_batch("COMMIT").is_ok()
    }
    fn rollback(&self) -> bool {
        self.execute_batch("ROLLBACK").is_ok()
    }
    fn active(&self) -> bool {
        !self.is_autocommit()
    }
}
fn transact(control: &impl Transaction, body: impl FnOnce() -> Result<()>) -> Result<()> {
    if !control.begin() {
        return Err(RuntimeError::storage());
    }
    let outcome = body();
    if outcome.is_ok() && control.commit() {
        return Ok(());
    }
    if control.active() && control.rollback() {
        return Err(RuntimeError::storage());
    }
    Err(RuntimeError::new(
        ErrorCode::PersistenceUncertain,
        "Runtime save or recovery persistence is uncertain. Refresh authoritative saved configuration before further setup work; do not repeat the action.",
    ))
}
impl RuntimeSqlite {
    fn readable(&mut self) -> Result<()> {
        if self.dirty || self.connection.as_ref().is_none_or(|c| !c.is_autocommit()) {
            // Replace even an active/poisoned connection. Reads never observe its own
            // uncommitted writes, and no failed write is retried.
            // Close the old transaction before opening a new connection. Otherwise
            // its own unresolved lock can prevent authoritative readback.
            self.connection.take();
            self.connection =
                Some(super::SqliteStore::connect(&self.path).map_err(|_| RuntimeError::storage())?);
            self.dirty = false;
        }
        Ok(())
    }
    fn write(&mut self, body: impl FnOnce(&Connection) -> Result<()>) -> Result<()> {
        self.readable()?;
        let db = self.connection.as_ref().ok_or_else(RuntimeError::storage)?;
        let result = transact(db, || body(db));
        if result.is_err() {
            self.dirty = true;
        }
        result
    }
}
impl RuntimeStore for RuntimeSqlite {
    fn configuration(&mut self) -> Result<Configuration> {
        self.readable()?;
        self.connection
            .as_ref()
            .ok_or_else(RuntimeError::storage)?
            .query_row(
                "SELECT record FROM runtime_configuration WHERE id=1",
                [],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| RuntimeError::storage())?
            .map(decode)
            .transpose()
            .map(|c| c.unwrap_or_default())
    }
    fn action(&mut self, id: Uuid) -> Result<Option<Action>> {
        self.readable()?;
        self.connection
            .as_ref()
            .ok_or_else(RuntimeError::storage)?
            .query_row(
                "SELECT record FROM runtime_actions WHERE id=?1",
                [id.to_string()],
                |r| r.get::<_, String>(0),
            )
            .optional()
            .map_err(|_| RuntimeError::storage())?
            .map(decode)
            .transpose()
    }
    fn blocked(&mut self) -> Result<bool> {
        self.readable()?;
        let previous: Option<String> = self
            .connection
            .as_ref()
            .ok_or_else(RuntimeError::storage)?
            .query_row("SELECT boot FROM runtime_recovery WHERE id=1", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|_| RuntimeError::storage())?;
        Ok(previous.is_some_and(|old| old == self.boot || old.is_empty() || self.boot.is_empty()))
    }
    fn intent(&mut self, action: &Action, boot: &str) -> Result<()> {
        let record = encode(action)?;
        self.write(|db| {
            db.execute(
                "INSERT INTO runtime_actions(id,record) VALUES(?1,?2)",
                params![action.action_id.to_string(), record],
            )
            .map_err(|_| RuntimeError::storage())?;
            if matches!(
                action.request,
                crate::runtimes::RuntimeRequest::Check { .. }
            ) {
                db.execute(
                    "INSERT OR REPLACE INTO runtime_recovery(id,action_id,boot) VALUES(1,?1,?2)",
                    params![action.action_id.to_string(), boot],
                )
                .map_err(|_| RuntimeError::storage())?;
            }
            Ok(())
        })
    }
    fn save(&mut self, action: &Action, configuration: &Configuration) -> Result<()> {
        let action_record = encode(action)?;
        let configuration = encode(configuration)?;
        self.write(|db| {
            db.execute(
                "INSERT OR REPLACE INTO runtime_configuration(id,record) VALUES(1,?1)",
                [configuration],
            )
            .map_err(|_| RuntimeError::storage())?;
            db.execute(
                "UPDATE runtime_actions SET record=?2 WHERE id=?1",
                params![action.action_id.to_string(), action_record],
            )
            .map_err(|_| RuntimeError::storage())?;
            Ok(())
        })
    }
    fn finish(&mut self, action: &Action, cleanup_confirmed: bool) -> Result<()> {
        let record = encode(action)?;
        self.write(|db| {
            db.execute(
                "UPDATE runtime_actions SET record=?2 WHERE id=?1",
                params![action.action_id.to_string(), record],
            )
            .map_err(|_| RuntimeError::storage())?;
            if cleanup_confirmed {
                db.execute(
                    "DELETE FROM runtime_recovery WHERE action_id=?1",
                    [action.action_id.to_string()],
                )
                .map_err(|_| RuntimeError::storage())?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    struct Commit {
        active: Cell<bool>,
        committed: bool,
        rollback: bool,
    }
    impl Transaction for Commit {
        fn begin(&self) -> bool {
            self.active.set(true);
            true
        }
        fn commit(&self) -> bool {
            if self.committed {
                self.active.set(false);
            }
            false
        }
        fn rollback(&self) -> bool {
            self.active.set(!self.rollback);
            self.rollback
        }
        fn active(&self) -> bool {
            self.active.get()
        }
    }
    #[test]
    fn commit_failure_with_confirmed_rollback_is_definite_rejection() {
        let c = Commit {
            active: Cell::new(false),
            committed: false,
            rollback: true,
        };
        assert_eq!(
            transact(&c, || Ok(())).unwrap_err().code,
            ErrorCode::Storage
        );
    }
    #[test]
    fn commit_error_after_transaction_end_or_failed_rollback_requires_readback() {
        for (committed, rollback) in [(true, true), (false, false)] {
            let c = Commit {
                active: Cell::new(false),
                committed,
                rollback,
            };
            assert_eq!(
                transact(&c, || Ok(())).unwrap_err().code,
                ErrorCode::PersistenceUncertain
            );
        }
    }
}
