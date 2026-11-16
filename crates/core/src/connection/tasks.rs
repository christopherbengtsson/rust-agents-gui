//! Physical work remains owned when its result receiver is dropped.
use super::*;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::{sync::oneshot, task::JoinSet};

#[derive(Default)]
pub(crate) struct Tasks {
    set: Mutex<JoinSet<()>>,
    cleanup: tokio::sync::Mutex<()>,
    closed: AtomicBool,
    failure: Arc<Mutex<Vec<String>>>,
    pub(crate) cancel: CancellationToken,
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
    pub(crate) fn diagnostic(&self) -> Option<String> {
        self.reap(&mut self.set.lock().unwrap_or_else(|e| e.into_inner()));
        let failures = self.failure.lock().unwrap_or_else(|e| e.into_inner());
        (!failures.is_empty()).then(|| failures.join("; "))
    }
    pub(crate) fn spawn<T: Send + 'static>(
        &self,
        work: impl Future<Output = T> + Send + 'static,
    ) -> io::Result<oneshot::Receiver<T>> {
        let mut set = self.set.lock().unwrap_or_else(|e| e.into_inner());
        self.reap(&mut set);
        if self.closed.load(Ordering::Acquire)
            || !self
                .failure
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .is_empty()
        {
            return Err(io::Error::other("Native request cleanup has begun"));
        }
        let (send, receive) = oneshot::channel();
        set.spawn(async move {
            let _ = send.send(work.await);
        });
        Ok(receive)
    }
    pub(crate) async fn close_and_wait(&self) -> io::Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn discovered_panic_fences_next_dispatch_and_stays_in_cleanup_report() {
        let tasks = Tasks::default();
        let receiver = tasks
            .spawn::<()>(async { panic!("controlled previous panic") })
            .unwrap();
        assert!(receiver.await.is_err());
        assert!(tasks.spawn(async {}).is_err());
        assert!(tasks.diagnostic().is_some());
        assert!(tasks.close_and_wait().await.is_err());
    }
    #[tokio::test]
    async fn abandoned_cleanup_keeps_actual_work_until_it_finishes() {
        let tasks = Tasks::default();
        let (release, finish) = oneshot::channel();
        drop(tasks.spawn(async { finish.await.unwrap() }).unwrap());
        {
            let cleanup = tasks.close_and_wait();
            tokio::pin!(cleanup);
            assert!(
                std::future::poll_fn(|cx| std::task::Poll::Ready(
                    cleanup.as_mut().poll(cx).is_pending()
                ))
                .await
            );
        }
        assert!(tasks.spawn(async {}).is_err());
        release.send(()).unwrap();
        tasks.close_and_wait().await.unwrap();
    }
}
