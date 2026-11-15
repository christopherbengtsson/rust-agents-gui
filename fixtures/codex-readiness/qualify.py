#!/usr/bin/python3
"""Explicit actual-process/desktop qualification, excluded from unit discovery."""
import hashlib
import json
import os
from pathlib import Path
import select
import signal
import socket
import struct
import subprocess
import sys
import tempfile
import time

root = Path(tempfile.mkdtemp(prefix="aw-ready-", dir="/private/tmp"))
root.chmod(0o700)
state = root / "state"
state.mkdir(mode=0o700)
control = root / "control.json"
control.write_text(json.dumps({"scenario": "ready"}))
supervisor_path = Path(sys.argv[1]).resolve(strict=True)
runtime = Path(__file__).with_name("runtime.py").resolve(strict=True)
missing = "--missing" in sys.argv
if missing:
    runtime = root / "missing-codex"
env = {**os.environ, "AGENT_WORKSPACE_STATE_DIR": str(state), "AGENT_WORKSPACE_CODEX_FIXTURE": str(runtime), "AW_FIXTURE_CONTROL": str(control)}
env.pop("AGENT_WORKSPACE_SUPERVISOR", None)
supervisor = subprocess.Popen([str(supervisor_path), str(state)], env=env, stdout=subprocess.PIPE, stderr=(root / "supervisor.log").open("w"))
assert select.select([supervisor.stdout], [], [], 5)[0], "supervisor readiness timed out"
assert supervisor.stdout.readline() == b"ready\n"


def exchange(**fields):
    payload = json.dumps({"protocol": 10, "application_version": "0.1.0", "after": None, **fields}).encode()
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(3)
        peer.connect(str(state / "supervisor.sock"))
        peer.sendall(struct.pack(">I", len(payload)) + payload)

        def exact(size):
            result = b""
            while len(result) < size:
                data = peer.recv(size - len(result))
                if not data:
                    raise EOFError("truncated supervisor frame")
                result += data
            return result

        size = struct.unpack(">I", exact(4))[0]
        assert size <= 65532
        return json.loads(exact(size))


owner = exchange()["snapshot"]["cursor"]["owner"]


def provider(request, expected=owner):
    reply = exchange(expected_owner=expected, provider=request)
    assert reply["owner"] == expected
    return reply["result"]["Ok"]


target = provider({"operation": "target"})["target"]


def observe(refresh=False):
    return provider({"operation": "observe", "target_id": target["id"], "refresh": refresh})["observation"]


def settled():
    deadline = time.monotonic() + 25
    while time.monotonic() < deadline:
        result = observe()
        if result["phase"] == "observed":
            return result
        time.sleep(.02)
    raise TimeoutError("fixture setup failed to settle")


def events():
    file = control.with_suffix(".events.jsonl")
    return [json.loads(line) for line in file.read_text().splitlines()] if file.exists() else []


report = {"root": str(root), "owner": owner, "supervisor_pid": supervisor.pid, "supervisor": str(supervisor_path), "supervisor_sha256": hashlib.sha256(supervisor_path.read_bytes()).hexdigest(), "target": target, "checks": [], "limits": "Synthetic runtime; no native authentication, login, thread, turn or Claude process."}
output = Path(sys.argv[2])
assert not output.exists(), "Choose a new evidence destination; preserve prior attempts"
desktop = None


def save():
    output.write_text(json.dumps(report, indent=2) + "\n")


def check(name, action):
    started = time.monotonic()
    try:
        value = action()
        report["checks"].append({"name": name, "passed": True, "seconds": time.monotonic() - started, "result": value})
    except Exception as error:
        report["checks"].append({"name": name, "passed": False, "seconds": time.monotonic() - started, "error": repr(error)})
        raise
    finally:
        save()


try:
    if len(sys.argv) > 3 and sys.argv[3] != "--missing":
        desktop_path = Path(sys.argv[3]).resolve(strict=True)

        def start_desktop():
            return subprocess.Popen([str(desktop_path)], env=env, stdout=(root / "desktop.log").open("a"), stderr=subprocess.STDOUT)

        desktop = start_desktop()
        report["desktop"] = str(desktop_path)
        report["desktop_sha256"] = hashlib.sha256(desktop_path.read_bytes()).hexdigest()
        report["desktop_pid"] = desktop.pid
        save()
        print(json.dumps(report), flush=True)
        for command in sys.stdin:
            command = command.strip()
            if command == "stop":
                break
            if command.startswith("scenario "):
                control.write_text(json.dumps({"scenario": command.split(" ", 1)[1]}))
            elif command == "snapshot":
                print(json.dumps({"observation": observe(), "events": events()}), flush=True)
            elif command == "restart-desktop":
                assert desktop.poll() is not None, "Quit the desktop first"
                desktop = start_desktop()
                report["desktop_reopened_pid"] = desktop.pid
            save()
    elif missing:
        def missing_executable():
            assert events() == []
            observe()
            result = settled()
            assert result["problem"] == "missing_executable"
            assert result["evidence"] is None
            assert events() == []
            return result
        check("missing executable without fallback or authentication effects", missing_executable)
    else:
        def readiness():
            assert events() == [], "descriptor lookup must not start a runtime"
            first = observe()
            assert first["phase"] == "checking"
            for _ in range(3):
                assert observe(True)["phase"] == "checking"
            result = settled()
            assert result["evidence"]["authentication"] == "chatgpt"
            assert len([e for e in events() if e["event"] == "start"]) == 2
            return result
        check("initial readiness and active refresh coalescing", readiness)
        for scenario, problem, auth in [
            ("api_key", None, "api_key"), ("no_auth", "authentication_unavailable", "unavailable"),
            ("ready", None, "chatgpt"), ("config_failure", "status_read_failed", "chatgpt"),
            ("incompatible", "incompatible_protocol", "chatgpt"), ("version_exit", "status_read_failed", "chatgpt"),
            ("malformed", "incompatible_protocol", "chatgpt"), ("oversized", "incompatible_protocol", "chatgpt"),
            ("unrelated_id", "incompatible_protocol", "chatgpt"), ("unexpected_request", "incompatible_protocol", "chatgpt"),
            ("eof", "status_read_failed", "chatgpt"), ("deadline", "status_read_failed", "chatgpt"),
            ("cleanup_delay", "status_read_failed", "chatgpt"), ("ready", None, "chatgpt"),
        ]:
            def run_case(scenario=scenario, problem=problem, auth=auth):
                control.write_text(json.dumps({"scenario": scenario}))
                observe(True)
                result = settled()
                assert result["problem"] == problem, result
                assert result["evidence"]["authentication"] == auth, result
                return result
            check(scenario, run_case)

        def wrong_owner():
            before = len(events())
            reply = exchange(expected_owner="late-owner", provider={"operation": "observe", "target_id": target["id"], "refresh": True})
            assert reply["kind"] == "transport"
            assert reply["error"]["code"] == "owner_changed"
            assert len(events()) == before
            return reply
        check("wrong owner rejected without effects", wrong_owner)
finally:
    if desktop and desktop.poll() is None:
        desktop.terminate()
        desktop.wait(timeout=5)
    # Only the exact child returned by Popen is signalled/joined. This is fixture
    # shutdown, not qualification of the deferred full product Quit lifecycle.
    supervisor.terminate()
    supervisor.wait(timeout=5)
    report["supervisor_exit"] = supervisor.returncode
    records = events()
    assert not any(e["event"].startswith("forbidden") for e in records)
    pids = sorted({e["pid"] for e in records})
    remaining = []
    for pid in pids:
        try:
            os.kill(pid, 0)
            remaining.append(pid)
        except ProcessLookupError:
            pass
    report["fixture_pids"] = pids
    report["remaining_fixture_pids"] = remaining
    save()
    assert not remaining, remaining
