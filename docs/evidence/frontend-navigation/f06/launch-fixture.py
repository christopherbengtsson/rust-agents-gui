"""Launch only the copied release bundle with private F06 state."""
from pathlib import Path
import hashlib,json,os,plistlib,shutil,subprocess
out=Path(__file__).parent
context=json.loads((out/'desktop-context.json').read_text());root=Path(context['root'])
bundle=root/'F06 Routes.app'
if not bundle.exists():
    shutil.copytree('target/release/bundle/macos/Agent Workspace.app',bundle)
    plist=bundle/'Contents/Info.plist';data=plistlib.loads(plist.read_bytes())
    data.update(CFBundleIdentifier='dev.christopherbengtsson.f06.routes',CFBundleName='F06 Routes')
    plist.write_bytes(plistlib.dumps(data))
executable=bundle/'Contents/MacOS/workspace-desktop'
process=subprocess.Popen([str(executable)],env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':context['state']},stdout=(root/'desktop.log').open('a'),stderr=subprocess.STDOUT)
context.update(bundle=str(bundle),desktop_pid=process.pid,desktop_sha256=hashlib.sha256(executable.read_bytes()).hexdigest())
(out/'desktop-context.json').write_text(json.dumps(context,indent=2)+'\n')
print(json.dumps({'desktop_pid':process.pid,'bundle':str(bundle)}),flush=True)
raise SystemExit(process.wait())
