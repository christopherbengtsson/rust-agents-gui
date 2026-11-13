"""Opt-in actual-process evidence harness. Not part of the unit gate.

Uses only disposable repositories/state and an explicitly supplied built supervisor.
A synthetic Git wrapper pauses one mutation to qualify the real inherited lease.
It makes no provider submissions. All launched supervisor handles are owned here.
"""
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time
import uuid

binary = Path(sys.argv[1]).resolve()
output = Path(sys.argv[2])
root = Path(tempfile.mkdtemp(prefix="rag04-process-", dir="/private/tmp"))
state = root / "state"
state.mkdir(mode=0o700)
repo = root / "repo"
repo.mkdir()


def git(*args):
    return subprocess.check_output(
        ["/usr/bin/git", "-C", str(repo), *args], stderr=subprocess.STDOUT, text=True
    ).strip()


git("init", "--initial-branch=main")
(repo / "tracked.txt").write_text("base\n")
git("add", ".")
git("-c", "user.name=Qualification", "-c", "user.email=q@example.invalid", "commit", "-m", "base")
pinned = git("rev-parse", "HEAD")
(repo / "tracked.txt").write_text("dirty original\n")
(repo / "untracked.txt").write_text("preserve\n")
original = {name: hashlib.sha256((repo / name).read_bytes()).hexdigest() for name in ["tracked.txt", "untracked.txt"]}
bin_dir = root / "bin"
bin_dir.mkdir()
wrapper = bin_dir / "git"
wrapper.write_text("""#!/usr/bin/python3
import os,sys,time
from pathlib import Path
root=Path(os.environ['QUALIFICATION_GATE'])
if 'worktree' in sys.argv and 'add' in sys.argv:
    (root/'entered').write_text(str(os.getpid()))
    while not (root/'release').exists(): time.sleep(.01)
os.execv('/usr/bin/git',['/usr/bin/git',*sys.argv[1:]])
""")
wrapper.chmod(0o700)
env = {**os.environ, "PATH": f"{bin_dir}:{os.environ['PATH']}", "QUALIFICATION_GATE": str(root)}
owned = []


def launch():
    log = (root / f"owner-{len(owned)}.log").open("w")
    process = subprocess.Popen([str(binary), str(state)], env=env, stdout=log, stderr=log)
    owned.append(process)
    deadline = time.monotonic() + 5
    while time.monotonic() < deadline:
        try:
            snapshot = exchange(None, None)
            if snapshot["kind"] == "snapshot":
                return process, snapshot["snapshot"]
        except (OSError, KeyError):
            pass
        if process.poll() is not None:
            raise RuntimeError(f"Supervisor exited {process.returncode}; inspect {root}")
        time.sleep(.01)
    raise RuntimeError("Supervisor did not start")


def exchange(field, request):
    payload = {"protocol": 3, "application_version": "0.1.0", "after": None}
    if field:
        payload[field] = request
    with socket.socket(socket.AF_UNIX) as connection:
        connection.settimeout(3)
        connection.connect(str(state / "supervisor.sock"))
        connection.sendall((json.dumps(payload) + "\n").encode())
        data = b""
        while not data.endswith(b"\n"):
            part = connection.recv(65536)
            if not part:
                raise RuntimeError("Reply lost")
            data += part
        return json.loads(data)


def request(field, body):
    result = exchange(field, body)["result"]
    if "Err" in result:
        raise RuntimeError(result["Err"])
    return result["Ok"]


def wait_for(read, predicate, seconds=8):
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        value = read()
        if predicate(value):
            return value
        time.sleep(.02)
    raise RuntimeError(f"Timed out: {value}")


results = {"root": str(root), "binary": str(binary), "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "synthetic_fault": "Git wrapper pauses worktree add; real kernel lease and supervisor restart"}
started = time.monotonic()
try:
    first, first_snapshot = launch()
    action_id = str(uuid.uuid4())
    register = {"operation": "register", "request": {"action_id": action_id, "path": str(repo), "create_initial_workspace": True}}
    request("repository", register)
    registration = wait_for(lambda: request("repository", {"operation": "action", "action_id": action_id})["registration"], lambda a: a["state"] == "completed")
    workspace_id = registration["initial_workspace_id"]
    get = lambda: request("workspace", {"operation": "get", "id": workspace_id})["workspace"]
    pending = wait_for(get, lambda w: w["status"] == {"state": "running", "stage": "creating_worktree"} and (root / "entered").exists())
    # Kill only this harness's directly owned supervisor. The paused Git process survives.
    first.kill()
    first.wait(timeout=5)
    second, second_snapshot = launch()
    restored = get()
    assert restored["status"]["state"] == "uncertain"
    assert restored["id"] == pending["id"] and restored["base"]["commit"] == pinned
    recovery_id = str(uuid.uuid4())
    request("workspace", {"operation": "reconcile", "action_id": recovery_id, "workspace_id": workspace_id})
    leased = wait_for(get, lambda w: w["status"]["state"] == "uncertain" and w["cleanup_unconfirmed"])
    denied = exchange("workspace", {"operation": "create", "action_id": str(uuid.uuid4()), "repository_id": pending["repository_id"]})["result"]
    assert denied["Err"]["code"] == "cleanup_unconfirmed"
    # Releasing the test fixture lets the already-owned mutation finish naturally.
    (root / "release").touch()
    wait_for(lambda: (Path(pending["path"]) / ".git").is_file(), bool)
    time.sleep(.1)
    request("workspace", {"operation": "reconcile", "action_id": str(uuid.uuid4()), "workspace_id": workspace_id})
    ready = wait_for(get, lambda w: w["status"]["state"] not in ["running", "queued"])
    assert ready["status"]["state"] == "ready", ready
    assert ready["id"] == pending["id"] and ready["branch"] == pending["branch"] and ready["path"] == pending["path"]
    replay = request("workspace", {"operation": "create", "action_id": pending["creation_action_id"], "repository_id": pending["repository_id"]})
    assert replay["workspace"]["id"] == workspace_id
    conflict = exchange("workspace", {"operation": "reconcile", "action_id": pending["creation_action_id"], "workspace_id": workspace_id})["result"]
    assert conflict["Err"]["code"] == "conflict"
    assert len(git("for-each-ref", "--format=%(refname)", "refs/heads/workspace/").splitlines()) == 1
    assert len(git("worktree", "list", "--porcelain").split("worktree ")) - 1 == 2
    assert {name: hashlib.sha256((repo / name).read_bytes()).hexdigest() for name in original} == original
    assert (Path(ready["path"]) / "tracked.txt").read_text() == "base\n"
    results.update({"result": "pass", "first_owner": first_snapshot, "replacement_owner": second_snapshot, "registration": registration, "pending": pending, "restored": restored, "lease_blocked": leased, "new_creation_denied": denied, "ready": ready, "conflicting_action": conflict, "original_file_hashes": original, "workspace_branch_count": 1, "worktree_count_including_original": 2})
finally:
    # Ensure the fixture cannot remain paused even after a failed assertion.
    (root / "release").touch()
    for process in owned:
        if process.poll() is None:
            process.terminate()
            process.wait(timeout=5)
    results["wall_seconds"] = time.monotonic() - started
    output.write_text(json.dumps(results, indent=2) + "\n")
print(json.dumps({"result": results.get("result"), "root": str(root), "wall_seconds": results["wall_seconds"]}))
