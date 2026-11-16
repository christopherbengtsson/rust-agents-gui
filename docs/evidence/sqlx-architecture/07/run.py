#!/usr/bin/env python3
"""Sequential ticket-07 evidence recorder; commands are explicit, never live providers."""
import hashlib, json, os, pathlib, subprocess, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[4]
OUT = pathlib.Path(__file__).resolve().parent
record = OUT / 'verification.json'
report = json.loads(record.read_text()) if record.exists() else {'checks': []}
label, *command = sys.argv[1:]
assert command and not (OUT / (label + '.log')).exists(), 'Preserve prior attempts'
env = dict(os.environ)
env['PATH'] = str(pathlib.Path.home()/'.local/share/fnm/node-versions/v26.13.1/installation/bin') + os.pathsep + env['PATH']
start = time.monotonic()
with (OUT / (label + '.log')).open('w') as log:
    result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
report['checks'].append({'label': label, 'command': command, 'exit': result.returncode, 'seconds': time.monotonic()-start,
    'source': subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    'dirty': subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True),
    'source_hashes': {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted({p for folder in ['crates/core/src', 'src-tauri/src', 'src', 'scripts'] for p in (ROOT/folder).rglob('*') if p.is_file() and p.suffix in ['.rs','.ts','.tsx','.mjs','.js','.py']})}})
record.write_text(json.dumps(report,indent=2)+'\n')
subprocess.run(['corepack','pnpm','exec','oxfmt','--write',str(record)],cwd=ROOT,env=env,stdout=subprocess.DEVNULL,check=True)
print(label, 'exit', result.returncode, 'seconds', round(time.monotonic()-start,3), flush=True)
sys.exit(result.returncode)
