"""Start the same baseline release with the identical synthetic Git-delay adapter."""
from pathlib import Path
import json,os,signal,subprocess
out=Path(__file__).parent;c=json.loads((out/'inspector-context.json').read_text())['baseline'];seed=json.loads((out/'baseline-seed.json').read_text());owner=seed['snapshot']['snapshot']['status']['supervisor_pid'];records=[]
for pid in [c['desktop_pid'],owner]:
    actual=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip();assert c['root'] in actual,actual
    records.append({'pid':pid,'command':actual});os.kill(pid,signal.SIGKILL)
(out/'baseline-slow-restart.json').write_text(json.dumps({'terminated_owned_fixture_processes':records,'purpose':'same Git wrapper and delay as current; no providers or original checkout edits'},indent=2)+'\n')
p=out/'launch-inspector.py';s=p.read_text();s=s.replace("if name=='current':env['PATH']=str(root/'bin')+':'+env['PATH']", "common_root=Path(json.loads((out/'desktop-context.json').read_text())['root'])\nenv['PATH']=str(common_root/'bin')+':'+env['PATH']");p.write_text(s)
Path(json.loads((out/'desktop-context.json').read_text())['root'],'delay-git').touch()
