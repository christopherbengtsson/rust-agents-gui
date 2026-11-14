//! Nonwaiting physical connection budgets shared by the desktop and server.
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

#[derive(Clone)]
pub struct ConnectionSlots {
    active: Arc<AtomicUsize>,
    limit: usize,
}

impl ConnectionSlots {
    pub fn new(limit: usize) -> Self {
        Self {
            active: Arc::new(AtomicUsize::new(0)),
            limit,
        }
    }

    pub fn try_acquire(&self) -> Option<ConnectionPermit> {
        self.active
            .try_update(Ordering::AcqRel, Ordering::Acquire, |active| {
                (active < self.limit).then(|| active + 1)
            })
            .ok()?;
        Some(ConnectionPermit(self.active.clone()))
    }
}

/// Move into the physical worker, never into its disposable response waiter.
pub struct ConnectionPermit(Arc<AtomicUsize>);
impl Drop for ConnectionPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::AcqRel);
    }
}

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
