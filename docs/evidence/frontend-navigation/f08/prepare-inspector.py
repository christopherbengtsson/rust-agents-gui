"""Copy matched optimized inspector-enabled bundles; preserve production copies."""
from pathlib import Path
import hashlib,json,plistlib,shutil,subprocess,tempfile
out=Path(__file__).parent;c=json.loads((out/'desktop-context.json').read_text());b=json.loads((out/'baseline-context.json').read_text())
r={}
for label,source in [('current',Path('target/release/bundle/macos/Agent Workspace.app')),('baseline',Path(b['target'])/'release/bundle/macos/Agent Workspace.app')]:
    root=Path(c['root']) if label=='current' else Path(tempfile.mkdtemp(prefix='aw-f08-baseline-desktop-',dir='/private/tmp'))
    bundle=root/f'F08 Inspector {label}.app';shutil.copytree(source,bundle)
    plist=bundle/'Contents/Info.plist';data=plistlib.loads(plist.read_bytes());data.update(CFBundleIdentifier=f'dev.christopherbengtsson.f08.inspector.{label}',CFBundleName=f'F08 Inspector {label}');plist.write_bytes(plistlib.dumps(data))
    state=Path(c['state']) if label=='current' else root/'state';state.mkdir(mode=0o700,exist_ok=True)
    r[label]={'root':str(root),'bundle':str(bundle),'state':str(state),'bundle_id':data['CFBundleIdentifier'],'profile':'release + tauri/devtools, optimized minified frontend, no HMR','desktop_sha256':hashlib.sha256((bundle/'Contents/MacOS/workspace-desktop').read_bytes()).hexdigest(),'sidecar_sha256':hashlib.sha256((bundle/'Contents/Resources/workspace-supervisor').read_bytes()).hexdigest()}
# Frontend bytes are identical to the measured production assets; feature flag is native only.
a=json.loads((out/'artifacts.json').read_text())
for label,root in [('current',Path.cwd()),('baseline',Path(b['checkout']))]:
    actual={p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in (root/'dist/assets').glob('*.js')}
    r[label]['frontend_matches_production']=actual=={name:item['sha256'] for name,item in a[label]['assets'].items()}
    assert r[label]['frontend_matches_production'],label
(out/'inspector-context.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
