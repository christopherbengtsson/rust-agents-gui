"""Opt-in disposable desktop fixture preparation; never part of the unit suite.

All product actions are taken through the actual desktop. A Git wrapper logs
commands and optionally adds a synthetic 1.5 s status delay for pending-state QA.
No provider executable is invoked. Pass `setup` once; `observe` retains read-only
private-state observations and never admits workspace or provider actions.
"""
from pathlib import Path
import hashlib
import json
import os
import shutil
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time

EVIDENCE = Path(__file__).resolve().parent
CONTEXT = EVIDENCE / 'desktop-context.json'


def git(root, *args):
    return subprocess.check_output([shutil.which('git'), '-C', str(root), *args], text=True).strip()


if sys.argv[1] == 'setup':
    if CONTEXT.exists():
        raise SystemExit('Fixture already exists; preserve its evidence')
    root = Path(tempfile.mkdtemp(prefix='agent-workspace-05c-', dir='/private/tmp'))
    root.chmod(0o700)
    repositories = []
    for name in ['Inspection Alpha', 'Inspection Beta']:
        repo = root / name
        repo.mkdir()
        git(repo, '-c', 'init.defaultBranch=main', 'init')
        (repo / 'text.txt').write_text('original text\n')
        (repo / 'delete.txt').write_text('delete this\n')
        (repo / 'empty.txt').touch()
        git(repo, 'add', '--', 'text.txt', 'delete.txt', 'empty.txt')
        git(repo, '-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', '-c', 'core.hooksPath=/dev/null', 'commit', '--no-gpg-sign', '-m', 'Disposable inspection baseline')
        repositories.append(str(repo))
    bin_dir = root / 'bin'
    bin_dir.mkdir()
    wrapper = bin_dir / 'git'
    actual_git = shutil.which('git')
    wrapper.write_text('''#!/usr/bin/python3
import json,os,sys,time
from pathlib import Path
root=Path(os.environ['INSPECTION_QUALIFICATION_ROOT'])
with (root/'git-commands.jsonl').open('a') as log:
    log.write(json.dumps({'time':time.time(),'pid':os.getpid(),'args':sys.argv[1:]})+'\\n')
if 'status' in sys.argv and (root/'delay-status').exists():
    (root/'status-delay-entered').write_text(str(os.getpid()))
    time.sleep(1.5)
os.execv(''' + repr(actual_git) + ''',['git',*sys.argv[1:]])
''')
    wrapper.chmod(0o700)
    context = {'root':str(root),'state':str(root/'state'),'repositories':repositories,'git':actual_git,'synthetic_faults':['optional 1.5 s Git status wrapper delay','chmod 000 selected disposable file','temporarily hidden disposable .git indirection'],'original_hashes':{str(Path(repo)/name):hashlib.sha256((Path(repo)/name).read_bytes()).hexdigest() for repo in repositories for name in ['text.txt','delete.txt','empty.txt']}}
    CONTEXT.write_text(json.dumps(context,indent=2)+'\n')
    print(json.dumps(context,indent=2))
elif sys.argv[1] == 'observe':
    context=json.loads(CONTEXT.read_text())
    state=Path(context['state'])
    payload={'protocol':5,'application_version':'0.1.0','after':None}
    with socket.socket(socket.AF_UNIX) as connection:
        connection.settimeout(5)
        connection.connect(str(state/'supervisor.sock'))
        connection.sendall((json.dumps(payload)+'\n').encode())
        data=b''
        while not data.endswith(b'\n'):
            part=connection.recv(65536)
            if not part: raise RuntimeError('Reply lost')
            data+=part
    context['snapshot']=json.loads(data)
    databases=list(state.glob('*.sqlite*'))
    db=next((p for p in databases if p.suffix not in ['.shm','.wal'] and not str(p).endswith(('-shm','-wal'))),None)
    if db:
        connection=sqlite3.connect(f'file:{db}?mode=ro',uri=True)
        connection.row_factory=sqlite3.Row
        tables=[r[0] for r in connection.execute("select name from sqlite_master where type='table'")]
        context['database']=str(db)
        context['table_counts']={table:connection.execute('select count(*) from "'+table+'"').fetchone()[0] for table in tables}
        context['workspaces']=[dict(row) for row in connection.execute('select * from workspaces')]
        connection.close()
    CONTEXT.write_text(json.dumps(context,indent=2)+'\n')
    print(json.dumps(context,indent=2))
else:
    raise SystemExit('usage: fixture.py setup|observe')
