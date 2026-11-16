#!/usr/bin/env python3
"""Normal native package, incompatible-state rehearsal and launcher qualification.
Only new private state and retained direct child handles; no real providers/auth.
"""
import fcntl,hashlib,json,os,pathlib,plistlib,shutil,signal,sqlite3,subprocess,tempfile,time,sys
REPO=pathlib.Path(__file__).resolve().parents[4];OUT=pathlib.Path(__file__).resolve().parent
prefix=sys.argv[1] if len(sys.argv)>1 else 'normal'
root=pathlib.Path(tempfile.mkdtemp(prefix='aw-native07-normal-',dir='/private/tmp'));root.chmod(0o700)
source=REPO/'target/release/bundle/macos/Agent Workspace.app';bundle=root/'Normal07.app';shutil.copytree(source,bundle)
plist=bundle/'Contents/Info.plist';info=plistlib.loads(plist.read_bytes());info['CFBundleIdentifier']='dev.christopherbengtsson.native07.normal.'+root.name;plist.write_bytes(plistlib.dumps(info))
runtime=root/'synthetic-codex';shutil.copyfile(REPO/'fixtures/codex-readiness/runtime.py',runtime);runtime.chmod(0o700)
home=root/'codex-home';home.mkdir(mode=0o700)
binary=bundle/'Contents/MacOS/workspace-desktop'
frontend=list((REPO/'dist/assets').glob('*.js'));assert frontend and all(b'native-retention-fixture' not in p.read_bytes() for p in frontend)
assert b'Old WebView destruction has not completed' not in binary.read_bytes()
base={**os.environ,'AGENT_WORKSPACE_CODEX_FIXTURE':str(runtime),'CODEX_HOME':str(home)}
for key in ['AGENT_WORKSPACE_SUPERVISOR','AW_NATIVE_SCRIPT','AW_NATIVE_REPORT','AW_NATIVE_QUIT_ON_START']:base.pop(key,None)
report={'root':str(root),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'checks':['normal frontend excludes retention fixture','normal native executable excludes WebView qualification driver'],'processes':[],'files':[{'path':str(p.relative_to(source)),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(source.rglob('*')) if p.is_file()]}
assert not any('supervisor' in p['path'] or 'sidecar' in p['path'] for p in report['files'])
children=[]
def save(): (OUT/(prefix+'-smoke.json')).write_text(json.dumps(report,indent=2)+'\n')
def launch(label,state,command=None):
 state.mkdir(mode=0o700,exist_ok=True)
 log=root/(label+'.log');p=subprocess.Popen(command or [str(binary)],cwd=REPO,env={**base,'AGENT_WORKSPACE_STATE_DIR':str(state)},stdout=log.open('w'),stderr=subprocess.STDOUT,start_new_session=True);children.append((label,p,log));report['processes'].append({'label':label,'pid':p.pid,'state':str(state)});return p,log
def wait(log,text,child,seconds=25):
 end=time.monotonic()+seconds
 while time.monotonic()<end:
  if text in log.read_text():return
  assert child.poll() is None,(child.returncode,log.read_text())
  time.sleep(.05)
 raise TimeoutError((text,log.read_text()))
def quit(p,log,group=False):
 if group:os.killpg(p.pid,signal.SIGINT)
 else:p.send_signal(signal.SIGTERM)
 assert p.wait(25)==0,log.read_text();assert 'failures=0' in log.read_text(),log.read_text()
def unlocked(state):
 with (state/'owner.lock').open('r') as lock:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
try:
 state=root/'native-schema-4';host,log=launch('package',state);wait(log,'phase=Some(Ready)',host)
 assert not (state/'supervisor.sock').exists();inode=(state/'owner.lock').stat().st_ino
 with (state/'owner.lock').open('r') as lock:
  try:fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
  except BlockingIOError:pass
  else:raise AssertionError('Native failed to retain lease')
 competitor,other=launch('competing',state);wait(other,'phase=None',competitor);quit(competitor,other)
 quit(host,log);unlocked(state)
 host,log=launch('handover',state);wait(log,'phase=Some(Ready)',host);assert (state/'owner.lock').stat().st_ino==inode;quit(host,log)
 report['checks']+=['normal package startup ready without supervisor resources','same-root competitor excluded; ordinary Quit completes','current-schema same-root handover keeps identical lease inode']
 incompatible=root/'incompatible';incompatible.mkdir(mode=0o700)
 db=incompatible/'metadata.sqlite3'
 with sqlite3.connect(db) as connection:connection.executescript('PRAGMA user_version=777; CREATE TABLE investigation(value TEXT); INSERT INTO investigation VALUES (\'retained\');')
 db.chmod(0o600)
 host,log=launch('incompatible',incompatible);wait(log,'phase=Some(Failed)',host)
 with sqlite3.connect(db) as connection:
  assert connection.execute('PRAGMA user_version').fetchone()[0]==777
  assert connection.execute('SELECT value FROM investigation').fetchone()[0]=='retained'
 host.send_signal(signal.SIGTERM);time.sleep(.2);assert host.poll() is None
 host.kill();host.wait(10);report['checks'].append('incompatible metadata preserved; failed cleanup retains owner; deliberate recovery kill is not success')
 # Documented reset: select a new root, preserving the incompatible root and all worktrees.
 fresh=root/'reset-rehearsal'/'native-schema-4';fresh.parent.mkdir(mode=0o700)
 host,log=launch('reset-fresh',fresh);wait(log,'phase=Some(Ready)',host);quit(host,log)
 assert db.exists();report['checks'].append('deliberate fresh-root reset initializes schema 4 without deleting prior metadata or worktrees')
 # The real launcher owns Vite and native directly, builds debug once, and drains on signal.
 dev=root/'dev-override';host,log=launch('dev-launcher',dev,['node','scripts/start.mjs','dev']);wait(log,'phase=Some(Ready)',host,120)
 assert 'Native dev state: '+str(dev) in log.read_text();assert not (dev/'supervisor.sock').exists();quit(host,log,group=True);unlocked(dev)
 report['checks'].append('actual dev launcher with exact override starts Vite/native and awaits foreground process-group Ctrl-C completed drain')
 report['outcome']='passed'
except BaseException as error:report['outcome']='failed';report['error']=repr(error);raise
finally:
 for label,p,log in children:
  if p.poll() is None:
   p.send_signal(signal.SIGTERM)
   try:p.wait(25)
   except subprocess.TimeoutExpired:p.kill();p.wait(10);report.setdefault('forced_cleanup',[]).append(label)
  report.setdefault('exits',{})[label]=p.returncode
  shutil.copyfile(log,OUT/(prefix+'-'+label+'.log'))
 save();print(json.dumps({'outcome':report.get('outcome'),'root':str(root)}))
