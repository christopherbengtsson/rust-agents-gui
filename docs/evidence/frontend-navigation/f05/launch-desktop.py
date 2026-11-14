"""Launch only the copied F05 fixture bundle with private supervisor state."""
from pathlib import Path
import hashlib,json,os,plistlib,shutil,subprocess
out=Path('docs/evidence/frontend-navigation/f05');context=json.loads((out/'desktop-context.json').read_text());root=Path(context['root'])
source=Path('target/release/bundle/macos/Agent Workspace.app');bundle=root/'F05 Context.app'
shutil.copytree(source,bundle)
plist=bundle/'Contents/Info.plist';data=plistlib.loads(plist.read_bytes());data['CFBundleIdentifier']='dev.christopherbengtsson.f05.context';data['CFBundleName']='F05 Context';plist.write_bytes(plistlib.dumps(data))
executable=bundle/'Contents/MacOS/workspace-desktop'
process=subprocess.Popen([str(executable)],env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':context['state']},stdout=(root/'desktop.log').open('w'),stderr=subprocess.STDOUT)
context.update(bundle=str(bundle),desktop_pid=process.pid,desktop_hash=hashlib.sha256(executable.read_bytes()).hexdigest(),bundle_identifier=data['CFBundleIdentifier'])
(out/'desktop-context.json').write_text(json.dumps(context,indent=2)+'\n');print(json.dumps(context,indent=2))
