#!/usr/bin/env python3
"""Actual native Tauri/WebView Q4 driver. Synthetic Codex only; no credentials.
Creates and retains a disposable app/state/repository and a loopback fixture page.
"""
import fcntl, hashlib, http.server, json, os, pathlib, plistlib, shutil, signal, subprocess, sys, tempfile, threading, time
REPO = pathlib.Path(__file__).resolve().parents[4]
OUT = pathlib.Path(__file__).resolve().parent
source = pathlib.Path(sys.argv[1]).resolve(strict=True)
report_path = OUT / sys.argv[2]
assert not report_path.exists(), 'Preserve prior qualification attempts'
root = pathlib.Path(tempfile.mkdtemp(prefix='aw-native06-', dir='/private/tmp')); root.chmod(0o700)
state = root/'state'; state.mkdir(mode=0o700)
home = root/'codex-home'; home.mkdir(mode=0o700)
runtime = root/'synthetic-codex'; shutil.copyfile(REPO/'fixtures/codex-readiness/runtime.py', runtime); runtime.chmod(0o700)
bundle = root/'Native06.app'; shutil.copytree(source, bundle)
plist = bundle/'Contents/Info.plist'; info = plistlib.loads(plist.read_bytes()); info['CFBundleIdentifier'] = 'dev.christopherbengtsson.native06.'+root.name; plist.write_bytes(plistlib.dumps(info))
executable = bundle/'Contents/MacOS/workspace-desktop'
repository = root/'repository'; repository.mkdir()
def git(*args): return subprocess.run(['git','-C',str(repository),*args], check=True, capture_output=True)
git('init','--initial-branch=main'); (repository/'fixture.txt').write_text('native host fixture\n'); git('add','.'); git('-c','user.name=Fixture','-c','user.email=fixture@example.test','commit','-m','Fixture')
class Landing(http.server.BaseHTTPRequestHandler):
 def do_GET(self):
  body=b'<html><title>Synthetic sign-in fixture</title><h1>Local synthetic sign-in</h1><p>No authentication or credentials.</p></html>'
  self.send_response(200); self.end_headers(); self.wfile.write(body)
 def log_message(self,*args): pass
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Landing); threading.Thread(target=server.serve_forever,daemon=True).start()
control=root/'control.json'; control.write_text(json.dumps({'scenario':'browser_pending','browser_url':f'http://127.0.0.1:{server.server_port}/fixture'}))
script=root/'driver.js'; raw= (OUT/'native-driver.js').read_text(); records=root/'webview.jsonl'
base_env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':str(state),'AGENT_WORKSPACE_CODEX_FIXTURE':str(runtime),'AW_FIXTURE_CONTROL':str(control),'CODEX_HOME':str(home),'AW_NATIVE_SCRIPT':str(script),'AW_NATIVE_REPORT':str(records)}
base_env.pop('AGENT_WORKSPACE_SUPERVISOR',None)
report={'root':str(root),'bundle':str(bundle),'repository':str(repository),'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=REPO,text=True).strip(),'binary_sha256':hashlib.sha256(executable.read_bytes()).hexdigest(),'fixture_sha256':hashlib.sha256(runtime.read_bytes()).hexdigest(),'checks':[],'processes':[],'records':str(records),'limits':['Synthetic Codex protocol; no real authentication/agent calls','SIGKILL establishes process-crash recovery, not power loss','Retained supervisor resource is unreachable; deletion/package audit belongs to ticket 07']}
processes=[]
def save(): report_path.write_text(json.dumps(report,indent=2)+'\n')
def configure(mode,**kw): script.write_text('window.nativeFixture='+json.dumps({'mode':mode,'repository':str(repository),**kw})+';\n'+raw)
def events():
 p=control.with_suffix('.events.jsonl'); return [json.loads(s) for s in p.read_text().splitlines()] if p.exists() else []
def all_records(): return [json.loads(s) for s in records.read_text().splitlines()] if records.exists() else []
def wait_stage(stage,offset=0,timeout=40):
 end=time.monotonic()+timeout
 while time.monotonic()<end:
  for item in all_records()[offset:]:
   if item['stage']=='failed': raise AssertionError(item)
   if item['stage']==stage: return item
  time.sleep(.05)
 raise TimeoutError((stage,all_records()[offset:]))
def launch(label,env=None):
 log=(root/(label+'.log')).open('w'); p=subprocess.Popen([str(executable)],env=env or base_env,stdout=log,stderr=subprocess.STDOUT); processes.append(p); report['processes'].append({'label':label,'pid':p.pid});save();return p
try:
 configure('initial'); host=launch('initial')
 replace=wait_stage('replace-ready'); configure('replacement',previousIncarnation=replace['incarnation'],attempt=replace['attempt'])
 cancelled=wait_stage('cancelled'); assert cancelled['incarnation']==replace['incarnation']
 report['checks']+=['registration plus first creation','two concurrent creations with inspection','synthetic readiness/login across navigation','actual WebView destruction/recreation preserves incarnation and exact login; cancellation settles']
 assert not (state/'supervisor.sock').exists(); report['checks'].append('no application supervisor socket created')
 offset=len(all_records()); configure('compete'); competitor=launch('competing-owner'); excluded=wait_stage('excluded',offset); assert 'own' in excluded['snapshot']['diagnostic']; competitor.send_signal(signal.SIGTERM); assert competitor.wait(15)==0
 report['checks'].append('same-root competitor rejected before storage/effects and can quit')
 host.send_signal(signal.SIGTERM); time.sleep(.03)
 if host.poll() is None: host.send_signal(signal.SIGTERM)
 assert host.wait(20)==0; report['checks'].append('repeated native Quit after cancelled login completes with exit 0')
 offset=len(all_records()); configure('active-quit'); active=launch('active-quit'); wait_stage('active-quit-ready',offset)
 active.send_signal(signal.SIGTERM); time.sleep(.02)
 if active.poll() is None: active.send_signal(signal.SIGTERM)
 assert active.wait(20)==0;report['checks'].append('actual repeated Quit during active synthetic login cancels/joins work and exits 0')
 offset=len(all_records()); configure('crash'); host=launch('before-crash'); pending=wait_stage('kill-ready',offset); starts_before=len([x for x in events() if x.get('method')=='account/login/start']); host.kill(); host.wait(10)
 offset=len(all_records()); configure('reopen',previousIncarnation=pending['incarnation'],attempt=pending['attempt']); host=launch('reopen'); recovered=wait_stage('recovered',offset); starts_after=len([x for x in events() if x.get('method')=='account/login/start']); assert starts_before==starts_after
 report['checks'].append('native kill/reopen changes incarnation, retains exact interrupted login and saved work, fences replacement without replay')
 host.send_signal(signal.SIGTERM); time.sleep(.1); host.send_signal(signal.SIGTERM); time.sleep(1)
 assert host.poll() is None; assert 'native shutdown settled' in (root/'reopen.log').read_text()
 with (state/'owner.lock').open('r') as lease:
  try: fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
  except BlockingIOError: pass
  else: raise AssertionError('Failed cleanup released state lease')
 report['checks'].append('failed cleanup retains native process and actual OS state lease through repeated Quit')
 host.kill();host.wait(10);report['checks'].append('explicit recovery kill after failed drain; not successful Quit')
 startup_state=root/'startup-state'; startup_state.mkdir(mode=0o700); startup_records=root/'startup.jsonl'
 env={**base_env,'AGENT_WORKSPACE_STATE_DIR':str(startup_state),'AW_NATIVE_REPORT':str(startup_records),'AW_NATIVE_QUIT_ON_START':'1'}; env.pop('AW_NATIVE_SCRIPT',None)
 startup=launch('startup-quit',env);assert startup.wait(20)==0
 first=json.loads(startup_records.read_text().splitlines()[0]); assert first['snapshot']['status']['state']=='starting'; report['checks'].append('actual Quit during native startup retains initialization and exits after drain')
 report['outcome']='passed'
except BaseException as error:
 report['outcome']='failed';report['error']=repr(error);raise
finally:
 report['fixture_events']=events(); report['webview_records']=all_records()
 for p in processes:
  if p.poll() is None:p.kill();p.wait(10);report.setdefault('forced_cleanup',[]).append(p.pid)
 report['exit_codes']={str(p.pid):p.returncode for p in processes}
 report['native_logs']=[]
 for item in report['processes']:
  src=root/(item['label']+'.log');dest=OUT/(report_path.stem+'-'+item['label']+'.log')
  if src.exists():shutil.copyfile(src,dest);report['native_logs'].append(str(dest.relative_to(REPO)))
 save();server.shutdown()
 print(json.dumps({'report':str(report_path),'outcome':report.get('outcome'),'root':str(root)}),flush=True)
