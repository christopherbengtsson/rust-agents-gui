#!/usr/bin/env python3
"""Sequential evidence capture. Never rewrites an earlier command result."""
import datetime, hashlib, json, pathlib, subprocess, sys, time
ROOT = pathlib.Path(__file__).resolve().parents[4]
OUT = pathlib.Path(__file__).resolve().parent
label, *command = sys.argv[1:]
log = OUT / (label + '.log')
if log.exists(): raise SystemExit('Choose a new label; preserve earlier attempts')
start = time.monotonic()
with log.open('x') as output:
    result = subprocess.run(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT)
entry = {'label':label, 'command':command, 'exit':result.returncode, 'seconds':time.monotonic()-start,
 'date':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
 'dirty':subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True),
 'diff_sha256':hashlib.sha256(subprocess.check_output(['git','diff','HEAD'],cwd=ROOT)).hexdigest()}
with (OUT/'commands.jsonl').open('a') as output: output.write(json.dumps(entry)+'\n')
print(json.dumps({key:entry[key] for key in ['label','exit','seconds']}),flush=True)
raise SystemExit(result.returncode)
