"""Opt-in private Git/SQLite/process qualification. No provider invocation.

Usage: python3 qualify_paths.py <Cargo-reported-supervisor> <result.json>
Only the retained Popen handles are terminated; disposable state remains as evidence.
"""
import hashlib
import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
import uuid

binary = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2])
root = Path(tempfile.mkdtemp(prefix="pr4-fixed-", dir="/private/tmp"))
root.chmod(0o700)
owned = []
result = {
    "root": str(root), "binary": str(binary),
    "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
    "source_head": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "source_dirty": True, "cases": [],
}


def git(repo, *args):
    return subprocess.check_output(["/usr/bin/git", "-C", str(repo), *args], stderr=subprocess.STDOUT, text=True).strip()


def exchange(state, field=None, body=None):
    data = {"protocol": 4, "application_version": "0.1.0", "after": None}
    if field:
        data[field] = body
    with socket.socket(socket.AF_UNIX) as connection:
        connection.settimeout(3)
        connection.connect(str(state / "supervisor.sock"))
        connection.sendall((json.dumps(data) + "\n").encode())
        return json.loads(connection.makefile("rb").readline())


def request(state, field, body):
    reply = exchange(state, field, body)["result"]
    assert "Ok" in reply, reply
    return reply["Ok"]


def wait(read, done):
    deadline = time.monotonic() + 8
    while time.monotonic() < deadline:
        try:
            value = read()
            if done(value):
                return value
        except (OSError, ValueError):
            pass
        time.sleep(.01)
    raise AssertionError("bounded qualification deadline expired")


def launch(argument, cwd, state):
    with (root / f"owner-{len(owned)}.log").open("w") as log:
        proc = subprocess.Popen([str(binary), argument], cwd=cwd, stdout=log, stderr=log)
    owned.append(proc)
    wait(lambda: exchange(state), lambda value: value["kind"] == "snapshot")
    return proc


started = time.monotonic()
try:
    for case in ["relative", "dotdot"]:
        directory = root / case
        launcher = directory / "launcher"
        launcher.mkdir(parents=True)
        state = launcher / "state"
        state.mkdir(mode=0o700)
        repo = directory / "repo"
        repo.mkdir()
        hooks = directory / "hooks"
        hooks.mkdir()
        git(repo, "-c", "init.templateDir=", "init", "--initial-branch=main")
        git(repo, "config", "core.hooksPath", str(hooks))
        (repo / "tracked.txt").write_text("base\n")
        git(repo, "add", ".")
        git(repo, "-c", "commit.gpgsign=false", "-c", "user.name=Qualification", "-c", "user.email=q@example.invalid", "commit", "-m", "base")
        (repo / "tracked.txt").write_text("dirty original\n")
        (repo / "untracked.txt").write_text("preserve\n")
        before = git(repo, "status", "--porcelain")
        hashes = {name: hashlib.sha256((repo / name).read_bytes()).hexdigest() for name in ["tracked.txt", "untracked.txt"]}
        # An ambiguous base supplies an actual actionable failure before explicit recovery.
        git(repo, "branch", "master")
        argument = "state" if case == "relative" else str(state) + "/../state"
        owner = launch(argument, launcher, state)
        aid = str(uuid.uuid4())
        request(state, "repository", {"operation": "register", "request": {"action_id": aid, "path": str(repo), "create_initial_workspace": True}})
        registration = wait(lambda: request(state, "repository", {"operation": "action", "action_id": aid})["registration"], lambda value: value["state"] != "pending")
        wid = registration["initial_workspace_id"]
        get = lambda: request(state, "workspace", {"operation": "get", "id": wid})["workspace"]
        done = lambda value: value["status"]["state"] not in ["queued", "running"]
        failed = wait(get, done)
        assert failed["status"]["state"] == "failed", failed
        assert Path(failed["path"]).is_absolute() and str(Path(failed["path"]).resolve()) == failed["path"]
        assert Path(failed["path"]).is_relative_to(state)
        git(repo, "branch", "-D", "master")
        request(state, "workspace", {"operation": "reconcile", "action_id": str(uuid.uuid4()), "workspace_id": wid})
        ready = wait(get, done)
        assert ready["status"]["state"] == "ready", ready
        assert all(ready[key] == failed[key] for key in ["id", "creation_action_id", "name", "branch", "path"])
        assert Path(ready["path"]).is_dir()
        assert git(repo, "status", "--porcelain") == before
        assert all(hashlib.sha256((repo / name).read_bytes()).hexdigest() == value for name, value in hashes.items())
        assert not (repo / "state").exists()
        owner.terminate()
        owner.wait(timeout=5)
        owner = launch(argument, launcher, state)
        restored = get()
        assert restored == ready
        entry = {"case": case, "argument": argument, "cwd": str(launcher), "failed": failed, "ready": ready, "restored": restored, "original_status_unchanged": before, "original_hashes": hashes}
        if case == "relative":
            # Deliberately incompatible development data must fail without adopting a different target.
            owner.terminate()
            owner.wait(timeout=5)
            db = sqlite3.connect(state / "workspace.sqlite3")
            malformed = {**ready, "path": "state/workspaces/foreign/checkout"}
            db.execute("UPDATE workspaces SET record=?,path=? WHERE id=?", (json.dumps(malformed), malformed["path"], wid))
            db.commit()
            db.close()
            owner = launch(argument, launcher, state)
            rejected = exchange(state, "workspace", {"operation": "get", "id": wid})["result"]
            assert "Err" in rejected and "Incompatible workspace path" in rejected["Err"]["message"], rejected
            assert Path(ready["path"]).is_dir() and git(repo, "status", "--porcelain") == before
            entry["incompatible_record_rejected"] = rejected
        result["cases"].append(entry)
    result["result"] = "pass"
finally:
    for proc in owned:
        if proc.poll() is None:
            proc.terminate()
            proc.wait(timeout=5)
    result["owned_exits"] = [proc.returncode for proc in owned]
    result["wall_seconds"] = time.monotonic() - started
    output.write_text(json.dumps(result, indent=2) + "\n")
print(json.dumps({"result": result.get("result"), "root": str(root), "wall_seconds": result["wall_seconds"]}))
