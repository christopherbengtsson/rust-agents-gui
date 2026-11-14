"""Overflow the real journal while the routed frontend retains its accepted cursor."""
from pathlib import Path
import json,socket,struct,time
out=Path(__file__).parent;c=json.loads((out/'desktop-context.json').read_text());owner=json.loads((out/'owner-reopened.json').read_text())['snapshot']['snapshot']['cursor']['owner'];records=[]
def request(refresh):
    data=json.dumps({'protocol':8,'application_version':'0.1.0','after':None,'expected_owner':owner,'repository':{'operation':'get','id':c['repository_id'],'refresh':refresh}}).encode()
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(5);peer.connect(str(Path(c['state'])/'supervisor.sock'));peer.sendall(struct.pack('>I',len(data))+data)
        def exact(n):
            data=b''
            while len(data)<n:
                chunk=peer.recv(n-len(data))
                if not chunk:raise EOFError('overflow producer reply truncated')
                data+=chunk
            return data
        result=json.loads(exact(struct.unpack('>I',exact(4))[0]))
    assert result['kind']=='repository' and 'Ok' in result['result'],result
    return result['result']['Ok']['repository']
start=time.monotonic()
for i in range(1):
    checking=request(True)
    for _ in range(300):
        settled=request(False)
        if settled['availability']['state']!='checking':break
        time.sleep(.01)
    else:raise AssertionError('probe did not settle')
    assert settled['availability']['state']=='available',settled
    records.append({'checking':checking['availability']['state'],'settled':settled['availability']['state']})
value={'owner':owner,'repository_id':c['repository_id'],'explicit_probes':records,'minimum_published_transitions':2,'seconds':time.monotonic()-start}
(out/'settings-hidden-hint-producer.json').write_text(json.dumps(value,indent=2)+'\n');print(json.dumps({'probes':len(records),'minimum_transitions':2,'seconds':value['seconds']}))
