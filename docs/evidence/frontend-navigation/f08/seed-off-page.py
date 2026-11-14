"""Actual opt-in catalog qualification: 33+ records via targeted native actions."""
from pathlib import Path
import json,socket,struct,subprocess,time,uuid
out=Path(__file__).parent;c=json.loads((out/'desktop-context.json').read_text());state=Path(c['state']);root=Path(c['root']);owner=json.loads((out/'owner-reopened.json').read_text())['snapshot']['snapshot']['cursor']['owner']
def request(kind,**fields):
    payload=json.dumps({'protocol':8,'application_version':'0.1.0','after':None,'expected_owner':owner,kind:fields}).encode()
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(5);peer.connect(str(state/'supervisor.sock'));peer.sendall(struct.pack('>I',len(payload))+payload)
        def exact(n):
            data=b''
            while len(data)<n:
                part=peer.recv(n-len(data))
                if not part:raise EOFError('qualification reply truncated')
                data+=part
            return data
        reply=json.loads(exact(struct.unpack('>I',exact(4))[0]))
    assert reply['kind']==kind and 'Ok' in reply['result'],reply
    return reply['result']['Ok']
created=[]
for _ in range(32):
    action=str(uuid.uuid4());r=request('workspace',operation='create',action_id=action,repository_id=c['repository_id'])
    for _ in range(500):
        r=request('workspace',operation='action',action_id=action)
        if r['workspace'] and r['workspace']['status']['state']=='ready':break
        if r['workspace'] and r['workspace']['status']['state']=='failed':raise AssertionError(r)
        time.sleep(.02)
    else:raise AssertionError(r)
    created.append(r['workspace'])
page=request('workspace',operation='list',repository_id=c['repository_id'],after=None)['page'];listed={w['id'] for w in page['workspaces']}
off_workspace=next(w for w in created if w['id'] not in listed)
(root/'catalog').mkdir()
registrations=[]
for i in range(33):
    repo=root/'catalog'/f'Catalog-{i:02}';subprocess.run(['git','clone','--shared','--no-checkout',c['repositories'][1],str(repo)],check=True,capture_output=True)
    action=str(uuid.uuid4());request('repository',operation='register',request={'action_id':action,'path':str(repo),'create_initial_workspace':False})
    for _ in range(500):
        r=request('repository',operation='action',action_id=action)['registration']
        if r and r['state']=='completed':break
        if r and r['state']=='failed':raise AssertionError(r)
        time.sleep(.02)
    else:raise AssertionError(r)
    registrations.append(r)
repos=request('repository',operation='list',after=None,refresh=False)['page'];listed_repos={r['id'] for r in repos['repositories']};off_repository=next(r for r in registrations if r['repository_id'] not in listed_repos)
result={'source_owner':owner,'created_workspaces':created,'registrations':registrations,'workspace_first_page':page,'repository_first_page':repos,'off_page_workspace':off_workspace,'off_page_repository':off_repository}
(out/'off-page-fixture.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'workspaces':len(created),'repositories':len(registrations),'off_page_workspace':off_workspace,'off_page_repository':off_repository}))
