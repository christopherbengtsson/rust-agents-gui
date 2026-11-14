"""Prepare a new isolated F08 release cohort; never adopt preexisting owners."""
from pathlib import Path
import hashlib,json,os,plistlib,shutil,subprocess,tempfile
out=Path(__file__).parent
root=Path(tempfile.mkdtemp(prefix='aw-f08-qualified-',dir='/private/tmp'));root.chmod(0o700)
actual_git=shutil.which('git');(root/'bin').mkdir()
wrapper=root/'bin/git'
wrapper.write_text('#!/usr/bin/python3\nimport os,sys,time,json\nfrom pathlib import Path\nr=Path('+repr(str(root))+')\nwith (r/"git.jsonl").open("a") as f:f.write(json.dumps({"time":time.time(),"pid":os.getpid(),"args":sys.argv[1:]})+"\\n")\nif (r/"delay-git").exists() and "status" in sys.argv:time.sleep(1.5)\nos.execv('+repr(actual_git)+',["git",*sys.argv[1:]])\n');wrapper.chmod(0o700)
repositories=[]
for name in ['Alpha','Beta']:
    repo=root/name;repo.mkdir()
    subprocess.run([actual_git,'init','--initial-branch=main',str(repo)],check=True,capture_output=True)
    (repo/'text.txt').write_text('original text\n')
    (repo/'large.txt').write_text(''.join(f'Line {i}: original text\n' for i in range(120)))
    subprocess.run([actual_git,'-C',str(repo),'add','.'],check=True)
    subprocess.run([actual_git,'-C',str(repo),'-c','user.name=Fixture','-c','user.email=fixture@example.invalid','-c','core.hooksPath=/dev/null','commit','--no-gpg-sign','-m','Disposable F08 comparison'],check=True,capture_output=True)
    repositories.append(str(repo))
state=root/'state';state.mkdir(mode=0o700)
bundle=root/'F08 Qualified.app';shutil.copytree('target/release/bundle/macos/Agent Workspace.app',bundle)
plist=bundle/'Contents/Info.plist';data=plistlib.loads(plist.read_bytes());data.update(CFBundleIdentifier='dev.christopherbengtsson.f08.qualified',CFBundleName='F08 Qualified');plist.write_bytes(plistlib.dumps(data))
context={'root':str(root),'state':str(state),'repositories':repositories,'bundle':str(bundle),'bundle_id':data['CFBundleIdentifier'],'hashes':{str(p.relative_to(bundle)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (bundle/'Contents/MacOS').glob('*')},'sidecar_sha256':hashlib.sha256((bundle/'Contents/Resources/workspace-supervisor').read_bytes()).hexdigest(),'original_files':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for repo in repositories for p in Path(repo).glob('*.txt')}}
(out/'desktop-context.json').write_text(json.dumps(context,indent=2)+'\n')
print(json.dumps(context,indent=2))
