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
import threading
import uuid
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

root = Path(tempfile.mkdtemp(prefix="aw-ready-", dir="/private/tmp"))
root.chmod(0o700)
state = root / "state"
state.mkdir(mode=0o700)
control = root / "control.json"
control.write_text(json.dumps({"scenario": "ready"}))
browser = "--browser" in sys.argv
landing = None
def scenario(value):
    settings = json.loads(control.read_text())
    settings["scenario"] = value
    temporary = control.with_suffix(".new")
    temporary.write_text(json.dumps(settings))
    temporary.replace(control)

if browser:
    class Landing(BaseHTTPRequestHandler):
        def do_GET(self):
            if self.path == "/complete": scenario("browser_complete")
            if self.path == "/failure": scenario("browser_failure")
            body = b'<!doctype html><title>Fixture Codex browser sign-in</title><h1>Fixture Codex browser sign-in</h1><p>Synthetic local authentication. No credentials are collected.</p><p><a href="/complete">Complete fixture sign-in</a></p><p><a href="/failure">Fail fixture sign-in</a></p>'
            self.send_response(200); self.send_header("Content-Type", "text/html"); self.end_headers(); self.wfile.write(body)
        def log_message(self, *_): pass
    landing = ThreadingHTTPServer(("127.0.0.1", 0), Landing)
    threading.Thread(target=landing.serve_forever, daemon=True).start()
    control.write_text(json.dumps({"scenario": "browser_pending", "browser_url": "aw-invalid:headless-fixture" if "--headless" in sys.argv else f"http://127.0.0.1:{landing.server_port}/fixture-sign-in"}))
supervisor_path = Path(sys.argv[1]).resolve(strict=True)
protocol = int(subprocess.check_output([str(supervisor_path), "--protocol-version"], text=True))
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
    payload = json.dumps({"protocol": protocol, "application_version": "0.1.0", "after": None, **fields}).encode()
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


report = {"protocol": protocol, "root": str(root), "owner": owner, "supervisor_pid": supervisor.pid, "supervisor": str(supervisor_path), "supervisor_sha256": hashlib.sha256(supervisor_path.read_bytes()).hexdigest(), "target": target, "checks": [], "limits": "Synthetic runtime only; no real authentication, thread, turn or Claude process."}
report["headless_browser"] = "--headless" in sys.argv
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
    if len(sys.argv) > 3 and not sys.argv[3].startswith("--"):
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
                scenario(command.split(" ", 1)[1])
            elif command == "snapshot":
                print(json.dumps({"observation": observe(), "events": events()}), flush=True)
            elif command == "restart-desktop":
                assert desktop.poll() is not None, "Quit the desktop first"
                desktop = start_desktop()
                report["desktop_reopened_pid"] = desktop.pid
            elif command == "kill-desktop":
                desktop.kill(); desktop.wait(timeout=5)
            elif command == "processes":
                print(json.dumps({"supervisor_alive": supervisor.poll() is None, "desktop_alive": desktop.poll() is None, "events": events()}), flush=True)
            save()
    elif browser:
        def login_settled(phase):
            deadline = time.monotonic()+25
            while time.monotonic() < deadline:
                value = observe()
                if value.get("login", {}).get("phase") == phase: return value
                time.sleep(.02)
            raise TimeoutError(f"login did not reach {phase}: {value}")

        def start_login(lost=False):
            current = observe()
            request = {"operation": "start_browser_login", "target_id": target["id"], "attempt_id": str(uuid.uuid4()), "expected_revision": current["revision"]}
            if lost:
                payload = json.dumps({"protocol": protocol, "application_version": "0.1.0", "after": None, "expected_owner": owner, "provider": request}).encode()
                with socket.socket(socket.AF_UNIX) as peer:
                    peer.connect(str(state/"supervisor.sock")); peer.sendall(struct.pack(">I",len(payload))+payload)
            else: provider(request)
            return request

        def pending_case():
            assert events() == []
            assert observe()["phase"] == "unknown"
            assert events() == []
            observe(True); settled()
            request = start_login()
            value = login_settled("pending")
            before = len([e for e in events() if e.get("method") == "account/login/start"])
            assert provider(request)["observation"]["login"]["id"] == request["attempt_id"]
            competitor = {**request, "attempt_id": str(uuid.uuid4()), "expected_revision": value["revision"]}
            assert "Err" in exchange(expected_owner=owner, provider=competitor)["result"]
            observe(True)
            assert len([e for e in events() if e.get("method") == "account/login/start"]) == before == 1
            scenario("browser_complete")
            result = login_settled("completed")
            assert result["evidence"]["authentication"] == "chatgpt"
            return result
        check("browser handoff, duplicate exclusion, check fencing and authoritative completion", pending_case)

        def lost_case():
            scenario("browser_pending")
            request = start_login(lost=True)
            result = login_settled("pending")
            status = provider({"operation":"reconcile_login", "target_id":target["id"], "attempt_id":request["attempt_id"]})
            assert status["observation"]["login"]["id"] == request["attempt_id"]
            provider({"operation":"cancel_login", "target_id":target["id"], "attempt_id":request["attempt_id"]})
            result = login_settled("cancelled")
            return result
        check("lost start reply retrieves same attempt and exact runtime cancellation", lost_case)

        def browser_failure_case():
            scenario("browser_open_failure")
            request = start_login(); result = login_settled("pending")
            deadline = time.monotonic()+5
            while not result["login"]["browser_open_failed"] and time.monotonic()<deadline:
                time.sleep(.02); result=observe()
            assert result["login"]["browser_open_failed"]
            provider({"operation":"reopen_browser", "target_id":target["id"], "attempt_id":request["attempt_id"]})
            assert observe()["login"]["id"] == request["attempt_id"]
            provider({"operation":"cancel_login", "target_id":target["id"], "attempt_id":request["attempt_id"]})
            return login_settled("cancelled")
        check("browser-open failure and explicit same-attempt reopen", browser_failure_case)

        def late_setup_case():
            scenario("version_delay")
            before = len(events())
            login_starts = len([event for event in events() if event.get("method") == "account/login/start"])
            request = start_login()
            deadline = time.monotonic()+5
            while time.monotonic()<deadline:
                if any(event["event"] == "start" and event.get("argument") == ["--version"] for event in events()[before:]): break
                time.sleep(.02)
            else: raise TimeoutError("late version child did not start")
            scenario("browser_pending")
            provider({"operation":"cancel_login", "target_id":target["id"], "attempt_id":request["attempt_id"]})
            result = login_settled("cancelled")
            assert len([event for event in events() if event.get("method") == "account/login/start"]) == login_starts
            return result
        check("cancellation during late version setup stops owned work before authentication", late_setup_case)

        def descendant_case():
            scenario("fork_descendant")
            request = start_login(); login_settled("pending")
            assert any(event["event"] == "descendant_start" for event in events())
            scenario("browser_pending")
            payload = json.dumps({"protocol": protocol, "application_version": "0.1.0", "after": None, "expected_owner": owner, "provider": {"operation":"cancel_login", "target_id":target["id"], "attempt_id":request["attempt_id"]}}).encode()
            with socket.socket(socket.AF_UNIX) as peer:
                peer.connect(str(state/"supervisor.sock")); peer.sendall(struct.pack(">I",len(payload))+payload)
            result=login_settled("cancelled")
            assert provider({"operation":"reconcile_login", "target_id":target["id"], "attempt_id":request["attempt_id"]})["observation"]["login"]["phase"] == "cancelled"
            return result
        check("lost cancel reply and owned descendant cleanup", descendant_case)

        for end_scenario, phase, problem in [("browser_failure","failed","authentication_unavailable"),("browser_verify_failure","completed","status_read_failed")]:
            def failure_case(end_scenario=end_scenario, phase=phase, problem=problem):
                scenario("browser_pending"); start_login(); login_settled("pending"); scenario(end_scenario)
                result=login_settled(phase); assert result["problem"]==problem, result
                return result
            check(end_scenario, failure_case)

        def shutdown_case():
            scenario("browser_pending"); start_login(); login_settled("pending")
            provider({"operation":"begin_shutdown"})
            deadline=time.monotonic()+25
            while time.monotonic()<deadline:
                status=provider({"operation":"shutdown_status"})
                if status["complete"]: break
                assert not status["diagnostic"], status
                time.sleep(.05)
            assert status["complete"], status
            provider({"operation":"finish_shutdown"})
            supervisor.wait(timeout=5)
            assert supervisor.returncode == 0
            return status
        check("owned login shutdown and supervisor completion", shutdown_case)
    elif missing:
        def missing_executable():
            assert events() == []
            observe(True)
            result = settled()
            assert result["problem"] == "missing_executable"
            assert result["evidence"] is None
            assert events() == []
            return result
        check("missing executable without fallback or authentication effects", missing_executable)
    else:
        def readiness():
            assert events() == [], "descriptor lookup must not start a runtime"
            first = observe(True)
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
            ("cleanup_delay", "status_read_failed", "chatgpt"), ("fork_descendant", None, "chatgpt"), ("ready", None, "chatgpt"),
        ]:
            def run_case(scenario=scenario, problem=problem, auth=auth):
                control.write_text(json.dumps({"scenario": scenario}))
                observe(True)
                result = settled()
                assert result["problem"] == problem, result
                assert result["evidence"]["authentication"] == auth, result
                if protocol >= 11 and problem and problem != "authentication_unavailable":
                    assert result["diagnostic"], result
                if protocol >= 11 and scenario == "version_exit":
                    assert "Some(7)" in result["diagnostic"], result
                if protocol >= 11 and scenario == "config_failure":
                    assert "-32603" in result["diagnostic"], result
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
    if browser and supervisor.poll() is None:
        try:
            provider({"operation":"begin_shutdown"})
            deadline=time.monotonic()+25
            while time.monotonic()<deadline:
                status=provider({"operation":"shutdown_status"})
                if status["complete"]: break
                if status["diagnostic"]: raise RuntimeError(status["diagnostic"])
                time.sleep(.05)
            if status["complete"]:
                provider({"operation":"finish_shutdown"}); supervisor.wait(timeout=5)
        except Exception as error:
            report["cleanup_failure"] = repr(error)
    if supervisor.poll() is None:
        supervisor.terminate()
        supervisor.wait(timeout=5)
    if landing: landing.shutdown(); landing.server_close()
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
