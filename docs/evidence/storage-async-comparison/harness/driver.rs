use super::*;
#[cfg(feature = "sqlx-driver")]
mod imp {
    use super::*;
    use sqlx::{
        AssertSqlSafe, Row, SqlitePool,
        sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    };
    pub const NAME: &str = "sqlx-0.9.0";
    #[derive(Clone)]
    pub struct Db {
        w: SqlitePool,
        r: SqlitePool,
    }
    impl Db {
        pub async fn open(path: &Path, n: usize) -> R<Self> {
            let options = SqliteConnectOptions::new()
                .filename(path)
                .create_if_missing(true)
                .busy_timeout(Duration::from_millis(100))
                .statement_cache_capacity(100)
                .pragma("journal_mode", "WAL")
                .pragma("synchronous", "FULL")
                .pragma("fullfsync", "ON")
                .pragma("foreign_keys", "ON")
                .pragma("cache_size", "-2048")
                .pragma("wal_autocheckpoint", "1000");
            let w = SqlitePoolOptions::new()
                .max_connections(1)
                .min_connections(1)
                .connect_with(options.clone())
                .await?;
            sqlx::raw_sql(SCHEMA).execute(&w).await?;
            let mut tx = w.begin().await?;
            for id in 0..n * 25 {
                sqlx::query("INSERT INTO streams VALUES(?,?,?,?)")
                    .bind(id as i64)
                    .bind((id / 25) as i64)
                    .bind((id / 5 * 5) as i64)
                    .bind(if id % 5 == 0 {
                        None
                    } else {
                        Some((id / 5 * 5) as i64)
                    })
                    .execute(&mut *tx)
                    .await?;
            }
            for ws in 0..n {
                sqlx::query("INSERT INTO context VALUES(?,0,0)")
                    .bind(ws as i64)
                    .execute(&mut *tx)
                    .await?;
                sqlx::query("INSERT INTO drafts VALUES(?,0,?,'human-edit')")
                    .bind(ws as i64)
                    .bind(format!("local-external-operation-{ws}"))
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
            let r = SqlitePoolOptions::new()
                .max_connections(2)
                .min_connections(2)
                .connect_with(options)
                .await?;
            Ok(Self { w, r })
        }
        pub async fn seed(&self, n: usize, streams: usize) -> R<()> {
            sqlx::query("WITH RECURSIVE x(i) AS(SELECT 1 UNION ALL SELECT i+1 FROM x WHERE i<?) INSERT INTO events SELECT -i,i%?,-i,printf('%0256d',i) FROM x").bind(n as i64).bind(streams as i64).execute(&self.w).await?;
            Ok(())
        }
        pub async fn write(&self, b: &[Event]) -> R<()> {
            let mut tx = self.w.begin().await?;
            for e in b {
                sqlx::query("INSERT INTO events VALUES(?,?,?,?)")
                    .bind(e.id)
                    .bind(e.stream)
                    .bind(e.seq)
                    .bind(&e.payload)
                    .execute(&mut *tx)
                    .await?;
            }
            tx.commit().await?;
            Ok(())
        }
        pub async fn intent(&self, id: i64, w: usize) -> R<()> {
            let mut tx = self.w.begin().await?;
            sqlx::query(
                "UPDATE context SET version=version+1,completed=completed+1 WHERE workspace=?",
            )
            .bind(w as i64)
            .execute(&mut *tx)
            .await?;
            sqlx::query(
                "INSERT INTO intents SELECT ?,workspace,version FROM context WHERE workspace=?",
            )
            .bind(id)
            .bind(w as i64)
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE drafts SET revision=revision+1 WHERE workspace=?")
                .bind(w as i64)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            Ok(())
        }
        pub async fn read(&self, _idx: usize, kind: usize, stream: usize) -> R<()> {
            let mut tx = self.r.begin().await?;
            match kind {
                0 | 1 => {
                    let rows = sqlx::query(PAGE)
                        .bind(stream as i64)
                        .bind(if kind == 0 { i64::MAX } else { -10000 })
                        .fetch_all(&mut *tx)
                        .await?;
                    let materialized: Vec<(i64, i64, String)> = rows
                        .iter()
                        .map(|r| (r.get(0), r.get(1), r.get(2)))
                        .collect();
                    let bytes: usize = materialized.iter().map(|r| r.2.len()).sum();
                    assert!(bytes <= 262144);
                }
                2 => {
                    let rows = sqlx::query(
                        "SELECT workspace,version,completed FROM context ORDER BY workspace",
                    )
                    .fetch_all(&mut *tx)
                    .await?;
                    for r in rows {
                        let w: i64 = r.get(0);
                        let v: i64 = r.get(1);
                        let c: i64 = r.get(2);
                        let max:i64=sqlx::query_scalar("SELECT coalesce(max(context_version),0) FROM intents WHERE workspace=?").bind(w).fetch_one(&mut *tx).await?;
                        assert_eq!(v, c);
                        assert_eq!(v, max);
                    }
                }
                _ => {
                    let rows=sqlx::query("SELECT d.workspace,d.revision,d.external_identity,d.body,c.version FROM drafts d JOIN context c USING(workspace) ORDER BY d.workspace LIMIT 32").fetch_all(&mut *tx).await?;
                    let materialized: Vec<(i64, i64, String, String, i64)> = rows
                        .iter()
                        .map(|r| (r.get(0), r.get(1), r.get(2), r.get(3), r.get(4)))
                        .collect();
                    assert!(materialized.len() <= 32);
                }
            }
            tx.commit().await?;
            Ok(())
        }
        pub async fn intent_audit(&self) -> R<Vec<(i64, usize)>> {
            let rows = sqlx::query("SELECT id,workspace FROM intents ORDER BY id")
                .fetch_all(&self.w)
                .await?;
            Ok(rows
                .iter()
                .map(|r| (r.get(0), r.get::<i64, _>(1) as usize))
                .collect())
        }
        pub async fn verify(&self) -> R<(i64, i64)> {
            let r = sqlx::query(VERIFY).fetch_one(&self.w).await?;
            let invalid:i64=sqlx::query_scalar("SELECT count(*) FROM events e JOIN streams s ON s.id=e.stream WHERE e.id>0 AND (s.workspace!=s.id/25 OR s.root!=s.id/5*5 OR (s.id%5!=0 AND s.parent!=s.root) OR e.seq!=e.id)").fetch_one(&self.w).await?;
            assert_eq!(invalid, 0);
            Ok((r.get(0), r.get(1)))
        }
        pub async fn audit(&self) -> R<Vec<(i64, u64)>> {
            let rows=sqlx::query("SELECT e.id,e.stream,e.seq,s.workspace,s.root,s.parent,e.payload FROM events e JOIN streams s ON s.id=e.stream WHERE e.id>0 ORDER BY e.id").fetch_all(&self.w).await?;
            Ok(rows
                .iter()
                .map(|r| {
                    let id = r.get(0);
                    (
                        id,
                        record_hash(
                            id,
                            r.get(1),
                            r.get(2),
                            r.get(3),
                            r.get(4),
                            r.get(5),
                            r.get::<&str, _>(6),
                        ),
                    )
                })
                .collect())
        }
        pub async fn integrity(&self) -> R<String> {
            let s = sqlx::query_scalar("PRAGMA integrity_check")
                .fetch_one(&self.w)
                .await?;
            let fk = sqlx::query("PRAGMA foreign_key_check")
                .fetch_all(&self.w)
                .await?;
            assert!(fk.is_empty());
            Ok(s)
        }
        pub async fn checkpoint(&self, trunc: bool) -> R<Vec<i64>> {
            let s = if trunc {
                "PRAGMA wal_checkpoint(TRUNCATE)"
            } else {
                "PRAGMA wal_checkpoint(PASSIVE)"
            };
            let r = sqlx::query(AssertSqlSafe(s)).fetch_one(&self.w).await?;
            Ok(vec![r.get(0), r.get(1), r.get(2)])
        }
        pub async fn environment(&self) -> R<Value> {
            let row = sqlx::query("SELECT sqlite_version(),sqlite_source_id()")
                .fetch_one(&self.w)
                .await?;
            let options: Vec<String> = sqlx::query_scalar("PRAGMA compile_options")
                .fetch_all(&self.w)
                .await?;
            let mut pragmas = serde_json::Map::new();
            for p in [
                "journal_mode",
                "synchronous",
                "fullfsync",
                "foreign_keys",
                "busy_timeout",
                "cache_size",
                "wal_autocheckpoint",
            ] {
                let r = sqlx::query(AssertSqlSafe(format!("PRAGMA {p}")))
                    .fetch_one(&self.w)
                    .await?;
                pragmas.insert(
                    p.into(),
                    if p == "journal_mode" {
                        json!(r.get::<String, _>(0))
                    } else {
                        json!(r.get::<i64, _>(0))
                    },
                );
            }
            Ok(
                json!({"version":row.get::<String,_>(0),"sourceid":row.get::<String,_>(1),"compile_options":options,"pragmas":pragmas}),
            )
        }
        pub async fn future_probe(&self) -> R<Value> {
            self.intent(99999, 0).await?;
            self.read(0, 2, 0).await?;
            let r=sqlx::query("SELECT c.version,c.completed,d.revision,i.context_version,d.external_identity FROM context c JOIN drafts d USING(workspace) JOIN intents i USING(workspace) WHERE i.id=99999").fetch_one(&self.w).await?;
            let v: i64 = r.get(0);
            let c: i64 = r.get(1);
            let d: i64 = r.get(2);
            let iv: i64 = r.get(3);
            assert_eq!((v, c, d, iv), (1, 1, 1, 1));
            Ok(
                json!({"atomic_completion_next_intent_context_version":true,"consistent_context_snapshot":true,"draft_revision":d,"external_operation_identity":r.get::<String,_>(4)}),
            )
        }
        pub async fn busy_probe(&self, path: &Path) -> R<Value> {
            let p = path.with_extension("busy");
            let opts = SqliteConnectOptions::new()
                .filename(&p)
                .create_if_missing(true)
                .busy_timeout(Duration::from_millis(100))
                .pragma("journal_mode", "DELETE")
                .pragma("synchronous", "FULL");
            let a = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(opts.clone())
                .await?;
            let b = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(opts)
                .await?;
            sqlx::raw_sql("CREATE TABLE t(x);INSERT INTO t VALUES(1)")
                .execute(&a)
                .await?;
            let mut ar = a.acquire().await?;
            let mut br = b.acquire().await?;
            sqlx::raw_sql("BEGIN;SELECT * FROM t")
                .execute(&mut *br)
                .await?;
            sqlx::raw_sql("BEGIN IMMEDIATE;INSERT INTO t VALUES(2)")
                .execute(&mut *ar)
                .await?;
            let err = sqlx::raw_sql("COMMIT").execute(&mut *ar).await.unwrap_err();
            let code = err
                .as_database_error()
                .and_then(|e| e.code())
                .map(|x| x.to_string());
            assert_eq!(code.as_deref(), Some("5"));
            sqlx::raw_sql("ROLLBACK").execute(&mut *br).await?;
            sqlx::raw_sql("COMMIT").execute(&mut *ar).await?;
            let count: i64 = sqlx::query_scalar("SELECT count(*) FROM t")
                .fetch_one(&mut *ar)
                .await?;
            assert_eq!(count, 2);
            drop(ar);
            drop(br);
            a.close().await;
            b.close().await;
            Ok(
                json!({"busy_code":code,"same_transaction_commit_retry":true,"persisted_rows":count}),
            )
        }
        pub async fn private_probe(&self, path: &Path) -> R<Value> {
            let alias = path.with_extension("alias");
            std::os::unix::fs::symlink(path, &alias)?;
            let options = SqliteConnectOptions::new()
                .filename(&alias)
                .create_if_missing(false);
            let p = SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await;
            let opened = p.is_ok();
            if let Ok(p) = p {
                p.close().await
            }
            std::fs::remove_file(alias)?;
            Ok(
                json!({"native_nofollow_supported":false,"alias_opened_by_default":opened,"adoption_gate":"FAILED: supported API exposes no native SQLITE_OPEN_NOFOLLOW; no fork/VFS workaround attempted"}),
            )
        }
        pub async fn close(self) -> R<()> {
            self.r.close().await;
            self.w.close().await;
            Ok(())
        }
    }
}
#[cfg(not(feature = "sqlx-driver"))]
mod imp {
    use super::*;
    use rusqlite::{OpenFlags, params};
    use tokio_rusqlite::Connection;
    pub const NAME: &str = "tokio-rusqlite-0.9.0-rusqlite-0.42.0";
    #[derive(Clone)]
    pub struct Db {
        w: Connection,
        r: Vec<Connection>,
    }
    impl Db {
        pub async fn open(path: &Path, n: usize) -> R<Self> {
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_NOFOLLOW;
            let w = Connection::open_with_flags(path, flags).await?;
            w.call(move |c| {
                c.execute_batch(CONFIG)?;
                c.execute_batch(SCHEMA)?;
                let tx = c.transaction()?;
                for id in 0..n * 25 {
                    tx.execute(
                        "INSERT INTO streams VALUES(?,?,?,?)",
                        params![
                            id as i64,
                            (id / 25) as i64,
                            (id / 5 * 5) as i64,
                            if id % 5 == 0 {
                                None
                            } else {
                                Some((id / 5 * 5) as i64)
                            }
                        ],
                    )?;
                }
                for ws in 0..n {
                    tx.execute("INSERT INTO context VALUES(?,0,0)", [ws as i64])?;
                    tx.execute(
                        "INSERT INTO drafts VALUES(?,0,?,'human-edit')",
                        params![ws as i64, format!("local-external-operation-{ws}")],
                    )?;
                }
                tx.commit()
            })
            .await?;
            let mut r = vec![];
            for _ in 0..2 {
                let c = Connection::open_with_flags(path, flags).await?;
                c.call(|c| c.execute_batch(CONFIG)).await?;
                r.push(c)
            }
            Ok(Self { w, r })
        }
        pub async fn seed(&self, n: usize, streams: usize) -> R<()> {
            self.w.call(move|c|c.execute("WITH RECURSIVE x(i) AS(SELECT 1 UNION ALL SELECT i+1 FROM x WHERE i<?) INSERT INTO events SELECT -i,i%?,-i,printf('%0256d',i) FROM x",params![n as i64,streams as i64])).await?;
            Ok(())
        }
        pub async fn write(&self, b: &[Event]) -> R<()> {
            let b = b.to_vec();
            self.w
                .call(move |c| {
                    let tx = c.transaction()?;
                    {
                        let mut stmt = tx.prepare_cached("INSERT INTO events VALUES(?,?,?,?)")?;
                        for e in b {
                            stmt.execute(params![e.id, e.stream, e.seq, e.payload])?;
                        }
                    }
                    tx.commit()
                })
                .await?;
            Ok(())
        }
        pub async fn intent(&self, id: i64, w: usize) -> R<()> {
            self.w.call(move|c|{let tx=c.transaction()?;tx.execute("UPDATE context SET version=version+1,completed=completed+1 WHERE workspace=?",[w as i64])?;tx.execute("INSERT INTO intents SELECT ?,workspace,version FROM context WHERE workspace=?",params![id,w as i64])?;tx.execute("UPDATE drafts SET revision=revision+1 WHERE workspace=?",[w as i64])?;tx.commit()}).await?;
            Ok(())
        }
        pub async fn read(&self, idx: usize, kind: usize, stream: usize) -> R<()> {
            self.r[idx].call(move|c|{let tx=c.transaction()?;match kind{0|1=>{let mut s=tx.prepare_cached(PAGE)?;let rows=s.query_map(params![stream as i64,if kind==0{i64::MAX}else{-10000}],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?)))?;let materialized=rows.collect::<Result<Vec<_>,_>>()?;let bytes:usize=materialized.iter().map(|r|r.2.len()).sum();assert!(bytes<=262144);},2=>{let mut s=tx.prepare_cached("SELECT workspace,version,completed FROM context ORDER BY workspace")?;let rows=s.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?)))?;for row in rows{let(w,v,c)=row?;let max:i64=tx.query_row("SELECT coalesce(max(context_version),0) FROM intents WHERE workspace=?",[w],|r|r.get(0))?;assert_eq!(v,c);assert_eq!(v,max)}},_=>{let mut s=tx.prepare_cached("SELECT d.workspace,d.revision,d.external_identity,d.body,c.version FROM drafts d JOIN context c USING(workspace) ORDER BY d.workspace LIMIT 32")?;let rows=s.query_map([],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?,r.get::<_,i64>(4)?)))?;let materialized=rows.collect::<Result<Vec<_>,_>>()?;assert!(materialized.len()<=32);}}tx.commit()}).await?;
            Ok(())
        }
        pub async fn intent_audit(&self) -> R<Vec<(i64, usize)>> {
            Ok(self
                .w
                .call(|c| {
                    c.prepare("SELECT id,workspace FROM intents ORDER BY id")?
                        .query_map([], |r| Ok((r.get(0)?, r.get::<_, i64>(1)? as usize)))?
                        .collect::<Result<Vec<_>, _>>()
                })
                .await?)
        }
        pub async fn verify(&self) -> R<(i64, i64)> {
            Ok(self.w.call(|c|{let invalid:i64=c.query_row("SELECT count(*) FROM events e JOIN streams s ON s.id=e.stream WHERE e.id>0 AND (s.workspace!=s.id/25 OR s.root!=s.id/5*5 OR (s.id%5!=0 AND s.parent!=s.root) OR e.seq!=e.id)",[],|r|r.get(0))?;assert_eq!(invalid,0);c.query_row(VERIFY,[],|r|Ok((r.get(0)?,r.get(1)?)))}).await?)
        }
        pub async fn audit(&self) -> R<Vec<(i64, u64)>> {
            Ok(self.w.call(|c|{c.prepare("SELECT e.id,e.stream,e.seq,s.workspace,s.root,s.parent,e.payload FROM events e JOIN streams s ON s.id=e.stream WHERE e.id>0 ORDER BY e.id")?.query_map([],|r|{let id=r.get(0)?;let p:String=r.get(6)?;Ok((id,record_hash(id,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?,&p)))})?.collect::<Result<Vec<_>,_>>()}).await?)
        }
        pub async fn integrity(&self) -> R<String> {
            Ok(self
                .w
                .call(|c| {
                    let n = c
                        .prepare("PRAGMA foreign_key_check")?
                        .query_map([], |_| Ok(()))?
                        .count();
                    assert_eq!(n, 0);
                    c.query_row("PRAGMA integrity_check", [], |r| r.get(0))
                })
                .await?)
        }
        pub async fn checkpoint(&self, trunc: bool) -> R<Vec<i64>> {
            Ok(self
                .w
                .call(move |c| {
                    c.query_row(
                        if trunc {
                            "PRAGMA wal_checkpoint(TRUNCATE)"
                        } else {
                            "PRAGMA wal_checkpoint(PASSIVE)"
                        },
                        [],
                        |r| Ok(vec![r.get(0)?, r.get(1)?, r.get(2)?]),
                    )
                })
                .await?)
        }
        pub async fn environment(&self) -> R<Value> {
            Ok(self.w.call(|c|{let(v,s):(String,String)=c.query_row("SELECT sqlite_version(),sqlite_source_id()",[],|r|Ok((r.get(0)?,r.get(1)?)))?;let options=c.prepare("PRAGMA compile_options")?.query_map([],|r|r.get::<_,String>(0))?.collect::<Result<Vec<_>,_>>()?;let mut pragmas=serde_json::Map::new();for p in ["journal_mode","synchronous","fullfsync","foreign_keys","busy_timeout","cache_size","wal_autocheckpoint"]{let x=if p=="journal_mode"{json!(c.query_row(&format!("PRAGMA {p}"),[],|r|r.get::<_,String>(0))?)}else{json!(c.query_row(&format!("PRAGMA {p}"),[],|r|r.get::<_,i64>(0))?)};pragmas.insert(p.into(),x);}Ok::<_,rusqlite::Error>(json!({"version":v,"sourceid":s,"compile_options":options,"pragmas":pragmas}))}).await?)
        }
        pub async fn future_probe(&self) -> R<Value> {
            self.intent(99999, 0).await?;
            self.read(0, 2, 0).await?;
            Ok(self.w.call(|c|{let(v,co,d,iv,s):(i64,i64,i64,i64,String)=c.query_row("SELECT c.version,c.completed,d.revision,i.context_version,d.external_identity FROM context c JOIN drafts d USING(workspace) JOIN intents i USING(workspace) WHERE i.id=99999",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?)))?;assert_eq!((v,co,d,iv),(1,1,1,1));Ok::<_,rusqlite::Error>(json!({"atomic_completion_next_intent_context_version":true,"consistent_context_snapshot":true,"draft_revision":d,"external_operation_identity":s}))}).await?)
        }
        pub async fn busy_probe(&self, path: &Path) -> R<Value> {
            let p = path.with_extension("busy");
            let a = Connection::open(&p).await?;
            let b = Connection::open(&p).await?;
            a.call(|c|c.execute_batch("PRAGMA journal_mode=DELETE;PRAGMA synchronous=FULL;PRAGMA busy_timeout=100;CREATE TABLE t(x);INSERT INTO t VALUES(1)")).await?;
            b.call(|c| c.execute_batch("BEGIN;SELECT * FROM t")).await?;
            a.call(|c| c.execute_batch("BEGIN IMMEDIATE;INSERT INTO t VALUES(2)"))
                .await?;
            let busy = a
                .call(|c| {
                    let e = c.execute_batch("COMMIT").unwrap_err();
                    assert_eq!(
                        e.sqlite_error_code(),
                        Some(rusqlite::ErrorCode::DatabaseBusy)
                    );
                    Ok::<_, rusqlite::Error>(!c.is_autocommit())
                })
                .await?;
            assert!(busy);
            b.call(|c| c.execute_batch("ROLLBACK")).await?;
            a.call(|c| c.execute_batch("COMMIT")).await?;
            let count: i64 = a
                .call(|c| c.query_row("SELECT count(*) FROM t", [], |r| r.get(0)))
                .await?;
            assert_eq!(count, 2);
            a.close().await?;
            b.close().await?;
            Ok(
                json!({"busy_code":5,"transaction_retained_after_error":busy,"same_transaction_commit_retry":true,"persisted_rows":count}),
            )
        }
        pub async fn private_probe(&self, path: &Path) -> R<Value> {
            let alias = path.with_extension("alias");
            std::os::unix::fs::symlink(path, &alias)?;
            let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NOFOLLOW;
            let opened = Connection::open_with_flags(&alias, flags).await;
            let rejected = opened.is_err();
            if let Ok(c) = opened {
                c.close().await?
            }
            std::fs::remove_file(alias)?;
            assert!(rejected);
            Ok(
                json!({"native_nofollow_supported":true,"alias_rejected":rejected,"adoption_gate":"passed real alias probe; not complete filesystem security qualification"}),
            )
        }
        pub async fn close(self) -> R<()> {
            for c in self.r {
                c.close().await?
            }
            self.w.close().await?;
            Ok(())
        }
    }
}
pub use imp::*;
