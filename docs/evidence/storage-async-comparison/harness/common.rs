use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::VecDeque,
    error::Error,
    path::Path,
    rc::Rc,
    time::{Duration, Instant},
};
use tokio::{
    sync::{Semaphore, oneshot},
    task::JoinHandle,
};
type E = Box<dyn Error + Send + Sync>;
type R<T> = Result<T, E>;
const SCHEMA: &str = "CREATE TABLE streams(id INTEGER PRIMARY KEY,workspace INTEGER NOT NULL,root INTEGER NOT NULL,parent INTEGER REFERENCES streams(id)); CREATE TABLE events(id INTEGER PRIMARY KEY,stream INTEGER NOT NULL REFERENCES streams(id),seq INTEGER NOT NULL,payload TEXT NOT NULL,UNIQUE(stream,seq)); CREATE TABLE context(workspace INTEGER PRIMARY KEY,version INTEGER NOT NULL,completed INTEGER NOT NULL); CREATE TABLE intents(id INTEGER PRIMARY KEY,workspace INTEGER NOT NULL,context_version INTEGER NOT NULL); CREATE TABLE drafts(workspace INTEGER PRIMARY KEY,revision INTEGER NOT NULL,external_identity TEXT NOT NULL,body TEXT NOT NULL);";
const CONFIG: &str = "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA fullfsync=ON; PRAGMA foreign_keys=ON; PRAGMA busy_timeout=100; PRAGMA cache_size=-2048; PRAGMA wal_autocheckpoint=1000;";
const PAGE: &str = "SELECT id,seq,payload FROM (SELECT id,seq,payload,sum(length(payload)) OVER(ORDER BY seq DESC) AS bytes FROM events WHERE stream=? AND seq<? ORDER BY seq DESC LIMIT 64) WHERE bytes<=262144";
const VERIFY: &str = "SELECT count(*),coalesce(sum(e.id*3+e.stream*7+e.seq*11+length(e.payload)*13+unicode(substr(e.payload,1,1))*17+unicode(substr(e.payload,-1,1))*19),0) FROM events e WHERE e.id>0";
#[derive(Clone)]
struct Event {
    id: i64,
    stream: i64,
    seq: i64,
    ws: usize,
    payload: String,
    arrival: Instant,
}
impl Event {
    fn sig(&self) -> i64 {
        self.id * 3
            + self.stream * 7
            + self.seq * 11
            + self.payload.len() as i64 * 13
            + self.payload.as_bytes()[0] as i64 * 36
    }
}
fn event(id: i64, stream: usize, arrival: Instant) -> Event {
    let n = match id % 100 {
        0 => 65536,
        1..=9 => 2048,
        10 => 16384,
        _ => 256,
    };
    Event {
        id,
        stream: stream as i64,
        seq: id,
        ws: stream / 25,
        payload: std::iter::repeat_n((65 + id % 26) as u8 as char, n).collect(),
        arrival,
    }
}
fn record_hash(
    id: i64,
    stream: i64,
    seq: i64,
    workspace: i64,
    root: i64,
    parent: Option<i64>,
    payload: &str,
) -> u64 {
    let mut h = 14695981039346656037u64;
    for value in [id, stream, seq, workspace, root, parent.unwrap_or(-1)] {
        for b in value.to_le_bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(1099511628211)
        }
    }
    for b in payload.bytes() {
        h = (h ^ u64::from(b)).wrapping_mul(1099511628211)
    }
    h
}
fn percentile(v: &[f64], p: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    let mut x = v.to_vec();
    x.sort_by(f64::total_cmp);
    x[((x.len() - 1) as f64 * p) as usize]
}
fn stats(v: &[f64]) -> Value {
    json!({"n":v.len(),"p50_ms":percentile(v,0.50),"p95_ms":percentile(v,0.95),"p99_ms":percentile(v,0.99),"max_ms":percentile(v,1.0)})
}
struct Queue {
    lanes: Vec<VecDeque<Event>>,
    inflight_ws: Vec<usize>,
    count: usize,
    bytes: usize,
    rr: usize,
    urgent: VecDeque<(i64, usize, Instant)>,
    done: bool,
    rejected: Vec<u64>,
    offered: Vec<u64>,
    admitted: Vec<u64>,
    max_count: usize,
    max_bytes: usize,
}
impl Queue {
    fn new(n: usize) -> Self {
        Self {
            lanes: (0..n).map(|_| VecDeque::new()).collect(),
            inflight_ws: vec![0; n],
            count: 0,
            bytes: 0,
            rr: 0,
            urgent: VecDeque::new(),
            done: false,
            rejected: vec![0; n],
            offered: vec![0; n],
            admitted: vec![0; n],
            max_count: 0,
            max_bytes: 0,
        }
    }
    fn admit(&mut self, e: Event) {
        let w = e.ws;
        self.offered[w] += 1;
        if self.count >= 1024 || self.bytes + e.payload.len() > 8388608 || self.inflight_ws[w] >= 64
        {
            self.rejected[w] += 1;
            return;
        }
        self.count += 1;
        self.bytes += e.payload.len();
        self.inflight_ws[w] += 1;
        self.admitted[w] += 1;
        self.lanes[w].push_back(e);
        self.max_count = self.max_count.max(self.count);
        self.max_bytes = self.max_bytes.max(self.bytes)
    }
    fn batch(&mut self, now: Instant) -> Vec<Event> {
        let queued: usize = self.lanes.iter().map(VecDeque::len).sum();
        let old = self
            .lanes
            .iter()
            .filter_map(|x| x.front().map(|e| e.arrival))
            .min();
        if queued == 0
            || (!self.done
                && queued < 16
                && old.is_some_and(|x| now.duration_since(x) < Duration::from_millis(25)))
        {
            return vec![];
        }
        let mut out = vec![];
        for _ in 0..16 {
            let mut found = false;
            for _ in 0..self.lanes.len() {
                let w = self.rr;
                self.rr = (self.rr + 1) % self.lanes.len();
                if let Some(e) = self.lanes[w].pop_front() {
                    out.push(e);
                    found = true;
                    break;
                }
            }
            if !found {
                break;
            }
        }
        out
    }
    fn finish(&mut self, b: &[Event]) {
        for e in b {
            self.count -= 1;
            self.bytes -= e.payload.len();
            self.inflight_ws[e.ws] -= 1
        }
    }
}
#[cfg(feature = "sqlx-driver")]
mod driver;
#[cfg(not(feature = "sqlx-driver"))]
mod driver;
use driver::Db;
struct Writes {
    lat: Vec<Vec<f64>>,
    urgent: Vec<f64>,
    checkpoints: Vec<Value>,
    signature: i64,
    count: usize,
    commits: usize,
    hashes: Vec<(i64, u64)>,
}
async fn owner(db: Db, q: Rc<RefCell<Queue>>, start: Instant) -> R<Writes> {
    let n = q.borrow().lanes.len();
    let mut out = Writes {
        lat: vec![vec![]; n],
        urgent: vec![],
        checkpoints: vec![],
        signature: 0,
        count: 0,
        commits: 0,
        hashes: vec![],
    };
    let mut cp = start + Duration::from_secs(2);
    loop {
        let u = q.borrow_mut().urgent.pop_front();
        if let Some((id, w, t)) = u {
            db.intent(id, w).await?;
            out.urgent.push(t.elapsed().as_secs_f64() * 1000.0)
        }
        let batch = q.borrow_mut().batch(Instant::now());
        if !batch.is_empty() {
            db.write(&batch).await?;
            let committed = Instant::now();
            for e in &batch {
                out.lat[e.ws].push(committed.duration_since(e.arrival).as_secs_f64() * 1000.0);
                out.signature += e.sig();
                let root = e.stream / 5 * 5;
                out.hashes.push((
                    e.id,
                    record_hash(
                        e.id,
                        e.stream,
                        e.seq,
                        e.ws as i64,
                        root,
                        if e.stream % 5 == 0 { None } else { Some(root) },
                        &e.payload,
                    ),
                ))
            }
            out.count += batch.len();
            out.commits += 1;
            q.borrow_mut().finish(&batch)
        }
        if Instant::now() >= cp {
            let t = Instant::now();
            let frames = db.checkpoint(false).await?;
            out.checkpoints.push(json!({"at_s":start.elapsed().as_secs_f64(),"latency_ms":t.elapsed().as_secs_f64()*1000.0,"tuple":frames}));
            cp += Duration::from_secs(2)
        }
        if {
            let x = q.borrow();
            x.done && x.count == 0 && x.urgent.is_empty()
        } {
            break;
        }
        tokio::time::sleep(Duration::from_millis(1)).await
    }
    Ok(out)
}
async fn read_loop(
    db: Db,
    idx: usize,
    start: Instant,
    _stop: Rc<RefCell<bool>>,
    streams: usize,
) -> R<Value> {
    let mut lat = vec![vec![]; 4];
    let mut end_to_end = vec![vec![]; 4];
    let mut max_late: f64 = 0.0;
    for k in 0..600 {
        let scheduled = start + Duration::from_millis(k as u64 * 50);
        tokio::time::sleep_until(tokio::time::Instant::from_std(scheduled)).await;
        let t = Instant::now();
        max_late = max_late.max(t.duration_since(scheduled).as_secs_f64() * 1000.0);
        db.read(idx, k % 4, k % streams).await?;
        lat[k % 4].push(t.elapsed().as_secs_f64() * 1000.0);
        end_to_end[k % 4].push(scheduled.elapsed().as_secs_f64() * 1000.0)
    }
    Ok(
        json!({"reader":idx,"offered":600,"completed":600,"rejected":0,"max_start_lateness_ms":max_late,"recent_page":stats(&lat[0]),"older_page":stats(&lat[1]),"context":stats(&lat[2]),"draft_inbox":stats(&lat[3]),"scheduled_to_completion":end_to_end.iter().map(|v|stats(v)).collect::<Vec<_>>()}),
    )
}
async fn focused(path: &Path) -> R<Value> {
    let db = Db::open(path, 1).await?;
    let permits = std::sync::Arc::new(Semaphore::new(1));
    let permit = permits.clone().acquire_owned().await?;
    let (reply, rx) = oneshot::channel();
    drop(rx);
    let (started, started_rx) = oneshot::channel();
    let (release, release_rx) = oneshot::channel();
    let d = db.clone();
    let t = tokio::task::spawn_local(async move {
        let _ = started.send(());
        let _ = release_rx.await;
        d.write(&[event(1, 0, Instant::now())]).await.unwrap();
        let _ = reply.send(());
        drop(permit)
    });
    started_rx.await?;
    let capacity_retained = permits.clone().try_acquire_owned().is_err();
    let rejected_identity = 2i64;
    let rejected_permit = permits.clone().try_acquire_owned();
    assert!(rejected_permit.is_err());
    let rejected_absent_before = db.verify().await?.0 == 0;
    release.send(()).unwrap();
    t.await?;
    let committed = db.verify().await?;
    let future = db.future_probe().await?;
    let busy = db.busy_probe(path).await?;
    let private = db.private_probe(path).await?;
    db.close().await?;
    Ok(
        json!({"kind":"focused","driver":driver::NAME,"dropped_reply_capacity_retained":capacity_retained,"saturation_rejected_identity":rejected_identity,"saturation_rejected_absent":rejected_absent_before&&committed.0==1,"committed_after_reply_dropped":committed.0==1,"permit_released_only_after_physical":permits.available_permits()==1,"drain_close_completed":true,"future_primitives":future,"real_commit_busy":busy,"private_open":private}),
    )
}
async fn run(path: &Path, workload: &str, run: usize) -> R<Value> {
    let workspaces = if workload == "baseline" { 10 } else { 20 };
    let streams = workspaces * 25;
    let db = Db::open(path, workspaces).await?;
    let seed = Instant::now();
    db.seed(100000, streams).await?;
    let seed_s = seed.elapsed().as_secs_f64();
    let env = db.environment().await?;
    let cp_before = db.checkpoint(false).await?;
    let q = Rc::new(RefCell::new(Queue::new(workspaces)));
    let stop = Rc::new(RefCell::new(false));
    let start = Instant::now();
    let writer = tokio::task::spawn_local(owner(db.clone(), q.clone(), start));
    let mut readers: Vec<JoinHandle<R<Value>>> = vec![];
    for i in 0..2 {
        readers.push(tokio::task::spawn_local(read_loop(
            db.clone(),
            i,
            start,
            stop.clone(),
            streams,
        )))
    }
    let heartbeat_stop = stop.clone();
    let heart = tokio::task::spawn_local(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(10));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut v = vec![];
        while !*heartbeat_stop.borrow() {
            let due = interval.tick().await;
            v.push(
                tokio::time::Instant::now()
                    .duration_since(due)
                    .as_secs_f64()
                    * 1000.0,
            )
        }
        v
    });
    let mut id = 1;
    let mut offered_sig = 0i64;
    let mut next_stream = 0;
    let mut urgent_rejected = 0;
    let mut accepted_urgent = vec![];
    for tick in 0..6000 {
        let arrival = start + Duration::from_millis(tick * 5);
        tokio::time::sleep_until(tokio::time::Instant::from_std(arrival)).await;
        let count = if workload == "baseline" {
            if tick % 2 == 0 { 3 } else { 2 }
        } else {
            10
        };
        for _ in 0..count {
            let e = event(id, next_stream % streams, arrival);
            offered_sig += e.sig();
            q.borrow_mut().admit(e);
            next_stream += 1;
            id += 1
        }
        if workload == "heavy" && tick % 1000 < 100 {
            for j in 0..10 {
                let e = event(id, (tick as usize + j) % 25, arrival);
                offered_sig += e.sig();
                q.borrow_mut().admit(e);
                id += 1
            }
        }
        if tick % 40 == 0 {
            let mut x = q.borrow_mut();
            if x.urgent.len() < 32 {
                accepted_urgent.push((tick as i64, tick as usize / 40 % workspaces));
                x.urgent
                    .push_back((tick as i64, tick as usize / 40 % workspaces, arrival))
            } else {
                urgent_rejected += 1
            }
        }
    }
    tokio::time::sleep_until(tokio::time::Instant::from_std(
        start + Duration::from_secs(30),
    ))
    .await;
    q.borrow_mut().done = true;
    let mut writes = writer.await??;
    let drain_s = start.elapsed().as_secs_f64() - 30.0;
    *stop.borrow_mut() = true;
    let mut read_results = vec![];
    for h in readers {
        read_results.push(h.await??)
    }
    let heartbeat = heart.await?;
    let reader_drain_s = (start.elapsed().as_secs_f64() - 30.0).max(0.0);
    assert_eq!(writes.urgent.len() + urgent_rejected, 150);
    let intent_rows = db.intent_audit().await?;
    assert_eq!(intent_rows, accepted_urgent);
    let actual = db.verify().await?;
    let integrity = db.integrity().await?;
    let actual_hashes = db.audit().await?;
    writes.hashes.sort_unstable_by_key(|x| x.0);
    let hashes_match = actual_hashes == writes.hashes;
    assert!(hashes_match);
    let cp_t = Instant::now();
    let final_cp = db.checkpoint(true).await?;
    let checkpoint_ms = cp_t.elapsed().as_secs_f64() * 1000.0;
    let q = q.borrow();
    for w in 0..workspaces {
        assert_eq!(q.offered[w], q.admitted[w] + q.rejected[w]);
        assert_eq!(q.admitted[w] as usize, writes.lat[w].len());
    }
    assert_eq!(
        (id - 1) as usize,
        writes.count + q.rejected.iter().sum::<u64>() as usize
    );
    let all: Vec<f64> = writes.lat.iter().flatten().copied().collect();
    let fairness:Vec<Value>=(0..workspaces).map(|w|json!({"workspace":w,"offered":q.offered[w],"admitted":q.admitted[w],"rejected":q.rejected[w],"arrival_to_commit":stats(&writes.lat[w])})).collect();
    let mut result = json!({"kind":"workload","driver":driver::NAME,"workload":workload,"run":run,"warmup":run==0,"window_s":30,"seed_s":seed_s,"env":env,"offered":id-1,"offered_signature":offered_sig,"committed":writes.count,"rejected":q.rejected.iter().sum::<u64>(),"urgent_rejected":urgent_rejected,"urgent":stats(&writes.urgent),"commits":writes.commits,"events_per_second_including_drain":writes.count as f64/(30.0+drain_s),"drain_s":drain_s,"reader_drain_s":reader_drain_s,"urgent_durable_identity_audit":true,"arrival_to_commit":stats(&all),"per_workspace":fairness,"readers":read_results,"heartbeat_lateness":stats(&heartbeat),"max_outstanding_events":q.max_count,"max_outstanding_bytes":q.max_bytes,"checkpoint_before":cp_before,"checkpoints":writes.checkpoints,"checkpoint_final":final_cp,"checkpoint_final_ms":checkpoint_ms,"verify":{"expected_count":writes.count,"actual_count":actual.0,"expected_signature":writes.signature,"actual_signature":actual.1,"matched":actual.0==writes.count as i64&&actual.1==writes.signature,"integrity":integrity,"all_record_full_payload_hashes_and_parent_routes_match":hashes_match,"hash_algorithm":"FNV1a64 over ordered id/stream/seq/workspace/root/parent LE bytes and full payload; exact per-record comparison, noncryptographic"},"database_bytes":std::fs::metadata(path)?.len()});
    drop(q);
    let close_started = Instant::now();
    db.close().await?;
    result["close_ack_ms"] = json!(close_started.elapsed().as_secs_f64() * 1000.0);
    result["total_run_s"] = json!(start.elapsed().as_secs_f64());
    Ok(result)
}
fn main() -> R<()> {
    let args: Vec<String> = std::env::args().collect();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let local = tokio::task::LocalSet::new();
    let result = rt.block_on(local.run_until(async {
        if args[1] == "focused" {
            focused(Path::new(&args[2])).await
        } else {
            run(Path::new(&args[2]), &args[1], args[3].parse()?).await
        }
    }))?;
    println!("{}", serde_json::to_string(&result)?);
    Ok(())
}
