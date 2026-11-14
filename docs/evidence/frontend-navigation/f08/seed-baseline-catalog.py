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
seed=json.loads((out/'baseline-seed.json').read_text());records=[];registrations=[]
for _ in range(32):
    action=str(uuid.uuid4());exchange(workspace={'operation':'create','action_id':action,'repository_id':seed['workspaces'][0]['repository_id']})
    for _ in range(500):
        w=exchange(workspace={'operation':'action','action_id':action})['result']['Ok']['workspace']
        if w and w['status']['state']=='ready':break
        if w and w['status']['state']=='failed':raise AssertionError(w)
        time.sleep(.02)
    else:raise AssertionError(w)
    records.append(w)
for path in sorted((Path(current['root'])/'catalog').iterdir()):
    action=str(uuid.uuid4());exchange(repository={'operation':'register','request':{'action_id':action,'path':str(path),'create_initial_workspace':False}})
    for _ in range(500):
        r=exchange(repository={'operation':'action','action_id':action})['result']['Ok']['registration']
        if r and r['state']=='completed':break
        if r and r['state']=='failed':raise AssertionError(r)
        time.sleep(.02)
    else:raise AssertionError(r)
    registrations.append(r)
(out/'baseline-matched-catalog.json').write_text(json.dumps({'snapshot':exchange(),'created_workspaces':records,'registrations':registrations,'conditions':'35 repositories/34 workspaces, selected Alpha 120-line diff, same Git wrapper and source repositories as current'},indent=2)+'\n');print(json.dumps({'workspaces_added':len(records),'repositories_added':len(registrations)}))
