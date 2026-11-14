"""Create isolated historical source/target; never reset the active branch."""
from pathlib import Path
import json,subprocess,tempfile
out=Path(__file__).parent
root=Path(tempfile.mkdtemp(prefix='aw-f08-matched-baseline-',dir='/private/tmp'))
checkout=root/'checkout'
revision='99adb6366ef7eed1cc72d310f75b6fbec361d436'
subprocess.run(['git','worktree','add','--detach',str(checkout),revision],check=True)
context={'root':str(root),'checkout':str(checkout),'target':str(root/'target'),'revision':revision}
(out/'baseline-context.json').write_text(json.dumps(context,indent=2)+'\n')
print(json.dumps(context))
