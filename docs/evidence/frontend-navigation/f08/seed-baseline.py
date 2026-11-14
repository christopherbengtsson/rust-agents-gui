"""Explicitly seed baseline protocol-5 private state through its native seam."""
from pathlib import Path
import json,socket,time,uuid
out=Path(__file__).parent;c=json.loads((out/'inspector-context.json').read_text())['baseline'];current=json.loads((out/'desktop-context.json').read_text())
def exchange(**fields):
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(5);peer.connect(str(Path(c['state'])/'supervisor.sock'));peer.sendall((json.dumps({'protocol':5,'application_version':'0.1.0','after':None,**fields})+'\n').encode());data=b''
        while not data.endswith(b'\n'):
            part=peer.recv(65536)
            if not part:raise EOFError('baseline response truncated')
            data+=part
        return json.loads(data)
records=[]
for path in current['repositories']:
    action=str(uuid.uuid4());exchange(repository={'operation':'register','request':{'action_id':action,'path':path,'create_initial_workspace':True}})
    for _ in range(200):
        r=exchange(repository={'operation':'action','action_id':action})['result']['Ok']['registration']
        if r['state']=='completed':break
        time.sleep(.02)
    else:raise AssertionError(r)
    for _ in range(200):
        w=exchange(workspace={'operation':'get','id':r['initial_workspace_id']})['result']['Ok']['workspace']
        if w['status']['state']=='ready':break
        time.sleep(.02)
    else:raise AssertionError(w)
    p=Path(w['path']);(p/'text.txt').write_text('changed text\n');(p/'large.txt').write_text(''.join(f'Line {i}: '+('changed ' if i%3==0 else '')+'original text\n' for i in range(120)));records.append(w)
(out/'baseline-seed.json').write_text(json.dumps({'snapshot':exchange(),'workspaces':records},indent=2)+'\n');print(json.dumps(records,indent=2))
