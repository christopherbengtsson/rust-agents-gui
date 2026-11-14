import json,statistics,subprocess,time,sys
from pathlib import Path
out=Path('docs/evidence/frontend-navigation/f04g')
output=Path(sys.argv[1])
if output.exists():raise SystemExit('Choose a fresh result path; retain existing evidence')
start=time.perf_counter();built=subprocess.check_output(['node','scripts/cargo.mjs','test','--locked','-p','workspace-supervisor','--features','process-qualification','--test','startup','--test','discovery','--no-run','--message-format=json-render-diagnostics'],text=True)
compilation=time.perf_counter()-start
artifacts=[json.loads(l) for l in built.splitlines() if l.startswith('{')];exes=[x['executable'] for x in artifacts if x.get('reason')=='compiler-artifact' and x.get('profile',{}).get('test') and x.get('executable') and x['target']['kind']==['test']]
assert len(exes)==2,exes
runs=[]
for i in range(6):
 start=time.perf_counter()
 for exe in exes:subprocess.run([exe,'--test-threads=1'],check=True)
 elapsed=time.perf_counter()-start;runs.append(elapsed);print('warm-up' if i==0 else f'run {i}',elapsed,flush=True)
result={'compilation_seconds':compilation,'executables':exes,'threads':1,'warmup':runs[0],'runs':runs[1:],'median':statistics.median(runs[1:]),'range':[min(runs[1:]),max(runs[1:])]};output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
