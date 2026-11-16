#!/usr/bin/env python3
"""One separately labeled diagnostic pair after the frozen v1 cohort, never a capacity qualification."""
import hashlib,json,os,pathlib,shutil,sqlite3,subprocess,tempfile,time
HERE=pathlib.Path(__file__).resolve().parent
OUT=HERE.parent
private=pathlib.Path(tempfile.mkdtemp(prefix='storage-async-group64-')).resolve()
os.chmod(private,0o700)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
original=(HERE/'common.rs').read_text()
assert original.count('queued < 16')==1 and original.count('0..16')==1
variant=original.replace('queued < 16','queued < 64').replace('0..16','0..64')
(HERE/'group64-common.rs').write_text(variant)
(private/'common.rs').write_text(variant)
shutil.copyfile(HERE/'driver.rs',private/'driver.rs')
manifest=json.loads((HERE/'manifest.json').read_text())
manifest['variant']='single paired group64 diagnostic; not five-run qualified capacity'
manifest['cohort']={'warmups':0,'measured':1,'admission_seconds':30,'seed_events':100000}
manifest['workloads']=[x for x in manifest['workloads'] if x['name']=='heavy']
manifest['admission']['group_max_events']=64
manifest['logical_budget_note']='unchanged8MiB admission including physical original; max physical batch64x64KiB=4MiB; bridge copy additional; no total-RSS guarantee'
manifest['source_change']='Only queued<16 and range0..16 changed to64; all trace/SQL/read/admission/urgent logic preserved'
(HERE/'group64-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
provenance={'common_v1_sha256':sha(HERE/'common.rs'),'common_group64_sha256':sha(HERE/'group64-common.rs'),'driver_unchanged_sha256':sha(HERE/'driver.rs'),'manifest':manifest,'commands':[],'builds':[]}
def record(cmd,env=None,time_path=None,log_path=None):
    begin=time.monotonic();p=subprocess.run(cmd,capture_output=True,text=True,env=env)
    x={'command':cmd,'CARGO_TARGET_DIR':env.get('CARGO_TARGET_DIR') if env else None,'exit':p.returncode,'wall_s':time.monotonic()-begin,'stdout':p.stdout,'stderr':p.stderr}
    if time_path and time_path.exists():x['time_l']=time_path.read_text()
    if log_path:log_path.write_text(p.stdout+p.stderr)
    provenance['commands'].append(x);(OUT/'results-group64-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n');p.check_returncode();return p,x
for driver in ['bridge','sqlx']:
    shutil.copytree(HERE/driver,private/driver)
    target=pathlib.Path(tempfile.mkdtemp(prefix=f'storage-async-group64-{driver}-target-')).resolve()
    env=os.environ.copy();env['CARGO_TARGET_DIR']=str(target)
    timing=private/f'{driver}-build.time'
    cmd=['/usr/bin/time','-l','-o',str(timing),'cargo','build','--locked','--release','--manifest-path',str(private/driver/'Cargo.toml')]
    p,e=record(cmd,env,timing,OUT/f'results-group64-build-{driver}.log')
    binary=target/'release'/f'storage-async-{driver}'
    provenance['builds'].append({'driver':driver,'binary_sha256':sha(binary),'binary_bytes':binary.stat().st_size,'lock_sha256':sha(private/driver/'Cargo.lock'),'matches_original_lock':sha(private/driver/'Cargo.lock')==sha(HERE/driver/'Cargo.lock'),'fresh_private_target':str(target)})
    db=private/f'{driver}.sqlite';timing=private/f'{driver}-runtime.time'
    cmd=['/usr/bin/time','-l','-o',str(timing),str(binary),'heavy',str(db),'1']
    p,e=record(cmd,None,timing)
    result=json.loads(p.stdout);result.update({'variant':'group64-single-diagnostic','label':f'group64-heavy-{driver}','binary_sha256':sha(binary),'process_wall_s':e['wall_s'],'time_l':e['time_l'],'group_max_events':64,'five_run_capacity_qualified':False})
    conn=sqlite3.connect(db);digest=hashlib.sha256();count=0
    for row in conn.execute('SELECT e.id,e.stream,e.seq,s.workspace,s.root,s.parent,e.payload FROM events e JOIN streams s ON s.id=e.stream WHERE e.id>0 ORDER BY e.id'):
        digest.update(json.dumps(row,separators=(',',':')).encode());digest.update(b'\n');count+=1
    result['post_close_live_rows_sha256']=digest.hexdigest();result['post_close_live_rows_count']=count;result['post_close_integrity']=conn.execute('PRAGMA integrity_check').fetchone()[0];conn.close()
    with (OUT/'results-group64.jsonl').open('a') as f:f.write(json.dumps(result)+'\n')
    print(driver,'group64',result['committed'],'committed',result['rejected'],'rejected',flush=True)
    shutil.rmtree(target)
    for f in private.glob(f'{driver}.sqlite*'):f.unlink()
provenance['v1_sources_unchanged']=sha(HERE/'common.rs')==provenance['common_v1_sha256'] and sha(HERE/'driver.rs')==provenance['driver_unchanged_sha256']
provenance['private_database_and_targets_removed']=True
(OUT/'results-group64-provenance.json').write_text(json.dumps(provenance,indent=2)+'\n')
shutil.rmtree(private)
