"""Audit disposable effects, then stop only exact verified processes created by F08."""
from pathlib import Path
import hashlib,json,os,signal,sqlite3,subprocess,time
out=Path(__file__).parent;c=json.loads((out/'desktop-context.json').read_text());ins=json.loads((out/'inspector-context.json').read_text());release=json.loads((out/'final-release-context.json').read_text())
action='fc16e29e-c473-4aa8-9d15-4afc8dd5070b'
with sqlite3.connect('file:'+str(Path(c['state'])/'workspace.sqlite3')+'?mode=ro',uri=True) as db:
    counts={table:db.execute('SELECT COUNT(*) FROM '+table).fetchone()[0] for table in ['registration_actions','workspace_actions','repositories','workspaces']}
    rows=db.execute('SELECT COUNT(*) FROM workspace_actions WHERE id=?',(action,)).fetchone()[0]
    workspaces=db.execute("SELECT COUNT(*) FROM workspaces WHERE json_extract(record, '$.creation_action_id')=?",(action,)).fetchone()[0]
assert rows==1 and workspaces==1,(rows,workspaces)
originals={p:{'expected':h,'actual':hashlib.sha256(Path(p).read_bytes()).hexdigest()} for p,h in c['original_files'].items()};assert all(v['expected']==v['actual'] for v in originals.values())
processes=subprocess.check_output(['ps','-axo','pid=,command='],text=True).splitlines();owned=[]
for label,item in [*ins.items(),('production',release)]:
    expected=str(Path(item['bundle'])/'Contents/MacOS/workspace-desktop');pid=item['desktop_pid'];actual=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip();assert actual==expected,(pid,actual,expected);owned.append({'label':label,'pid':pid,'command':actual})
for label,item in ins.items():
    expected=str(Path(item['bundle'])/'Contents/Resources/workspace-supervisor')+' '+item['state'];matches=[line for line in processes if line.strip().split(None,1)[1]==expected];assert len(matches)==1,matches;pid=int(matches[0].strip().split(None,1)[0]);owned.append({'label':label+' supervisor','pid':pid,'command':expected})
result={'counts':counts,'pending_action':{'action_id':action,'action_rows':rows,'workspace_rows':workspaces},'original_files':originals,'delay_marker_present':(Path(c['root'])/'delay-git').exists(),'processes':owned,'scope':'Explicit fixture termination; no product owned-Quit or provider-descendant claim. Preexisting owners were not signalled.'}
assert not result['delay_marker_present']
for p in owned:os.kill(p['pid'],signal.SIGTERM)
for p in owned:
    deadline=time.monotonic()+10
    while time.monotonic()<deadline:
        state=subprocess.run(['ps','-p',str(p['pid']),'-o','stat='],text=True,capture_output=True).stdout.strip()
        if not state or state.startswith('Z'):break
        time.sleep(.05)
    else:raise AssertionError('Fixture did not exit: '+str(p))
    p['exited']=True
(out/'fixture-cleanup.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
