import hashlib,json,os,plistlib,shutil,subprocess,tempfile
from pathlib import Path
repo=Path.cwd()
root=Path(tempfile.mkdtemp(prefix='aw-core01-ui-',dir='/private/tmp'));root.chmod(0o700)
state=root/'state';state.mkdir(mode=0o700)
home=root/'codex-home';home.mkdir(mode=0o700)
bundle=root/'Core01 Fixture.app'
shutil.copytree(repo/'target/release/bundle/macos/Agent Workspace.app',bundle)
plist=bundle/'Contents/Info.plist';info=plistlib.loads(plist.read_bytes());info['CFBundleIdentifier']='dev.christopherbengtsson.sqlx01.fixture';plist.write_bytes(plistlib.dumps(info))
subprocess.run(['codesign','--force','--deep','--sign','-',str(bundle)],check=True,capture_output=True)
control=root/'control.json';control.write_text(json.dumps({'scenario':'ready'}))
exe=bundle/'Contents/MacOS'/info['CFBundleExecutable']
env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':str(state),'AGENT_WORKSPACE_SUPERVISOR':str(bundle/'Contents/Resources/workspace-supervisor'),'AGENT_WORKSPACE_CODEX_FIXTURE':str(repo/'fixtures/codex-readiness/runtime.py'),'AW_FIXTURE_CONTROL':str(control),'CODEX_HOME':str(home)}
log=(root/'desktop.log').open('w')
process=subprocess.Popen([str(exe)],env=env,stdout=log,stderr=log,start_new_session=True)
context={'root':str(root),'state':str(state),'bundle':str(bundle),'pid':process.pid,'executable':str(exe),'sha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'supervisor_sha256':hashlib.sha256((bundle/'Contents/Resources/workspace-supervisor').read_bytes()).hexdigest()}
(repo/'docs/evidence/sqlx-architecture/01/desktop-context.json').write_text(json.dumps(context,indent=2)+'\n')
print(json.dumps(context))

raise SystemExit(process.wait())
