"""Launch the recorded F08 bundle with explicit private state and fixture Git."""
from pathlib import Path
import json,os,subprocess,time
out=Path(__file__).parent
context=json.loads((out/'desktop-context.json').read_text());root=Path(context['root'])
exe=Path(context['bundle'])/'Contents/MacOS/workspace-desktop'
start=time.monotonic()
with (root/'desktop.log').open('a') as log:
    process=subprocess.Popen([str(exe)],env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':context['state'],'PATH':str(root/'bin')+':'+os.environ['PATH']},stdout=log,stderr=subprocess.STDOUT)
context.update(desktop_pid=process.pid,launch_monotonic=start)
(out/'desktop-context.json').write_text(json.dumps(context,indent=2)+'\n')
print(json.dumps({'desktop_pid':process.pid,'bundle':context['bundle']}),flush=True)
raise SystemExit(process.wait())
