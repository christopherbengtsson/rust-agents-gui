"""Explicit protocol-8 OS qualification; never selected by application unit gates.

Usage: python3 scripts/qualify-observations.py <supervisor> <results.json>
Uses only disposable /private/tmp state/repositories. Faults are synthetic,
application-owned peer bytes/delays, not native-provider evidence. Retains private
fixtures and failed results; terminates and joins only its own child process.
"""
import hashlib
import json
import os
from pathlib import Path
import select
import shutil
import socket
import sqlite3
import struct
import subprocess
import sys
import tempfile
import time
import uuid


def encode(value):
    payload = json.dumps(value, separators=(',', ':')).encode()
    return struct.pack('>I', len(payload)) + payload


def exact(peer, size):
    result = b''
    while len(result) < size:
        part = peer.recv(size - len(result))
        if not part:
            raise EOFError('incomplete frame')
        result += part
    return result


def receive(peer):
    size = struct.unpack('>I', exact(peer, 4))[0]
    assert size <= 65532, size
    return json.loads(exact(peer, size))


def main():
    executable = Path(sys.argv[1]).resolve(strict=True)
    output = Path(sys.argv[2])
    if output.exists():
        raise SystemExit("Preserve existing results; choose a new output filename")
    root = Path(tempfile.mkdtemp(prefix='aw-f04g-', dir='/private/tmp'))
    root.chmod(0o700)
    state = root / 'state'
    state.mkdir(mode=0o700)
    records = []
    process = subprocess.Popen([str(executable), str(state)], stdout=subprocess.PIPE,
                               stderr=(root / 'supervisor.log').open('w'))
    result = {'root': str(root), 'pid': process.pid, 'executable': str(executable),
              'sha256': hashlib.sha256(executable.read_bytes()).hexdigest(), 'checks': records}

    def save():
        output.write_text(json.dumps(result, indent=2) + '\n')

    def check(name, run):
        start = time.monotonic()
        try:
            detail = run()
            records.append({'name': name, 'status': 'pass', 'seconds': time.monotonic() - start,
                            'detail': detail})
        except BaseException as error:
            records.append({'name': name, 'status': 'fail', 'seconds': time.monotonic() - start,
                            'error': repr(error)})
            raise
        finally:
            save()
        print(json.dumps(records[-1]), flush=True)

    def connect():
        peer = socket.socket(socket.AF_UNIX)
        peer.settimeout(4)
        peer.connect(str(state / 'supervisor.sock'))
        return peer

    def request(**fields):
        return {'protocol': 8, 'application_version': '0.1.0', 'after': None, **fields}

    def exchange(value):
        with connect() as peer:
            peer.sendall(encode(value))
            return receive(peer)

    def closed(peer):
        try:
            assert peer.recv(1) == b'', 'unexpected data'
        except ConnectionResetError:
            pass

    def rejected_bytes(payload):
        with connect() as peer:
            peer.sendall(payload)
            peer.shutdown(socket.SHUT_WR)
            closed(peer)

    def subscribe(after=None):
        peer = connect()
        identity = str(uuid.uuid4())
        peer.sendall(encode(request(expected_owner=owner, subscribe={
            'subscription_id': identity, 'after': after})))
        return peer

    def ack(peer, batch):
        peer.sendall(encode({key: batch[key] for key in ['subscription_id', 'batch_id', 'cursor']}))

    def repo(**fields):
        response = exchange(request(expected_owner=owner, repository=fields))
        assert response['kind'] == 'repository', response
        assert 'Ok' in response['result'], response
        return response['result']['Ok']

    try:
        deadline = time.monotonic() + 8
        readiness = b''
        while b'\n' not in readiness:
            remaining = deadline - time.monotonic()
            assert remaining > 0 and select.select([process.stdout], [], [], remaining)[0], 'startup timeout'
            part = os.read(process.stdout.fileno(), 128 - len(readiness))
            assert part, 'supervisor exited before readiness'
            readiness += part
            assert len(readiness) < 128, 'unbounded readiness output'
        assert readiness == b'ready\n', readiness
        snapshot = exchange(request())['snapshot']
        owner = snapshot['cursor']['owner']
        result['owner'] = owner
        def permissions():
            assert state.stat().st_mode & 0o777 == 0o700
            assert (state / 'supervisor.sock').stat().st_mode & 0o777 == 0o600
            return {'directory': '0700', 'endpoint': '0600'}
        check('private_directory_and_endpoint', permissions)

        def fragmented():
            frame = encode(request())
            with connect() as peer:
                for part in [frame[:1], frame[1:3], frame[3:5], frame[5:]]:
                    peer.sendall(part)
                    time.sleep(0.02)
                assert receive(peer)['kind'] == 'snapshot'
        check('fragmented_header_and_body', fragmented)

        def limits():
            payload = json.dumps(request()).encode()
            with connect() as peer:
                peer.sendall(struct.pack('>I', 65532) + payload + b' ' * (65532-len(payload)))
                assert receive(peer)['kind'] == 'snapshot'
            rejected_bytes(struct.pack('>I', 65533))
            rejected_bytes(struct.pack('>I', 0xffffffff))
        check('exact_payload_limit_and_oversized_header', limits)
        for label, payload in [('partial_header', b'\0\0'), ('header_without_body', struct.pack('>I', 100)),
                               ('partial_body', struct.pack('>I', 100) + b'{'), ('empty_json', b'\0\0\0\0'),
                               ('legacy_v6_v7_newline', b'{"protocol":7}\n')]:
            check(label, lambda payload=payload: rejected_bytes(payload))

        def incompatible():
            for fields in [{'protocol': 6}, {'protocol': 7}, {'protocol': 999},
                           {'application_version': '99.0.0'}]:
                reply = exchange(request(**fields))
                assert reply['kind'] == 'incompatible', reply
            reply = exchange(request(expected_owner=str(uuid.uuid4()), repository={
                'operation': 'list', 'after': None, 'refresh': False}))
            assert reply['kind'] == 'transport' and reply['error']['code'] == 'owner_changed', reply
        check('incompatible_versions_and_owner_before_dispatch', incompatible)

        def slow():
            with connect() as peer:
                peer.sendall(struct.pack('>I', 100) + b'{')
                start = time.monotonic()
                for _ in range(3):
                    time.sleep(0.6)
                    peer.sendall(b' ')
                closed(peer)
                elapsed = time.monotonic() - start
                assert 1.8 <= elapsed < 3.5, elapsed
                return elapsed
        check('absolute_transfer_deadline_not_renewed_by_fragments', slow)

        def slots():
            peers = [connect() for _ in range(8)]
            try:
                for peer in peers:
                    peer.sendall(b'\0')
                # Status is the ninth accepted peer; the eight occupied handlers
                # must not wait for it or enqueue another handler.
                with connect() as ninth:
                    try:
                        ninth.sendall(encode(request()))
                    except BrokenPipeError:
                        pass
                    closed(ninth)
            finally:
                for peer in peers:
                    peer.close()
            # An actual snapshot proves handler slots became usable again.
            for _ in range(100):
                try:
                    assert exchange(request())['kind'] == 'snapshot'
                    return
                except (OSError, EOFError):
                    time.sleep(0.02)
            raise AssertionError('handlers did not drain')
        check('eight_server_clients_then_handler_recovery', slots)

        def heartbeat():
            with subscribe() as peer:
                baseline = receive(peer)['batch']
                assert baseline['sample']['kind'] == 'resync_required'
                ack(peer, baseline)
                start = time.monotonic()
                next_batch = receive_with_timeout(peer, 8)['batch']
                assert next_batch['sample']['kind'] == 'heartbeat'
                assert next_batch['cursor'] == baseline['cursor']
                assert next_batch['batch_id'] == baseline['batch_id'] + 1
                elapsed = time.monotonic() - start
                assert 4.5 <= elapsed < 7.5, elapsed
                ack(peer, next_batch)
                return {'seconds': elapsed, 'cursor': baseline['cursor']}
        check('baseline_exact_ack_and_five_second_heartbeat', heartbeat)

        def bad_ack():
            with subscribe() as peer:
                batch = receive(peer)['batch']
                batch['batch_id'] += 1
                ack(peer, batch)
                closed(peer)
        check('mismatched_ack_closes_subscription', bad_ack)

        git = shutil.which('git')
        repository = root / 'repository'
        repository.mkdir()
        subprocess.run([git, '-c', 'init.defaultBranch=main', 'init', str(repository)], check=True,
                       stdout=subprocess.DEVNULL)
        (repository/'text.txt').write_text('original\n')
        subprocess.run([git, '-C', str(repository), 'add', '.'], check=True)
        subprocess.run([git, '-C', str(repository), '-c', 'user.name=Fixture', '-c',
                        'user.email=fixture@example.invalid', '-c', 'core.hooksPath=/dev/null',
                        'commit', '--no-gpg-sign', '-m', 'Disposable baseline'], check=True,
                       stdout=subprocess.DEVNULL)

        def progress():
            with subscribe() as peer:
                baseline = receive(peer)['batch']
                accepted = baseline['cursor']
                ack(peer, baseline)
                action = str(uuid.uuid4())
                admission = repo(operation='register', request={'action_id': action,
                    'path': str(repository), 'create_initial_workspace': True})
                seen = []
                complete = False
                for _ in range(40):
                    batch = receive_with_timeout(peer, 8)['batch']
                    seen.append(batch)
                    assert len(encode(batch)) - 4 <= 4096
                    if batch['sample']['kind'] == 'hints':
                        assert len(batch['sample']['targets']) <= 32
                    ack(peer, batch)
                    current = repo(operation='action', action_id=action)['registration']
                    if current and current['state'] != 'pending':
                        complete = True
                        result['registration'] = current
                        break
                assert complete, seen
                # Do not use the newer status cursor; reconnect from acceptance.
                with subscribe(accepted) as reopened:
                    replay = receive_with_timeout(reopened, 8)['batch']
                    assert replay['cursor']['sequence'] > accepted['sequence']
                    assert replay['sample']['kind'] == 'hints', replay
                    ack(reopened, replay)
                return {'admission': admission, 'batches': seen, 'accepted_reconnect': accepted}
        check('repository_workspace_push_progress_and_accepted_cursor_reconnect', progress)

        def blocking_admission():
            # Lock only this disposable database. Production repository reads
            # enter real synchronous SQLite work, then wait on its lock/core
            # mutex. Closing their peers must not release handler capacity.
            database = sqlite3.connect(state / 'workspace.sqlite3', timeout=5)
            database.execute('BEGIN EXCLUSIVE')
            peers = []
            try:
                for _ in range(8):
                    peer = connect()
                    peer.sendall(encode(request(expected_owner=owner, repository={
                        'operation': 'list', 'after': None, 'refresh': False})))
                    peers.append(peer)
                time.sleep(0.015)
                for peer in peers:
                    peer.close()
                time.sleep(0.005)
                with connect() as ninth:
                    try:
                        ninth.sendall(encode(request()))
                    except BrokenPipeError:
                        pass
                    closed(ninth)
            finally:
                for peer in peers:
                    peer.close()
                database.rollback()
                database.close()
            for _ in range(200):
                try:
                    assert exchange(request())['kind'] == 'snapshot'
                    return {'blocked_handlers': 8, 'capacity_retained_after_peer_close': True,
                            'recovered_after_database_unlock': True}
                except (OSError, EOFError):
                    time.sleep(0.01)
            raise AssertionError('blocking handlers did not physically complete')
        check('started_blocking_work_keeps_physical_admission_after_peer_close', blocking_admission)

        def inspection():
            def workspace(**fields):
                reply = exchange(request(expected_owner=owner, workspace=fields))
                assert reply['kind'] == 'workspace' and 'Ok' in reply['result'], reply
                return reply['result']['Ok']
            identity = result['registration']['initial_workspace_id']
            for _ in range(200):
                record = workspace(operation='get', id=identity)['workspace']
                if record['status']['state'] == 'ready':
                    break
                time.sleep(0.01)
            else:
                raise AssertionError('workspace did not become ready')
            path = Path(record['path']) / 'text.txt'
            path.write_text(''.join('changed ' + 'x' * 180 + '\n' for _ in range(250)))
            target = dict(operation='changes', workspace_id=identity, comparison_id=None, after=None)
            workspace(**target, refresh=True)
            for _ in range(300):
                observed = workspace(**target, refresh=False)['observation']
                if observed['phase'] == 'ready':
                    break
                time.sleep(0.01)
            else:
                raise AssertionError(observed)
            page = observed['page']
            assert page['total'] == 1, page
            preview_target = {'workspace_id': identity, 'comparison_id': page['comparison_id'],
                              'file_id': page['files'][0]['id']}
            workspace(operation='change_preview', target=preview_target, load=True)
            for _ in range(300):
                preview = workspace(operation='change_preview', target=preview_target, load=False)['observation']
                if preview['phase'] == 'ready':
                    break
                time.sleep(0.01)
            else:
                raise AssertionError(preview)
            assert preview['preview']['limited'] and len(preview['preview']['rows']) <= 200
            frame = request(expected_owner=owner, workspace={'operation': 'change_preview',
                             'target': preview_target, 'load': False})
            reply = exchange(frame)
            assert len(encode(reply)) <= 65536
            # Real Unix socket backpressure. Keep a small receive window unread
            # until the server's absolute write budget has expired.
            with connect() as peer:
                peer.setsockopt(socket.SOL_SOCKET, socket.SO_RCVBUF, 1024)
                peer.sendall(encode(frame))
                time.sleep(2.3)
                data = b''
                while True:
                    chunk = peer.recv(65536)
                    if not chunk:
                        break
                    data += chunk
                assert len(data) >= 4
                advertised = struct.unpack('>I', data[:4])[0]
                assert advertised <= 65532
                assert len(data)-4 < advertised, (len(data), advertised)
            assert exchange(request())['kind'] == 'snapshot'
            return {'rows': len(preview['preview']['rows']), 'wire_bytes': len(encode(reply)),
                    'partial_write_bytes': len(data), 'advertised_payload': advertised,
                    'small_receive_buffer': 1024}
        check('inspection_push_result_large_preview_and_absolute_partial_write', inspection)

        def overflow():
            with subscribe() as peer:
                baseline = receive(peer)['batch']
                for _ in range(140):
                    repo(operation='list', after=None, refresh=True)
                    for _ in range(100):
                        page = repo(operation='list', after=None, refresh=False)['page']
                        if all(item['availability']['state'] != 'checking' for item in page['repositories']):
                            break
                        time.sleep(0.005)
                    else:
                        raise AssertionError('availability did not settle')
                # One delivery in flight: no second bytes before baseline ack.
                assert not select.select([peer], [], [], 0.2)[0]
                ack(peer, baseline)
                batch = receive(peer)['batch']
                assert batch['sample']['kind'] == 'resync_required', batch
                ack(peer, batch)
                return batch
        check('one_inflight_and_256_entry_overflow_resync', overflow)

        def stalled():
            with subscribe() as peer:
                receive(peer)
                start = time.monotonic()
                peer.settimeout(18)
                closed(peer)
                elapsed = time.monotonic()-start
                assert 14 <= elapsed < 17, elapsed
                return elapsed
        check('unacknowledged_delivery_closes_at_15_seconds', stalled)
        check('owner_alive_after_fault_cohort', lambda: exchange(request())['snapshot'])
    finally:
        process.terminate()
        try:
            process.wait(timeout=8)
        except subprocess.TimeoutExpired:
            process.kill()
            process.wait(timeout=8)
        result['cleanup'] = {'child_joined': process.returncode is not None,
                             'returncode': process.returncode}
        save()


def receive_with_timeout(peer, timeout):
    peer.settimeout(timeout)
    return receive(peer)


if __name__ == '__main__':
    main()
