//! Nonwaiting physical connection budgets shared by the desktop and server.
use std::sync::Arc;
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

/// Move into the physical worker, never into its disposable response waiter.
pub type ConnectionPermit = OwnedSemaphorePermit;

/// Only admitted work can cross the desktop's blocking-task boundary.
pub struct Admitted<F> {
    pub(super) permit: ConnectionPermit,
    pub(super) work: F,
}
impl<F> Admitted<F> {
    pub fn run<T>(self) -> T
    where
        F: FnOnce() -> T,
    {
        let _permit = self.permit;
        (self.work)()
    }
}
