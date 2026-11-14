"""Stop the verified fixture owner; preserve private files and original checkout."""
from pathlib import Path
import hashlib,json,os,signal,socket,struct,subprocess,time
out=Path('docs/evidence/frontend-navigation/f05');context=json.loads((out/'desktop-context.json').read_text());state=Path(context['state'])
with socket.socket(socket.AF_UNIX) as peer:
 peer.settimeout(4);peer.connect(str(state/'supervisor.sock'));payload=json.dumps({'protocol':8,'application_version':'0.1.0','after':None}).encode();peer.sendall(struct.pack('>I',len(payload))+payload)
 def exact(n):
  data=b''
  while len(data)<n:
   part=peer.recv(n-len(data))
   if not part:raise EOFError()
   data+=part
  return data
 snapshot=json.loads(exact(struct.unpack('>I',exact(4))[0]))['snapshot']
pid=snapshot['status']['supervisor_pid'];command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()
assert str(state) in command and context['root'] in command
os.kill(pid,signal.SIGTERM)
for attempt in range(100):
 try:os.kill(pid,0)
 except ProcessLookupError:break
 time.sleep(.05)
else:raise RuntimeError('Fixture owner did not exit')
repo=Path(context['repository']);expected={'text.txt':'original\n','large.txt':''.join(f'Line {i}: original text\n' for i in range(120))}
for name,text in expected.items():assert (repo/name).read_text()==text
assert subprocess.check_output(['git','-C',str(repo),'status','--porcelain'],text=True)==''
result={'owner':snapshot,'command':command,'owner_exited':True,'original_checkout_preserved':True,'original_hashes':{name:hashlib.sha256((repo/name).read_bytes()).hexdigest() for name in expected},'private_fixtures_retained':context['root'],'full_owned_quit_qualified':False}
(out/'cleanup.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
