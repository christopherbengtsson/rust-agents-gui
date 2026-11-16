#!/usr/bin/env python3
"""Run only after the serial runtime cohort; final-source fresh private target costs."""
import hashlib,json,os,pathlib,shutil,subprocess,tempfile,time
OUT=pathlib.Path(__file__).resolve().parent
rows=[]
for driver in ['bridge','sqlx']:
    target=pathlib.Path(tempfile.mkdtemp(prefix=f'storage-async-final-{driver}-')).resolve()
    manifest=OUT/'harness'/driver/'Cargo.toml'
    profile=OUT/f'results-build-{driver}-fresh.time'
    log=OUT/f'results-build-{driver}-fresh.log'
    cmd=['/usr/bin/time','-l','-o',str(profile),'cargo','build','--locked','--release','--manifest-path',str(manifest)]
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(target)
    begin=time.monotonic()
    with log.open('w') as f:p=subprocess.run(cmd,stdout=f,stderr=subprocess.STDOUT,env=env)
    wall=time.monotonic()-begin
    binary=target/'release'/f'storage-async-{driver}'
    measured=pathlib.Path('/tmp')/f'storage-async-{driver}-target/release/storage-async-{driver}'
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    tree_cmd=['cargo','tree','--locked','--manifest-path',str(manifest),'--edges','normal','--prefix','none','--no-dedupe','--format','{p}']
    tree=subprocess.run(tree_cmd,capture_output=True,text=True,env=env)
    normal=sorted({x.removesuffix(' (*)') for x in tree.stdout.splitlines() if x})
    row={'driver':driver,'command':cmd,'CARGO_TARGET_DIR':str(target),'fresh_target':True,'registry_toolchain_OS_caches':'warm/not controlled','exit':p.returncode,'wall_s':wall,'time_l':profile.read_text(),'profile':'release; strip=true','lock_sha256':sha(manifest.with_name('Cargo.lock')),'normal_dependency_graph_command':tree_cmd,'normal_dependency_graph_exit':tree.returncode,'normal_dependency_packages_including_root':normal,'normal_dependency_package_count':len(normal),'compiled_package_lines':[x.strip() for x in log.read_text().splitlines() if x.strip().startswith('Compiling ')]}
    if p.returncode==0:row.update({'binary_sha256':sha(binary),'binary_bytes':binary.stat().st_size,'matches_runtime_binary':sha(binary)==sha(measured)})
    rows.append(row)
    (OUT/'results-build-costs.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(driver,'exit',p.returncode,'wall',wall,'hashmatch',row.get('matches_runtime_binary'),flush=True)
    shutil.rmtree(target)
    if p.returncode:raise RuntimeError('build failed; raw log retained')
