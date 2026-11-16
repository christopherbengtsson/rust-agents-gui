// Standalone real-storage experiment; not an application unit/integration test.
use serde_json::{Value, json};
use std::{
    error::Error,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
const INSERT: &str = "INSERT INTO events(conversation,seq,body) VALUES(?1,?2,?3)";
const UPDATE: &str = "INSERT INTO metadata(id,value) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET value=excluded.value";
const PAGE: &str =
    "SELECT body FROM events WHERE conversation=?1 AND seq<?2 ORDER BY seq DESC LIMIT 100";
trait Store {
    fn open(path: &Path, journal: &str, sync: &str) -> Result<Self>
    where
        Self: Sized;
    fn exec(&mut self, sql: &str) -> Result<()>;
    fn text(&mut self, sql: &str) -> Result<String>;
    fn insert(&mut self, c: i64, s: i64, b: &str) -> Result<()>;
    fn metadata(&mut self, id: i64, value: i64) -> Result<()>;
    fn page(&mut self, c: i64, before: i64) -> Result<usize>;
    fn close(self) -> Result<()>;
}
#[cfg(feature = "rusqlite")]
struct Database(rusqlite::Connection);
#[cfg(feature = "rusqlite")]
impl Store for Database {
    fn open(path: &Path, journal: &str, sync: &str) -> Result<Self> {
        let c = rusqlite::Connection::open(path)?;
        c.busy_timeout(Duration::from_millis(100))?;
        c.set_prepared_statement_cache_capacity(100);
        let mut d = Self(c);
        configure(&mut d, journal, sync)?;
        Ok(d)
    }
    fn exec(&mut self, s: &str) -> Result<()> {
        self.0.execute_batch(s)?;
        Ok(())
    }
    fn text(&mut self, s: &str) -> Result<String> {
        Ok(self.0.query_row(s, [], |r| r.get(0))?)
    }
    fn insert(&mut self, c: i64, s: i64, b: &str) -> Result<()> {
        self.0
            .prepare_cached(INSERT)?
            .execute(rusqlite::params![c, s, b])?;
        Ok(())
    }
    fn metadata(&mut self, id: i64, v: i64) -> Result<()> {
        self.0
            .prepare_cached(UPDATE)?
            .execute(rusqlite::params![id, v])?;
        Ok(())
    }
    fn page(&mut self, c: i64, b: i64) -> Result<usize> {
        let mut q = self.0.prepare_cached(PAGE)?;
        let mut rows = q.query(rusqlite::params![c, b])?;
        let mut values = Vec::new();
        while let Some(r) = rows.next()? {
            values.push(r.get::<_, String>(0)?);
        }
        Ok(values.iter().map(String::len).sum())
    }
    fn close(self) -> Result<()> {
        self.0.close().map_err(|(_, e)| e)?;
        Ok(())
    }
}
#[cfg(feature = "sqlx")]
struct Database {
    rt: tokio::runtime::Runtime,
    c: sqlx::SqliteConnection,
}
#[cfg(feature = "sqlx")]
impl Store for Database {
    fn open(path: &Path, journal: &str, sync: &str) -> Result<Self> {
        use sqlx::Connection;
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(true)
            .busy_timeout(Duration::from_millis(100))
            .statement_cache_capacity(100);
        let c = rt.block_on(sqlx::SqliteConnection::connect_with(&options))?;
        let mut d = Self { rt, c };
        configure(&mut d, journal, sync)?;
        Ok(d)
    }
    fn exec(&mut self, s: &str) -> Result<()> {
        self.rt
            .block_on(sqlx::raw_sql(sqlx::AssertSqlSafe(s)).execute(&mut self.c))?;
        Ok(())
    }
    fn text(&mut self, s: &str) -> Result<String> {
        Ok(self.rt.block_on(
            sqlx::query_scalar::<_, String>(sqlx::AssertSqlSafe(s)).fetch_one(&mut self.c),
        )?)
    }
    fn insert(&mut self, c: i64, s: i64, b: &str) -> Result<()> {
        self.rt.block_on(
            sqlx::query(INSERT)
                .bind(c)
                .bind(s)
                .bind(b)
                .execute(&mut self.c),
        )?;
        Ok(())
    }
    fn metadata(&mut self, id: i64, v: i64) -> Result<()> {
        self.rt
            .block_on(sqlx::query(UPDATE).bind(id).bind(v).execute(&mut self.c))?;
        Ok(())
    }
    fn page(&mut self, c: i64, b: i64) -> Result<usize> {
        let rows = self.rt.block_on(
            sqlx::query_scalar::<_, String>(PAGE)
                .bind(c)
                .bind(b)
                .fetch_all(&mut self.c),
        )?;
        Ok(rows.iter().map(String::len).sum())
    }
    fn close(self) -> Result<()> {
        use sqlx::Connection;
        self.rt.block_on(self.c.close())?;
        Ok(())
    }
}
fn configure(d: &mut impl Store, journal: &str, sync: &str) -> Result<()> {
    assert!(["DELETE", "WAL"].contains(&journal));
    assert!(["FULL", "EXTRA", "NORMAL"].contains(&sync));
    d.exec(&format!("PRAGMA journal_mode={journal}; PRAGMA synchronous={sync}; PRAGMA fullfsync=ON; PRAGMA foreign_keys=ON; PRAGMA cache_size=-2048; PRAGMA wal_autocheckpoint=1000;"))
}
fn payload(i: usize) -> String {
    let n = match i % 100 {
        0..=89 => 256,
        90..=98 => 2048,
        _ => 16384,
    };
    "x".repeat(n)
}
fn percentiles(mut a: Vec<f64>) -> Value {
    a.sort_by(f64::total_cmp);
    if a.is_empty() {
        return json!({"n":0});
    }
    let at = |p: f64| a[((a.len() - 1) as f64 * p).round() as usize];
    json!({"n":a.len(),"p50_ms":at(0.50),"p95_ms":at(0.95),"p99_ms":at(0.99),"max_ms":a[a.len()-1]})
}
fn bytes(path: &Path) -> u64 {
    ["", "-wal", "-shm"]
        .iter()
        .map(|s| std::fs::metadata(format!("{}{s}", path.display())).map_or(0, |m| m.len()))
        .sum()
}
fn info(d: &mut impl Store) -> Result<Value> {
    Ok(
        json!({"sqlite_version":d.text("SELECT sqlite_version()")?,"sqlite_source_id":d.text("SELECT sqlite_source_id()")?,"compile_options":d.text("SELECT group_concat(compile_options,',') FROM pragma_compile_options")?,"journal":d.text("SELECT journal_mode FROM pragma_journal_mode")?,"synchronous":d.text("SELECT CAST(synchronous AS TEXT) FROM pragma_synchronous")?,"fullfsync":d.text("SELECT CAST(fullfsync AS TEXT) FROM pragma_fullfsync")?}),
    )
}
fn seed(path: &Path, n: usize) -> Result<Value> {
    let mut d = Database::open(path, "DELETE", "FULL")?;
    d.exec("CREATE TABLE events(id INTEGER PRIMARY KEY,conversation INTEGER NOT NULL,seq INTEGER NOT NULL,body TEXT NOT NULL,UNIQUE(conversation,seq)); CREATE TABLE metadata(id INTEGER PRIMARY KEY,value INTEGER NOT NULL);")?;
    let t = Instant::now();
    d.exec("BEGIN IMMEDIATE")?;
    for i in 0..n {
        d.insert(
            (i % 1000) as i64,
            (i / 1000) as i64,
            &payload(i / 1000 + i % 1000),
        )?;
        if i % 10000 == 9999 {
            d.exec("COMMIT;BEGIN IMMEDIATE")?;
        }
    }
    d.exec("COMMIT")?;
    let v = json!({"seed":n,"seed_ms":t.elapsed().as_secs_f64()*1000.,"bytes":bytes(path),"info":info(&mut d)?});
    d.close()?;
    Ok(v)
}
struct Job {
    queued: Instant,
    rows: Vec<(i64, i64)>,
    index: usize,
}
fn run(
    path: &Path,
    journal: &str,
    sync: &str,
    active: usize,
    batch: usize,
    duration: f64,
    reader_count: usize,
    rate: f64,
    reopen: bool,
    coalesce: bool,
) -> Result<Value> {
    let mut d = Database::open(path, journal, sync)?;
    let inf = info(&mut d)?;
    let max_seq = d
        .text("SELECT CAST(COALESCE(MAX(seq),0)+1 AS TEXT) FROM events")?
        .parse::<i64>()?;
    let stop = Arc::new(AtomicBool::new(false));
    let mut readers = Vec::new();
    for i in 0..reader_count {
        let p = path.to_owned();
        let s = stop.clone();
        let j = journal.to_owned();
        let sy = sync.to_owned();
        let mut rd = Database::open(&p, &j, &sy)?;
        readers.push(thread::spawn(move || -> Result<_> {
            let mut times = Vec::new();
            let mut errors = 0;
            while !s.load(Ordering::Relaxed) {
                let t = Instant::now();
                if rd.page(i as i64, i64::MAX).is_err() {
                    errors += 1;
                }
                times.push(t.elapsed().as_secs_f64() * 1000.);
                thread::sleep(Duration::from_millis(10));
            }
            rd.close()?;
            Ok((times, errors))
        }));
    }
    let (tx, rx) = mpsc::sync_channel::<Job>(64);
    let worker_path = path.to_owned();
    let worker_journal = journal.to_owned();
    let worker_sync = sync.to_owned();
    let worker = thread::spawn(move || -> Result<_> {
        let (mut q, mut service, mut reads, mut errors) =
            (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut count, mut checksum) = (0, 0);
        let mut peak = bytes(&worker_path);
        for job in rx {
            q.push(job.queued.elapsed().as_secs_f64() * 1000.);
            let t = Instant::now();
            if reopen {
                d.close()?;
                d = Database::open(&worker_path, &worker_journal, &worker_sync)?;
            }
            let outcome = (|| -> Result<()> {
                d.exec("BEGIN IMMEDIATE")?;
                for &(c, seq) in &job.rows {
                    d.insert(c, seq, &payload((c + seq) as usize))?;
                }
                if job.index % 10 == 0 {
                    d.metadata(job.rows[0].0, job.rows[0].1)?;
                }
                d.exec("COMMIT")?;
                Ok(())
            })();
            match outcome {
                Ok(()) => count += job.rows.len(),
                Err(e) => {
                    errors.push(e.to_string());
                    let _ = d.exec("ROLLBACK");
                }
            }
            service.push(t.elapsed().as_secs_f64() * 1000.);
            if reader_count == 0 && job.index % 2 == 0 {
                let t = Instant::now();
                let before = if job.index % 4 == 0 {
                    max_seq / 2
                } else {
                    i64::MAX
                };
                checksum += d.page(job.rows[0].0, before)?;
                reads.push(t.elapsed().as_secs_f64() * 1000.);
            }
            peak = peak.max(bytes(&worker_path));
        }
        let close = Instant::now();
        d.close()?;
        Ok((
            q,
            service,
            reads,
            errors,
            count,
            checksum,
            peak,
            close.elapsed().as_secs_f64() * 1000.,
        ))
    });
    let t = Instant::now();
    let (mut admitted, mut rejected, mut index) = (0, 0, 0);
    let mut formation = Vec::new();
    if coalesce {
        assert!(rate > 0.);
        let interval = Duration::from_secs_f64(1. / (rate * active as f64));
        let mut next = t;
        let mut first: Option<Instant> = None;
        let mut rows = Vec::new();
        let mut arrivals = Vec::new();
        let mut event = 0;
        loop {
            let now = Instant::now();
            let ending = now.duration_since(t).as_secs_f64() >= duration;
            if !ending && now >= next {
                let c = (event % active) as i64;
                let seq = max_seq + (event / active) as i64;
                rows.push((c, seq));
                arrivals.push(now);
                first.get_or_insert(now);
                event += 1;
                next += interval;
            }
            let flush = rows.len() >= batch
                || first.is_some_and(|f| f.elapsed() >= Duration::from_millis(25))
                || ending;
            if flush && !rows.is_empty() {
                let queued = Instant::now();
                formation.extend(
                    arrivals
                        .drain(..)
                        .map(|a| queued.duration_since(a).as_secs_f64() * 1000.),
                );
                let job = Job {
                    queued,
                    rows: std::mem::take(&mut rows),
                    index,
                };
                match tx.try_send(job) {
                    Ok(()) => admitted += 1,
                    Err(mpsc::TrySendError::Full(_)) => rejected += 1,
                    Err(mpsc::TrySendError::Disconnected(_)) => {
                        return Err("worker disconnected".into());
                    }
                }
                index += 1;
                first = None;
            }
            if ending {
                break;
            }
            let deadline = first.map_or(next, |f| next.min(f + Duration::from_millis(25)));
            if let Some(wait) = deadline.checked_duration_since(Instant::now()) {
                thread::sleep(wait);
            }
        }
    } else {
        let interval = if rate > 0. {
            Duration::from_secs_f64(batch as f64 / (rate * active as f64))
        } else {
            Duration::ZERO
        };
        let mut next = t;
        while t.elapsed().as_secs_f64() < duration {
            let rows = (0..batch)
                .map(|k| {
                    (
                        (index % active) as i64,
                        max_seq + (index / active * batch + k) as i64,
                    )
                })
                .collect();
            let job = Job {
                queued: Instant::now(),
                rows,
                index,
            };
            match tx.try_send(job) {
                Ok(()) => admitted += 1,
                Err(mpsc::TrySendError::Full(_)) => rejected += 1,
                Err(mpsc::TrySendError::Disconnected(_)) => {
                    return Err("worker disconnected".into());
                }
            }
            index += 1;
            if interval.is_zero() {
                thread::yield_now();
            } else {
                next += interval;
                if let Some(wait) = next.checked_duration_since(Instant::now()) {
                    thread::sleep(wait);
                }
            }
        }
    }
    drop(tx);
    let drain = Instant::now();
    let (q, service, reads, errors, count, checksum, peak, close_ms) =
        worker.join().map_err(|_| "worker panic")??;
    let drain_ms = drain.elapsed().as_secs_f64() * 1000.;
    stop.store(true, Ordering::Relaxed);
    let (mut rt, mut re) = (Vec::new(), 0);
    for h in readers {
        let (a, b) = h.join().map_err(|_| "reader panic")??;
        rt.extend(a);
        re += b;
    }
    let elapsed = t.elapsed().as_secs_f64();
    let mut verify = Database::open(path, journal, sync)?;
    let integrity = verify.text("PRAGMA integrity_check")?;
    verify.close()?;
    Ok(
        json!({"coalesce":coalesce,"batch_formation":percentiles(formation),"reopen_per_transaction":reopen,"journal":journal,"sync":sync,"active":active,"batch":batch,"duration_s":duration,"readers":reader_count,"offered_events_per_chat_s":rate,"admitted_transactions":admitted,"rejected_transactions":rejected,"committed_events":count,"elapsed_s":elapsed,"events_per_s":count as f64/elapsed,"queue":percentiles(q),"service":percentiles(service),"history":percentiles(reads),"reader_history":percentiles(rt),"reader_errors":re,"errors":errors,"read_checksum":checksum,"sampled_main_wal_shm_bytes":peak,"closed_bytes":bytes(path),"writer_drain_ms":drain_ms,"writer_close_ms":close_ms,"integrity":integrity,"info":inf}),
    )
}
fn lifecycle(path: &Path) -> Result<Value> {
    let mut d = Database::open(path, "WAL", "FULL")?;
    let (tx, rx) = mpsc::sync_channel::<i64>(1);
    let (gate_send, gate_receive) = mpsc::channel();
    let (reply_send, reply_receive) = mpsc::channel();
    let h = thread::spawn(move || -> Result<_> {
        gate_receive.recv()?;
        for id in rx {
            d.exec("BEGIN IMMEDIATE")?;
            d.metadata(id, 7)?;
            d.exec("COMMIT")?;
            let _ = reply_send.send(id);
        }
        d.close()?;
        Ok(())
    });
    tx.try_send(999001)?;
    let saturated = matches!(tx.try_send(999002), Err(mpsc::TrySendError::Full(_)));
    drop(reply_receive);
    drop(tx);
    gate_send.send(())?;
    h.join().map_err(|_| "worker panic")??;
    let mut d = Database::open(path, "WAL", "FULL")?;
    let rows = d.text("SELECT CAST(COUNT(*) AS TEXT) FROM metadata WHERE id=999001 AND value=7")?;
    let absent = d.text("SELECT CAST(COUNT(*) AS TEXT) FROM metadata WHERE id=999002")?;
    d.close()?;
    Ok(
        json!({"saturation_rejected":saturated,"admitted_write_committed_after_reply_receiver_drop":rows=="1","rejected_write_absent":absent=="0","physical_worker_joined":true}),
    )
}
fn main() -> Result<()> {
    let a = std::env::args().collect::<Vec<_>>();
    let path = Path::new(&a[2]);
    let v = match a[1].as_str() {
        "seed" => seed(path, a[3].parse()?)?,
        "lifecycle" => lifecycle(path)?,
        "run" => run(
            path,
            &a[3],
            &a[4],
            a[5].parse()?,
            a[6].parse()?,
            a[7].parse()?,
            a[8].parse()?,
            a[9].parse()?,
            a.get(10).is_some_and(|s| s == "reopen"),
            a.iter().any(|s| s == "coalesce"),
        )?,
        _ => return Err("mode".into()),
    };
    println!("{}", v);
    Ok(())
}
