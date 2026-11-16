//! Provider policy awaits one shared storage owner; storage never decides to replay effects.
use super::*;
use crate::storage::{Error, ProviderAction, Read, Storage, Write};
use futures_util::future::BoxFuture;

type Result<T> = std::result::Result<T, Error>;
pub(super) trait Persistence: Send + Sync {
    /// False means this exact action already exists. It grants no replay authority.
    fn begin<'a>(&'a self, action: &'a ProviderAction) -> BoxFuture<'a, Result<bool>>;
    fn finish<'a>(
        &'a self,
        action: &'a ProviderAction,
        remember: bool,
    ) -> BoxFuture<'a, Result<()>>;
}
async fn one<T: serde::de::DeserializeOwned>(storage: &Storage, read: Read) -> Result<Option<T>> {
    Ok(storage
        .read(read)?
        .await
        .map_err(|_| Error::Panic)?
        .into_records::<T>()?
        .into_items()
        .into_iter()
        .next())
}
impl Persistence for Storage {
    fn begin<'a>(&'a self, action: &'a ProviderAction) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            if let Some((saved, _)) =
                one::<(ProviderAction, i64)>(self, Read::ProviderAction(action.id)).await?
            {
                if saved != *action {
                    return Err(Error::Invalid("Provider action identity/payload mismatch"));
                }
                return Ok(false);
            }
            self.write(Write::ProviderIntent(action.clone()))?
                .await
                .map_err(|_| Error::Panic)?
                .revision()?;
            Ok(true)
        })
    }
    fn finish<'a>(
        &'a self,
        action: &'a ProviderAction,
        remember: bool,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.write(Write::ProviderFinish {
                action: action.clone(),
                remember,
            })?
            .await
            .map_err(|_| Error::Panic)?
            .revision()?;
            Ok(())
        })
    }
}
pub(super) async fn restore(
    storage: &Storage,
    scope: &str,
) -> Result<(Option<(uuid::Uuid, String)>, Option<ProviderAction>)> {
    let selected = one(storage, Read::ProviderSelection(scope.to_owned())).await?;
    let pending = one::<(ProviderAction, i64)>(storage, Read::ProviderPending(scope.to_owned()))
        .await?
        .map(|(action, _)| action);
    Ok((selected, pending))
}
impl ProviderConnections {
    pub(super) fn action(&self, target: uuid::Uuid, id: uuid::Uuid) -> Result<ProviderAction> {
        self.records
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(&target)
            .and_then(|entry| {
                entry
                    .selection_action
                    .iter()
                    .chain(entry.login_action.iter())
                    .find(|action| action.id == id)
            })
            .cloned()
            .ok_or(Error::Invalid("Provider action target changed"))
    }
    pub(super) async fn begin_action(
        &self,
        action: &ProviderAction,
    ) -> std::result::Result<(), SetupFailure> {
        let result = match &self.persistence {
            Some(store) => store.begin(action).await,
            None => Ok(true),
        };
        match result {
            Ok(true) => Ok(()),
            Ok(false) => Err(SetupFailure::at(
                SetupProblem::CleanupUncertain,
                "This action is already recorded. Retrieve status; it cannot be replayed.",
            )),
            Err(error) => Err(self.persistence_failure(action, error)),
        }
    }
    pub(super) async fn finish_action(
        &self,
        action: &ProviderAction,
        remember: bool,
    ) -> std::result::Result<(), SetupFailure> {
        if let Some(store) = &self.persistence {
            store
                .finish(action, remember)
                .await
                .map_err(|error| self.persistence_failure(action, error))?;
        }
        Ok(())
    }
    fn persistence_failure(&self, action: &ProviderAction, error: Error) -> SetupFailure {
        // The retained domain fault survives observation replacement and successful close.
        self.persistence_faults
            .lock()
            .unwrap()
            .push((action.clone(), error.clone()));
        SetupFailure::at(
            SetupProblem::CleanupUncertain,
            format!("Provider action {} persistence: {error}", action.id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct HeldIntent {
        entered: tokio::sync::Notify,
        release: Mutex<Option<tokio::sync::oneshot::Receiver<()>>>,
        finishes: AtomicUsize,
    }
    impl Persistence for HeldIntent {
        fn begin<'a>(&'a self, _: &'a ProviderAction) -> BoxFuture<'a, Result<bool>> {
            Box::pin(async {
                let receive = self.release.lock().unwrap().take().unwrap();
                self.entered.notify_one();
                receive.await.unwrap();
                Ok(true)
            })
        }
        fn finish<'a>(
            &'a self,
            _: &'a ProviderAction,
            remember: bool,
        ) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                assert!(!remember);
                self.finishes.fetch_add(1, Ordering::SeqCst);
                Ok(())
            })
        }
    }
    #[test]
    fn quit_during_storage_acquisition_keeps_status_usable_and_settles_without_effects() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                let (release, receive) = tokio::sync::oneshot::channel();
                let store = Arc::new(HeldIntent {
                    entered: tokio::sync::Notify::new(),
                    release: Mutex::new(Some(receive)),
                    finishes: AtomicUsize::new(0),
                });
                let mut raw = ProviderConnections::test_owner();
                raw.persistence = Some(store.clone());
                let owner = Arc::new(raw);
                let target = owner.test_target("codex");
                let other = owner.test_target("other");
                let action = uuid::Uuid::new_v4();
                owner
                    .check_with(
                        target,
                        true,
                        Some((action, 0, "/chosen/codex".into())),
                        || async { panic!("Quit must fence the delayed effect") },
                    )
                    .unwrap();
                store.entered.notified().await;
                // Losing the original reply and storage pressure cannot consume control capacity.
                assert_eq!(owner.read(target).unwrap().phase, SetupPhase::Checking);
                assert_eq!(owner.read(other).unwrap().phase, SetupPhase::Unknown);
                assert!(owner.admit_login(target, uuid::Uuid::new_v4(), 1).is_err());
                owner.fence();
                assert_eq!(owner.read(target).unwrap().selection.unwrap().id, action);
                release.send(()).unwrap();
                owner.finish().await.unwrap();
                assert_eq!(store.finishes.load(Ordering::SeqCst), 1);
            });
    }
}
