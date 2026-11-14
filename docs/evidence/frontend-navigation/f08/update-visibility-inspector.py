"""Retain the pre-visibility-fix bundle and deploy the final frontend."""
from pathlib import Path
import hashlib,json,os,plistlib,shutil,signal,subprocess
out=Path(__file__).parent
all_context=json.loads((out/'inspector-context.json').read_text());c=all_context['current'];pid=c['desktop_pid'];expected=str(Path(c['bundle'])/'Contents/MacOS/workspace-desktop')
actual=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip();assert actual==expected
os.kill(pid,signal.SIGKILL)
(out/'inspector-context-before-visibility-fix.json').write_text(json.dumps(all_context,indent=2)+'\n')
bundle=Path(c['bundle']);bundle.rename(bundle.with_name('F08 Inspector current before visibility fix.app'))
shutil.copytree('target/release/bundle/macos/Agent Workspace.app',bundle)
plist=bundle/'Contents/Info.plist';data=plistlib.loads(plist.read_bytes());data.update(CFBundleIdentifier=c['bundle_id'],CFBundleName='F08 Inspector current');plist.write_bytes(plistlib.dumps(data))
for key,relative in [('desktop_sha256','Contents/MacOS/workspace-desktop'),('sidecar_sha256','Contents/Resources/workspace-supervisor')]:c[key]=hashlib.sha256((bundle/relative).read_bytes()).hexdigest()
a=json.loads((out/'artifacts-final.json').read_text());c['frontend_matches_production']={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in Path('dist/assets').glob('*.js')}=={name:item['sha256'] for name,item in a['current']['assets'].items()};assert c['frontend_matches_production']
(out/'inspector-context.json').write_text(json.dumps(all_context,indent=2)+'\n');print(json.dumps(c))
