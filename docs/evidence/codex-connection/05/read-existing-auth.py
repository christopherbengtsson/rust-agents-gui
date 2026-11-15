#!/usr/bin/python3
"""Agreed read-only existing-authentication cohort through packaged supervisor.
No browser/login/thread/turn action; no helper access to credential contents.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import struct
import subprocess
import tempfile
import time

folder = Path(__file__).resolve().parent
output = folder / "existing-auth.json"
assert not output.exists()
source = Path("target/release/bundle/macos/Agent Workspace.app/Contents/Resources/workspace-supervisor").resolve(strict=True)
root = Path(tempfile.mkdtemp(prefix="aw-discovery-real-", dir="/private/tmp")); root.chmod(0o700)
state = root / "state"; state.mkdir(mode=0o700)
installed = Path(shutil.which("codex")).resolve(strict=True)
hash_file = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
before = hash_file(installed)
env = dict(os.environ)
for key in ["AGENT_WORKSPACE_CODEX_FIXTURE", "AW_FIXTURE_CONTROL", "AW_FIXTURE_SCENARIO", "AGENT_WORKSPACE_SUPERVISOR"]: env.pop(key, None)
env["AGENT_WORKSPACE_STATE_DIR"] = str(state)
protocol = int(subprocess.check_output([str(source), "--protocol-version"]))
process = subprocess.Popen([str(source), str(state)], env=env, stdout=(root / "supervisor.log").open("w"), stderr=subprocess.STDOUT, start_new_session=True)
requests = []
def exchange(**fields):
    data = json.dumps({"protocol": protocol, "application_version": "0.1.0", "after": None, **fields}).encode()
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(5); peer.connect(str(state / "supervisor.sock")); peer.sendall(struct.pack(">I", len(data)) + data)
        def exact(n):
            result=b""
            while len(result)<n:
                part=peer.recv(n-len(result))
                if not part: raise EOFError("Short reply")
                result+=part
            return result
        length=struct.unpack(">I",exact(4))[0]; assert length<=65532
        return json.loads(exact(length))
def provider(request):
    requests.append(request)
    return exchange(expected_owner=owner, provider=request)["result"]["Ok"]
deadline=time.monotonic()+10
while time.monotonic()<deadline:
    try: owner=exchange()["snapshot"]["cursor"]["owner"]; break
    except OSError: time.sleep(.02)
target=provider({"operation":"target"})["target"]
provider({"operation":"observe","target_id":target["id"],"refresh":True})
deadline=time.monotonic()+25
while time.monotonic()<deadline:
    observation=provider({"operation":"observe","target_id":target["id"],"refresh":False})["observation"]
    if observation["phase"]=="observed": break
    time.sleep(.02)
assert observation["phase"]=="observed", observation["phase"]
# Discard account identity before any repository artifact is written.
if observation["evidence"]:
    observation["evidence"]["account"] = "<redacted>" if observation["evidence"]["account"] else None
provider({"operation":"begin_shutdown"})
deadline=time.monotonic()+20
while time.monotonic()<deadline:
    status=provider({"operation":"shutdown_status"})
    if status["complete"]: break
    time.sleep(.02)
assert status["complete"],status
provider({"operation":"finish_shutdown"}); process.wait(timeout=5)
after=hash_file(installed)
result={"root":str(root),"source":str(source),"supervisor_sha256":hash_file(source),"protocol":protocol,"installed_executable":str(installed),"runtime_sha256_before":before,"runtime_sha256_after":after,"observation":observation,"requests":requests,"shutdown":status,"supervisor_exit":process.returncode,"limits":"Read-only existing authentication via packaged supervisor. No desktop/Finder UI, browser sign-in, credential-file inspection or native agent work. Account redacted before publication."}
output.write_text(json.dumps(result,indent=2)+"\n")
assert before==after and process.returncode==0
assert observation["problem"] is None, observation["problem"]
assert observation["evidence"]["authentication"]=="chatgpt",observation["evidence"]["authentication"]
print(json.dumps({"runtime":observation['evidence']['version'],"authentication":observation['evidence']['authentication'],"supervisor_exit":process.returncode,"runtime_unchanged":before==after}))
