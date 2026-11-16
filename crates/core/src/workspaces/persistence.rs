//! Typed storage outcomes; workspace policy owns reconciliation, never SQL.
use super::*;
use crate::storage::{Delivery, Read, Storage, Write};
use futures_util::future::BoxFuture;

pub(super) trait Metadata: Send + Sync {
    fn read(
        &self,
        read: Read,
    ) -> BoxFuture<'_, std::result::Result<Delivery, crate::storage::Error>>;
    fn write(
        &self,
        write: Write,
    ) -> BoxFuture<'_, std::result::Result<Delivery, crate::storage::Error>>;
}
impl Metadata for Storage {
    fn read(
        &self,
        read: Read,
    ) -> BoxFuture<'_, std::result::Result<Delivery, crate::storage::Error>> {
        Box::pin(async move {
            Storage::read(self, read)?
                .await
                .map_err(|_| crate::storage::Error::Panic)
        })
    }
    fn write(
        &self,
        write: Write,
    ) -> BoxFuture<'_, std::result::Result<Delivery, crate::storage::Error>> {
        Box::pin(async move {
            Storage::write(self, write)?
                .await
                .map_err(|_| crate::storage::Error::Panic)
        })
    }
}
pub(super) async fn page<T: serde::de::DeserializeOwned>(
    store: &dyn Metadata,
    read: Read,
) -> Result<(Vec<T>, Option<String>)> {
    let records = store
        .read(read)
        .await
        .map_err(RepositoryError::storage)?
        .into_records::<T>()
        .map_err(RepositoryError::storage)?;
    let next = records.next.clone();
    Ok((records.into_items(), next))
}
pub(super) async fn one<T: serde::de::DeserializeOwned>(
    store: &dyn Metadata,
    read: Read,
) -> Result<Option<T>> {
    Ok(page(store, read).await?.0.into_iter().next())
}
pub(super) async fn save(store: &dyn Metadata, write: Write) -> Result<()> {
    store
        .write(write)
        .await
        .map_err(RepositoryError::storage)?
        .revision()
        .map(|_| ())
        .map_err(RepositoryError::storage)
}
