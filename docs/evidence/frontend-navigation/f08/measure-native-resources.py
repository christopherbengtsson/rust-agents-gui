"""Five native-process RSS/thread samples; excludes WebContent and inspector heaps."""
from pathlib import Path
import json,statistics,subprocess,sys,time
out=Path(__file__).parent;label=sys.argv[1];c=json.loads((out/'inspector-context.json').read_text())[label]
if label=='baseline':supervisor=json.loads((out/'baseline-seed.json').read_text())['snapshot']['snapshot']['status']['supervisor_pid']
else:supervisor=json.loads((out/'owner-reopened.json').read_text())['snapshot']['snapshot']['status']['supervisor_pid']
pids=[c['desktop_pid'],supervisor];samples=[]
for i in range(5):
    values=[]
    for pid in pids:
        line=subprocess.check_output(['ps','-p',str(pid),'-o','pid=,rss=,command='],text=True).strip();parts=line.split(None,2)
        assert str(Path(c['root'])) in parts[2],line
        threads=subprocess.check_output(['ps','-M','-p',str(pid)],text=True).splitlines()
        values.append({'pid':pid,'rss_kib':int(parts[1]),'threads':len(threads)-1,'command':parts[2]})
    samples.append({'time':time.monotonic(),'processes':values,'total_rss_kib':sum(v['rss_kib'] for v in values)})
    if i<4:time.sleep(0.5)
value={'label':label,'scope':'native desktop + supervisor RSS only; excludes WebContent, total JS heap and inspector process memory','samples':samples,'median_kib':statistics.median(s['total_rss_kib'] for s in samples),'range_kib':[min(s['total_rss_kib'] for s in samples),max(s['total_rss_kib'] for s in samples)]}
(out/(label+'-native-resources.json')).write_text(json.dumps(value,indent=2)+'\n');print(json.dumps(value))
