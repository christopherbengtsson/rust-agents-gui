#!/usr/bin/env python3
"""Ticket05 packaged incumbent host/desktop with retained synthetic peer only.
No real runtime, credentials or authentication service is accessed.
Usage: qualify.py <source.app> <new report.json> [--desktop]
Desktop mode accepts capture <label>, scenario <name>, select, stop on stdin;
all actual UI interactions are performed separately through CUA.
"""
import hashlib, json, os, pathlib, plistlib, select, shutil, socket, struct, subprocess, sys, tempfile, time, uuid
source = pathlib.Path(sys.argv[1]).resolve(strict=True)
output = pathlib.Path(sys.argv[2]).resolve()
assert not output.exists(), 'Preserve existing evidence'
desktop_mode = '--desktop' in sys.argv
repo = pathlib.Path(__file__).resolve().parents[4]
root = pathlib.Path(tempfile.mkdtemp(prefix='aw-sqlx05-',dir='/private/tmp')); root.chmod(0o700)
state = root/'state'; state.mkdir(mode=0o700)
control = root/'control.json'
control.write_text(json.dumps({'scenario':'ready','browser_url':'aw-invalid:ticket05-fixture'}))
runtime = root/'synthetic-codex'; shutil.copyfile(repo/'fixtures/codex-readiness/runtime.py',runtime); runtime.chmod(0o700)
bundle = root/'SQLx05.app'; shutil.copytree(source,bundle)
plist = bundle/'Contents/Info.plist'; info=plistlib.loads(plist.read_bytes()); info.update(CFBundleIdentifier='dev.christopherbengtsson.sqlx05.'+root.name,CFBundleName='SQLx05'); plist.write_bytes(plistlib.dumps(info))
host_binary = bundle/'Contents/Resources/workspace-supervisor'
desktop_binary = bundle/'Contents/MacOS/workspace-desktop'
env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':str(state),'AGENT_WORKSPACE_CODEX_FIXTURE':str(runtime),'AW_FIXTURE_CONTROL':str(control)}
env.pop('AGENT_WORKSPACE_SUPERVISOR',None)
protocol=int(subprocess.check_output([str(host_binary),'--protocol-version'],env=env))
fixture_repo=root/'repository'; fixture_repo.mkdir()
def git(*args): subprocess.run(['git','-C',str(fixture_repo),*args],check=True,capture_output=True)
git('init','--initial-branch=main'); (fixture_repo/'fixture.txt').write_text('ticket05 fixture\n'); git('add','.'); git('-c','user.name=Fixture','-c','user.email=fixture@example.test','commit','-m','Fixture')
report={'root':str(root),'bundle':str(bundle),'repository':str(fixture_repo),'runtime':str(runtime),'protocol':protocol,'mode':'fixture-desktop' if desktop_mode else 'actual-incumbent-process','checks':[], 'captures':[], 'binary_hashes':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [host_binary,desktop_binary,runtime]},'schema_source':'Codex 0.169.0 / d32063f96c2dff1dc96af839b3952abb6acfeda4; all peer messages synthetic'}
processes=[]; desktop=None; owner=None; target=None

def save(): output.write_text(json.dumps(report,indent=2)+'\n')
def events():
 path=control.with_suffix('.events.jsonl'); return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []
def wait(predicate,seconds=20):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  value=predicate()
  if value: return value
  time.sleep(.02)
 raise TimeoutError('Controlled condition did not settle')
def until(get, predicate, seconds=20):
 def check():
  value=get()
  return value if predicate(value) else None
 return wait(check,seconds)
def exchange(abandon=False,**fields):
 payload=json.dumps({'protocol':protocol,'application_version':'0.1.0','after':None,**fields}).encode()
 with socket.socket(socket.AF_UNIX) as peer:
  peer.settimeout(5); peer.connect(str(state/'supervisor.sock')); peer.sendall(struct.pack('>I',len(payload))+payload)
  if abandon:return
  def exact(n):
   data=b''
   while len(data)<n:
    part=peer.recv(n-len(data))
    if not part: raise EOFError('Truncated frame')
    data+=part
   return data
  size=struct.unpack('>I',exact(4))[0]; assert size<=65532
  return json.loads(exact(size))
def start_host():
 global host,owner,target
 host=subprocess.Popen([str(host_binary),str(state)],env=env,stdout=subprocess.PIPE,stderr=(root/f'host-{len(processes)}.log').open('w')); processes.append(host)
 assert select.select([host.stdout],[],[],15)[0] and host.stdout.readline()==b'ready\n'
 owner=exchange()['snapshot']['cursor']['owner']; target=provider({'operation':'target'})['target']; return host
def provider(request): return exchange(expected_owner=owner,provider=request)['result']['Ok']
def observe(): return provider({'operation':'observe','target_id':target['id'],'refresh':False})['observation']
def settled(): return until(observe,lambda value:value['phase']!='checking')
def refresh():
 provider({'operation':'observe','target_id':target['id'],'refresh':True}); return settled()
def scenario(name):
 settings=json.loads(control.read_text()); settings['scenario']=name; control.write_text(json.dumps(settings))
def capture(label):
 value={'label':label,'snapshot':exchange(),'provider':observe(),'events':events(),'host_exit':host.poll(),'desktop_exit':desktop.poll() if desktop else None}
 path=output.with_name(output.stem+'-'+label+'.json');path.write_text(json.dumps(value,indent=2)+'\n');report['captures'].append(str(path));save();print(json.dumps({'capture':label,'provider':value['provider']}),flush=True)
def quit_owned():
 exchange(expected_owner=owner,quit='begin')
 status=until(lambda:exchange(expected_owner=owner,quit='status')['status'],lambda value:value['complete'] or value.get('diagnostic'))
 assert status['complete'],status
 exchange(expected_owner=owner,quit='finish');host.wait(timeout=10);assert host.returncode==0
 return status
def select_runtime():
 value=observe(); action={'operation':'select_executable','target_id':target['id'],'action_id':str(uuid.uuid4()),'expected_revision':value['revision'],'path':str(runtime)}
 provider(action); value=settled(); assert value['target']['selected_executable']==str(runtime),value
 return action
try:
 start_host();save();print(json.dumps(report),flush=True)
 if desktop_mode:
  desktop=subprocess.Popen([str(desktop_binary)],env=env,stdout=(root/'desktop.out').open('w'),stderr=(root/'desktop.err').open('w'));report['desktop_pid']=desktop.pid;save()
  for line in sys.stdin:
   command=line.strip()
   if command.startswith('capture '):capture(command.split(' ',1)[1])
   elif command.startswith('scenario '):scenario(command.split(' ',1)[1])
   elif command=='select':select_runtime()
   elif command=='stop':
    desktop.wait(timeout=15);host.wait(timeout=15);assert desktop.returncode==host.returncode==0;report['checks'].append('actual desktop owned Quit exits desktop and host 0');break
   else:raise ValueError(command)

 else:
  assert events()==[]
  assert refresh()['evidence']['authentication']=='chatgpt';report['checks'].append('bounded synthetic readiness through actual incumbent host')
  selected=select_runtime();provider(selected);assert settled()['selection']['id']==selected['action_id'];report['checks'].append('selection intent/result and same-ID duplicate')
  registration={'operation':'register','request':{'action_id':str(uuid.uuid4()),'path':str(fixture_repo),'create_initial_workspace':True}}
  exchange(expected_owner=owner,repository=registration,abandon=True)
  registered=until(lambda:exchange(expected_owner=owner,repository={'operation':'action','action_id':registration['request']['action_id']})['result'].get('Ok',{}).get('registration'),lambda value:value and value['state']!='pending')
  assert 'completed' in registered['state'];report['checks'].append('abandoned registration reply retains first-creation continuation')
  capture('before-quit');quit_owned();before_restart=len(events());start_host();assert len(events())==before_restart
  baseline=len(events());assert observe()['target']['selected_executable']==str(runtime);time.sleep(.1);assert len(events())==baseline
  report['checks'].append('current-build reopen restores preference without login replay');quit_owned()
 report['result']='passed'
except BaseException as error:
 report['result']='failed: '+repr(error);raise
finally:
 # Bounded normal cleanup first; direct children remain owned if qualification fails.
 if desktop and desktop.poll() is None:
  desktop.terminate();desktop.wait(timeout=5);report['desktop_harness_termination']=desktop.returncode
 for process in processes:
  if process.poll() is None:
   try:quit_owned()
   except BaseException as error:
    report.setdefault('cleanup_failures',[]).append(repr(error));process.terminate();process.wait(timeout=5)
 report['host_exits']=[process.returncode for process in processes]
 records=events();report['events']=records
 report['forbidden_methods']=[event for event in records if event['event'].startswith('forbidden') or str(event.get('method','')).startswith(('thread/','turn/','account/login/'))]
 remaining=[]
 for pid in sorted({event['pid'] for event in records}):
  try:os.kill(pid,0);remaining.append(pid)
  except ProcessLookupError:pass
 report['remaining_fixture_pids']=remaining;save()
 assert not report['forbidden_methods'] and not remaining,report
