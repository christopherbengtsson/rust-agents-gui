"""Analyze retained WebKit timing markers and conservative composited feedback images.

Run with the full lossless export gzip path. Keep all non-screenshot records and
only the first screenshot after two animation frames for each sample in the repository trace.
"""
from pathlib import Path
import base64
import copy
import gzip
import hashlib
import json
import statistics
import sys

out=Path(__file__).resolve().parent
source=Path(sys.argv[1])
raw=gzip.decompress(source.read_bytes())
data=json.loads(raw)
recording=data['recording']
markers={m['details']:m['time'] for m in recording['markers'] if m['type']=='timestamp'}
images=[r for r in recording['records'] if r['type']=='timeline-record-type-screenshots']
observations=json.loads((out/'navigation-observations.json').read_text())['records']
selected=[]
results=[]
for record in observations:
    i=record['sample']
    start=markers[f'05c.nav.{i}.input']
    commit=markers[f'05c.nav.{i}.commit']
    second_frame=markers[f'05c.nav.{i}.secondFrame']
    image=next(r for r in images if r['timestamp']>=second_frame)
    name=f'navigation-frame-{i:02d}.png'
    image_bytes=base64.b64decode(image['imageData'].split(',',1)[1])
    (out/name).write_bytes(image_bytes)
    selected.append(image)
    results.append({**record,'warmup':i<=2,'input_marker':start,'commit_marker':commit,
                    'first_feedback_frame':image['timestamp'],
                    'feedback_ms':(image['timestamp']-start)*1000,
                    'screenshot':name,'screenshot_sha256':hashlib.sha256(image_bytes).hexdigest()})
summary={}
for target in ['Workspace','Changes']:
    samples=[r['feedback_ms'] for r in results if not r['warmup'] and r['target']==target]
    summary[target]={'count':len(samples),'median_ms':statistics.median(samples),'range_ms':[min(samples),max(samples)],'all_under_100_ms':max(samples)<100}
identity=json.loads((out/'navigation-trace-identity.json').read_text())
identity.update(full_lossless_export=str(source),source_screenshots=len(images),retained_screenshots=len(selected),pruning='All non-screenshot records, samples, markers and metadata retained; only first post-second-animation-frame screenshot per native navigation sample retained in repository trace.')
kept_times={r['timestamp'] for r in selected}
recording['records']=[r for r in recording['records'] if r['type']!='timeline-record-type-screenshots' or r['timestamp'] in kept_times]
compact=json.dumps(data,separators=(',',':')).encode()
(out/'navigation-timeline.json.gz').write_bytes(gzip.compress(compact,compresslevel=9,mtime=0))
identity['retained_gzip_sha256']=hashlib.sha256((out/'navigation-timeline.json.gz').read_bytes()).hexdigest()
identity['retained_gzip_bytes']=(out/'navigation-timeline.json.gz').stat().st_size
(out/'navigation-trace-identity.json').write_text(json.dumps(identity,indent=2)+'\n')
(out/'navigation-results.json').write_text(json.dumps({'method':'native trusted click input marker to first WebKit composite screenshot at/after the second animation frame following selected-tab DOM commit; inspect retained feedback screenshots; first cycle warm-up; next five cycles measured','viewport_css_pixels':results[0]['viewport'],'display_target_ms':100,'summary':summary,'samples':results},indent=2)+'\n')
print(json.dumps({'summary':summary,'retained_trace_bytes':identity['retained_gzip_bytes']},indent=2))
