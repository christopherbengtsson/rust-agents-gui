import json,os,subprocess,sys,time
from pathlib import Path
out=Path('docs/evidence/frontend-navigation/f04g')
name,*cmd=sys.argv[1:]
start=time.perf_counter()
with (out/(name+'.log')).open('w') as log:
 p=subprocess.run(cmd,stdout=log,stderr=subprocess.STDOUT)
r={'name':name,'command':cmd,'status':p.returncode,'elapsed_seconds':time.perf_counter()-start}
with (out/'checks.jsonl').open('a') as f:f.write(json.dumps(r)+'\n')
print(json.dumps(r));print((out/(name+'.log')).read_text()[-2500:]);sys.exit(p.returncode)
