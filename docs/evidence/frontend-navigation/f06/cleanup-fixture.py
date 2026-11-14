"""Stop only the verified copied desktop and private F06 supervisor."""
from pathlib import Path
import hashlib,json,os,signal,socket,struct,subprocess,time
out=Path(__file__).parent;context=json.loads((out/'desktop-context.json').read_text());state=Path(context['state'])
with socket.socket(socket.AF_UNIX) as peer:
    peer.settimeout(4);peer.connect(str(state/'supervisor.sock'))
    payload=json.dumps({'protocol':8,'application_version':'0.1.0','after':None}).encode();peer.sendall(struct.pack('>I',len(payload))+payload)
    def exact(n):
        data=b''
        while len(data)<n:
            part=peer.recv(n-len(data))
            if not part:raise EOFError()
            data+=part
        return data
    snapshot=json.loads(exact(struct.unpack('>I',exact(4))[0]))['snapshot']
owner=snapshot['status']['supervisor_pid'];commands={}
for pid in [context['desktop_pid'],owner]:
    command=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()
    assert context['root'] in command
    if pid==owner:assert str(state) in command
    commands[pid]=command;os.kill(pid,signal.SIGTERM)
for pid in commands:
    for attempt in range(100):
        try:os.kill(pid,0)
        except ProcessLookupError:break
        time.sleep(.05)
    else:raise RuntimeError(f'Fixture process {pid} did not exit')
repository=Path(context['repository']);digest=hashlib.sha256((repository/'text.txt').read_bytes()).hexdigest()
assert digest==context['original_sha256']
assert subprocess.check_output(['git','-C',str(repository),'status','--porcelain'],text=True)==''
result={'snapshot':snapshot,'commands':commands,'fixture_processes_exited':True,'original_checkout_preserved':True,'original_sha256':digest,'private_files_retained':context['root'],'native_provider_submissions':0,'full_owned_quit_qualified':False}
(out/'cleanup.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
