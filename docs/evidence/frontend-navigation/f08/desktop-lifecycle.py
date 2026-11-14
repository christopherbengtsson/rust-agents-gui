"""Record snapshots and stop only the verified private fixture desktop process."""
from pathlib import Path
import json,os,signal,socket,struct,subprocess,sys,time
out=Path(__file__).parent
context=json.loads((out/'desktop-context.json').read_text())
command=sys.argv[1]
def snapshot():
    with socket.socket(socket.AF_UNIX) as peer:
        peer.settimeout(4);peer.connect(str(Path(context['state'])/'supervisor.sock'))
        data=json.dumps({'protocol':8,'application_version':'0.1.0','after':None}).encode();peer.sendall(struct.pack('>I',len(data))+data)
        def exact(n):
            result=b''
            while len(result)<n:
                chunk=peer.recv(n-len(result))
                if not chunk:raise EOFError('incomplete snapshot')
                result+=chunk
            return result
        return json.loads(exact(struct.unpack('>I',exact(4))[0]))
record={'command':command,'snapshot':snapshot()}
if command=='kill-desktop':
    pid=context['desktop_pid'];actual=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()
    expected=str(Path(context['bundle'])/'Contents/MacOS/workspace-desktop')
    assert actual==expected,(actual,expected)
    os.kill(pid,signal.SIGKILL)
    record.update(pid=pid,verified_executable=actual,signal='SIGKILL')
    time.sleep(0.1);record['after']=snapshot()
    assert record['snapshot']['snapshot']['cursor']['owner']==record['after']['snapshot']['cursor']['owner']
if command=='kill-owner':
    pid=record['snapshot']['snapshot']['status']['supervisor_pid']
    actual=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip()
    assert context['state'] in actual and 'workspace-supervisor' in actual,actual
    os.kill(pid,signal.SIGKILL)
    record.update(pid=pid,verified_executable=actual,signal='SIGKILL')
name=sys.argv[2]
(out/name).write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record))
