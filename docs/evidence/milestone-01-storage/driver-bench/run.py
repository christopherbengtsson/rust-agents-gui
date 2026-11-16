#!/usr/bin/env python3
"""Reproducible, disposable real-storage experiment runner; not a unit test."""
import argparse, hashlib, json, os, pathlib, platform, resource, shutil, statistics, subprocess, tempfile, time
BASE=pathlib.Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--private-root',type=pathlib.Path,required=True);p.add_argument('--phase',choices=['screen','finalists','scale','resources','all'],default='all');p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();os.umask(0o077)
requested=a.private_root.absolute()
if requested.is_symlink():raise RuntimeError('private root must not be a symlink')
root=requested.resolve();root.mkdir(mode=0o700,parents=True,exist_ok=True)
if root.stat().st_uid!=os.getuid() or root.stat().st_mode & 0o077:raise RuntimeError('private root must be owned and mode0700')
data_root=pathlib.Path(tempfile.mkdtemp(prefix='data-',dir=root))
results=[]
def emit(v):
    with a.output.open('a')as f:f.write(json.dumps(v,sort_keys=True)+'\n')
    print(json.dumps({k:v[k]for k in ['phase','driver','label','exit_code']if k in v}),flush=True)
def binary(driver):return root/f'fresh-{driver}'/'release'/f'storage-bench-{driver}'
def invoke(driver,args,label,phase,repeat=None,timed=False):
    cmd=[str(binary(driver)),*map(str,args)];cmd=['/usr/bin/time','-l',*cmd]if timed else cmd
    t=time.monotonic()
    try:
        telemetry=[]
        if timed:
            import signal
            child=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,start_new_session=True)
            while child.poll() is None:
                if time.monotonic()-t>120:
                    os.killpg(child.pid,signal.SIGKILL)
                    child.communicate()
                    raise subprocess.TimeoutExpired(cmd,120)
                descendants=subprocess.run(['pgrep','-P',str(child.pid)],capture_output=True,text=True).stdout.split()
                observed=descendants[0] if descendants else str(child.pid)
                thread_output=subprocess.run(['ps','-M','-p',observed],capture_output=True,text=True).stdout
                threads=thread_output.splitlines()[1:]
                rss=subprocess.run(['ps','-p',observed,'-o','rss='],capture_output=True,text=True).stdout.strip()
                telemetry.append({'pid':observed,'threads':len(threads),'ps_thread_output':thread_output,'rss_kib':int(rss) if rss.isdigit() else None})
                time.sleep(0.05)
            out,err=child.communicate()
            proc=subprocess.CompletedProcess(cmd,child.returncode,out,err)
        else:
            proc=subprocess.run(cmd,capture_output=True,text=True,timeout=120)
        v={'phase':phase,'driver':driver,'label':label,'repeat':repeat,'command':cmd,'exit_code':proc.returncode,'wall_s':time.monotonic()-t,'stderr':proc.stderr}
        if timed:v['resource_telemetry']=telemetry
        try:v['result']=json.loads(proc.stdout)
        except json.JSONDecodeError:v['stdout']=proc.stdout
    except subprocess.TimeoutExpired as e:v={'phase':phase,'driver':driver,'label':label,'repeat':repeat,'command':cmd,'exit_code':'timeout','wall_s':time.monotonic()-t,'stdout':str(e.stdout),'stderr':str(e.stderr)}
    emit(v)
    if v['exit_code']!=0:raise RuntimeError(v)
    return v

def seed(driver,n):
    db=data_root/f'seed-{driver}-{n}.db'
    if not db.exists():invoke(driver,['seed',db,n],str(n),'seed')
    return db

def case(driver,label,*,journal='WAL',sync='FULL',active=25,batch=16,n=10000,readers=0,rate=20,duration=3,repeat=0,phase='screen',timed=False,reopen=False,coalesce=False):
    if sum(f.stat().st_size for f in data_root.iterdir() if f.is_file())>8589934592:raise RuntimeError("8GiB private data budget exceeded")
    source=seed(driver,n)
    db=data_root/f'run-{driver}.db'
    for suffix in ['', '-wal','-shm','-journal']:
        candidate=pathlib.Path(str(db)+suffix)
        if candidate.exists():candidate.unlink()
    shutil.copyfile(source,db)
    return invoke(driver,['run',db,journal,sync,active,batch,duration,readers,rate,*(['reopen'] if reopen else []),*(['coalesce'] if coalesce else [])],label,phase,repeat,timed)

emit({'phase':'provenance','manifest_sha256':hashlib.sha256((BASE/'manifest.json').read_bytes()).hexdigest(),'common_sha256':hashlib.sha256((BASE/'common.rs').read_bytes()).hexdigest(),'platform':platform.platform(),'private_root':str(root),'fresh_data_root':str(data_root),'runner_sha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'phase_selected':a.phase})
if a.phase in ['screen','all']:
    for driver in ['rusqlite','sqlx']:
        for journal,sync in [('DELETE','FULL'),('DELETE','EXTRA'),('WAL','FULL'),('WAL','NORMAL')]:case(driver,f'{journal}-{sync}-b1',journal=journal,sync=sync,batch=1)
        case(driver,'DELETE-FULL-b1-reopen',journal='DELETE',sync='FULL',batch=1,reopen=True)
        case(driver,'WAL-FULL-b16')
        case(driver,'WAL-FULL-b16-readers2',readers=2)
        db=data_root/f'run-{driver}.db';shutil.copyfile(seed(driver,10000),db)
        invoke(driver,['lifecycle',db],'queue-drop-drain','lifecycle')
if a.phase in ['finalists','all']:
    for driver in ['rusqlite','sqlx']:
        for label,opts in [('WAL-FULL-b1',{'batch':1}),('WAL-FULL-coalesce25ms',{'coalesce':True}),('WAL-FULL-coalesce25ms-readers2',{'readers':2,'coalesce':True})]:
            for repeat in range(6):case(driver,label,repeat=repeat,phase='finalists',**opts)
if a.phase in ['scale','all']:
    for driver in ['rusqlite','sqlx']:
        for active in [1,3,10,25]:case(driver,f'active-{active}',active=active,phase='scale',coalesce=True)
        for n in [100000,1000000]:
            for repeat in range(6):case(driver,f'history-{n}',n=n,repeat=repeat,phase='scale',coalesce=True)
if a.phase in ['resources','all']:
    for driver in ['rusqlite','sqlx']:
        case(driver,'saturation-b16',rate=0,phase='resources',timed=True)
        b=binary(driver)
        emit({'phase':'artifact','driver':driver,'binary_bytes':b.stat().st_size,'binary_sha256':hashlib.sha256(b.read_bytes()).hexdigest()})
