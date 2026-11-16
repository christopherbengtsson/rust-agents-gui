#!/usr/bin/env python3
"""Normal package startup/drain with isolated state and captured synthetic runtime."""
import hashlib,json,os,pathlib,plistlib,shutil,signal,subprocess,tempfile,time
REPO=pathlib.Path(__file__).resolve().parents[4]; OUT=pathlib.Path(__file__).resolve().parent
root=pathlib.Path(tempfile.mkdtemp(prefix='aw-native06-normal-',dir='/private/tmp'));root.chmod(0o700)
source=REPO/'target/release/bundle/macos/Agent Workspace.app';bundle=root/'Normal06.app';shutil.copytree(source,bundle)
plist=bundle/'Contents/Info.plist';info=plistlib.loads(plist.read_bytes());info['CFBundleIdentifier']='dev.christopherbengtsson.native06.normal.'+root.name;plist.write_bytes(plistlib.dumps(info))
runtime=root/'synthetic-codex';shutil.copyfile(REPO/'fixtures/codex-readiness/runtime.py',runtime);runtime.chmod(0o700)
state=root/'state';state.mkdir(mode=0o700);home=root/'codex-home';home.mkdir(mode=0o700)
binary=bundle/'Contents/MacOS/workspace-desktop';log=root/'normal.log'
frontend=list((REPO/'dist/assets').glob('*.js'));assert frontend and all(b'native-retention-fixture' not in p.read_bytes() for p in frontend)
assert b'Old WebView destruction has not completed' not in binary.read_bytes()
env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':str(state),'AGENT_WORKSPACE_CODEX_FIXTURE':str(runtime),'CODEX_HOME':str(home)}
for key in ['AGENT_WORKSPACE_SUPERVISOR','AW_NATIVE_SCRIPT','AW_NATIVE_REPORT','AW_NATIVE_QUIT_ON_START']:env.pop(key,None)
report={'root':str(root),'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'checks':['normal frontend contains no retention fixture','normal binary contains no qualification replacement driver'],'profile':'release, normal features','provenance':json.loads((REPO/'src-tauri/gen/sidecar/provenance.json').read_text())}
process=None
try:
 with log.open('w') as output:
  process=subprocess.Popen([str(binary)],env=env,stdout=output,stderr=subprocess.STDOUT)
  deadline=time.monotonic()+15
  while time.monotonic()<deadline and 'phase=Some(Ready)' not in log.read_text():
   if process.poll() is not None:raise AssertionError(('early exit',process.returncode,log.read_text()))
   time.sleep(.05)
  assert 'phase=Some(Ready)' in log.read_text(),log.read_text()
  assert not (state/'supervisor.sock').exists()
  process.send_signal(signal.SIGTERM);assert process.wait(20)==0
  assert 'failures=0' in log.read_text()
 report['checks']+=['normal packaged core ready on private state without a supervisor socket','ordinary native signal Quit completes required drain with exit 0'];report['outcome']='passed'
finally:
 if process is not None and process.poll() is None:process.kill();process.wait(10);report['forced_cleanup']=True
 report['exit']=process.returncode if process else None
 shutil.copyfile(log,OUT/'normal-smoke-native.log');(OUT/'normal-smoke.json').write_text(json.dumps(report,indent=2)+'\n')
 print(json.dumps(report,indent=2))
