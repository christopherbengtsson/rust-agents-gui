"""Create a disposable F06 Git checkout and private application-state directory."""
from pathlib import Path
import hashlib,json,os,subprocess,tempfile
out=Path(__file__).parent
root=Path(tempfile.mkdtemp(prefix='aw-f06-',dir='/private/tmp'))
repository=root/'route-fixture';repository.mkdir()
subprocess.run(['git','init','--initial-branch=main',str(repository)],check=True,capture_output=True)
text='original\n'
(repository/'text.txt').write_text(text)
subprocess.run(['git','-C',str(repository),'add','text.txt'],check=True)
subprocess.run(['git','-C',str(repository),'-c','user.name=F06 Fixture','-c','user.email=f06@example.invalid','commit','-m','fixture baseline'],check=True,capture_output=True)
state=root/'state';state.mkdir(mode=0o700)
context={'root':str(root),'repository':str(repository),'state':str(state),'original_sha256':hashlib.sha256(text.encode()).hexdigest()}
(out/'desktop-context.json').write_text(json.dumps(context,indent=2)+'\n')
print(json.dumps(context,indent=2))
