#!/usr/bin/env python3
"""Ticket03 sequential checkpoint verification; no provider/authentication calls."""
import glob, hashlib, json, os, pathlib, platform, re, subprocess, sys, time, uuid
CORRECTED="--corrected" in sys.argv
SUPPORTED_NODE="--supported-node" in sys.argv
CHECK_ENV=dict(os.environ)
if SUPPORTED_NODE:
    node_bin=pathlib.Path.home()/".local/share/fnm/node-versions/v26.13.1/installation/bin"
    CHECK_ENV["PATH"]=str(node_bin)+os.pathsep+CHECK_ENV["PATH"]
REPORT_NAME="verification-corrected.json" if CORRECTED else "verification-initial.json"
ROOT=pathlib.Path(__file__).resolve().parents[4]
OUT=pathlib.Path(__file__).resolve().parent
if SUPPORTED_NODE: REPORT_NAME="verification-node26.json"
report={'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'dirty':subprocess.check_output(['git','status','--short'],cwd=ROOT,text=True),'profile':'worktree-private default target; warm caches; one Rust harness thread','checks':[]}
def run(label, command):
    if SUPPORTED_NODE: label='node26-'+label
    elif CORRECTED: label='corrected-'+label
    print('Running '+label,flush=True)
    start=time.monotonic()
    result=subprocess.run(command,cwd=ROOT,env=CHECK_ENV,text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
    (OUT/(label+'.log')).write_text(result.stdout)
    record={'label':label,'command':command,'exit':result.returncode,'seconds':time.monotonic()-start}
    report['checks'].append(record)
    (OUT/REPORT_NAME).write_text(json.dumps(report,indent=2)+'\n')
    print(f'{label}: exit {result.returncode}, {record["seconds"]:.3f}s',flush=True)
    return result.stdout
if SUPPORTED_NODE:
    run('environment-node',['node','--version'])
    run('environment-pnpm',['corepack','pnpm','--version'])
    run('complete-verify',['corepack','pnpm','verify'])
    run('typescript',['corepack','pnpm','exec','tsc','--noEmit'])
    run('tooling-units',['node','--test','--test-concurrency=1']+sorted(glob.glob('scripts/*.test.mjs',root_dir=ROOT)))
    run('ui-units',['node','node_modules/vitest/vitest.mjs','run','--maxWorkers=1'])
    run('lint-final',['corepack','pnpm','lint'])
    report['note']='Rust executable measurements/Clippy and physical qualifications remain in verification-corrected.json; same source, no runtime code changes.'
    (OUT/REPORT_NAME).write_text(json.dumps(report,indent=2)+'\n')
    print('Supported Node verification recorded',flush=True)
    sys.exit(0)
if not CORRECTED:
    for label,command in [('host',['uname','-a']),('hardware',['sysctl','hw.model','hw.ncpu','hw.memsize']),('macos',['sw_vers']),('rust',['rustc','--version']),('cargo',['cargo','--version']),('node',['node','--version']),('pnpm',['corepack','pnpm','--version']),('git',['git','--version'])]:
        run('environment-'+label,command)
    run('complete-verify',['corepack','pnpm','verify'])
run('checkpoint-units-measured',['corepack','pnpm','test:workspaces','--measure'])
run('checkpoint-clippy',['node','scripts/cargo.mjs','clippy','--locked','-p','workspace-checkpoint','--all-targets','--all-features','--','-D','warnings'])
run('core-tests-check-final',['node','scripts/cargo.mjs','check','--locked','-p','workspace-core','--tests','--features','codegen'])
if not CORRECTED:
    run('typescript',['corepack','pnpm','exec','tsc','--noEmit'])
    run('tooling-units',['node','--test','--test-concurrency=1']+sorted(glob.glob('scripts/*.test.mjs',root_dir=ROOT)))
    run('ui-units',['node','node_modules/vitest/vitest.mjs','run','--maxWorkers=1'])
    run('format-final',['corepack','pnpm','format:check'])
    run('lint-final',['corepack','pnpm','lint'])
for package,binary in [('workspace-checkpoint','qualify-workspaces'),('storage-checkpoint','qualify-storage')]:
    output=run(package+'-qualification-build',['node','scripts/cargo.mjs','build','--locked','-p',package,'--features','qualification','--bin',binary,'--message-format=json-render-diagnostics'])
    artifacts=[json.loads(line) for line in output.splitlines() if line.startswith('{')]
    executables=[a['executable'] for a in artifacts if a.get('reason')=='compiler-artifact' and a.get('executable') and a['target']['name']==binary]
    if not executables:
        continue
    executable=executables[-1]
    disposable='/tmp/ticket03-'+package+'-'+uuid.uuid4().hex
    report[package]={'executable':executable,'sha256':hashlib.sha256(pathlib.Path(executable).read_bytes()).hexdigest(),'disposable_root':disposable}
    run(package+'-qualification',[executable,disposable])
report['source_files']={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((ROOT/'crates/core/src').rglob('*.rs'))}
(OUT/REPORT_NAME).write_text(json.dumps(report,indent=2)+'\n')
print('Verification recorded',flush=True)
