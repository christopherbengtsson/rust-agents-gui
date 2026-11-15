#!/usr/bin/python3
"""Synthetic official-schema setup peer; never a live provider or a unit dependency."""
import json
import os
from pathlib import Path
import signal
import sys
import time

control = os.environ.get("AW_FIXTURE_CONTROL")
settings = json.loads(Path(control).read_text()) if control else {}
scenario = settings.get("scenario", os.environ.get("AW_FIXTURE_SCENARIO", "ready"))


def record(event, **fields):
    if control:
        with Path(control).with_suffix(".events.jsonl").open("a") as stream:
            stream.write(json.dumps({"pid": os.getpid(), "event": event, **fields}) + "\n")


record("start", argument=sys.argv[1:])
if sys.argv[1:] == ["--version"]:
    if scenario == "version_exit":
        sys.exit(7)
    print("codex-cli 0.167.1" if scenario == "incompatible" else "codex-cli 0.169.0")
    sys.exit(0)
if sys.argv[1:] != ["app-server"]:
    record("forbidden_command")
    sys.exit(9)
if scenario == "cleanup_delay":
    signal.signal(signal.SIGTERM, signal.SIG_IGN)
if scenario == "fork_descendant":
    child = os.fork()
    if child == 0:
        signal.signal(signal.SIGTERM, signal.SIG_IGN)
        record("descendant_start")
        # Keep inherited pipes open after the parent exits. Only the owned group
        # cleanup can stop this synthetic descendant; it is never native proof.
        while True:
            time.sleep(1)


def send(value):
    print(json.dumps(value), flush=True)


try:
    for line in sys.stdin:
        request = json.loads(line)
        method = request.get("method")
        record("request", method=method)
        if method == "initialized":
            continue
        identifier = request["id"]
        if method == "initialize":
            send({"id": identifier, "result": {"userAgent": "fixture/0.169.0"}})
        elif method == "account/read":
            assert request["params"] == {"refreshToken": False}
            if scenario == "malformed":
                print("invalid-json", flush=True)
                continue
            if scenario == "oversized":
                print("x" * (256 * 1024 + 1), flush=True)
                continue
            if scenario == "eof":
                break
            if scenario in ("deadline", "cleanup_delay"):
                time.sleep(60)
                continue
            if scenario == "unexpected_request":
                send({"id": identifier, "method": "item/commandExecution/requestApproval", "params": {}})
                continue
            if scenario == "unrelated_id":
                send({"id": 999, "result": {}})
                continue
            account = None if scenario == "no_auth" else {"type": "apiKey"} if scenario == "api_key" else {"type": "chatgpt", "email": "fixture@example.test", "planType": "pro", "workspaceName": None}
            send({"id": identifier, "result": {"account": account, "userId": None, "requiresOpenaiAuth": True}})
        elif method == "config/read":
            assert request["params"] == {"includeLayers": False}
            if scenario == "config_failure":
                send({"id": identifier, "error": {"code": -32603, "message": "synthetic config failure"}})
            else:
                send({"id": identifier, "result": {"config": {"model_provider": "openai", "model": "fixture-model"}, "origins": {}, "layers": None}})
        else:
            record("forbidden_method", method=method)
            sys.exit(9)
finally:
    record("exit")
