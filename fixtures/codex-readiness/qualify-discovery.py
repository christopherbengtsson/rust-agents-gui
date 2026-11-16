#!/usr/bin/python3
"""Explicit controlled native discovery/desktop qualification, never a unit test.

Usage: qualify-discovery.py <packaged .app> <new report.json>
Uses native discovery (no forced executable), private shell startup files and a
minimal environment. Protocol/account outcomes are synthetic, not real authentication.
Interact with the actual desktop; stdin commands only change private fixture inputs
or capture authoritative observations. Quit the product before `restart`/`stop`.
"""
import hashlib
import json
import os
from pathlib import Path
import plistlib
import shlex
import shutil
import socket
import struct
import subprocess
import sys
import tempfile
import time
import uuid

source = Path(sys.argv[1]).resolve(strict=True)
output = Path(sys.argv[2]).resolve()
process_only = "--process" in sys.argv
assert not output.exists(), "Preserve earlier qualification reports"
root = Path(tempfile.mkdtemp(prefix="aw-discovery-", dir="/private/tmp"))
root.chmod(0o700)
for name in ["state", "home", "shell", "tools", "new-tools", "codex-home"]:
    (root / name).mkdir(mode=0o700)
runtime = Path(__file__).with_name("runtime.py").resolve(strict=True)
for name in ["codex-v1", "codex-v2"]:
    shutil.copyfile(runtime, root / name)
    (root / name).chmod(0o700)
(root / "manual-codex").symlink_to(root / "codex-v1")
(root / "replacement-codex").symlink_to(root / "codex-v2")
control = root / "control.json"
control.write_text(json.dumps({"scenario": "ready", "browser_url": "aw-invalid:controlled-fixture"}))
(root / "shell/.zprofile").write_text("export PATH=/usr/bin:/bin\n")
def shell_path(directory):
    (root / "shell/.zshrc").write_text("export PATH=" + shlex.quote(str(directory) + ":/usr/bin:/bin") + "\n")
shell_path(root / "tools")
bundle = root / "Discovery Refinement.app"
shutil.copytree(source, bundle)
plist = bundle / "Contents/Info.plist"
info = plistlib.loads(plist.read_bytes())
info.update(CFBundleIdentifier="dev.christopherbengtsson.discovery." + root.name, CFBundleName="Discovery Refinement")
plist.write_bytes(plistlib.dumps(info))
desktop_path = bundle / "Contents/MacOS/workspace-desktop"
supervisor_path = bundle / "Contents/Resources/workspace-supervisor"
# No inherited user credential/configuration environment reaches the synthetic peer.
env = {"PATH": "/usr/bin:/bin", "HOME": str(root / "home"), "SHELL": "/bin/zsh", "ZDOTDIR": str(root / "shell"),
       "CODEX_HOME": str(root / "codex-home"), "AGENT_WORKSPACE_STATE_DIR": str(root / "state"), "AW_FIXTURE_CONTROL": str(control)}
protocol = int(subprocess.check_output([str(supervisor_path), "--protocol-version"]))
report = {"root": str(root), "bundle": str(bundle), "protocol": protocol, "environment": env,
          "desktop_sha256": hashlib.sha256(desktop_path.read_bytes()).hexdigest(),
          "supervisor_sha256": hashlib.sha256(supervisor_path.read_bytes()).hexdigest(),
          "limits": "Controlled native filesystem/shell/process discovery; synthetic account/protocol evidence. No user credentials, real authentication or agent work.", "commands": [], "captures": [], "mode": "packaged-supervisor-process" if process_only else "packaged-desktop"}

def save(): output.write_text(json.dumps(report, indent=2) + "\n")
def exchange(**fields):
    payload = json.dumps({"protocol": protocol, "application_version": "0.1.0", "after": None, **fields}).encode()
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(5); peer.connect(str(root / "state/supervisor.sock")); peer.sendall(struct.pack(">I", len(payload)) + payload)
        def exact(count):
            data = b""
            while len(data) < count:
                part = peer.recv(count-len(data))
                if not part: raise EOFError("Truncated reply")
                data += part
            return data
        size = struct.unpack(">I", exact(4))[0]; assert size <= 65532
        return json.loads(exact(size))
def attach():
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        try:
            snapshot = exchange(); owner = snapshot["snapshot"]["cursor"]["owner"]
            target = exchange(expected_owner=owner, provider={"operation": "target"})["result"]["Ok"]["target"]
            return owner, target
        except (OSError, KeyError): time.sleep(.05)
    raise TimeoutError("Desktop supervisor did not become available")
def provider(request):
    return exchange(expected_owner=owner, provider=request)["result"]["Ok"]
def settled():
    deadline = time.monotonic() + 25
    while time.monotonic() < deadline:
        value = provider({"operation": "observe", "target_id": target["id"], "refresh": False})["observation"]
        if value["phase"] == "observed": return value
        time.sleep(.02)
    raise TimeoutError("Check did not settle")
def start():
    return subprocess.Popen([str(supervisor_path), str(root / "state")] if process_only else [str(desktop_path)], env=env, stdout=(root / "desktop.log").open("a"), stderr=subprocess.STDOUT, start_new_session=True)
def events():
    path = control.with_suffix(".events.jsonl")
    return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []
def capture(label):
    observation = exchange(expected_owner=owner, provider={"operation": "observe", "target_id": target["id"], "refresh": False})["result"]["Ok"]["observation"]
    data = {"label": label, "owner": owner, "observation": observation, "events": events(), "desktop_exit": desktop.poll(), "snapshot": exchange()}
    destination = output.with_name(output.stem + "-" + label + ".json")
    destination.write_text(json.dumps(data, indent=2) + "\n"); report["captures"].append(str(destination)); save()
    print(json.dumps({"capture": label, "observation": observation}), flush=True)

def remaining():
    return [line for line in subprocess.check_output(["/bin/ps", "-axo", "pid=,command="], text=True).splitlines() if str(root) in line]
def shutdown_owned():
    provider({"operation": "begin_shutdown"})
    deadline = time.monotonic()+25
    while time.monotonic() < deadline:
        status = provider({"operation": "shutdown_status"})
        if status["complete"]: break
        if status["diagnostic"]: raise RuntimeError(status["diagnostic"])
        time.sleep(.02)
    assert status["complete"], status
    provider({"operation": "finish_shutdown"})
    if process_only: desktop.wait(timeout=5)
    report.setdefault("owned_shutdowns", []).append({"pid": desktop.pid, "exit": desktop.poll(), "status": status})

desktop = None
owner = None
target = None
try:
    desktop = start()
    report["desktop_pid"] = desktop.pid
    save()  # Retain identity even when initial attachment fails.
    owner, target = attach()
    print(json.dumps(report), flush=True)
    for line in sys.stdin:
        command = line.strip(); report["commands"].append({"command": command, "time": time.time()})
        if command == "check":
            provider({"operation": "observe", "target_id": target["id"], "refresh": True}); settled()
        elif command in ("select-manual", "select-replacement"):
            value = provider({"operation": "observe", "target_id": target["id"], "refresh": False})["observation"]
            path = root / ("manual-codex" if command == "select-manual" else "replacement-codex")
            provider({"operation": "select_executable", "target_id": target["id"], "action_id": str(uuid.uuid4()), "expected_revision": value["revision"], "path": str(path)})
            settled()
        elif command == "start-pending":
            value = provider({"operation": "observe", "target_id": target["id"], "refresh": False})["observation"]
            provider({"operation": "start_browser_login", "target_id": target["id"], "attempt_id": str(uuid.uuid4()), "expected_revision": value["revision"]})
            deadline = time.monotonic()+20
            while time.monotonic() < deadline:
                value = provider({"operation": "observe", "target_id": target["id"], "refresh": False})["observation"]
                if value.get("login") and value["login"]["phase"] == "pending": break
                time.sleep(.02)
            assert value["login"]["phase"] == "pending", value
        elif command in ("reopen", "cancel"):
            value = provider({"operation": "observe", "target_id": target["id"], "refresh": False})["observation"]
            provider({"operation": "reopen_browser" if command == "reopen" else "cancel_login", "target_id": target["id"], "attempt_id": value["login"]["id"]})
            if command == "cancel":
                deadline = time.monotonic()+20
                while time.monotonic() < deadline:
                    value = provider({"operation": "observe", "target_id": target["id"], "refresh": False})["observation"]
                    if value["login"]["phase"] == "cancelled": break
                    time.sleep(.02)
                assert value["login"]["phase"] == "cancelled", value
        elif command == "shutdown":
            shutdown_owned()
        elif command == "install": (root / "tools/codex").symlink_to(root / "codex-v1")
        elif command == "update":
            (root / "tools/codex").unlink(missing_ok=True)
            (root / "new-tools/codex").symlink_to(root / "codex-v2")
            shell_path(root / "new-tools")
        elif command == "remove-auto":
            for path in [root / "tools/codex", root / "new-tools/codex"]: path.unlink(missing_ok=True)
        elif command == "remove-selected": (root / "manual-codex").unlink()
        elif command.startswith("scenario "):
            settings = json.loads(control.read_text()); settings["scenario"] = command.split(" ", 1)[1]; control.write_text(json.dumps(settings))
        elif command.startswith("capture "): capture(command.split(" ", 1)[1])
        elif command == "restart":
            desktop.wait(timeout=5); assert desktop.returncode == 0
            owner = None
            desktop = start(); report["restarted_pid"] = desktop.pid; save()
            owner, target = attach()
        elif command == "stop":
            desktop.wait(timeout=5); assert desktop.returncode == 0
            report["desktop_exit"] = desktop.returncode
            report["remaining_processes"] = subprocess.check_output(["/bin/ps", "-axo", "pid=,command="], text=True).splitlines()
            report["remaining_processes"] = [line for line in report["remaining_processes"] if str(root) in line]
            assert not report["remaining_processes"], report["remaining_processes"]
            assert not any(event.get("event") == "forbidden_command" or str(event.get("method", "")).startswith(("thread/", "turn/")) for event in events())
            save(); break
        save()
except BaseException as error:
    report["failure"] = repr(error); save(); raise

finally:
    # EOF, startup failure and command failure all keep a completion owner.
    if desktop is not None and desktop.poll() is None:
        try:
            if owner is not None:
                shutdown_owned()
            else:
                # No provider intent is possible before attachment succeeds.
                report["cleanup_before_attachment"] = "Harness terminated its retained direct child before provider effects. Not product Quit."
            if desktop.poll() is None:
                desktop.terminate(); desktop.wait(timeout=5)
                report["harness_desktop_termination"] = desktop.returncode
        except BaseException as cleanup_error:
            # Never signal saved PIDs or erase product fences after uncertain cleanup.
            report["cleanup_unconfirmed"] = repr(cleanup_error)
            report["retained_owner_pid"] = desktop.pid
    report["final_process_exit"] = desktop.poll() if desktop is not None else None
    report["final_remaining_processes"] = remaining()
    save()
