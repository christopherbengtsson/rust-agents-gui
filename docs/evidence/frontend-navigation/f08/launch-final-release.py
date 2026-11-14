from pathlib import Path
import json,os,subprocess
out=Path(__file__).parent;c=json.loads((out/'final-release-context.json').read_text());root=Path(c['bundle']).parent
env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':c['state'],'PATH':str(root/'bin')+':'+os.environ['PATH']}
with (root/'final-release.log').open('a') as log:
    process=subprocess.Popen([str(Path(c['bundle'])/'Contents/MacOS/workspace-desktop')],env=env,stdout=log,stderr=subprocess.STDOUT)
c['first_unretained_launcher_pid']=c['desktop_pid'];c['desktop_pid']=process.pid;(out/'final-release-context.json').write_text(json.dumps(c,indent=2)+'\n');print(json.dumps({'pid':process.pid}),flush=True)
raise SystemExit(process.wait())
