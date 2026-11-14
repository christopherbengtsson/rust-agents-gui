from pathlib import Path
import hashlib,json,os,shutil,subprocess,tempfile,plistlib
repo=Path.cwd(); evidence=repo/'docs/evidence/frontend-navigation/f04g'
root=Path(tempfile.mkdtemp(prefix='aw-f04g-desktop-',dir='/private/tmp'));root.chmod(0o700)
context={'root':str(root),'states':{},'repositories':[],'bundles':{}}
actual_git=shutil.which('git');(root/'bin').mkdir()
wrapper=root/'bin/git'
wrapper.write_text('''#!/usr/bin/python3
import json,os,sys,time
from pathlib import Path
root=Path(os.environ['F04G_FIXTURE_ROOT'])
with (root/'git.jsonl').open('a') as log: log.write(json.dumps({'time':time.time(),'pid':os.getpid(),'args':sys.argv[1:]})+'\\n')
if (root/'delay-git').exists() and 'status' in sys.argv:
 (root/'git-entered').write_text(str(os.getpid()))
 time.sleep(1.5)
os.execv('''+repr(actual_git)+''',['git',*sys.argv[1:]])
''');wrapper.chmod(0o700)
for label in ['Alpha','Beta']:
 p=root/label;p.mkdir();subprocess.run([actual_git,'-c','init.defaultBranch=main','init',str(p)],check=True,stdout=subprocess.DEVNULL)
 (p/'text.txt').write_text('original text\n');(p/'large.txt').write_text(''.join(f'Line {i}: original text\n' for i in range(120)))
 subprocess.run([actual_git,'-C',str(p),'add','.'],check=True)
 subprocess.run([actual_git,'-C',str(p),'-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','core.hooksPath=/dev/null','commit','--no-gpg-sign','-m','Disposable comparison'],check=True,stdout=subprocess.DEVNULL)
 context['repositories'].append(str(p))
base=json.load(open(evidence/'environment.json'))
for label,source in [('current',repo/'target/release/bundle/macos/Agent Workspace.app'),('baseline',Path(base['target'])/'release/bundle/macos/Agent Workspace.app')]:
 bundle=root/f'F04g {label}.app';shutil.copytree(source,bundle)
 plist=bundle/'Contents/Info.plist';data=plistlib.loads(plist.read_bytes());data['CFBundleIdentifier']=f'dev.christopherbengtsson.f04g.{label}';data['CFBundleName']=f'F04g {label}';plist.write_bytes(plistlib.dumps(data))
 state=root/f'state-{label}';state.mkdir(mode=0o700)
 context['states'][label]=str(state);context['bundles'][label]=str(bundle)
context['source_hashes']={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in root.glob('*.app/Contents/MacOS/*')}
(evidence/'desktop-context.json').write_text(json.dumps(context,indent=2)+'\n');print(json.dumps(context,indent=2))
