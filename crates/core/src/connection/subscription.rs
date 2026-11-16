//! One acknowledged feed. No discovery, domain actions or automatic action replay.
use super::*;
use crate::observations::{DELIVERY_BYTES, FeedCursor, MAX_SEQUENCE, Sample};

const HEARTBEAT: Duration = Duration::from_secs(5);
const ACK_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct FeedAck {
    pub subscription_id: String,
    pub batch_id: u64,
    pub cursor: FeedCursor,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
pub struct FeedBatch {
    #[serde(flatten)]
    pub acknowledgement: FeedAck,
    pub sample: Sample,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "codegen", derive(ts_rs::TS, schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FeedEvent {
    Batch {
        batch: FeedBatch,
    },
    Stopped {
        subscription_id: String,
        message: String,
    },
}
struct NativeSubscription {
    id: String,
    control: Arc<Control>,
    worker: tokio::task::JoinHandle<()>,
}
#[derive(Default)]
pub(crate) struct FeedOwner {
    current: Mutex<Option<NativeSubscription>>,
    failure: Mutex<Option<String>>,
    // A new WebView document has no old subscription ID. Admit at most one
    // replacement while its predecessor is physically draining.
    replacement: tokio::sync::Mutex<()>,
    // Serializes join polling; the handle stays in current if a stop waiter is dropped.
    cleanup: tokio::sync::Mutex<()>,
}
#[derive(Default)]
struct Pending {
    acknowledgement: Option<FeedAck>,
    accepted: bool,
}
#[derive(Default)]
struct Control {
    pending: Mutex<Pending>,
    wake: tokio::sync::Notify,
    cancel: CancellationToken,
}
impl Control {
    fn stop(&self) {
        self.cancel.cancel();
    }
    fn offer(&self, acknowledgement: FeedAck) -> io::Result<()> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if self.cancel.is_cancelled() || pending.acknowledgement.is_some() {
            return Err(io::Error::other(
                "Subscription stopped or batch already pending",
            ));
        }
        pending.acknowledgement = Some(acknowledgement);
        pending.accepted = false;
        Ok(())
    }
    fn acknowledge(&self, acknowledgement: &FeedAck) -> io::Result<()> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if self.cancel.is_cancelled()
            || pending.accepted
            || pending.acknowledgement.as_ref() != Some(acknowledgement)
        {
            return Err(io::Error::other(
                "Subscription acknowledgement does not match the pending batch",
            ));
        }
        pending.accepted = true;
        drop(pending);
        self.wake.notify_one();
        Ok(())
    }
    async fn wait_accepted(&self, deadline: Instant) -> io::Result<()> {
        loop {
            let wake = self.wake.notified();
            tokio::pin!(wake);
            wake.as_mut().enable();
            if self.cancel.is_cancelled() {
                return Err(io::ErrorKind::Interrupted.into());
            }
            if Instant::now() >= deadline {
                return Err(io::ErrorKind::TimedOut.into());
            }
            {
                let mut pending = self
                    .pending
                    .lock()
                    .map_err(|e| io::Error::other(e.to_string()))?;
                if pending.accepted {
                    pending.acknowledgement = None;
                    pending.accepted = false;
                    return Ok(());
                }
            }
            match self
                .cancel
                .run_until_cancelled(tokio::time::timeout_at(deadline, wake))
                .await
            {
                None => return Err(io::ErrorKind::Interrupted.into()),
                Some(Err(_)) => return Err(io::ErrorKind::TimedOut.into()),
                Some(Ok(())) => {}
            }
        }
    }
}
impl FeedOwner {
    async fn stop(&self, id: &str) -> Result<(), RequestFailure> {
        let control = {
            let current = self.current.lock().map_err(unavailable)?;
            let Some(subscription) = current.as_ref().filter(|s| s.id == id) else {
                return Ok(());
            };
            subscription.control.stop();
            subscription.control.clone()
        };
        let _cleanup = self.cleanup.lock().await;
        std::future::poll_fn(|cx| {
            let mut current = match self.current.lock() {
                Ok(current) => current,
                Err(error) => return std::task::Poll::Ready(Err(unavailable(error))),
            };
            let Some(subscription) = current
                .as_mut()
                .filter(|s| Arc::ptr_eq(&s.control, &control))
            else {
                return std::task::Poll::Ready(Ok(()));
            };
            match std::pin::Pin::new(&mut subscription.worker).poll(cx) {
                std::task::Poll::Pending => std::task::Poll::Pending,
                std::task::Poll::Ready(result) => {
                    current.take();
                    if let Err(error) = &result {
                        *self.failure.lock().unwrap_or_else(|e| e.into_inner()) =
                            Some(format!("Native subscription completion failed: {error}"));
                    }
                    std::task::Poll::Ready(result.map_err(unavailable))
                }
            }
        })
        .await
    }
    pub(crate) async fn stop_all(&self) -> Result<(), RequestFailure> {
        let id = self
            .current
            .lock()
            .map_err(unavailable)?
            .as_ref()
            .map(|s| s.id.clone());
        if let Some(id) = id {
            self.stop(&id).await?;
        }
        if let Some(error) = self
            .failure
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
        {
            return Err(unavailable(error));
        }
        Ok(())
    }
}
impl crate::native::NativeHost {
    pub async fn replace_subscription(
        &self,
        expected: String,
        subscription_id: String,
        after: Option<FeedCursor>,
        deliver: impl Fn(FeedEvent) -> io::Result<()> + Send + Sync + 'static,
    ) -> Result<(), RequestFailure> {
        validate_subscription_id(&subscription_id).map_err(unavailable)?;
        self.core(&expected)?;
        let _replacement = self.feed.replacement.try_lock().map_err(|_| {
            RequestFailure::transport(
                TransportCode::Busy,
                Delivery::NotSent,
                "Previous native subscription is draining",
            )
        })?;
        self.feed.stop_all().await?;
        let mut current = self.feed.current.lock().map_err(unavailable)?;
        let core = self.core(&expected)?;
        let control = Arc::new(Control::default());
        let owned = control.clone();
        let id = subscription_id.clone();
        let worker = tokio::spawn(async move {
            let result = run_direct(&core.journal, &expected, &id, after, &owned, &deliver).await;
            owned.stop();
            let message = match result {
                Ok(()) => "Observation delivery stopped".into(),
                Err(error) => {
                    format!("Observation delivery stopped: {error}. Refresh core status.")
                }
            };
            let _ = deliver(FeedEvent::Stopped {
                subscription_id: id,
                message,
            });
        });
        *current = Some(NativeSubscription {
            id: subscription_id,
            control,
            worker,
        });
        Ok(())
    }
    pub fn acknowledge_subscription(&self, acknowledgement: FeedAck) -> Result<(), RequestFailure> {
        let current = self.feed.current.lock().map_err(unavailable)?;
        current
            .as_ref()
            .filter(|s| s.id == acknowledgement.subscription_id)
            .ok_or_else(|| unavailable("Unknown native subscription"))?
            .control
            .acknowledge(&acknowledgement)
            .map_err(unavailable)
    }
    pub async fn stop_subscription(&self, id: &str) -> Result<(), RequestFailure> {
        self.feed.stop(id).await
    }
    pub(crate) async fn stop_delivery(&self) -> Result<(), RequestFailure> {
        let _replacement = self.feed.replacement.lock().await;
        self.feed.stop_all().await
    }
}
async fn run_direct(
    journal: &crate::observations::Journal,
    incarnation: &str,
    id: &str,
    after: Option<FeedCursor>,
    control: &Control,
    deliver: &(impl Fn(FeedEvent) -> io::Result<()> + Sync),
) -> io::Result<()> {
    let mut after = after
        .filter(|cursor| cursor.incarnation == incarnation)
        .map(|cursor| cursor.sequence);
    let mut batch_id = 0u64;
    loop {
        let sample = journal
            .wait_async(after, HEARTBEAT, &control.cancel)
            .await?;
        batch_id = batch_id
            .checked_add(1)
            .filter(|id| *id <= MAX_SEQUENCE)
            .ok_or_else(|| io::Error::other("Batch sequence exhausted"))?;
        let acknowledgement = FeedAck {
            subscription_id: id.into(),
            batch_id,
            cursor: FeedCursor {
                incarnation: incarnation.into(),
                sequence: sample.watermark(),
            },
        };
        let batch = FeedBatch {
            acknowledgement: acknowledgement.clone(),
            sample,
        };
        validate_batch(&batch, incarnation, id, batch_id - 1, after)?;
        control.offer(acknowledgement.clone())?;
        deliver(FeedEvent::Batch { batch })?;
        control.wait_accepted(Instant::now() + ACK_TIMEOUT).await?;
        after = Some(acknowledgement.cursor.sequence);
    }
}
fn validate_subscription_id(id: &str) -> io::Result<()> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid subscription identity",
        ))
    } else {
        Ok(())
    }
}
fn validate_batch(
    batch: &FeedBatch,
    incarnation: &str,
    id: &str,
    previous: u64,
    cursor: Option<u64>,
) -> io::Result<()> {
    let ack = &batch.acknowledgement;
    let sequence = batch.sample.watermark();
    let valid_sample = match &batch.sample {
        Sample::Hints { targets, .. } => {
            !targets.is_empty()
                && targets.len() <= crate::observations::DELIVERY_TAGS
                && cursor.is_none_or(|cursor| sequence > cursor)
        }
        Sample::Heartbeat { .. } => cursor == Some(sequence),
        Sample::ResyncRequired { .. } => true,
    };
    if ack.subscription_id != id
        || ack.cursor.incarnation != incarnation
        || Some(ack.batch_id) != previous.checked_add(1)
        || ack.batch_id > MAX_SEQUENCE
        || sequence == 0
        || sequence > MAX_SEQUENCE
        || ack.cursor.sequence != sequence
        || !valid_sample
        || serde_json::to_vec(batch)?.len() > DELIVERY_BYTES
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid subscription batch identity, sequence or bounds",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<FeedAck>();
    exports.add::<FeedBatch>();
    exports.add::<FeedEvent>();
}
