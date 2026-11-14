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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct Subscribe {
    pub subscription_id: String,
    pub after: Option<FeedCursor>,
}

struct NativeSubscription {
    id: String,
    control: Arc<Control>,
    worker: tokio::task::JoinHandle<()>,
}
#[derive(Default)]
pub(super) struct FeedOwner {
    current: Mutex<Option<NativeSubscription>>,
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
                    std::task::Poll::Ready(result.map_err(unavailable))
                }
            }
        })
        .await
    }
    pub(super) async fn stop_all(&self) -> Result<(), RequestFailure> {
        let id = self
            .current
            .lock()
            .map_err(unavailable)?
            .as_ref()
            .map(|s| s.id.clone());
        if let Some(id) = id {
            self.stop(&id).await?;
        }
        Ok(())
    }
}
impl SupervisorConnection {
    pub fn start_subscription(
        &self,
        expected_owner: String,
        subscription_id: String,
        after: Option<FeedCursor>,
        deliver: impl Fn(FeedEvent) -> io::Result<()> + Send + Sync + 'static,
    ) -> Result<(), RequestFailure> {
        let client = self.client();
        self.start_subscription_with(
            expected_owner,
            subscription_id,
            after,
            deliver,
            async move |owner, id, after, control, deliver| {
                run_client(&client, &owner, &id, after, &control, &*deliver).await
            },
        )
    }
    fn start_subscription_with<D, R, F>(
        &self,
        expected_owner: String,
        subscription_id: String,
        after: Option<FeedCursor>,
        deliver: D,
        run: R,
    ) -> Result<(), RequestFailure>
    where
        D: Fn(FeedEvent) -> io::Result<()> + Send + Sync + 'static,
        R: FnOnce(String, String, Option<FeedCursor>, Arc<Control>, Arc<D>) -> F + Send + 'static,
        F: Future<Output = io::Result<()>> + Send + 'static,
    {
        if self.tasks.cancel.is_cancelled() {
            return Err(unavailable("Connection cleanup has begun"));
        }
        validate_subscription_id(&subscription_id).map_err(unavailable)?;
        self.require_connection(&expected_owner)?;
        let permit = Self::admit(&self.subscription)?;
        let mut current = self.feed.current.lock().map_err(unavailable)?;
        if self.tasks.cancel.is_cancelled() {
            return Err(unavailable("Connection cleanup has begun"));
        }
        if current.is_some() {
            return Err(RequestFailure::transport(
                TransportCode::Busy,
                Delivery::NotSent,
                "Stop and await the previous subscription before replacement",
            ));
        }
        let control = Arc::new(Control::default());
        let owned_control = control.clone();
        let id = subscription_id.clone();
        let deliver = Arc::new(deliver);
        let worker = tokio::spawn(async move {
            let _permit = permit;
            let result = run(
                expected_owner,
                id.clone(),
                after,
                owned_control.clone(),
                deliver.clone(),
            )
            .await;
            owned_control.stop();
            let message = match result {
                Ok(()) => "Observation delivery stopped".into(),
                Err(error) => {
                    format!("Observation delivery stopped: {error}. Refresh connection explicitly.")
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
            .ok_or_else(|| unavailable("Unknown subscription acknowledgement"))?
            .control
            .acknowledge(&acknowledgement)
            .map_err(unavailable)
    }
    pub async fn stop_subscription(&self, id: &str) -> Result<(), RequestFailure> {
        self.feed.stop(id).await
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
fn request(owner: String, subscription_id: String, after: Option<FeedCursor>) -> Request {
    Request {
        protocol: PROTOCOL_VERSION,
        application_version: env!("CARGO_PKG_VERSION").into(),
        after: None,
        expected_owner: Some(owner),
        repository: None,
        workspace: None,
        subscribe: Some(Subscribe {
            subscription_id,
            after,
        }),
    }
}
async fn run_client(
    client: &SupervisorClient,
    owner: &str,
    id: &str,
    after: Option<FeedCursor>,
    control: &Control,
    deliver: &(impl Fn(FeedEvent) -> io::Result<()> + Sync),
) -> io::Result<()> {
    let socket = open_stream(&client.state, &mut client.effects(), &control.cancel).await?;
    let mut exchange = FramedExchange::new(socket);
    write_json(
        &mut exchange,
        &request(owner.into(), id.into(), after.clone()),
        Instant::now() + IO_TIMEOUT,
        &control.cancel,
    )
    .await?;
    run_subscribed(&mut exchange, owner, id, after, control, deliver).await
}
async fn run_subscribed<S: AsyncRead + AsyncWrite + Unpin + 'static>(
    exchange: &mut FramedExchange<S>,
    owner: &str,
    id: &str,
    after: Option<FeedCursor>,
    control: &Control,
    deliver: &(impl Fn(FeedEvent) -> io::Result<()> + Sync),
) -> io::Result<()> {
    let mut previous_batch = 0;
    let mut previous_cursor = after
        .filter(|cursor| cursor.owner == owner)
        .map(|cursor| cursor.sequence);
    loop {
        let response: Response = read_json(
            exchange,
            Instant::now() + HEARTBEAT + IO_TIMEOUT,
            &control.cancel,
        )
        .await?;
        let Response::Feed { batch } = response else {
            return Err(io::Error::other("Subscription rejected or incompatible"));
        };
        validate_batch(&batch, owner, id, previous_batch, previous_cursor)?;
        let deadline = Instant::now() + ACK_TIMEOUT;
        let acknowledgement = batch.acknowledgement.clone();
        control.offer(acknowledgement.clone())?;
        deliver(FeedEvent::Batch { batch })?;
        control.wait_accepted(deadline).await?;
        write_json(exchange, &acknowledgement, deadline, &control.cancel).await?;
        previous_batch = acknowledgement.batch_id;
        previous_cursor = Some(acknowledgement.cursor.sequence);
    }
}
fn validate_batch(
    batch: &FeedBatch,
    owner: &str,
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
        || ack.cursor.owner != owner
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

pub(super) async fn serve<S: AsyncRead + AsyncWrite + Unpin + 'static>(
    owner: &str,
    journal: &crate::observations::Journal,
    subscribe: Subscribe,
    exchange: &mut FramedExchange<S>,
    cancel: &CancellationToken,
) -> io::Result<()> {
    validate_subscription_id(&subscribe.subscription_id)?;
    let mut after = subscribe
        .after
        .filter(|cursor| cursor.owner == owner)
        .map(|cursor| cursor.sequence);
    let mut batch_id: u64 = 0;
    loop {
        let sample = journal.wait_async(after, HEARTBEAT, cancel).await?;
        batch_id = batch_id
            .checked_add(1)
            .filter(|id| *id <= MAX_SEQUENCE)
            .ok_or_else(|| io::Error::other("Subscription batch sequence exhausted"))?;
        let acknowledgement = FeedAck {
            subscription_id: subscribe.subscription_id.clone(),
            batch_id,
            cursor: FeedCursor {
                owner: owner.into(),
                sequence: sample.watermark(),
            },
        };
        let batch = FeedBatch {
            acknowledgement: acknowledgement.clone(),
            sample,
        };
        if serde_json::to_vec(&batch)?.len() > DELIVERY_BYTES {
            return Err(io::Error::other("Subscription batch exceeds byte budget"));
        }
        write_json(
            exchange,
            &Response::Feed { batch },
            Instant::now() + IO_TIMEOUT,
            cancel,
        )
        .await?;
        let ack: FeedAck = read_json(exchange, Instant::now() + ACK_TIMEOUT, cancel).await?;
        if ack != acknowledgement {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Mismatched subscription acknowledgement",
            ));
        }
        after = Some(ack.cursor.sequence);
    }
}
#[cfg(test)]
mod tests;

#[cfg(feature = "codegen")]
pub(crate) fn export_contracts(exports: &mut crate::codegen::Exports) {
    exports.add::<FeedAck>();
    exports.add::<FeedBatch>();
    exports.add::<FeedEvent>();
}
