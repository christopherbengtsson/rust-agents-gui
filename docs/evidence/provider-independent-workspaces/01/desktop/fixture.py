"""Explicit disposable desktop qualification host, outside automated test gates.

Run from workspace root. Commands on stdin are recorded; this host retains and
joins its exact production supervisor/desktop children. No provider is started.
"""
from pathlib import Path
import hashlib
import json
import os
import plistlib
import shutil
import signal
import socket
import sqlite3
import struct
import subprocess
import sys
import tempfile
import time

out = Path(__file__).resolve().parent
git_only = '--git-only' in sys.argv[1:]
if git_only:
    out = out / 'no-providers'
    out.mkdir(exist_ok=False)
root = Path(tempfile.mkdtemp(prefix='aw-piw-', dir='/private/tmp'))
root.chmod(0o700)
state = root / 'state'
state.mkdir(mode=0o700)
bundle = root / 'PIW Qualification.app'
shutil.copytree('target/release/bundle/macos/Agent Workspace.app', bundle)
plist = bundle / 'Contents/Info.plist'
info = plistlib.loads(plist.read_bytes())
info.update(CFBundleIdentifier='dev.christopherbengtsson.piw.qualification',
            CFBundleName='PIW Qualification')
if git_only:
    info['CFBundleIdentifier'] += '.noproviders'
plist.write_bytes(plistlib.dumps(info))
desktop_exe = bundle / 'Contents/MacOS/workspace-desktop'
supervisor_exe = bundle / 'Contents/Resources/workspace-supervisor'
env = {k: v for k, v in os.environ.items() if k not in
       ('AGENT_WORKSPACE_SUPERVISOR', 'AGENT_WORKSPACE_STATE_DIR')}
env['AGENT_WORKSPACE_STATE_DIR'] = str(state)
if git_only:
    bin_dir = root / 'bin'
    bin_dir.mkdir(mode=0o700)
    (bin_dir / 'git').symlink_to(shutil.which('git'))
    env['PATH'] = str(bin_dir)
context = {'root': str(root), 'state': str(state), 'bundle': str(bundle),
           'repositories': [], 'original_files': {}, 'processes': [],
           'commands': [], 'hashes': {str(p.relative_to(bundle)):
           hashlib.sha256(p.read_bytes()).hexdigest() for p in
           (desktop_exe, supervisor_exe)},
           'revision': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
           'sidecar_provenance': json.loads(Path('src-tauri/gen/sidecar/provenance.json').read_text()),
           'protocol': int(subprocess.check_output([str(supervisor_exe), '--protocol-version'], text=True, timeout=5))}
if git_only:
    context['runtime_path'] = env['PATH']
    context['provider_commands'] = {name: shutil.which(name, path=env['PATH'])
                                    for name in ['codex', 'claude', 'opencode']}
    assert not any(context['provider_commands'].values())

def save():
    (out / 'context.json').write_text(json.dumps(context, indent=2) + '\n')

def run(args):
    r = subprocess.run(args, text=True, capture_output=True)
    context['commands'].append({'argv': args, 'returncode': r.returncode,
                                'stdout': r.stdout, 'stderr': r.stderr})
    save()
    r.check_returncode()
    return r.stdout.strip()

for name in ['Alpha', 'Beta', 'Ambiguous']:
    repo = root / name
    run(['git', 'init', '--initial-branch=main', str(repo)])
    (repo / 'tracked.txt').write_text('committed fixture line\n')
    run(['git', '-C', str(repo), 'add', '.'])
    run(['git', '-C', str(repo), '-c', 'user.name=Fixture', '-c',
         'user.email=fixture@example.invalid', '-c', 'core.hooksPath=/dev/null',
         'commit', '--no-gpg-sign', '-m', 'Disposable provider-independent base'])
    commit = run(['git', '-C', str(repo), 'rev-parse', 'HEAD'])
    if name == 'Ambiguous':
        run(['git', '-C', str(repo), 'branch', 'master'])
    (repo / 'tracked.txt').write_text('dirty original checkout; preserve me\n')
    (repo / 'original-only.txt').write_text('untracked original; preserve me\n')
    for p in repo.glob('*.txt'):
        context['original_files'][str(p)] = hashlib.sha256(p.read_bytes()).hexdigest()
    context['repositories'].append({'name': name, 'path': str(repo), 'base': commit})

children = {}

def launch(kind):
    exe = supervisor_exe if kind == 'supervisor' else desktop_exe
    args = [str(exe), str(state)] if kind == 'supervisor' else [str(exe)]
    with (out / (kind + '.log')).open('a') as log:
        p = subprocess.Popen(args, env=env, stdout=log, stderr=subprocess.STDOUT)
    children[kind] = p
    context['processes'].append({'kind': kind, 'pid': p.pid, 'argv': args})
    save()
    if kind == 'supervisor':
        for _ in range(100):
            if (state / 'supervisor.sock').exists():
                break
            assert p.poll() is None
            time.sleep(.05)
    return p.pid

def exact(peer, size):
    data = b''
    while len(data) < size:
        part = peer.recv(size - len(data))
        assert part, 'incomplete response'
        data += part
    return data

def snapshot():
    data = json.dumps({'protocol': 9, 'application_version': '0.1.0', 'after': None}).encode()
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(4)
        peer.connect(str(state / 'supervisor.sock'))
        peer.sendall(struct.pack('>I', len(data)) + data)
        return json.loads(exact(peer, struct.unpack('>I', exact(peer, 4))[0]))

def capture(label):
    result = {'snapshot': snapshot()}
    db = state / 'workspace.sqlite3'
    if db.exists():
        with sqlite3.connect(f'file:{db}?mode=ro', uri=True) as conn:
            result['tables'] = {table: [dict(zip([c[0] for c in cur.description], row))
                for row in cur] for table in ['repositories', 'registration_actions',
                'workspaces', 'workspace_actions', 'repository_folders']
                for cur in [conn.execute('SELECT * FROM ' + table + ' ORDER BY 1')]}
    result['processes'] = {k: {'pid': p.pid, 'poll': p.poll(), 'ps':
        subprocess.run(['ps', '-p', str(p.pid), '-o', 'pid=,ppid=,command='],
        text=True, capture_output=True).stdout} for k, p in children.items()}
    result['original_files_unchanged'] = {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() == h
                                         for p, h in context['original_files'].items()}
    (out / (label + '.json')).write_text(json.dumps(result, indent=2) + '\n')
    return result

def join(kind, terminate=False):
    p = children[kind]
    expected = context['processes'][next(i for i in range(len(context['processes'])-1, -1, -1)
                                       if context['processes'][i]['pid'] == p.pid)]['argv']
    if terminate and p.poll() is None:
        command = subprocess.check_output(['ps', '-p', str(p.pid), '-o', 'command='], text=True).strip()
        assert command == ' '.join(expected), (command, expected)
        if kind == 'supervisor':
            assert snapshot()['snapshot']['status']['supervisor_pid'] == p.pid
        p.send_signal(signal.SIGTERM)
    code = p.wait(timeout=12)
    context['commands'].append({'join': kind, 'pid': p.pid, 'returncode': code,
                                'verified_before_signal': terminate})
    save()
    return code

launch('supervisor')
launch('desktop')
print(json.dumps({'ready': context}), flush=True)
for line in sys.stdin:
    try:
        command = json.loads(line)
        context['commands'].append({'control': command, 'time': time.time()})
        save()
        op = command['op']
        if op == 'capture':
            result = capture(command['label'])
            result = {'workspace_count': len(result.get('tables', {}).get('workspaces', [])),
                      'snapshot': result['snapshot']}
        elif op == 'join-desktop':
            result = join('desktop')
        elif op == 'launch-desktop':
            result = launch('desktop')
        elif op == 'restart-supervisor':
            result = {'old_returncode': join('supervisor', True), 'new_pid': launch('supervisor')}
        elif op == 'repair-base':
            repo = root / 'Ambiguous'
            result = run(['git', '-C', str(repo), 'branch', '-m', 'master', 'fixture-spare'])
        elif op == 'edit-workspace':
            with sqlite3.connect(f'file:{state / "workspace.sqlite3"}?mode=ro', uri=True) as conn:
                workspace = json.loads(conn.execute('SELECT record FROM workspaces WHERE id=?',
                                       [command['id']]).fetchone()[0])
            path = Path(workspace['path'])
            assert str(path).startswith(str(state) + '/'), path
            (path / 'tracked.txt').write_text('workspace tracked edit\n')
            (path / 'new-file.txt').write_text('workspace new file\n')
            result = {'path': str(path)}
        elif op == 'cleanup':
            join('desktop', True)
            join('supervisor', True)
            result = {'remaining_fixture_processes': [line for line in
                subprocess.check_output(['ps', '-axo', 'pid,ppid,command'], text=True).splitlines()
                if str(root) in line], 'retained_root': str(root)}
            (out / 'cleanup.json').write_text(json.dumps(result, indent=2) + '\n')
            print(json.dumps(result), flush=True)
            break
        else:
            raise ValueError(op)
        print(json.dumps({'op': op, 'result': result}), flush=True)
    except Exception as error:
        print(json.dumps({'error': repr(error)}), flush=True)
