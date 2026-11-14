import json,socket,time,uuid
from pathlib import Path
c=json.load(open('docs/evidence/frontend-navigation/f04g/desktop-context.json'));state=Path(c['states']['baseline'])
def exchange(**fields):
 with socket.socket(socket.AF_UNIX) as s:
  s.settimeout(5);s.connect(str(state/'supervisor.sock'));s.sendall((json.dumps({'protocol':7,'application_version':'0.1.0','after':None,**fields})+'\n').encode());data=b''
  while not data.endswith(b'\n'):
   part=s.recv(65536)
   if not part:raise EOFError()
   data+=part
  return json.loads(data)
snapshot=exchange()['snapshot'];owner=snapshot['cursor']['owner'];records=[]
for path in c['repositories']:
 action=str(uuid.uuid4());reply=exchange(expected_owner=owner,repository={'operation':'register','request':{'action_id':action,'path':path,'create_initial_workspace':True}})
 for _ in range(100):
  r=exchange(expected_owner=owner,repository={'operation':'action','action_id':action})['result']['Ok']['registration']
  if r['state']=='completed':break
  time.sleep(.05)
 for _ in range(100):
  w=exchange(expected_owner=owner,workspace={'operation':'get','id':r['initial_workspace_id']})['result']['Ok']['workspace']
  if w['status']['state']=='ready':break
  time.sleep(.05)
 p=Path(w['path']);(p/'text.txt').write_text('changed through external Git fixture\n');(p/'large.txt').write_text(''.join(f'Line {i}: changed '+'x'*150+'\n' for i in range(120)));records.append(w)
Path('docs/evidence/frontend-navigation/f04g/baseline-seed.json').write_text(json.dumps({'snapshot':snapshot,'workspaces':records},indent=2)+'\n');print(json.dumps(records,indent=2))
