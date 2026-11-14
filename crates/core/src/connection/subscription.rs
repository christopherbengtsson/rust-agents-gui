//! One acknowledged feed. No discovery, domain actions or automatic action replay.
use super::*;
use crate::observations::{DELIVERY_BYTES, FeedCursor, MAX_SEQUENCE, Sample};
use std::{net::Shutdown, sync::Condvar, thread::JoinHandle};

const HEARTBEAT: Duration = Duration::from_secs(5);
const ACK_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedAck {
    pub subscription_id: String,
    pub batch_id: u64,
    pub cursor: FeedCursor,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedBatch {
    #[serde(flatten)]
    pub acknowledgement: FeedAck,
    pub sample: Sample,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub(super) struct NativeSubscription {
    id: String,
    control: Arc<Control>,
    worker: JoinHandle<()>,
}
trait FeedSocket: Send {
    fn shutdown(&self) -> io::Result<()>;
}
impl FeedSocket for UnixStream {
    fn shutdown(&self) -> io::Result<()> {
        UnixStream::shutdown(self, Shutdown::Both)
    }
}
#[derive(Default)]
struct Pending {
    stopped: bool,
    socket: Option<Box<dyn FeedSocket>>,
    acknowledgement: Option<FeedAck>,
    accepted: bool,
}
#[derive(Default)]
struct Control {
    pending: Mutex<Pending>,
    wake: Condvar,
}
impl Control {
    fn stop(&self) {
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        pending.stopped = true;
        let socket = pending.socket.take();
        drop(pending);
        if let Some(socket) = socket {
            let _ = socket.shutdown();
        }
        self.wake.notify_all();
    }
    fn attach(&self, socket: Box<dyn FeedSocket>) -> io::Result<()> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if pending.stopped {
            drop(pending);
            socket.shutdown()?;
            return Err(io::Error::other("Subscription stopped before attachment"));
        }
        pending.socket = Some(socket);
        Ok(())
    }
    fn offer(&self, acknowledgement: FeedAck) -> io::Result<()> {
        let mut pending = self
            .pending
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        if pending.stopped || pending.acknowledgement.is_some() {
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
        if pending.stopped
            || pending.accepted
            || pending.acknowledgement.as_ref() != Some(acknowledgement)
        {
            return Err(io::Error::other(
                "Subscription acknowledgement does not match the pending batch",
            ));
        }
        pending.accepted = true;
        drop(pending);
        self.wake.notify_all();
        Ok(())
    }
    fn wait_accepted(&self, timeout: Duration) -> io::Result<()> {
        let pending = self
            .pending
            .lock()
            .map_err(|e| io::Error::other(e.to_string()))?;
        let (mut pending, _) = self
            .wake
            .wait_timeout_while(pending, timeout, |pending| {
                !pending.stopped && !pending.accepted
            })
            .map_err(|e| io::Error::other(e.to_string()))?;
        if pending.stopped || !pending.accepted {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Subscription stopped or frontend acknowledgement deadline exceeded",
            ));
        }
        pending.acknowledgement = None;
        pending.accepted = false;
        Ok(())
    }
}
impl SupervisorConnection {
    /// Admission is nonwaiting and occurs before creating the physical worker.
    pub fn start_subscription(
        &self,
        expected_owner: String,
        subscription_id: String,
        after: Option<FeedCursor>,
        deliver: impl Fn(FeedEvent) -> io::Result<()> + Send + 'static,
    ) -> Result<(), RequestFailure> {
        self.start_subscription_with(expected_owner, subscription_id, after, deliver, run_client)
    }
    fn start_subscription_with(
        &self,
        expected_owner: String,
        subscription_id: String,
        after: Option<FeedCursor>,
        deliver: impl Fn(FeedEvent) -> io::Result<()> + Send + 'static,
        run: impl FnOnce(
            &Path,
            &str,
            &str,
            Option<FeedCursor>,
            &Control,
            &dyn Fn(FeedEvent) -> io::Result<()>,
        ) -> io::Result<()>
        + Send
        + 'static,
    ) -> Result<(), RequestFailure> {
        validate_subscription_id(&subscription_id).map_err(unavailable)?;
        self.require_connection(&expected_owner)?;
        let permit = Self::admit(&self.subscription)?;
        let mut current = self.feed.lock().map_err(unavailable)?;
        if current
            .as_ref()
            .is_some_and(|subscription| !subscription.worker.is_finished())
        {
            return Err(RequestFailure::transport(
                TransportCode::Busy,
                Delivery::NotSent,
                "Subscription cleanup is still in progress",
            ));
        }
        if let Some(old) = current.take() {
            let _ = old.worker.join();
        }
        let control = Arc::new(Control::default());
        let owned_control = control.clone();
        let state = self.state.clone();
        let id = subscription_id.clone();
        let worker = std::thread::Builder::new()
            .name("observation-feed".into())
            .spawn(move || {
                let _permit = permit;
                let result = run(
                    &state,
                    &expected_owner,
                    &id,
                    after,
                    &owned_control,
                    &deliver,
                );
                owned_control.stop();
                let message = match result {
                    Ok(()) => "Observation delivery stopped".into(),
                    Err(error) => format!(
                        "Observation delivery stopped: {error}. Refresh connection explicitly."
                    ),
                };
                let _ = deliver(FeedEvent::Stopped {
                    subscription_id: id,
                    message,
                });
            })
            .map_err(unavailable)?;
        *current = Some(NativeSubscription {
            id: subscription_id,
            control,
            worker,
        });
        Ok(())
    }
    pub fn acknowledge_subscription(&self, acknowledgement: FeedAck) -> Result<(), RequestFailure> {
        let current = self.feed.lock().map_err(unavailable)?;
        let subscription = current
            .as_ref()
            .filter(|subscription| subscription.id == acknowledgement.subscription_id)
            .ok_or_else(|| unavailable("Unknown subscription acknowledgement"))?;
        subscription
            .control
            .acknowledge(&acknowledgement)
            .map_err(unavailable)
    }
    /// Invoke off the rendering thread. Taking a resource out never frees its permit.
    pub fn stop_subscription(&self, id: &str) -> Result<(), RequestFailure> {
        let subscription = {
            let mut current = self.feed.lock().map_err(unavailable)?;
            if current
                .as_ref()
                .is_none_or(|subscription| subscription.id != id)
            {
                return Ok(());
            }
            let subscription = current.take().expect("matching subscription exists");
            subscription.control.stop();
            subscription
        };
        subscription
            .worker
            .join()
            .map_err(|_| unavailable("Subscription worker panicked"))
    }
    pub fn stop_observations(&self) -> Result<(), RequestFailure> {
        let id = self
            .feed
            .lock()
            .map_err(unavailable)?
            .as_ref()
            .map(|subscription| subscription.id.clone());
        if let Some(id) = id {
            self.stop_subscription(&id)?;
        }
        Ok(())
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
fn run_client(
    state: &Path,
    owner: &str,
    id: &str,
    after: Option<FeedCursor>,
    control: &Control,
    deliver: &dyn Fn(FeedEvent) -> io::Result<()>,
) -> io::Result<()> {
    check_private(state, true)?;
    check_private(&socket_path(state), false)?;
    let socket = UnixStream::connect(socket_path(state))?;
    configure(&socket)?;
    control.attach(Box::new(socket.try_clone()?))?;
    let mut reader = BufReader::new(socket);
    write_timed_frame(
        reader.get_mut(),
        &request(owner.into(), id.into(), after.clone()),
    )?;
    run_subscribed(&mut reader, owner, id, after, control, deliver)
}
fn run_subscribed<S: FrameStream>(
    reader: &mut BufReader<S>,
    owner: &str,
    id: &str,
    after: Option<FeedCursor>,
    control: &Control,
    deliver: &dyn Fn(FeedEvent) -> io::Result<()>,
) -> io::Result<()> {
    let mut previous_batch = 0;
    let mut previous_cursor = after
        .filter(|cursor| cursor.owner == owner)
        .map(|cursor| cursor.sequence);
    loop {
        let response: Response = read_buffered_frame(reader, HEARTBEAT + IO_TIMEOUT)?;
        let Response::Feed { batch } = response else {
            return Err(io::Error::other("Subscription rejected or incompatible"));
        };
        validate_batch(&batch, owner, id, previous_batch, previous_cursor)?;
        let deadline = Instant::now() + ACK_TIMEOUT;
        control.offer(batch.acknowledgement.clone())?;
        let acknowledgement = batch.acknowledgement.clone();
        deliver(FeedEvent::Batch { batch })?;
        control.wait_accepted(deadline.saturating_duration_since(Instant::now()))?;
        write_timed_frame(reader.get_mut(), &acknowledgement)?;
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
        || ack.batch_id != previous + 1
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

pub(super) fn serve(
    core: &Core,
    subscribe: Subscribe,
    reader: &mut BufReader<&mut impl FrameStream>,
) -> io::Result<()> {
    validate_subscription_id(&subscribe.subscription_id)?;
    let after = subscribe
        .after
        .filter(|cursor| cursor.owner == core.owner)
        .map(|cursor| cursor.sequence);
    serve_with(
        &core.owner,
        &subscribe.subscription_id,
        after,
        reader,
        |after| core.journal.wait(after, HEARTBEAT),
    )
}
fn serve_with<S: FrameStream>(
    owner: &str,
    id: &str,
    mut after: Option<u64>,
    reader: &mut BufReader<S>,
    mut sample: impl FnMut(Option<u64>) -> io::Result<Sample>,
) -> io::Result<()> {
    let mut batch_id: u64 = 0;
    loop {
        let sample = sample(after)?;
        batch_id = batch_id
            .checked_add(1)
            .filter(|id| *id <= MAX_SEQUENCE)
            .ok_or_else(|| io::Error::other("Subscription batch sequence exhausted"))?;
        let acknowledgement = FeedAck {
            subscription_id: id.into(),
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
        write_timed_frame(reader.get_mut(), &Response::Feed { batch })?;
        let ack: FeedAck = read_buffered_frame(reader, ACK_TIMEOUT)?;
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
mod tests {
    use super::super::tests::TransportScenario;
    use super::*;
    use crate::observations::Target;
    use std::cell::RefCell;
    fn batch(id: u64, sequence: u64, sample: Sample) -> FeedBatch {
        FeedBatch {
            acknowledgement: FeedAck {
                subscription_id: "feed-1".into(),
                batch_id: id,
                cursor: FeedCursor {
                    owner: "owner-a".into(),
                    sequence,
                },
            },
            sample,
        }
    }
    #[test]
    fn acknowledgements_match_subscription_batch_and_cursor_exactly() {
        let control = Control::default();
        let ack = batch(1, 1, Sample::ResyncRequired { watermark: 1 }).acknowledgement;
        control.offer(ack.clone()).unwrap();
        assert!(control.offer(ack.clone()).is_err());
        for wrong in [
            FeedAck {
                subscription_id: "old".into(),
                ..ack.clone()
            },
            FeedAck {
                batch_id: 2,
                ..ack.clone()
            },
            FeedAck {
                cursor: FeedCursor {
                    owner: "other".into(),
                    sequence: 1,
                },
                ..ack.clone()
            },
            FeedAck {
                cursor: FeedCursor {
                    owner: "owner-a".into(),
                    sequence: 2,
                },
                ..ack.clone()
            },
        ] {
            assert!(control.acknowledge(&wrong).is_err());
        }
        control.acknowledge(&ack).unwrap();
        assert!(control.acknowledge(&ack).is_err());
        control.wait_accepted(Duration::ZERO).unwrap();
        control.offer(ack).unwrap();
        assert_eq!(
            control.wait_accepted(Duration::ZERO).unwrap_err().kind(),
            io::ErrorKind::TimedOut
        );
        control.stop();
        assert!(control.wait_accepted(Duration::ZERO).is_err());
    }
    #[test]
    fn native_delivery_waits_for_frontend_acceptance_before_acknowledging() {
        let control = Control::default();
        let baseline = batch(1, 1, Sample::ResyncRequired { watermark: 1 });
        let hint = batch(
            2,
            2,
            Sample::Hints {
                watermark: 2,
                targets: vec![Target::Repositories],
            },
        );
        let mut bytes = encode_frame(&Response::Feed {
            batch: baseline.clone(),
        })
        .unwrap();
        bytes.extend(
            encode_frame(&Response::Feed {
                batch: hint.clone(),
            })
            .unwrap(),
        );
        let stream = TransportScenario::new(bytes);
        let trace = stream.trace.clone();
        let mut reader = BufReader::new(stream);
        let accepted = RefCell::new(Vec::new());
        let result = run_subscribed(&mut reader, "owner-a", "feed-1", None, &control, &|event| {
            let FeedEvent::Batch { batch } = event else {
                panic!()
            };
            assert_eq!(
                trace
                    .borrow()
                    .output
                    .split(|byte| *byte == b'\n')
                    .filter(|bytes| !bytes.is_empty())
                    .count(),
                accepted.borrow().len()
            );
            accepted.borrow_mut().push(batch.acknowledgement.clone());
            control.acknowledge(&batch.acknowledgement)
        });
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::UnexpectedEof);
        assert_eq!(
            *accepted.borrow(),
            [
                baseline.acknowledgement.clone(),
                hint.acknowledgement.clone()
            ]
        );
        let acks: Vec<FeedAck> = trace
            .borrow()
            .output
            .split(|byte| *byte == b'\n')
            .filter(|bytes| !bytes.is_empty())
            .map(|bytes| serde_json::from_slice(bytes).unwrap())
            .collect();
        assert_eq!(acks, *accepted.borrow());
    }
    #[test]
    fn server_stops_on_wrong_ack_and_preserves_changes_during_resync() {
        for wrong in [false, true] {
            let first = batch(1, 1, Sample::ResyncRequired { watermark: 1 });
            let mut ack = first.acknowledgement.clone();
            if wrong {
                ack.batch_id += 1;
            }
            let mut reader = BufReader::new(TransportScenario::new(encode_frame(&ack).unwrap()));
            let mut calls = 0;
            let result = serve_with("owner-a", "feed-1", None, &mut reader, |after| {
                calls += 1;
                if calls == 1 {
                    assert_eq!(after, None);
                    Ok(Sample::ResyncRequired { watermark: 1 })
                } else {
                    assert_eq!(after, Some(1));
                    Err(io::ErrorKind::Interrupted.into())
                }
            });
            assert!(result.is_err());
            assert_eq!(calls, if wrong { 1 } else { 2 });
        }
    }
    #[test]
    fn owner_and_batch_bounds_are_checked_before_frontend_delivery() {
        let good = batch(
            1,
            2,
            Sample::Hints {
                watermark: 2,
                targets: vec![Target::Repositories],
            },
        );
        assert!(validate_batch(&good, "owner-a", "feed-1", 0, Some(1)).is_ok());
        assert!(validate_batch(&good, "other", "feed-1", 0, Some(1)).is_err());
        assert!(validate_batch(&good, "owner-a", "old", 0, Some(1)).is_err());
        assert!(validate_batch(&good, "owner-a", "feed-1", 1, Some(1)).is_err());
        assert!(validate_batch(&good, "owner-a", "feed-1", 0, Some(2)).is_err());
        assert!(
            validate_batch(
                &batch(
                    1,
                    2,
                    Sample::Hints {
                        watermark: 2,
                        targets: vec![Target::Repositories; 33]
                    }
                ),
                "owner-a",
                "feed-1",
                0,
                Some(1)
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use std::sync::mpsc;
    #[test]
    fn stop_wakes_and_joins_physical_work_before_releasing_the_subscription_lane() {
        let connection = SupervisorConnection::new("/state".into(), "/supervisor".into());
        {
            let mut state = connection.observed.lock().unwrap();
            state.initialized = true;
            state.cursor = Some(SnapshotCursor {
                owner: "owner-a".into(),
                revision: 999,
            });
        }
        let (entered, started) = mpsc::channel();
        let (exited, stopped) = mpsc::channel();
        connection
            .start_subscription_with(
                "owner-a".into(),
                "feed-1".into(),
                Some(FeedCursor {
                    owner: "owner-a".into(),
                    sequence: 7,
                }),
                |_| Ok(()),
                move |_, owner, id, after, control, _| {
                    assert_eq!(owner, "owner-a");
                    assert_eq!(id, "feed-1");
                    assert_eq!(after.unwrap().sequence, 7);
                    let pending = control.pending.lock().unwrap();
                    entered.send(()).unwrap();
                    drop(
                        control
                            .wake
                            .wait_while(pending, |pending| !pending.stopped)
                            .unwrap(),
                    );
                    exited.send(()).unwrap();
                    Ok(())
                },
            )
            .unwrap();
        started.recv().unwrap();
        assert!(connection.subscription.try_acquire().is_none());
        assert!(connection.prepare_snapshot().is_ok());
        assert!(connection.observations.try_acquire().is_some());
        connection.stop_subscription("old-generation").unwrap();
        assert!(stopped.try_recv().is_err());
        connection.stop_subscription("feed-1").unwrap();
        stopped.recv().unwrap();
        assert!(connection.subscription.try_acquire().is_some());
        assert!(connection.feed.lock().unwrap().is_none());
    }
    #[test]
    fn stopped_before_delivery_and_sink_failure_do_not_acknowledge() {
        let control = Control::default();
        control.stop();
        assert!(
            control
                .offer(FeedAck {
                    subscription_id: "feed".into(),
                    batch_id: 1,
                    cursor: FeedCursor {
                        owner: "owner".into(),
                        sequence: 1
                    }
                })
                .is_err()
        );
    }
}

#[cfg(test)]
mod cleanup_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    struct Socket {
        control: Arc<Control>,
        shutdowns: Arc<AtomicUsize>,
    }
    impl FeedSocket for Socket {
        fn shutdown(&self) -> io::Result<()> {
            assert!(
                self.control.pending.try_lock().is_ok(),
                "socket shutdown holds no control lock"
            );
            self.shutdowns.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }
    #[test]
    fn cancellation_shuts_down_attached_and_late_attaching_sockets_without_locks() {
        for late in [false, true] {
            let control = Arc::new(Control::default());
            let shutdowns = Arc::new(AtomicUsize::new(0));
            if late {
                control.stop();
            }
            let result = control.attach(Box::new(Socket {
                control: control.clone(),
                shutdowns: shutdowns.clone(),
            }));
            assert_eq!(result.is_err(), late);
            control.stop();
            assert_eq!(shutdowns.load(Ordering::SeqCst), 1);
        }
    }
    #[test]
    fn sink_failure_closes_without_acknowledging_the_unaccepted_batch() {
        let batch = FeedBatch {
            acknowledgement: FeedAck {
                subscription_id: "feed-1".into(),
                batch_id: 1,
                cursor: FeedCursor {
                    owner: "owner-a".into(),
                    sequence: 1,
                },
            },
            sample: Sample::ResyncRequired { watermark: 1 },
        };
        let stream = super::super::tests::TransportScenario::new(
            encode_frame(&Response::Feed { batch }).unwrap(),
        );
        let trace = stream.trace.clone();
        let mut reader = BufReader::new(stream);
        let result = run_subscribed(
            &mut reader,
            "owner-a",
            "feed-1",
            None,
            &Control::default(),
            &|_| Err(io::ErrorKind::BrokenPipe.into()),
        );
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::BrokenPipe);
        assert!(trace.borrow().output.is_empty());
    }
}

#[cfg(test)]
mod corpus_tests {
    use super::*;
    #[test]
    fn shared_corpus_matches_rust_feed_serialization_and_delivery_bounds() {
        let corpus: serde_json::Value =
            serde_json::from_str(include_str!("../../../../contracts/observations-v7.json"))
                .unwrap();
        for value in corpus["feed"].as_array().unwrap() {
            let batch: FeedBatch = serde_json::from_value(value.clone()).unwrap();
            assert_eq!(serde_json::to_value(&batch).unwrap(), *value);
            assert!(encode_frame(&Response::Feed { batch }).unwrap().len() <= DELIVERY_BYTES);
        }
    }
}
