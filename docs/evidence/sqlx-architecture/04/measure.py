#!/usr/bin/env python3
"""Measure the already compiled isolated Rust unit executable; no build or provider calls."""
import hashlib,json,pathlib,re,statistics,subprocess,time
out=pathlib.Path(__file__).resolve().parent
log=(out/'complete-verify-final.log').read_text()
executable=re.search(r'Unit execution 1: [\d.]+ s; (.+workspace_core-[a-z0-9]+) --test-threads=1',log)[1]
report={'executable':executable,'sha256':hashlib.sha256(pathlib.Path(executable).read_bytes()).hexdigest(),'profile':'warm default target, test profile, one harness thread','runs':[]}
for index in range(6):
 start=time.monotonic();result=subprocess.run([executable,'--test-threads=1'],text=True,capture_output=True);elapsed=time.monotonic()-start
 assert result.returncode==0,result.stdout+result.stderr
 report['runs'].append({'warmup':index==0,'exit':result.returncode,'seconds':elapsed,'result':re.search(r'test result:.*',result.stdout)[0]})
values=[run['seconds'] for run in report['runs'][1:]]
report.update(median=statistics.median(values),range=[min(values),max(values)])
(out/'unit-measurements.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
