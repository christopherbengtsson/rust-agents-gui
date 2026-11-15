"""Read-only audit of this retained actual desktop qualification cohort."""
from pathlib import Path
import hashlib
import json
import re
import socket
import sqlite3
import struct
import subprocess

out = Path(__file__).resolve().parent
context = json.loads((out / 'context.json').read_text())
state = Path(context['state'])
commands = []

def run(argv):
    r = subprocess.run(argv, capture_output=True, text=True)
    commands.append({'argv': argv, 'exit_code': r.returncode,
                     'stdout': r.stdout, 'stderr': r.stderr})
    r.check_returncode()
    return r.stdout.strip()

def read(label):
    return json.loads((out / (label + '.json')).read_text())

def exact(peer, length):
    data = b''
    while len(data) < length:
        part = peer.recv(length-len(data))
        assert part
        data += part
    return data

def exchange(**fields):
    request = {'protocol': 9, 'application_version': '0.1.0', 'after': None, **fields}
    data = json.dumps(request).encode()
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(4)
        peer.connect(str(state / 'supervisor.sock'))
        peer.sendall(struct.pack('>I', len(data)) + data)
        reply = json.loads(exact(peer, struct.unpack('>I', exact(peer, 4))[0]))
    commands.append({'native_request': request, 'native_reply': reply})
    return reply

before = read('08-failed-before-quit')
reopen = read('12-failed-after-reopen')
cold_before = read('15-before-supervisor-restart')
cold_after = read('16b-after-supervisor-restart')
repaired = read('18-base-repaired-no-auto-reconcile')
reconciled = read('21-after-explicit-reconcile')
final_before = read('26-ready-before-final-restart')
final_after = read('27-ready-after-final-restart')
final = read('31-final-durable-state')
assert before['tables'] == reopen['tables'] == cold_before['tables']
assert cold_before['tables'] == cold_after['tables'] == repaired['tables']
assert final_before['tables'] == final_after['tables'] == final['tables']
records = lambda capture: {r['id']: json.loads(r['record']) for r in capture['tables']['workspaces']}
old, new = records(before), records(final)
assert set(old) == set(new) and len(new) == 3
failed_id = 'e3afb8c7-c5bd-407d-bbd4-fb1bcf4c81e2'
assert old[failed_id]['status']['state'] == 'failed'
assert old[failed_id]['base'] is None
assert old[failed_id]['artifacts'] == {'branch': 'unknown', 'folder': 'unknown'}
assert old[failed_id]['cleanup_unconfirmed'] is False
for identity, record in new.items():
    assert 'provider' not in record
    assert record['status']['state'] == 'ready'
    for key in ['id', 'repository_id', 'creation_action_id', 'name', 'branch', 'path']:
        assert old[identity][key] == record[key]
    if identity != failed_id:
        assert old[identity] == record
assert len(final['tables']['workspace_actions']) == len(before['tables']['workspace_actions']) + 1
actions = [json.loads(r['record']) for r in final['tables']['workspace_actions']]
assert sum(a['operation'] == 'create' for a in actions) == 3
assert sum(a['operation'] == 'reconcile' for a in actions) == 1
assert all(final['original_files_unchanged'].values())
snapshot = exchange()['snapshot']
owner = snapshot['cursor']['owner']
assert snapshot['status']['supervisor_pid'] == 41501
observed = {}
for identity in new:
    reply = exchange(expected_owner=owner, workspace={'operation': 'get', 'id': identity})
    record = reply['result']['Ok']['workspace']
    assert record == new[identity] and 'provider' not in record
    observed[identity] = record
lists = []
for repo in final['tables']['repositories']:
    reply = exchange(expected_owner=owner, workspace={'operation': 'list', 'repository_id': repo['id'], 'after': None})
    page = reply['result']['Ok']['page']
    assert page['next'] is None
    assert {w['id'] for w in page['workspaces']} == {w['id'] for w in new.values() if w['repository_id'] == repo['id']}
    assert all('provider' not in w for w in page['workspaces'])
    lists.append(page)
git_facts = []
for record in new.values():
    path = record['path']
    branch = run(['git', '-C', path, 'symbolic-ref', '--short', 'HEAD'])
    head = run(['git', '-C', path, 'rev-parse', 'HEAD'])
    assert branch == record['branch'] and head == record['base']['commit']
    git_facts.append({'workspace_id': record['id'], 'path': path, 'branch': branch, 'commit': head,
                      'status': run(['git', '-C', path, 'status', '--porcelain=v1'])})
for repo in context['repositories']:
    run(['git', '-C', repo['path'], 'worktree', 'list', '--porcelain'])
    run(['git', '-C', repo['path'], 'show-ref'])
    run(['git', '-C', repo['path'], 'remote', '-v'])
with sqlite3.connect(f'file:{state / "workspace.sqlite3"}?mode=ro', uri=True) as conn:
    indexes = conn.execute("SELECT name,sql FROM sqlite_master WHERE type='index' AND name LIKE 'reserved_workspace_%' ORDER BY name").fetchall()
assert len(indexes) == 3
source_paths = run(['git', 'ls-files', 'src', 'src-tauri', 'crates', 'scripts', 'Cargo.toml', 'Cargo.lock',
                    'package.json', 'pnpm-lock.yaml', 'vite.config.ts', 'tsconfig.json']).splitlines()
source_hashes = {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in source_paths if Path(p).is_file()}
(out / 'source-sha256.json').write_text(json.dumps(source_hashes, indent=2) + '\n')
screens = {}
for p in out.glob('*.jpg'):
    description = run(['file', str(p)])
    width, height = re.search(r'precision 8, (\d+)x(\d+),', description).groups()
    screens[p.name] = {'width': int(width), 'height': int(height)}
assert screens['07-failed-ordinary.jpg'] == {'width': 1800, 'height': 1360}
assert screens['19-fixed-base-still-failed-keyboard.jpg'] == {'width': 1200, 'height': 960}
ideas_hash = hashlib.sha256(Path('docs/IDEAS.md').read_bytes()).hexdigest()
assert ideas_hash == read('before')['ideas_sha256']
result = {'result': 'pass', 'identical_tables_after_desktop_reopen': True,
          'identical_tables_after_failed_owner_restart': True,
          'identical_tables_after_recovered_owner_restart': True,
          'all_original_files_unchanged': True, 'ideas_sha256_unchanged': ideas_hash,
          'same_workspace_reservations_and_creation_actions': True,
          'counts': {k: len(v) for k, v in final['tables'].items()},
          'reservation_indexes': indexes, 'workspace_observations': observed,
          'workspace_pages': lists, 'git_facts': git_facts, 'screenshots': screens,
          'commands': commands}
(out / 'audit.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps({k: v for k, v in result.items() if k not in
                 ['commands', 'workspace_observations', 'workspace_pages', 'screenshots', 'git_facts']}, indent=2))
