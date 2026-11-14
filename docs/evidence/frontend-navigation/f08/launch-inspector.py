from pathlib import Path
import json,os,subprocess,sys,time
out=Path(__file__).parent;name=sys.argv[1];all_context=json.loads((out/'inspector-context.json').read_text());c=all_context[name];root=Path(c['root']);env={**os.environ,'AGENT_WORKSPACE_STATE_DIR':c['state']}
common_root=Path(json.loads((out/'desktop-context.json').read_text())['root'])
env['PATH']=str(common_root/'bin')+':'+env['PATH']
start=time.monotonic()
with (root/f'inspector-{name}.log').open('a') as log:
    process=subprocess.Popen([str(Path(c['bundle'])/'Contents/MacOS/workspace-desktop')],env=env,stdout=log,stderr=subprocess.STDOUT)
c.update(desktop_pid=process.pid,launch_monotonic=start);(out/'inspector-context.json').write_text(json.dumps(all_context,indent=2)+'\n');print(json.dumps({'name':name,'pid':process.pid}),flush=True)
raise SystemExit(process.wait())
