"""Explicit native fixture refresh while the desktop has released inspection."""
from pathlib import Path
import json,socket,struct,time
out=Path('docs/evidence/frontend-navigation/f05');context=json.loads((out/'desktop-context.json').read_text())
def exchange(**fields):
 with socket.socket(socket.AF_UNIX) as peer:
  peer.settimeout(4);peer.connect(str(Path(context['state'])/'supervisor.sock'))
  payload=json.dumps({'protocol':8,'application_version':'0.1.0','after':None,**fields}).encode();peer.sendall(struct.pack('>I',len(payload))+payload)
  def exact(n):
   data=b''
   while len(data)<n:
    part=peer.recv(n-len(data))
    if not part:raise EOFError()
    data+=part
   return data
  return json.loads(exact(struct.unpack('>I',exact(4))[0]))
snapshot=exchange()['snapshot'];owner=snapshot['cursor']['owner']
workspace='2a799c18-e502-4440-a42c-6cda4e44d08a'
request={'operation':'changes','workspace_id':workspace,'comparison_id':None,'after':None,'refresh':False}
before=exchange(expected_owner=owner,workspace=request)['result']['Ok']['observation']
admitted=exchange(expected_owner=owner,workspace={**request,'refresh':True})
for attempt in range(100):
 after=exchange(expected_owner=owner,workspace=request)['result']['Ok']['observation']
 if after['phase']=='ready' and after['page']['comparison_id']!=before['page']['comparison_id']:break
 time.sleep(.025)
else:raise RuntimeError('Comparison refresh did not complete')
(out/'changed-comparison-native.json').write_text(json.dumps({'snapshot':snapshot,'before':before,'admitted':admitted,'after':after},indent=2)+'\n')
print(json.dumps({'owner':owner,'before':before['page']['comparison_id'],'after':after['page']['comparison_id']}))
