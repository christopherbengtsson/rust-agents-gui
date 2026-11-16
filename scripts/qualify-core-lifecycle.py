#!/usr/bin/env python3
"""Bounded incumbent-host qualification, separate from substituted application units.
Only disposable local Git/state and synthetic dependencies; no provider/auth actions.
"""
import json
import os
from pathlib import Path
import select
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import uuid

binary = Path(sys.argv[1]).resolve(strict=True)
report = Path(sys.argv[2])
assert not report.exists(), "Preserve previous evidence"
root = Path(tempfile.mkdtemp(prefix="aw-core01-", dir="/private/tmp"))
root.chmod(0o700)
state = root / "state"
state.mkdir(mode=0o700)
repository = root / "repository"
subprocess.run(["git", "init", "--initial-branch=main", str(repository)], check=True, capture_output=True)
bin_dir = root / "bin"
bin_dir.mkdir()
wrapper = bin_dir / "git"
wrapper.write_text("#!/usr/bin/env python3\nimport os,time\nfrom pathlib import Path\nr=Path(" + repr(str(root)) + ")\n(r/'entered').touch()\nend=time.monotonic()+15\nwhile not (r/'release').exists():\n if time.monotonic()>end: raise SystemExit(99)\n time.sleep(.01)\nos.execv(" + repr(shutil.which("git")) + ",['git',*__import__('sys').argv[1:]])\n")
wrapper.chmod(0o700)
env = {**os.environ, "PATH": str(bin_dir) + os.pathsep + os.environ["PATH"]}
protocol = int(subprocess.check_output([str(binary), "--protocol-version"], text=True))
checks = []
processes = []
started = time.monotonic()

def wait_for(predicate, seconds=10):
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        if predicate(): return
        time.sleep(.01)
    raise TimeoutError("Controlled fixture condition did not complete")

def start():
    process = subprocess.Popen([str(binary), str(state)], env=env, stdout=subprocess.PIPE, stderr=(root / f"host-{len(processes)}.log").open("w"))
    processes.append(process)
    assert select.select([process.stdout], [], [], 10)[0]
    assert process.stdout.readline() == b"ready\n"
    return process

def exchange(owner=None, quit=None, repository=None, abandon=False):
    request = {"protocol": protocol, "application_version": "0.1.0", "after": None,
               "expected_owner": owner, "quit": quit, "repository": repository}
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(3)
        peer.connect(str(state / "supervisor.sock"))
        body = json.dumps(request).encode()
        peer.sendall(struct.pack(">I", len(body)) + body)
        if abandon: return
        def exact(count):
            data = b""
            while len(data) < count:
                part = peer.recv(count-len(data))
                if not part: raise EOFError("Incomplete frame")
                data += part
            return data
        size = struct.unpack(">I", exact(4))[0]
        assert size <= 65532
        return json.loads(exact(size))

def excluded():
    contender = subprocess.run([str(binary), str(state)], env=env, capture_output=True, timeout=3)
    assert contender.returncode != 0

try:
    host = start()
    snapshot = exchange()["snapshot"]
    assert snapshot["status"]["state"] == "ready"
    owner = snapshot["cursor"]["owner"]
    excluded()
    checks.append("actual same-root process exclusion")
    action = str(uuid.uuid4())
    exchange(owner, repository={"operation":"register", "request":{"action_id": action, "path":str(repository), "create_initial_workspace":False}}, abandon=True)
    wait_for(lambda: (root / "entered").exists())
    assert not exchange(owner, quit="begin")["status"]["complete"]
    assert not exchange(owner, quit="begin")["status"]["complete"]
    rejected = exchange(owner, repository={"operation":"register", "request":{"action_id":str(uuid.uuid4()), "path":str(repository), "create_initial_workspace":False}})
    assert "Err" in rejected["result"]
    excluded()
    checks.append("lost request retains active Git; repeated Quit pending; admission fenced; lease retained")
    (root / "release").touch()
    wait_for(lambda: exchange(owner, quit="status")["status"]["complete"])
    assert exchange(owner, quit="finish")["status"]["complete"]
    assert host.wait(timeout=10) == 0
    checks.append("whole-core physical drain and process exit")
    host = start()
    recovered = exchange()["snapshot"]
    owner2 = recovered["cursor"]["owner"]
    assert owner2 != owner
    recorded = exchange(owner2, repository={"operation":"action", "action_id":action})
    settled_action = recorded["result"]["Ok"]["registration"]
    # Ticket03 revalidates the fence after read-only inspection and before its
    # next effect. Reopen must retain that exact failure, never invent completion.
    assert settled_action["action_id"] == action and settled_action["path"] == str(repository), settled_action
    assert settled_action["state"] == "failed", settled_action
    assert settled_action["error"]["code"] == "unavailable", settled_action
    assert "fenced before its next effect" in settled_action["error"]["message"], settled_action
    exchange(owner2, quit="begin")
    wait_for(lambda: exchange(owner2, quit="status")["status"]["complete"])
    exchange(owner2, quit="finish")
    assert host.wait(timeout=10) == 0
    checks.append("fresh current-build restart reacquires lease and retains admitted action outcome")
    result = "passed"
except BaseException as error:
    result = "failed: " + repr(error)
    raise
finally:
    (root / "release").touch()
    for process in processes:
        if process.poll() is None:
            process.terminate()
            try: process.wait(timeout=3)
            except subprocess.TimeoutExpired: process.kill(); process.wait(timeout=3)
    report.write_text(json.dumps({"result":result,"binary":str(binary),"fixture_root":str(root),"protocol":protocol,"checks":checks,"seconds":time.monotonic()-started},indent=2)+"\n")
