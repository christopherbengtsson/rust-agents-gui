#!/usr/bin/env python3
"""Fresh normal-package startup/drain, with an isolated synthetic provider home."""
import hashlib, json, os, pathlib, shutil, signal, subprocess, tempfile, time
REPO = pathlib.Path(__file__).resolve().parents[4]
OUT = pathlib.Path(__file__).resolve().parent
report_path = OUT / 'normal-smoke.json'
assert not report_path.exists(), 'Preserve earlier evidence'
root = pathlib.Path(tempfile.mkdtemp(prefix='aw-foundation08-normal-',dir='/private/tmp'))
root.chmod(0o700)
source = REPO/'target/release/bundle/macos/Agent Workspace.app'
bundle = root/'Foundation08.app'
shutil.copytree(source,bundle)
binary = bundle/'Contents/MacOS/workspace-desktop'
state = root/'state'; state.mkdir(mode=0o700)
home = root/'synthetic-home'; home.mkdir(mode=0o700)
runtime = root/'synthetic-codex'; shutil.copyfile(REPO/'fixtures/codex-readiness/runtime.py',runtime); runtime.chmod(0o700)
env = {**os.environ,'AGENT_WORKSPACE_STATE_DIR':str(state),'AGENT_WORKSPACE_CODEX_FIXTURE':str(runtime),'CODEX_HOME':str(home)}
for key in ['AW_NATIVE_SCRIPT','AW_NATIVE_REPORT','AW_NATIVE_QUIT_ON_START']: env.pop(key,None)
files = [{'path':str(p.relative_to(source)),'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in sorted(source.rglob('*')) if p.is_file()]
report = {'root':str(root),'files':files,'total_bytes':sum(p['bytes'] for p in files),'checks':[]}
assert not any('supervisor' in p['path'] or 'sidecar' in p['path'] for p in files)
assert b'Old WebView destruction has not completed' not in binary.read_bytes()
assert all(b'native-retention-fixture' not in p.read_bytes() for p in (REPO/'dist/assets').glob('*.js'))
report['checks'].append('normal native/frontend exclude qualification instrumentation and retired supervisor resources')
log = OUT/'normal-native.log'
with log.open('x') as output:
    host = subprocess.Popen([str(binary)],env=env,stdout=output,stderr=subprocess.STDOUT)
    try:
        end = time.monotonic()+25
        while 'phase=Some(Ready)' not in log.read_text():
            assert host.poll() is None, log.read_text()
            if time.monotonic()>end: raise TimeoutError(log.read_text())
            time.sleep(.05)
        host.send_signal(signal.SIGTERM)
        report['exit'] = host.wait(25)
        assert report['exit']==0 and 'failures=0' in log.read_text(), log.read_text()
        report['checks'].append('fresh normal package ready, retained owned Quit, zero failures and exit 0')
        report['outcome']='passed'
    except BaseException as error:
        report.update(outcome='failed',error=repr(error)); raise
    finally:
        if host.poll() is None:
            host.kill(); host.wait(10); report['forced_cleanup']=True
        report_path.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'outcome':report['outcome'],'root':str(root)}))
