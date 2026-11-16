#!/usr/bin/env python3
"""Summarize retained short experiments; medians/ranges across five warm runs."""
import collections,json,pathlib,statistics,sys
base=pathlib.Path(__file__).resolve().parent.parent
groups=collections.defaultdict(list);failures=[];lifecycle=[];provenance=[]
for path in sorted(base.glob('driver-results-*.jsonl')):
 for line in path.read_text().splitlines():
  v=json.loads(line)
  if v.get('phase')=='provenance':provenance.append({'file':path.name,**v});continue
  if v.get('exit_code') not in [None,0]:failures.append({'file':path.name,**v})
  if v.get('phase')=='lifecycle':lifecycle.append(v)
  if v.get('phase') not in ['screen','finalists','scale','resources']:continue
  if path.name=='driver-results-resources.jsonl' and v.get('phase')=='resources':continue
  if v.get('repeat')==0 and (v['phase']=='finalists' or v['label'].startswith('history-')):continue
  groups[(v['phase'],v['driver'],v['label'])].append(v)
def at(d,path):
 for key in path.split('.'):
  if not isinstance(d,dict)or key not in d:return None
  d=d[key]
 return d
metrics=['events_per_s','elapsed_s','admitted_transactions','rejected_transactions','committed_events','queue.p50_ms','queue.p95_ms','queue.p99_ms','service.p50_ms','service.p95_ms','service.p99_ms','history.p50_ms','history.p95_ms','history.p99_ms','reader_history.p95_ms','batch_formation.p50_ms','batch_formation.p95_ms','batch_formation.p99_ms','sampled_main_wal_shm_bytes','peak_db_sidecar_bytes','closed_bytes','writer_drain_ms','writer_close_ms','drain_ms','close_ms']
summary=[]
for(phase,driver,label),vs in groups.items():
 row={'phase':phase,'driver':driver,'label':label,'samples':len(vs),'metrics':{},'all_integrity_ok':all(v['result']['integrity']=='ok'for v in vs),'writer_errors':sum(len(v['result']['errors'])for v in vs),'reader_errors':sum(v['result']['reader_errors']for v in vs)}
 for metric in metrics:
  vals=[x for v in vs if (x:=at(v['result'],metric))is not None]
  if vals:row['metrics'][metric]={'median':statistics.median(vals),'min':min(vals),'max':max(vals)}
 summary.append(row)
result={'excluded_cohort':'driver-results-resources.jsonl raw thread counter included ps header; use resources-corrected cohort', 'note':'Run zero excluded for finalists and repeated history scale cases. Screen/active sweep/resource cases are single observations, not warm-five distributions. Per-run percentile distributions are not combined into an end-to-end percentile.','groups':summary,'failed_invocations':failures,'lifecycle':lifecycle,'provenance':provenance}
(base/'driver-results-summary.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'groups':len(summary),'failed_invocations':len(failures),'all_integrity_ok':all(r['all_integrity_ok']for r in summary),'writer_errors':sum(r['writer_errors']for r in summary),'reader_errors':sum(r['reader_errors']for r in summary)}))
