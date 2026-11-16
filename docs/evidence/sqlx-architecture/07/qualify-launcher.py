#!/usr/bin/env python3
"""Actual terminal process-group Ctrl-C with controlled pending child completion.
The children are synthetic OS fixtures, separate from actual Tauri native Q4.
"""
import json,os,pathlib,signal,subprocess,tempfile,time
OUT=pathlib.Path(__file__).resolve().parent;ROOT=OUT.parents[3]
root=pathlib.Path(tempfile.mkdtemp(prefix='aw-launcher07-',dir='/private/tmp'));root.chmod(0o700)
fixture=root/'child.py';fixture.write_text('''import json,os,pathlib,signal,sys,time
role=sys.argv[1];root=pathlib.Path(sys.argv[2]);stopping=False
def term(*_):
 global stopping
 stopping=True;(root/(role+'.term')).touch()
 if role=='service':sys.exit(143)
def interrupt(*_):
 (root/(role+'.interrupt')).touch();sys.exit(130)
signal.signal(signal.SIGTERM,term);signal.signal(signal.SIGINT,interrupt)
(root/(role+'.ready')).write_text(json.dumps({'pid':os.getpid(),'group':os.getpgrp()}))
deadline=time.monotonic()+20
while time.monotonic()<deadline:
 if role=='native' and stopping and (root/'release').exists():sys.exit(0)
 time.sleep(.01)
sys.exit(4)
''')
launcher=root/'launcher.mjs';launcher.write_text('import { createLauncher } from '+json.dumps((ROOT/'scripts/start.mjs').as_uri())+';\n'+f'''
const launcher = createLauncher();
launcher.launch("python3", [{json.dumps(str(fixture))}, "service", {json.dumps(str(root))}], {{}}, "service");
launcher.launch("python3", [{json.dumps(str(fixture))}, "native", {json.dumps(str(root))}]);
process.on("SIGINT", () => launcher.stop().catch((error) => {{console.error(error);process.exitCode=1;}}));
''')
log=root/'launcher.log';p=subprocess.Popen(['node',str(launcher)],cwd=ROOT,stdout=log.open('w'),stderr=subprocess.STDOUT,start_new_session=True)
report={'root':str(root),'command':['node',str(launcher)],'checks':[],'limits':['controlled synthetic native/service children; actual Tauri is qualified separately']}
def until(predicate):
 deadline=time.monotonic()+10
 while time.monotonic()<deadline:
  if predicate():return
  assert p.poll() is None,(p.returncode,log.read_text())
  time.sleep(.02)
 raise TimeoutError(log.read_text())
try:
 until(lambda:all((root/(role+'.ready')).exists() for role in ['native','service']))
 report['children']={role:json.loads((root/(role+'.ready')).read_text()) for role in ['native','service']}
 assert all(child['group']==child['pid'] and child['group']!=p.pid for child in report['children'].values())
 # Only the fresh retained leader/session created above authorizes this group signal.
 os.killpg(p.pid,signal.SIGINT)
 until(lambda:(root/'native.term').exists())
 until(lambda:'Native shutdown is pending' in log.read_text())
 assert p.poll() is None and not (root/'service.term').exists()
 assert not any(root.glob('*.interrupt'))
 report['checks']+=['foreground Ctrl-C reaches launcher only','native receives requested SIGTERM; service remains alive through pending deadline','no automatic escalation or early service stop']
 os.killpg(p.pid,signal.SIGINT)
 (root/'release').touch();assert p.wait(10)==0,log.read_text()
 assert (root/'service.term').exists() and not any(root.glob('*.interrupt'))
 report['checks'].append('repeated Ctrl-C observes one drain; released native exits 0 before requested Vite-style service exit 143')
 report['outcome']='passed'
except BaseException as error:report['outcome']='failed';report['error']=repr(error);raise
finally:
 (root/'release').touch()
 if p.poll() is None:
  os.killpg(p.pid,signal.SIGINT)
  try:p.wait(8)
  except subprocess.TimeoutExpired:p.kill();p.wait(2);report['forced_cleanup']='recovery, not success'
 report['exit']=p.returncode
 (OUT/'launcher-process.json').write_text(json.dumps(report,indent=2)+'\n')
 (OUT/'launcher-process-native.log').write_text(log.read_text())
 print(json.dumps({'outcome':report.get('outcome'),'root':str(root)}))
