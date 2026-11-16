//! One private SQLx owner. Domain owners decide recovery; storage reports settlement.
//! The source-linked checkpoint compiles this exact module while 03/04 convert callers.
use crate::workspaces::*;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::{Connection, Row, Sqlite, SqliteConnection, SqlitePool, Transaction};
use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, mpsc, oneshot, watch};
#[path = "storage/opening.rs"]
mod opening;
#[path = "storage/operations.rs"]
mod operations;
#[path = "storage/policy.rs"]
mod policy;
use policy::{ITEM_BYTES, PAGE_BYTES, PAGE_ROWS};

// Retained decision seams for provider units; 04 replaces their synchronous consumers.
pub(crate) trait ProviderLoginPersistence: Send + Sync {
    fn record(&self, attempt: uuid::Uuid, pending: bool) -> crate::workspaces::Result<()>;
}
pub(crate) trait ProviderSelectionPersistence: Send + Sync {
    fn remember(&self, id: uuid::Uuid, path: &str) -> crate::workspaces::Result<()>;
}

type Result<T> = std::result::Result<T, Error>;
#[derive(Debug, Clone)]
pub(crate) enum Error {
    Sql(Arc<sqlx::Error>),
    Conflict(Arc<sqlx::Error>),
    CandidateConflict,
    Corrupt,
    Io(Arc<std::io::Error>),
    Invalid(&'static str),
    Capacity,
    Oversized,
    Closed,
    Fenced,
    Panic,
    Opening {
        original: Box<Error>,
        faults: Vec<Fault>,
        retirement_unknown: bool,
    },
    Write {
        original: Box<Error>,
        retirement: Option<Box<Error>>,
        uncertain: bool,
    },
}
impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        if matches!(&e, sqlx::Error::Database(db) if db.is_unique_violation()) {
            Self::Conflict(Arc::new(e))
        } else {
            Self::Sql(Arc::new(e))
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Arc::new(e))
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sql(e) | Self::Conflict(e) => Some(e.as_ref()),
            Self::Io(e) => Some(e.as_ref()),
            Self::Write { original, .. } | Self::Opening { original, .. } => {
                Some(original.as_ref())
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Reservation {
    pub workspace: Workspace,
    pub folder: String,
    pub continuation: u64,
}
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum Write {
    RegistrationIntent(RegisterRepository),
    RegistrationFinish {
        action: Registration,
        repository: Option<RepositoryRecord>,
        initial: Option<Workspace>,
    },
    WorkspaceSave {
        workspace: Workspace,
        action: Option<WorkspaceAction>,
    },
    Reservation(Reservation),
    AllocationCursor {
        workspace_id: WorkspaceId,
        cursor: u64,
    },
    ProviderSelection {
        scope: String,
        action: uuid::Uuid,
        path: String,
    },
    ProviderLogin {
        scope: String,
        attempt: uuid::Uuid,
        pending: bool,
    },
}
impl Write {
    fn identity(&self) -> uuid::Uuid {
        match self {
            Self::RegistrationIntent(r) => r.action_id.0,
            Self::RegistrationFinish { action, .. } => action.action_id.0,
            Self::WorkspaceSave { workspace, action } => action
                .as_ref()
                .map_or(workspace.creation_action_id.0, |a| a.id().0),
            Self::Reservation(r) => r.workspace.creation_action_id.0,
            Self::AllocationCursor { workspace_id, .. } => workspace_id.0,
            Self::ProviderSelection { action, .. } => *action,
            Self::ProviderLogin { attempt, .. } => *attempt,
        }
    }
}
#[derive(Debug)]
pub(crate) enum Read {
    Registration(ActionId),
    Repository(RepositoryId),
    CommonDirectory(String),
    Repositories {
        after: Option<RepositoryId>,
    },
    PendingRegistrations {
        after: Option<ActionId>,
    },
    Workspace(WorkspaceId),
    WorkspaceAction(ActionId),
    Workspaces {
        repository: RepositoryId,
        after: Option<WorkspaceId>,
    },
    PendingWorkspaces {
        after: Option<WorkspaceId>,
    },
    Folder {
        repository: RepositoryId,
        candidate: String,
    },
    Name {
        repository: RepositoryId,
        candidate: String,
    },
    AllocationCursor(WorkspaceId),
    CleanupBlocked(RepositoryId),
    ProviderSelection(String),
    ProviderLogin(String),
    ProviderAction(uuid::Uuid),
}
impl Read {
    fn validate(&self) -> Result<()> {
        let text = match self {
            Self::CommonDirectory(s) | Self::ProviderSelection(s) | Self::ProviderLogin(s) => {
                Some(s)
            }
            Self::Folder { candidate, .. } | Self::Name { candidate, .. } => Some(candidate),
            _ => None,
        };
        if text.is_some_and(|s| s.len() > 4096) {
            return Err(Error::Oversized);
        }
        Ok(())
    }
}
#[derive(Debug)]
pub(crate) struct Snapshot {
    pub revision: u64,
    /// Bounded current domain records, decoded by the typed helpers below.
    records: Vec<String>,
    pub next: Option<String>,
}
impl Snapshot {
    fn records<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        self.records
            .iter()
            .map(|r| serde_json::from_str(r).map_err(|_| Error::Corrupt))
            .collect()
    }
}
#[derive(Debug)]
pub(crate) enum Outcome {
    Written { revision: u64 },
    Snapshot(Snapshot),
}
#[derive(Debug)]
pub(crate) struct Delivery {
    outcome: Result<Outcome>,
    // Capacity covers completed but undelivered results as well as physical work.
    _slot: OwnedSemaphorePermit,
}
/// A decoded page retains its read interest through consumption, including decoded copies.
#[derive(Debug)]
pub(crate) struct Records<T> {
    pub revision: u64,
    pub next: Option<String>,
    records: Vec<T>,
    _slot: OwnedSemaphorePermit,
}
impl<T> Records<T> {
    pub(crate) fn items(&self) -> &[T] {
        &self.records
    }
    pub(crate) fn into_items(self) -> Vec<T> {
        self.records
    }
}
impl Delivery {
    #[cfg(test)]
    pub(crate) fn controlled(
        outcome: Result<Vec<serde_json::Value>>,
        next: Option<String>,
    ) -> Self {
        Self {
            outcome: outcome.map(|records| {
                Outcome::Snapshot(Snapshot {
                    revision: 1,
                    records: records.into_iter().map(|r| r.to_string()).collect(),
                    next,
                })
            }),
            _slot: Arc::new(Semaphore::new(1)).try_acquire_owned().unwrap(),
        }
    }
    pub(crate) fn revision(&self) -> Result<u64> {
        match &self.outcome {
            Ok(Outcome::Written { revision }) => Ok(*revision),
            Ok(Outcome::Snapshot(snapshot)) => Ok(snapshot.revision),
            Err(error) => Err(error.clone()),
        }
    }
    pub(crate) fn into_records<T: DeserializeOwned>(self) -> Result<Records<T>> {
        let snapshot = match self.outcome? {
            Outcome::Snapshot(snapshot) => snapshot,
            _ => return Err(Error::Invalid("Expected a metadata page")),
        };
        let records = snapshot.records()?;
        Ok(Records {
            revision: snapshot.revision,
            next: snapshot.next,
            records,
            _slot: self._slot,
        })
    }
}
#[derive(Debug, Clone)]
pub(crate) struct Fault {
    pub action: Option<uuid::Uuid>,
    pub error: Error,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct Checkpoint {
    pub busy: i64,
    pub log_frames: i64,
    pub checkpointed_frames: i64,
    pub wal_bytes: u64,
    pub elapsed: Duration,
}
#[derive(Debug, Clone, Default)]
pub(crate) struct CloseReport {
    pub faults: Vec<Fault>,
    pub checkpoint: Option<Checkpoint>,
}

struct Admission {
    closed: bool,
    fenced: bool,
    retirement_unknown: bool,
    retiring: usize,
    writes: std::collections::BTreeMap<u64, uuid::Uuid>,
    next_write: u64,
    faults: Vec<Fault>,
}
struct Shared {
    writer: SqlitePool,
    readers: SqlitePool,
    path: PathBuf,
    control: Arc<Control>,
}
struct Control {
    admission: Arc<Mutex<Admission>>,
    changes: watch::Sender<u64>,
}
impl Control {
    fn new() -> Self {
        let (changes, _) = watch::channel(0);
        Self {
            changes,
            admission: Arc::new(Mutex::new(Admission {
                closed: false,
                fenced: false,
                retirement_unknown: false,
                retiring: 0,
                writes: Default::default(),
                next_write: 0,
                faults: vec![],
            })),
        }
    }
    fn fault(&self, action: Option<uuid::Uuid>, error: Error) {
        let mut state = self.admission.lock().expect("storage admission poisoned");
        state.fenced = true;
        state.retirement_unknown |= matches!(
            &error,
            Error::Panic
                | Error::Write {
                    retirement: Some(_),
                    ..
                }
        );
        // At most 72 admitted operations can fail after the first failure fences admission.
        state.faults.push(Fault { action, error });
    }
    fn settle(&self, receipt: WriteReceipt) {
        let mut state = self.admission.lock().expect("storage admission poisoned");
        assert_eq!(
            state.writes.remove(&receipt.sequence),
            Some(receipt.action),
            "admitted write tracked"
        );
        self.changes.send_modify(|n| *n = n.wrapping_add(1));
    }
    #[cfg(test)]
    fn settled(
        &self,
        action: Option<uuid::Uuid>,
    ) -> impl std::future::Future<Output = Result<()>> + '_ {
        let pending = self
            .admission
            .lock()
            .expect("storage admission poisoned")
            .frontier(action);
        async move { self.wait_settled(&pending).await }
    }
    async fn wait_settled(&self, pending: &[u64]) -> Result<()> {
        let mut changes = self.changes.subscribe();
        loop {
            {
                let state = self.admission.lock().expect("storage admission poisoned");
                if state.retirement_unknown {
                    return Err(Error::Fenced);
                }
                if !pending
                    .iter()
                    .any(|sequence| state.writes.contains_key(sequence))
                {
                    return Ok(());
                }
            }
            changes.changed().await.map_err(|_| Error::Closed)?;
        }
    }
}
impl Admission {
    fn frontier(&self, action: Option<uuid::Uuid>) -> Vec<u64> {
        self.writes
            .iter()
            .filter(|(_, id)| action.is_none_or(|action| **id == action))
            .map(|(sequence, _)| *sequence)
            .collect()
    }
}
#[derive(Debug, Clone, Copy)]
struct WriteReceipt {
    sequence: u64,
    action: uuid::Uuid,
}

enum Operation {
    Write {
        receipt: WriteReceipt,
        payload: String,
    },
    Read {
        read: Read,
        pending: Vec<u64>,
    },
}
struct Job {
    operation: Operation,
    reply: oneshot::Sender<Delivery>,
    slot: OwnedSemaphorePermit,
}
#[derive(Clone)]
pub(crate) struct Storage {
    shared: Arc<Shared>,
    jobs: mpsc::Sender<Job>,
    write_slots: Arc<Semaphore>,
    read_slots: Arc<Semaphore>,
    done: watch::Receiver<Option<CloseReport>>,
}
pub(crate) struct Opening {
    result: watch::Receiver<Option<Result<Storage>>>,
}
impl Opening {
    pub(crate) async fn wait(&mut self) -> Result<Storage> {
        loop {
            if let Some(result) = self.result.borrow_and_update().clone() {
                return result;
            }
            self.result.changed().await.map_err(|_| Error::Panic)?;
        }
    }
}
impl Storage {
    /// Retain this receipt with core startup until observed; cancelling wait never cancels open/cleanup.
    pub(crate) fn start(state: PathBuf) -> Opening {
        let (result, receiver) = watch::channel(None);
        tokio::spawn(async move {
            result.send_replace(Some(Self::open(&state).await));
        });
        Opening { result: receiver }
    }

    /// Called by retained core startup while it holds the state lease. No per-call runtime.
    async fn open(state: &Path) -> Result<Self> {
        let control = Arc::new(Control::new());
        let (writer, readers, path) = opening::open(state, control.clone())
            .await
            .map_err(|error| opening::failure(&control, error))?;
        let shared = Arc::new(Shared {
            writer,
            readers,
            path,
            control,
        });
        let (jobs, receive) = mpsc::channel(72);
        let (done, completion) = watch::channel(None);
        tokio::spawn(run(shared.clone(), receive, done));
        Ok(Self {
            shared,
            jobs,
            write_slots: Arc::new(Semaphore::new(64)),
            read_slots: Arc::new(Semaphore::new(8)),
            done: completion,
        })
    }
    /// Synchronous admission prevents an unbounded semaphore waiter/task queue.
    pub(crate) fn write(&self, write: Write) -> Result<oneshot::Receiver<Delivery>> {
        let slot = self
            .write_slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::Capacity)?;
        let payload = encode(&write, ITEM_BYTES)?;
        let identity = write.identity();
        let mut state = self
            .shared
            .control
            .admission
            .lock()
            .expect("storage admission poisoned");
        if state.closed {
            return Err(Error::Closed);
        }
        if state.fenced {
            return Err(Error::Fenced);
        }
        state.next_write = state.next_write.checked_add(1).ok_or(Error::Capacity)?;
        let receipt = WriteReceipt {
            sequence: state.next_write,
            action: identity,
        };
        let (reply, result) = oneshot::channel();
        self.jobs
            .try_send(Job {
                operation: Operation::Write { receipt, payload },
                reply,
                slot,
            })
            .map_err(|_| Error::Capacity)?;
        state.writes.insert(receipt.sequence, identity);
        Ok(result)
    }
    pub(crate) fn read(&self, read: Read) -> Result<oneshot::Receiver<Delivery>> {
        read.validate()?;
        let slot = self
            .read_slots
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::Capacity)?;
        let state = self
            .shared
            .control
            .admission
            .lock()
            .expect("storage admission poisoned");
        if state.closed {
            return Err(Error::Closed);
        }
        if state.retirement_unknown || state.faults.len() >= 80 {
            return Err(Error::Fenced);
        }
        let target = match &read {
            Read::Registration(id) | Read::WorkspaceAction(id) => Some(id.0),
            Read::ProviderAction(id) => Some(*id),
            _ => None,
        };
        let pending = state.frontier(target);
        let (reply, result) = oneshot::channel();
        self.jobs
            .try_send(Job {
                operation: Operation::Read { read, pending },
                reply,
                slot,
            })
            .map_err(|_| Error::Capacity)?;
        Ok(result)
    }
    pub(crate) fn faults(&self) -> Vec<Fault> {
        self.shared
            .control
            .admission
            .lock()
            .expect("storage admission poisoned")
            .faults
            .clone()
    }
    /// Closing a requester never cancels this retained drain. Repeated calls observe it.
    pub(crate) async fn close(&self) -> CloseReport {
        {
            let mut state = self
                .shared
                .control
                .admission
                .lock()
                .expect("storage admission poisoned");
            state.closed = true;
            self.shared
                .control
                .changes
                .send_modify(|n| *n = n.wrapping_add(1));
        }
        let mut done = self.done.clone();
        loop {
            if let Some(report) = done.borrow_and_update().clone() {
                return report;
            }
            if done.changed().await.is_err() {
                return CloseReport {
                    faults: vec![Fault {
                        action: None,
                        error: Error::Panic,
                    }],
                    checkpoint: None,
                };
            }
        }
    }
}
async fn run(
    shared: Arc<Shared>,
    mut receive: mpsc::Receiver<Job>,
    done: watch::Sender<Option<CloseReport>>,
) {
    let mut tasks = tokio::task::JoinSet::new();
    let mut changes = shared.control.changes.subscribe();
    loop {
        if shared
            .control
            .admission
            .lock()
            .expect("storage admission poisoned")
            .closed
        {
            receive.close();
        }
        tokio::select! {
            biased;
            Some(joined) = tasks.join_next(), if !tasks.is_empty() => {
                if joined.is_err() { shared.control.fault(None, Error::Panic); }
            }
            job = receive.recv() => {
                let Some(Job { operation, reply, slot }) = job else { break; };
                let shared = shared.clone();
                tasks.spawn(async move {
                    let action = match &operation { Operation::Write { receipt, .. } => Some(*receipt), _ => None };
                    let effect = async {
                        match operation {
                            Operation::Write { payload, .. } => {
                                let write = serde_json::from_str(&payload).map_err(|_| Error::Invalid("Invalid internal command"))?;
                                operations::write(&shared, write).await.map(|revision| Outcome::Written { revision })
                            },
                            Operation::Read { read, pending } => {
                                shared.control.wait_settled(&pending).await?;
                                operations::read(&shared, read).await.map(Outcome::Snapshot)
                            }
                        }
                    };
                    complete_job(shared.control.clone(), action, reply, slot, effect).await;
                });
            }
            _ = changes.changed() => {}
        }
    }
    while let Some(joined) = tasks.join_next().await {
        if joined.is_err() {
            shared.control.fault(None, Error::Panic);
        }
    }
    let report = close_barriers(
        &shared.control,
        async { operations::checkpoint(&shared).await.map(Some) },
        || shared.readers.close(),
        || shared.writer.close(),
    )
    .await;
    done.send_replace(Some(report));
}

fn encode(value: &impl Serialize, max: usize) -> Result<String> {
    struct Bounded {
        bytes: Vec<u8>,
        max: usize,
    }
    impl std::io::Write for Bounded {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.max - self.bytes.len() {
                return Err(std::io::Error::other("metadata limit"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut writer = Bounded {
        bytes: Vec::with_capacity(max),
        max,
    };
    serde_json::to_writer(&mut writer, value).map_err(|_| Error::Oversized)?;
    String::from_utf8(writer.bytes).map_err(|_| Error::Invalid("Invalid metadata encoding"))
}

async fn complete_job(
    control: Arc<Control>,
    action: Option<WriteReceipt>,
    reply: oneshot::Sender<Delivery>,
    slot: OwnedSemaphorePermit,
    effect: impl std::future::Future<Output = Result<Outcome>>,
) {
    use futures_util::FutureExt;
    let outcome = std::panic::AssertUnwindSafe(effect)
        .catch_unwind()
        .await
        .unwrap_or(Err(Error::Panic));
    if let Err(error) = &outcome
        && !error.is_domain_rejection()
    {
        control.fault(action.map(|receipt| receipt.action), error.clone());
    }
    if let Some(action) = action {
        control.settle(action);
    }
    let _ = reply.send(Delivery {
        outcome,
        _slot: slot,
    });
}
impl Error {
    fn is_domain_rejection(&self) -> bool {
        match self {
            Self::CandidateConflict
            | Self::Invalid(_)
            | Self::Conflict(_)
            | Self::Oversized
            | Self::Capacity
            | Self::Fenced => true,
            Self::Write {
                original,
                retirement: None,
                uncertain: false,
            } => original.is_domain_rejection(),
            _ => false,
        }
    }
}
struct Retirement {
    control: Arc<Control>,
    confirmed: bool,
}
impl Retirement {
    fn begin(control: Arc<Control>) -> Self {
        control
            .admission
            .lock()
            .expect("storage admission poisoned")
            .retiring += 1;
        Self {
            control,
            confirmed: false,
        }
    }
}
impl Drop for Retirement {
    fn drop(&mut self) {
        let mut state = self
            .control
            .admission
            .lock()
            .expect("storage admission poisoned");
        state.retiring -= 1;
        if !self.confirmed {
            state.retirement_unknown = true;
            state.fenced = true;
        }
        self.control.changes.send_modify(|n| *n = n.wrapping_add(1));
    }
}
impl Control {
    async fn ready_connection(&self) -> Result<()> {
        let mut changes = self.changes.subscribe();
        loop {
            {
                let state = self.admission.lock().expect("storage admission poisoned");
                if state.retirement_unknown {
                    return Err(Error::Fenced);
                }
                if state.retiring == 0 {
                    return Ok(());
                }
            }
            changes.changed().await.map_err(|_| Error::Closed)?;
        }
    }
}
async fn retire_failed(
    control: Arc<Control>,
    original: Error,
    uncertain: bool,
    retirement: impl std::future::Future<Output = Result<()>>,
) -> Error {
    let mut guard = Retirement::begin(control);
    let retirement = retirement.await.err().map(Box::new);
    guard.confirmed = retirement.is_none();
    drop(guard);
    Error::Write {
        original: Box::new(original),
        retirement,
        uncertain,
    }
}

#[cfg(test)]
#[path = "storage/tests.rs"]
mod tests;

#[cfg(feature = "qualification")]
#[path = "storage/qualification.rs"]
pub(crate) mod qualification;

// Pool barriers have no returned error inventory. Retain checkpoint/operation faults
// separately and always visit both barriers, including after earlier failure.
async fn close_barriers<
    Reader: std::future::Future<Output = ()>,
    Writer: std::future::Future<Output = ()>,
>(
    control: &Control,
    checkpoint: impl std::future::Future<Output = Result<Option<Checkpoint>>>,
    readers: impl FnOnce() -> Reader,
    writer: impl FnOnce() -> Writer,
) -> CloseReport {
    let checkpoint = match checkpoint.await {
        Ok(value) => value,
        Err(error) => {
            control.fault(None, error);
            None
        }
    };
    // SQLx closes admission synchronously when close() constructs its future.
    // Defer those calls themselves, not only polling their returned futures.
    readers().await;
    writer().await;
    let faults = control
        .admission
        .lock()
        .expect("storage admission poisoned")
        .faults
        .clone();
    CloseReport { faults, checkpoint }
}
