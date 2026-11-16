//! Nonwaiting native request admission; permits follow physical work and retained results.
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
    pub fn try_acquire(&self) -> Option<OwnedSemaphorePermit> {
        self.semaphore.clone().try_acquire_owned().ok()
    }
}
