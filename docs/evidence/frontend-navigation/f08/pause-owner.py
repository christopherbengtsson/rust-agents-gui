"""Pause only the freshly observed private owner; always resume it after ten seconds."""
from pathlib import Path
import json,os,signal,subprocess,time
out=Path(__file__).parent;c=json.loads((out/'inspector-context.json').read_text())['current'];r=json.loads((out/'owner-reopened.json').read_text());pid=r['snapshot']['snapshot']['status']['supervisor_pid'];expected=str(Path(c['bundle'])/'Contents/Resources/workspace-supervisor')+' '+c['state'];actual=subprocess.check_output(['ps','-p',str(pid),'-o','command='],text=True).strip();assert actual==expected,(actual,expected)
start=time.monotonic();record={'pid':pid,'verified_command':actual,'start':start,'fault':'SIGSTOP for ten seconds, always SIGCONT in finally'};os.kill(pid,signal.SIGSTOP)
try:time.sleep(10)
finally:
    os.kill(pid,signal.SIGCONT);record['resumed']=time.monotonic();(out/'physical-owner-pause.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True)
