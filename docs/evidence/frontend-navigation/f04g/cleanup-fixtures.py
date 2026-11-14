import json,os,signal,socket,struct,subprocess,time,hashlib
from pathlib import Path
r=Path('docs/evidence/frontend-navigation/f04g');c=json.loads((r/'desktop-context.json').read_text());records=[]
def get_snapshot(state,protocol):
 with socket.socket(socket.AF_UNIX) as peer:
  peer.settimeout(4);peer.connect(str(Path(state)/'supervisor.sock'));payload=json.dumps({'protocol':protocol,'application_version':'0.1.0','after':None}).encode();peer.sendall(struct.pack('>I',len(payload))+payload if protocol==8 else payload+b'\n')
  def exact(n):
   data=b''
   while len(data)<n:
    part=peer.recv(n-len(data))
    if not part:raise EOFError()
    data+=part
   return data
  if protocol==8: data=exact(struct.unpack('>I',exact(4))[0])
  else:
   data=b''
   while not data.endswith(b'\n'):data+=exact(1)
  return json.loads(data)['snapshot']
for label,state,protocol in [('current',c['states']['current'],8),('baseline',c['states']['baseline'],7),('final-release',c['final_release_state'],8)]:
 snapshot=get_snapshot(state,protocol);pid=snapshot['status']['supervisor_pid'];command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip();assert state in command and c['root'] in command;os.kill(pid,signal.SIGTERM);deadline=time.monotonic()+8
 while time.monotonic()<deadline:
  try:os.kill(pid,0)
  except ProcessLookupError:break
  time.sleep(.05)
 else:raise RuntimeError(f'Fixture owner {pid} did not exit')
 records.append({'label':label,'snapshot':snapshot,'command':command,'exited':True})
originals={}
for name in ['Alpha','Beta']:
 repo=Path(c['root'])/name
 expected={'text.txt':'original text\n','large.txt':''.join(f'Line {i}: original text\n' for i in range(120))}
 for filename,value in expected.items():
  actual=(repo/filename).read_bytes();assert actual==value.encode();originals[str(repo/filename)]=hashlib.sha256(actual).hexdigest()
 assert not (repo/'.git-away').exists()
assert not (Path(c['root'])/'delay-git').exists()
(r/'cleanup.json').write_text(json.dumps({'owners':records,'original_checkout_hashes':originals,'original_checkouts_preserved':True,'fixture_faults_restored':True,'retained_private_fixtures':c['root'],'full_owned_quit_qualified':False},indent=2)+'\n');print(json.dumps({'owners_exited':[x['snapshot']['status']['supervisor_pid'] for x in records],'original_checkouts_preserved':True}))
