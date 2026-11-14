"""Opt-in application-owned lost-reply fault, protocol 8 only.

Run with a disposable state directory and its freshly observed supervisor PID.
Relays bounded frames unchanged. A drop-next-intent marker drops one real admitted
reply and pauses that exact private owner for five seconds. Never replays requests.
SIGTERM closes/joins proxy workers and restores the original endpoint name.
"""
import json
import os
from pathlib import Path
import signal
import socket
import struct
import subprocess
import sys
import threading
import time

state = Path(sys.argv[1]).resolve(strict=True)
owner_pid = int(sys.argv[2])
assert str(state).startswith('/private/tmp/aw-f04g-')
assert state.stat().st_uid == os.getuid() and state.stat().st_mode & 0o777 == 0o700
endpoint = state / 'supervisor.sock'
upstream = state / 'supervisor.upstream.sock'
assert not upstream.exists()
assert str(state) in subprocess.check_output(['ps', '-p', str(owner_pid), '-o', 'command='], text=True)
listener = socket.socket(socket.AF_UNIX)
renamed = False
bound = False
stop = threading.Event()
slots = threading.BoundedSemaphore(8)
workers = []
resumers = []


def log(**fields):
    print(json.dumps({'time': time.time(), **fields}), flush=True)


def frame(peer):
    def read(size):
        value = b''
        while len(value) < size:
            part = peer.recv(size - len(value))
            if not part:
                raise EOFError()
            value += part
        return value
    header = read(4)
    size = struct.unpack('>I', header)[0]
    assert size <= 65532
    return header + read(size)


def resume():
    # This exact executable/state was observed immediately before the fault.
    stop.wait(5)
    try:
        command = subprocess.check_output(['ps', '-p', str(owner_pid), '-o', 'command='], text=True)
        if str(state) in command:
            os.kill(owner_pid, signal.SIGCONT)
            log(event='owner-resumed', pid=owner_pid)
    except (subprocess.CalledProcessError, ProcessLookupError):
        pass


def serve(client):
    try:
        with client, socket.socket(socket.AF_UNIX) as server:
            client.settimeout(2)
            server.settimeout(2)
            request = frame(client)
            value = json.loads(request[4:])
            server.connect(str(upstream))
            server.sendall(request)
            target = value.get('workspace') or value.get('repository') or {}
            operation = target.get('operation')
            marker = state / 'drop-next-intent'
            if operation in ['register', 'create', 'reconcile'] and marker.exists():
                marker.unlink()
                reply = frame(server)
                admitted = json.loads(reply[4:])
                assert admitted.get('owner') == value['expected_owner'], admitted
                assert 'Ok' in admitted.get('result', {}), admitted
                body = admitted['result']['Ok']
                if operation in ['create', 'reconcile']:
                    assert body['action']['action_id'] == target['action_id'], body
                    assert body['workspace'] is not None, body
                else:
                    assert body['registration']['action_id'] == target['request']['action_id'], body
                log(event='real-intent-reply-dropped', request=value, reply=admitted)
                os.kill(owner_pid, signal.SIGSTOP)
                worker = threading.Thread(target=resume)
                resumers.append(worker)
                worker.start()
                return
            if not value.get('subscribe'):
                reply = frame(server)
                client.sendall(reply)
                log(event='exchange', operation=operation, action_id=target.get('action_id'))
                return
            log(event='subscription', after=value['subscribe']['after'])
            # One framed delivery and its exact frontend acknowledgement at a time.
            # Timeouts are retryable only while no frame bytes have been consumed;
            # socket readiness avoids restarting a partially read frame.
            import select
            while not stop.is_set():
                if not select.select([server], [], [], 0.2)[0]:
                    continue
                client.sendall(frame(server))
                deadline = time.monotonic() + 16
                while not select.select([client], [], [], 0.2)[0]:
                    if stop.is_set() or time.monotonic() >= deadline:
                        return
                server.sendall(frame(client))
    except (OSError, EOFError, AssertionError, ValueError) as error:
        log(event='peer-completed', detail=type(error).__name__)
    finally:
        slots.release()


def shutdown(*_):
    stop.set()


signal.signal(signal.SIGTERM, shutdown)
signal.signal(signal.SIGINT, shutdown)
try:
    endpoint.rename(upstream)
    renamed = True
    listener.bind(str(endpoint))
    bound = True
    endpoint.chmod(0o600)
    listener.listen(8)
    listener.settimeout(0.2)
    log(event='ready', state=str(state), owner_pid=owner_pid)
    while not stop.is_set():
        try:
            client, _ = listener.accept()
        except socket.timeout:
            continue
        workers = [worker for worker in workers if worker.is_alive()]
        if not slots.acquire(blocking=False):
            client.close()
            continue
        worker = threading.Thread(target=serve, args=(client,))
        workers.append(worker)
        worker.start()
finally:
    stop.set()
    listener.close()
    for worker in workers:
        worker.join(timeout=20)
        assert not worker.is_alive(), 'proxy worker did not join'
    for worker in resumers:
        worker.join(timeout=8)
        assert not worker.is_alive(), 'owner resume did not join'
    if bound:
        endpoint.unlink()
    if renamed:
        upstream.rename(endpoint)
    log(event='stopped-and-joined')
