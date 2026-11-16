#!/usr/bin/env python3
"""Serial standalone cohort runner; private DBs/targets, retained command/resource evidence."""
import datetime, hashlib, json, os, pathlib, platform, shutil, subprocess, tempfile, time
HERE=pathlib.Path(__file__).resolve().parent
OUT=HERE.parent
PRIVATE=pathlib.Path(tempfile.mkdtemp(prefix='storage-async-comparison-')).resolve()
os.chmod(PRIVATE,0o700)
LOG=OUT/'results-commands.jsonl'
RESULTS=OUT/'results.jsonl'
def command(cmd,**kwargs):
    begin=time.monotonic(); p=subprocess.run(cmd,capture_output=True,text=True,**kwargs)
    with LOG.open('a') as f:f.write(json.dumps({'command':cmd,'exit':p.returncode,'wall_s':time.monotonic()-begin,'stdout':p.stdout,'stderr':p.stderr})+'\n')
    p.check_returncode();return p.stdout

def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def trial(driver,args,label):
    binary=pathlib.Path('/tmp')/f'storage-async-{driver}-target/release/storage-async-{driver}'
    folder=PRIVATE/label;folder.mkdir(mode=0o700);db=folder/'events.sqlite'
    timing=folder/'time.txt'
    cmd=['/usr/bin/time','-l','-o',str(timing),str(binary),args[0],str(db),*args[1:]]
    begin=time.monotonic();p=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
    samples=[];next_sample=begin+1
    while p.poll() is None:
        if time.monotonic()>=next_sample:
            children=subprocess.run(['pgrep','-P',str(p.pid)],capture_output=True,text=True).stdout.split()
            for child in children:
                raw=subprocess.run(['ps','-M','-p',child],capture_output=True,text=True).stdout
                rss=subprocess.run(['ps','-o','rss=','-p',child],capture_output=True,text=True).stdout.strip()
                lines=[x for x in raw.splitlines() if x.strip()]
                samples.append({'at_s':time.monotonic()-begin,'pid':child,'threads_excluding_heading':max(0,len(lines)-1),'rss_kib':int(rss) if rss.isdigit() else None,'raw_ps_M':raw})
            next_sample+=5
        time.sleep(.1)
    stdout,stderr=p.communicate();wall=time.monotonic()-begin
    evidence={'label':label,'command':cmd,'exit':p.returncode,'wall_s':wall,'stderr':stderr,'stdout':stdout,'time_l':timing.read_text() if timing.exists() else None,'samples':samples,'binary_sha256':sha(binary)}
    with LOG.open('a') as f:f.write(json.dumps(evidence)+'\n')
    if p.returncode:raise RuntimeError(f'failed {label}: {stderr}')
    result=json.loads(stdout);result.update({'label':label,'process_wall_s':wall,'resource_samples':samples,'time_l':evidence['time_l'],'binary_sha256':evidence['binary_sha256']})
    # Closed private database audit through Python stdlib, outside timed binary window.
    import sqlite3
    connection=sqlite3.connect(db);digest=hashlib.sha256();count=0
    for row in connection.execute('SELECT e.id,e.stream,e.seq,s.workspace,s.root,s.parent,e.payload FROM events e JOIN streams s ON s.id=e.stream WHERE e.id>0 ORDER BY e.id'):
        digest.update(json.dumps(row,separators=(',',':')).encode());digest.update(b'\n');count+=1
    result['post_close_live_rows_sha256']=digest.hexdigest();result['post_close_live_rows_count']=count
    result['post_close_integrity']=connection.execute('PRAGMA integrity_check').fetchone()[0];connection.close()
    with RESULTS.open('a') as f:f.write(json.dumps(result)+'\n')
    print(label,'ok',result.get('committed'),result.get('rejected'),flush=True)
    shutil.rmtree(folder)
    return result

def main():
    environment={'timestamp_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'platform':platform.platform(),'machine':platform.machine(),'private_root':str(PRIVATE),'source_files':{str(p.relative_to(HERE)):sha(p) for p in sorted(HERE.rglob('*')) if p.is_file()},'rustc':command(['rustc','-Vv']),'cargo':command(['cargo','-V']),'hardware':command(['sysctl','hw.model','hw.memsize','hw.ncpu']),'source_revision':command(['git','rev-parse','HEAD']),'dirty':command(['git','status','--short']),'profiles':'release, strip=true; private target dirs; one current-thread Tokio runtime; no app/provider run'}
    (OUT/'results-environment.json').write_text(json.dumps(environment,indent=2)+'\n')
    for driver in ['bridge','sqlx']:trial(driver,['focused'],'before-'+driver)
    for workload in ['baseline','heavy']:
        for run in range(6):
            order=['bridge','sqlx'] if run%2==0 else ['sqlx','bridge']
            for driver in order:trial(driver,[workload,str(run)],f'{workload}-{run}-{driver}')
    for driver in ['sqlx','bridge']:trial(driver,['focused'],'after-'+driver)
    print('complete',flush=True)
    shutil.rmtree(PRIVATE)
if __name__=='__main__':main()
