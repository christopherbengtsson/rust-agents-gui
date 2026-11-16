//! Physical work remains owned when its result receiver is dropped.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{sync::oneshot, task::JoinSet};

#[derive(Default)]
pub(super) struct Tasks {
    set: Mutex<JoinSet<()>>,
    cleanup: tokio::sync::Mutex<()>,
    closed: AtomicBool,
    failure: Arc<Mutex<Vec<String>>>,
    pub(super) cancel: CancellationToken,
}
impl Tasks {
    fn reap(&self, set: &mut JoinSet<()>) {
        while let Some(result) = set.try_join_next() {
            if let Err(error) = result {
                self.failure
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .push(error.to_string());
                self.closed.store(true, Ordering::Release);
            }
        }
    }
    pub(super) fn spawn<T: Send + 'static>(
        &self,
        work: impl Future<Output = T> + Send + 'static,
    ) -> io::Result<oneshot::Receiver<T>> {
        let mut set = self.set.lock().unwrap_or_else(|e| e.into_inner());
        if self.closed.load(Ordering::Acquire)
            || !self
                .failure
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_empty()
        {
            return Err(io::Error::other("Connection cleanup has begun"));
        }
        self.reap(&mut set);
        let (send, receive) = oneshot::channel();
        set.spawn(async move {
            let _ = send.send(work.await);
        });
        Ok(receive)
    }
    pub(super) async fn blocking<T: Send + 'static>(
        &self,
        slots: &ConnectionSlots,
        work: impl FnOnce() -> io::Result<T> + Send + 'static,
    ) -> io::Result<T> {
        let permit = slots.try_acquire().ok_or_else(|| {
            io::Error::new(io::ErrorKind::WouldBlock, "Blocking dispatch is busy")
        })?;
        let receive = {
            let mut set = self.set.lock().unwrap_or_else(|e| e.into_inner());
            if self.closed.load(Ordering::Acquire)
                || !self
                    .failure
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .is_empty()
            {
                return Err(io::Error::other("Connection cleanup has begun"));
            }
            self.reap(&mut set);
            let (send, receive) = oneshot::channel();
            let failure = self.failure.clone();
            set.spawn_blocking(move || {
                let _permit = permit;
                if let Err(Err(error)) = send.send(work()) {
                    failure
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(format!("Undelivered blocking failure: {error}"));
                }
            });
            receive
        };
        receive
            .await
            .map_err(|error| io::Error::other(error.to_string()))?
    }
    pub(super) async fn close_and_wait(&self) -> io::Result<()> {
        self.cancel.cancel();
        let _cleanup = self.cleanup.lock().await;
        self.closed.store(true, Ordering::Release);
        std::future::poll_fn(|cx| {
            let mut set = self.set.lock().unwrap_or_else(|e| e.into_inner());
            loop {
                match set.poll_join_next(cx) {
                    std::task::Poll::Ready(Some(Err(error))) => self
                        .failure
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .push(error.to_string()),
                    std::task::Poll::Ready(Some(Ok(()))) => {}
                    std::task::Poll::Ready(None) => return std::task::Poll::Ready(()),
                    std::task::Poll::Pending => return std::task::Poll::Pending,
                }
            }
        })
        .await;
        let failures = self.failure.lock().unwrap_or_else(|e| e.into_inner());
        if failures.is_empty() {
            Ok(())
        } else {
            Err(io::Error::other(failures.join("; ")))
        }
    }
}
