#!/usr/bin/env python3
"""Disposable SQLite qualification probes; not application units or app qualification.

Usage: python3 storage-probes.py --library /private/path/libsqlite3.dylib
The supplied library must be built from the recorded supported amalgamation.
All databases/files are generated beneath a fresh mode-0700 temporary directory.
"""
import argparse
import ctypes as C
import fcntl
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import statistics
import stat
import subprocess
import sys
import tempfile
import threading
import time

LIB = None
CALLBACK = C.CFUNCTYPE(C.c_int, C.c_void_p, C.c_int, C.POINTER(C.c_char_p), C.POINTER(C.c_char_p))


def load(path):
    global LIB
    LIB = C.CDLL(path)
    LIB.sqlite3_open_v2.argtypes = [C.c_char_p, C.POINTER(C.c_void_p), C.c_int, C.c_char_p]
    LIB.sqlite3_close.argtypes = [C.c_void_p]
    LIB.sqlite3_exec.argtypes = [C.c_void_p, C.c_char_p, CALLBACK, C.c_void_p, C.POINTER(C.c_char_p)]
    LIB.sqlite3_free.argtypes = [C.c_void_p]
    LIB.sqlite3_get_autocommit.argtypes = [C.c_void_p]
    LIB.sqlite3_errmsg.argtypes = [C.c_void_p]
    LIB.sqlite3_errmsg.restype = C.c_char_p
    LIB.sqlite3_prepare_v2.argtypes = [C.c_void_p, C.c_char_p, C.c_int, C.POINTER(C.c_void_p), C.POINTER(C.c_char_p)]
    LIB.sqlite3_bind_int.argtypes = [C.c_void_p, C.c_int, C.c_int]
    LIB.sqlite3_bind_blob.argtypes = [C.c_void_p, C.c_int, C.c_void_p, C.c_int, C.c_void_p]
    LIB.sqlite3_bind_text.argtypes = [C.c_void_p, C.c_int, C.c_char_p, C.c_int, C.c_void_p]
    LIB.sqlite3_step.argtypes = [C.c_void_p]
    LIB.sqlite3_finalize.argtypes = [C.c_void_p]
    LIB.sqlite3_column_blob.argtypes = [C.c_void_p, C.c_int]
    LIB.sqlite3_column_blob.restype = C.c_void_p
    LIB.sqlite3_column_bytes.argtypes = [C.c_void_p, C.c_int]
    LIB.sqlite3_column_text.argtypes = [C.c_void_p, C.c_int]
    LIB.sqlite3_column_text.restype = C.c_char_p


def literal(value):
    return "'" + str(value).replace("'", "''") + "'"


class DB:
    def __init__(self, path, journal="WAL", sync="FULL", auto=1000):
        self.handle = C.c_void_p()
        code = LIB.sqlite3_open_v2(os.fsencode(path), C.byref(self.handle), 2 | 4 | 0x01000000, None)
        if code:
            raise RuntimeError(f"open: {code}")
        self.sql(f"PRAGMA busy_timeout=100; PRAGMA journal_mode={journal}; PRAGMA synchronous={sync}; PRAGMA fullfsync=ON; PRAGMA foreign_keys=ON; PRAGMA wal_autocheckpoint={auto}")

    def sql(self, sql, allow_error=False):
        rows = []

        @CALLBACK
        def receive(_, count, values, names):
            rows.append([values[i].decode() if values[i] is not None else None for i in range(count)])
            return 0

        error = C.c_char_p()
        code = LIB.sqlite3_exec(self.handle, sql.encode(), receive, None, C.byref(error))
        detail = error.value.decode() if error.value else None
        if error:
            LIB.sqlite3_free(error)
        if code and not allow_error:
            raise RuntimeError(f"sqlite {code}: {detail}; SQL: {sql[:160]}")
        return {"code": code, "error": detail, "rows": rows} if allow_error else rows

    def close(self):
        if self.handle:
            code = LIB.sqlite3_close(self.handle)
            if code:
                raise RuntimeError(f"close: {code}")
            self.handle = None

    def scalar(self, sql):
        return self.sql(sql)[0][0]

    def insert_blob(self, index, payload, digest):
        statement = C.c_void_p()
        sql = b"INSERT INTO payloads VALUES(?1,?2,NULL,?3)"
        if LIB.sqlite3_prepare_v2(self.handle, sql, -1, C.byref(statement), None):
            raise RuntimeError(LIB.sqlite3_errmsg(self.handle).decode())
        try:
            # Inputs are bounded by this experiment's 4MiB maximum and 18 rows.
            assert 0 <= index < 18 and len(payload) <= 4 * 1024 * 1024
            buffer = C.create_string_buffer(payload)
            codes = [LIB.sqlite3_bind_int(statement, 1, index), LIB.sqlite3_bind_blob(statement, 2, buffer, len(payload), C.c_void_p(-1)), LIB.sqlite3_bind_text(statement, 3, digest.encode(), -1, C.c_void_p(-1))]
            if any(codes) or LIB.sqlite3_step(statement) != 101:
                raise RuntimeError(LIB.sqlite3_errmsg(self.handle).decode())
        finally:
            if LIB.sqlite3_finalize(statement):
                raise RuntimeError("finalize failed")

    def verify_blobs(self):
        statement = C.c_void_p()
        if LIB.sqlite3_prepare_v2(self.handle, b"SELECT body,digest FROM payloads", -1, C.byref(statement), None):
            raise RuntimeError(LIB.sqlite3_errmsg(self.handle).decode())
        total = 0
        try:
            while True:
                code = LIB.sqlite3_step(statement)
                if code == 101:
                    return total
                if code != 100:
                    raise RuntimeError(LIB.sqlite3_errmsg(self.handle).decode())
                size = LIB.sqlite3_column_bytes(statement, 0)
                payload = C.string_at(LIB.sqlite3_column_blob(statement, 0), size)
                assert hashlib.sha256(payload).hexdigest() == LIB.sqlite3_column_text(statement, 1).decode()
                total += size
        finally:
            if LIB.sqlite3_finalize(statement):
                raise RuntimeError("finalize failed")


SCHEMA = """
CREATE TABLE actions(id INTEGER PRIMARY KEY, state TEXT NOT NULL);
CREATE TABLE workspaces(id INTEGER PRIMARY KEY, action INTEGER NOT NULL UNIQUE REFERENCES actions(id), name TEXT UNIQUE NOT NULL);
CREATE TABLE attachments(id INTEGER PRIMARY KEY, path TEXT NOT NULL, digest TEXT NOT NULL);
CREATE TABLE workflow(id INTEGER PRIMARY KEY, state TEXT NOT NULL, version INTEGER NOT NULL);
CREATE TABLE continuations(parent INTEGER PRIMARY KEY REFERENCES workflow(id), context_version INTEGER NOT NULL);
CREATE TABLE events(seq INTEGER PRIMARY KEY, conversation INTEGER NOT NULL, body BLOB NOT NULL);
CREATE INDEX history ON events(conversation,seq);
"""


def sync_file(handle):
    handle.flush()
    os.fsync(handle.fileno())
    if sys.platform == "darwin":
        fcntl.fcntl(handle.fileno(), 51)  # macOS F_FULLFSYNC; report failures.


def sync_directory(path):
    handle = os.open(path, os.O_RDONLY)
    try:
        os.fsync(handle)
    finally:
        os.close(handle)


def stop_at(stage):
    print(json.dumps({"ready": stage}), flush=True)
    while True:
        signal.pause()


def child(args):
    directory = Path(args.child)
    db = DB(directory / "main.db", args.journal, args.sync, auto=0)
    if args.kind == "transaction":
        db.sql("BEGIN IMMEDIATE")
        if args.stage == "after_begin":
            stop_at(args.stage)
        db.sql("INSERT INTO actions VALUES(1,'done'); INSERT INTO workspaces VALUES(1,1,'reserved'); INSERT INTO workflow VALUES(1,'done',7); INSERT INTO continuations VALUES(1,7)")
        if args.stage == "after_records":
            stop_at(args.stage)
        db.sql("COMMIT")
        stop_at(args.stage)  # committed, reply not yet consumed by caller
    elif args.kind == "attachment":
        payload = b"synthetic screenshot\n" * 16384
        temporary, final = directory / "upload.tmp", directory / "upload.bin"
        with temporary.open("wb") as handle:
            handle.write(payload)
            sync_file(handle)
        if args.stage == "after_temp_sync":
            stop_at(args.stage)
        os.rename(temporary, final)
        sync_directory(directory)
        if args.stage == "after_rename":
            stop_at(args.stage)
        digest = hashlib.sha256(payload).hexdigest()
        db.sql(f"INSERT INTO attachments VALUES(1,'upload.bin',{literal(digest)})")
        stop_at(args.stage)
    elif args.kind in ["checkpoint", "backup", "shutdown"]:
        db.sql("BEGIN IMMEDIATE;" + ";".join(f"INSERT INTO events VALUES({index},0,zeroblob(32768))" for index in range(128)) + ";COMMIT")
        if args.stage == "before_operation":
            stop_at(args.stage)
        if args.kind == "checkpoint":
            assert db.sql("PRAGMA wal_checkpoint(TRUNCATE)") == [["0", "0", "0"]]
        elif args.kind == "backup":
            db.sql(f"VACUUM INTO {literal(directory / 'backup.db')}")
        else:
            db.close()
        stop_at(args.stage)


def killed_probe(root, library, journal, sync, kind, stage):
    directory = root / f"kill-{journal}-{sync}-{kind}-{stage}"
    directory.mkdir(mode=0o700)
    db = DB(directory / "main.db", journal, sync)
    db.sql(SCHEMA)
    db.close()
    command = [sys.executable, str(Path(__file__).resolve()), "--library", library, "--child", str(directory), "--journal", journal, "--sync", sync, "--kind", kind, "--stage", stage]
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    try:
        if not select.select([process.stdout], [], [], 20)[0]:
            raise RuntimeError("child did not reach kill point within 20s")
        marker = process.stdout.readline()
        if json.loads(marker) != {"ready": stage}:
            raise RuntimeError(f"unexpected marker: {marker}")
        process.kill()
        _, stderr = process.communicate(timeout=20)
        assert process.returncode == -signal.SIGKILL, (process.returncode, stderr)
    finally:
        if process.poll() is None:
            process.kill()
            process.communicate(timeout=20)
    db = DB(directory / "main.db", journal, sync)
    assert db.scalar("PRAGMA integrity_check") == "ok"
    if kind == "transaction":
        expected = int(stage == "after_commit")
        counts = [int(db.scalar(f"SELECT count(*) FROM {table}")) for table in ["actions", "workspaces", "workflow", "continuations"]]
        assert counts == [expected] * 4, counts
        if expected:
            collision = db.sql("INSERT INTO workspaces VALUES(2,1,'reserved')", allow_error=True)
            assert collision["code"] == 19, collision
        result = {"counts": counts, "committed_after_lost_reply": bool(expected)}
    elif kind == "attachment":
        rows = db.sql("SELECT path,digest FROM attachments")
        expected = stage == "after_commit"
        assert bool(rows) == expected
        for path, digest in rows:
            assert hashlib.sha256((directory / path).read_bytes()).hexdigest() == digest
        result = {"metadata_present": bool(rows), "final_present": (directory / "upload.bin").exists(), "temp_present": (directory / "upload.tmp").exists()}
    else:
        assert db.scalar("SELECT count(*) FROM events") == "128"
        backup_valid = None
        if kind == "backup":
            if stage == "after_operation":
                backup = DB(directory / "backup.db")
                backup_valid = backup.scalar("PRAGMA integrity_check") == "ok" and backup.scalar("SELECT count(*) FROM events") == "128"
                assert backup_valid
                backup.close()
            else:
                assert not (directory / "backup.db").exists()
        result = {"committed_events": 128, "backup_valid": backup_valid, "limit": "kill at before/after operation markers; not an in-VFS interrupted-operation or power-loss test"}
    result.update({"journal": journal, "sync": sync, "kind": kind, "kill_point": stage, "pass": True})
    db.close()
    return result


def contention(root):
    path = root / "busy.db"
    first, second = DB(path, "DELETE", "EXTRA"), DB(path, "DELETE", "EXTRA")
    first.sql("CREATE TABLE value(n); INSERT INTO value VALUES(0); BEGIN; SELECT * FROM value")
    second.sql("BEGIN IMMEDIATE; UPDATE value SET n=1")
    start = time.perf_counter()
    commit = second.sql("COMMIT", allow_error=True)
    elapsed = (time.perf_counter() - start) * 1000
    active = not bool(LIB.sqlite3_get_autocommit(second.handle))
    assert commit["code"] == 5 and active, (commit, active)
    second.sql("ROLLBACK")
    assert second.scalar("SELECT n FROM value") == "0"
    first.sql("COMMIT")
    first.close()
    second.close()
    return {"commit_code": commit["code"], "active_after_busy_commit": active, "wait_ms": elapsed, "committed_read_after_rollback": 0}


def checkpoint_backup(root):
    path = root / "checkpoint.db"
    writer = DB(path, auto=0)
    writer.sql("CREATE TABLE body(id INTEGER PRIMARY KEY, value BLOB); CREATE TABLE markers(id INTEGER PRIMARY KEY); INSERT INTO body VALUES(0,zeroblob(32768)); INSERT INTO markers VALUES(0)")
    reader = DB(path, auto=0)
    reader.sql("BEGIN; SELECT count(*) FROM body")
    for index in range(1, 161):
        writer.sql(f"BEGIN IMMEDIATE; INSERT INTO body VALUES({index},zeroblob(32768)); INSERT INTO markers VALUES({index}); COMMIT")
    start = time.perf_counter()
    passive = writer.sql("PRAGMA wal_checkpoint(PASSIVE)")
    passive_ms = (time.perf_counter() - start) * 1000
    peak = Path(str(path) + "-wal").stat().st_size
    assert int(passive[0][1]) > int(passive[0][2]), passive
    reader.sql("COMMIT")
    start = time.perf_counter()
    truncate = writer.sql("PRAGMA wal_checkpoint(TRUNCATE)")
    truncate_ms = (time.perf_counter() - start) * 1000
    assert truncate == [["0", "0", "0"]], truncate
    assert Path(str(path) + "-wal").stat().st_size == 0
    reader.close()
    failures, samples = [], []
    started = threading.Event()
    backup_started = threading.Event()

    def append():
        connection = None
        try:
            connection = DB(path, auto=0)
            started.set()
            if not backup_started.wait(20):
                raise RuntimeError("backup start timed out")
            for index in range(161, 201):
                begin = time.perf_counter()
                connection.sql(f"BEGIN IMMEDIATE; INSERT INTO body VALUES({index},zeroblob(32768)); INSERT INTO markers VALUES({index}); COMMIT")
                samples.append((begin, time.perf_counter()))
                time.sleep(0.002)
        except Exception as error:
            failures.append(str(error))
            started.set()
        finally:
            if connection:
                connection.close()

    thread = threading.Thread(target=append)
    thread.start()
    assert started.wait(20)
    backup = root / "backup.db"
    start = time.perf_counter()
    backup_started.set()
    try:
        writer.sql(f"VACUUM INTO {literal(backup)}")
        backup_end = time.perf_counter()
        backup_ms = (backup_end - start) * 1000
    finally:
        thread.join(20)
    assert not thread.is_alive() and not failures, failures
    restored = DB(backup)
    count = int(restored.scalar("SELECT count(*) FROM body"))
    assert count == int(restored.scalar("SELECT count(*) FROM markers"))
    assert restored.scalar("PRAGMA integrity_check") == "ok"
    restored.close()
    writer.close()
    overlapping = [(end - begin) * 1000 for begin, end in samples if begin < backup_end and end > start]
    return {"passive": passive, "passive_ms": passive_ms, "pinned_wal_bytes": peak, "truncate": truncate, "truncate_ms": truncate_ms, "vacuum_into_ms": backup_ms, "backup_rows": count, "final_rows": 201, "foreground_commit_ms_all": distribution([(end - begin) * 1000 for begin, end in samples]), "commits_overlapping_backup": len(overlapping), "foreground_commit_ms_overlapping_backup": distribution(overlapping) if overlapping else None, "backup_consistent": True}


def workflow_consistency(root):
    path = root / "workflow.db"
    writer = DB(path)
    writer.sql("CREATE TABLE context(version INTEGER PRIMARY KEY,body TEXT); CREATE TABLE workflow(id INTEGER PRIMARY KEY,state TEXT,version INTEGER REFERENCES context(version)); CREATE TABLE continuation(parent INTEGER PRIMARY KEY REFERENCES workflow(id),context_version INTEGER REFERENCES context(version)); CREATE TABLE external_operations(id TEXT PRIMARY KEY,state TEXT); INSERT INTO context VALUES(1,'earlier'); INSERT INTO workflow VALUES(1,'running',1)")
    reader = DB(path)
    reader.sql("BEGIN")
    before = reader.sql("SELECT state,version,body FROM workflow JOIN context USING(version)")
    writer.sql("BEGIN IMMEDIATE; INSERT INTO context VALUES(2,'completed context'); UPDATE workflow SET state='done',version=2 WHERE id=1; INSERT INTO continuation VALUES(1,2); INSERT INTO external_operations VALUES('synthetic-review-1','pending'); COMMIT")
    during = reader.sql("SELECT state,version,body FROM workflow JOIN context USING(version)")
    assert before == during == [["running", "1", "earlier"]]
    assert reader.scalar("SELECT count(*) FROM continuation") == "0"
    reader.sql("COMMIT")
    after = reader.sql("SELECT state,version,body FROM workflow JOIN context USING(version)")
    assert after == [["done", "2", "completed context"]]
    assert reader.sql("SELECT context_version FROM continuation") == [["2"]]
    assert reader.sql("SELECT id,state FROM external_operations") == [["synthetic-review-1", "pending"]]
    reader.close()
    writer.close()
    return {"consistent_snapshot_during_completion": True, "completion_continuation_context_and_external_intent_committed_together": True, "external_effects_performed": 0, "limit": "SQLite primitives on synthetic records, not an implemented coordinator or remote idempotency policy"}


def distribution(values):
    ordered = sorted(values)
    return {"median": statistics.median(ordered), "min": min(ordered), "max": max(ordered), "p95": ordered[min(len(ordered) - 1, int(len(ordered) * .95))]}


def layout_run(root, run, mode):
    directory = root / f"layout-{mode}-{run}"
    directory.mkdir(mode=0o700)
    db = DB(directory / "main.db")
    db.sql("CREATE TABLE payloads(id INTEGER PRIMARY KEY, body BLOB, path TEXT, digest TEXT)")
    payloads = [b"s" * (256 * 1024)] * 16 + [b"l" * (4 * 1024 * 1024)] * 2
    start = time.perf_counter()
    for index, payload in enumerate(payloads):
        digest = hashlib.sha256(payload).hexdigest()
        if mode == "blob":
            db.insert_blob(index, payload, digest)
        else:
            path = directory / f"payload-{index}.bin"
            with path.with_suffix(".tmp").open("wb") as handle:
                handle.write(payload)
                sync_file(handle)
            os.rename(path.with_suffix(".tmp"), path)
            sync_directory(directory)
            db.sql(f"INSERT INTO payloads VALUES({index},NULL,{literal(path.name)},{literal(digest)})")
    write_ms = (time.perf_counter() - start) * 1000
    start = time.perf_counter()
    if mode == "blob":
        # Native length scan; avoids hex/text transfer being mistaken for payload read.
        assert int(db.scalar("SELECT sum(length(body)) FROM payloads")) == 12 * 1024 * 1024
    else:
        assert sum((directory / row[0]).stat().st_size for row in db.sql("SELECT path FROM payloads")) == 12 * 1024 * 1024
    metadata_ms = (time.perf_counter() - start) * 1000
    start = time.perf_counter()
    if mode == "blob":
        assert db.verify_blobs() == 12 * 1024 * 1024
    else:
        for path, digest in db.sql("SELECT path,digest FROM payloads"):
            assert hashlib.sha256((directory / path).read_bytes()).hexdigest() == digest
    read_ms = (time.perf_counter() - start) * 1000
    db.sql("PRAGMA wal_checkpoint(TRUNCATE)")
    db.close()
    disk = sum(p.stat().st_size for p in directory.iterdir() if p.is_file())
    return {"mode": mode, "run": run, "warmup": run == 0, "write_ms": write_ms, "metadata_scan_ms": metadata_ms, "payload_read_verify_ms": read_ms, "bytes": disk, "payload_bytes": 12 * 1024 * 1024}


def cache_run(root, run, split):
    directory = root / f"cache-{split}-{run}"
    directory.mkdir(mode=0o700)
    authoritative = DB(directory / "main.db")
    authoritative.sql("CREATE TABLE actions(id INTEGER PRIMARY KEY, version INTEGER); INSERT INTO actions VALUES(1,0)")
    cache = DB(directory / "cache.db") if split else authoritative
    cache.sql("CREATE TABLE diffs(id INTEGER PRIMARY KEY, body BLOB)")
    start = time.perf_counter()
    for batch in range(10):
        statements = ";".join(f"INSERT INTO diffs VALUES({batch * 100 + i},zeroblob(4096))" for i in range(100))
        cache.sql("BEGIN IMMEDIATE;" + statements + ";COMMIT")
        authoritative.sql("UPDATE actions SET version=version+1 WHERE id=1")
    elapsed = (time.perf_counter() - start) * 1000
    cache_bytes = int(cache.scalar("PRAGMA page_count")) * int(cache.scalar("PRAGMA page_size"))
    assert authoritative.scalar("SELECT version FROM actions WHERE id=1") == "10"
    if split:
        cache.close()
        for suffix in ["", "-wal", "-shm"]:
            (directory / ("cache.db" + suffix)).unlink(missing_ok=True)
        assert authoritative.scalar("SELECT version FROM actions WHERE id=1") == "10"
    else:
        authoritative.sql("DROP TABLE diffs")
    authoritative.close()
    return {"split": split, "run": run, "warmup": run == 0, "rebuild_ms": elapsed, "database_pages_bytes": cache_bytes, "authoritative_record_survived_discard": True}


def transcript_run(root, run, mode):
    directory = root / f"transcript-{mode}-{run}"
    directory.mkdir(mode=0o700)
    db = DB(directory / "main.db")
    db.sql("CREATE TABLE events(seq INTEGER PRIMARY KEY,body TEXT); CREATE TABLE batches(first_seq INTEGER PRIMARY KEY,last_seq INTEGER,first_byte INTEGER,last_byte INTEGER); CREATE TABLE turns(id INTEGER PRIMARY KEY,accepted_seq INTEGER); INSERT INTO turns VALUES(1,-1)")
    bodies = ["t" * (256 if i % 10 else 2048) for i in range(1024)]
    handle = (directory / "transcript.jsonl").open("w+b") if mode == "jsonl" else None
    start = time.perf_counter()
    try:
        for first in range(0, 1024, 16):
            if handle:
                first_byte = handle.tell()
                for seq in range(first, first + 16):
                    handle.write((json.dumps({"seq": seq, "body": bodies[seq]}, separators=(",", ":")) + "\n").encode())
                sync_file(handle)
                last_byte = handle.tell()
                db.sql(f"BEGIN IMMEDIATE; INSERT INTO batches VALUES({first},{first+15},{first_byte},{last_byte}); UPDATE turns SET accepted_seq={first+15} WHERE id=1; COMMIT")
            else:
                db.sql("BEGIN IMMEDIATE;" + ";".join(f"INSERT INTO events VALUES({seq},{literal(bodies[seq])})" for seq in range(first, first + 16)) + f";UPDATE turns SET accepted_seq={first+15} WHERE id=1;COMMIT")
        write_ms = (time.perf_counter() - start) * 1000
        start = time.perf_counter()
        lengths = []
        for cursor in [1023, 511, 127]:
            if handle:
                offsets = db.sql(f"SELECT first_byte,last_byte FROM batches WHERE first_seq<={cursor} ORDER BY first_seq DESC LIMIT 8")
                rows = []
                for first_byte, last_byte in reversed(offsets):
                    handle.seek(int(first_byte))
                    rows.extend(json.loads(line) for line in handle.read(int(last_byte)-int(first_byte)).splitlines())
                page = sorted((row for row in rows if row["seq"] <= cursor), key=lambda row: row["seq"], reverse=True)[:100]
                actual = [(row["seq"], row["body"]) for row in page]
            else:
                actual = [(int(seq), body) for seq, body in db.sql(f"SELECT seq,body FROM events WHERE seq<={cursor} ORDER BY seq DESC LIMIT 100")]
            assert actual == [(seq, bodies[seq]) for seq in range(cursor, cursor-100, -1)]
            lengths.append(sum(len(body) for _, body in actual))
        read_ms = (time.perf_counter() - start) * 1000
        assert db.scalar("SELECT accepted_seq FROM turns") == "1023"
    finally:
        if handle:
            handle.close()
        db.close()
    return {"mode": mode, "run": run, "warmup": run == 0, "events": 1024, "batch": 16, "write_ms": write_ms, "three_pages_ms": read_ms, "three_page_bytes": lengths, "bytes": sum(p.stat().st_size for p in directory.iterdir() if p.is_file()), "limits": "short warm history, ctypes/SQL text and JSON decode costs included; file directory first-publication sync not timed; no global layout winner established"}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--library", required=True)
    parser.add_argument("--child")
    parser.add_argument("--journal", default="WAL", choices=["WAL", "DELETE"])
    parser.add_argument("--sync", default="FULL", choices=["FULL", "EXTRA", "NORMAL"])
    parser.add_argument("--kind", choices=["transaction", "attachment", "checkpoint", "backup", "shutdown"])
    parser.add_argument("--stage")
    parser.add_argument("--maintenance-only", action="store_true")
    args = parser.parse_args()
    os.umask(0o077)
    load(args.library)
    if args.child:
        child(args)
        return
    with tempfile.TemporaryDirectory(prefix="storage-probes-06a-") as temporary:
        # macOS TMPDIR may start with /var, an alias for /private/var. Canonicalize
        # the newly allocated private root before SQLite's NOFOLLOW open.
        root = Path(temporary).resolve()
        db = DB(root / "version.db")
        version = db.sql("SELECT sqlite_version(),sqlite_source_id()")
        options = db.sql("PRAGMA compile_options")
        settings = {key: db.sql(f"PRAGMA {key}") for key in ["journal_mode", "synchronous", "fullfsync", "foreign_keys", "busy_timeout", "page_size"]}
        db.close()
        output = {"classification": "real disposable SQLite qualification, not application units", "version": version, "compile_options": options, "settings": settings, "manifest": {"kill_timeout_seconds": 20, "layout_warmups": 1, "layout_measured_runs": 5, "attachment_sizes": [262144, 4194304], "attachment_counts": [16, 2], "cache_diff_count": 1000, "cache_diff_bytes": 4096, "cache_batch": 100, "temporary_data_budget_bytes": 1073741824, "scope": "single SQLite3.54.1 native library via Python ctypes; serial runs; warm OS cache; cache churn sequential, not concurrency evidence"}}
        output["crashes"] = []
        if args.maintenance_only:
            for kind in ["checkpoint", "backup", "shutdown"]:
                for stage in ["before_operation", "after_operation"]:
                    output["crashes"].append(killed_probe(root, args.library, "WAL", "FULL", kind, stage))
            print(json.dumps(output, indent=2))
            return
        for journal, sync in [("DELETE", "FULL"), ("DELETE", "EXTRA"), ("WAL", "FULL")]:
            for kind, stages in [("transaction", ["after_begin", "after_records", "after_commit"]), ("attachment", ["after_temp_sync", "after_rename", "after_commit"])]:
                for stage in stages:
                    output["crashes"].append(killed_probe(root, args.library, journal, sync, kind, stage))
        output["busy_commit"] = contention(root)
        output["checkpoint_backup"] = checkpoint_backup(root)
        output["workflow_consistency"] = workflow_consistency(root)
        output["layouts"] = [layout_run(root, run, mode) for run in range(6) for mode in ["blob", "file"]]
        output["cache"] = [cache_run(root, run, split) for run in range(6) for split in [False, True]]
        output["transcripts"] = [transcript_run(root, run, mode) for run in range(6) for mode in ["sqlite", "jsonl"]]
        disk = sum(p.stat().st_size for p in root.rglob("*") if p.is_file())
        assert disk <= output["manifest"]["temporary_data_budget_bytes"], disk
        output["peak_retained_probe_bytes"] = disk
        files = [p for p in root.rglob("*") if p.is_file()]
        assert all(not p.is_symlink() and p.stat().st_nlink == 1 and stat.S_IMODE(p.stat().st_mode) == 0o600 for p in files)
        output["private_regular_single_link_files"] = len(files)
        print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
