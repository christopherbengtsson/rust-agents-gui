//! Nonwaiting physical connection budgets shared by the desktop and server.
use super::{RequestFailure, tasks::Tasks, unavailable};
use std::{future::Future, pin::Pin, sync::Arc};
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
#[derive(Clone)]
pub struct ConnectionSlots {
    semaphore: Arc<Semaphore>,
}
impl ConnectionSlots {
    pub fn new(limit: usize) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(limit)),
        }
    }
    pub fn try_acquire(&self) -> Option<ConnectionPermit> {
        self.semaphore.clone().try_acquire_owned().ok()
    }
}
pub type ConnectionPermit = OwnedSemaphorePermit;
/// An unstarted request owns admission; run transfers it to the connection's task owner.
pub struct Admitted<T> {
    permit: ConnectionPermit,
    tasks: Arc<Tasks>,
    work: Pin<Box<dyn Future<Output = Result<T, RequestFailure>> + Send>>,
}
impl<T: Send + 'static> Admitted<T> {
    pub(super) fn new(
        permit: ConnectionPermit,
        tasks: Arc<Tasks>,
        work: impl Future<Output = Result<T, RequestFailure>> + Send + 'static,
    ) -> Self {
        Self {
            permit,
            tasks,
            work: Box::pin(work),
        }
    }
    /// Starts synchronously. Dropping the returned waiter cannot cancel physical work.
    pub fn run(self) -> impl Future<Output = Result<T, RequestFailure>> + Send {
        let receive = self.tasks.spawn(async move {
            let _permit = self.permit;
            self.work.await
        });
        async move { receive.map_err(unavailable)?.await.map_err(unavailable)? }
    }
}
