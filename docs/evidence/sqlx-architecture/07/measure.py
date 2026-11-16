#!/usr/bin/env python3
"""Already compiled isolated Rust units: one warm-up, five warm runs, no build overlap."""
import hashlib,json,pathlib,re,statistics,subprocess,time
OUT=pathlib.Path(__file__).resolve().parent;ROOT=OUT.parents[3]
log=(OUT/'complete-final-current.log').read_text()
executable=re.search(r'Unit execution 1: [\d.]+ s; (.*?) --test-threads=1',log).group(1)
report={'command':[executable,'--test-threads=1'],'executable_sha256':hashlib.sha256(pathlib.Path(executable).read_bytes()).hexdigest(),'profile':'warm test, codegen enabled; one harness thread','source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'runs':[]}
for index in range(6):
 started=time.monotonic();result=subprocess.run(report['command'],cwd=ROOT,capture_output=True,text=True);elapsed=time.monotonic()-started
 (OUT/f'unit-measure-{index}.log').write_text(result.stdout+result.stderr)
 count=int(re.search(r'test result: ok\. (\d+) passed',result.stdout).group(1))
 assert result.returncode==0 and count==172
 report['runs'].append({'warmup':index==0,'seconds':elapsed,'exit':result.returncode,'passed':count})
samples=[r['seconds'] for r in report['runs'][1:]];report.update(median=statistics.median(samples),minimum=min(samples),maximum=max(samples),compilation='separate complete gate; fresh-private-target compilation not repeated')
(OUT/'unit-measurements.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ['median','minimum','maximum']}))
