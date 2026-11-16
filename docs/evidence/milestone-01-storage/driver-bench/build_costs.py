#!/usr/bin/env python3
"""Sequential final-source release builds in fresh private target directories."""
import argparse,hashlib,json,pathlib,subprocess,time
base=pathlib.Path(__file__).resolve().parent
p=argparse.ArgumentParser();p.add_argument('--private-root',type=pathlib.Path,required=True);a=p.parse_args();root=a.private_root.resolve()
result=[]
for driver in ['rusqlite','sqlx']:
 target=root/f'cost-final-{driver}'
 if target.exists():raise RuntimeError('Fresh build cost target already exists; choose a new private root')
 cmd=['/usr/bin/time','-l','cargo','build','--release','--locked','--manifest-path',str(base/driver/'Cargo.toml'),'--target-dir',str(target)]
 t=time.monotonic();p=subprocess.run(cmd,capture_output=True,text=True)
 log=base.parent/f'driver-results-build-{driver}-final-fresh.log';log.write_text(p.stdout+p.stderr)
 binary=target/'release'/f'storage-bench-{driver}'
 result.append({'driver':driver,'command':cmd,'exit_code':p.returncode,'wall_s':time.monotonic()-t,'log':log.name,'profile':'release,stripped,default optimization; warm registry/toolchain/OS caches, new target','common_sha256':hashlib.sha256((base/'common.rs').read_bytes()).hexdigest(),'manifest_sha256':hashlib.sha256((base/driver/'Cargo.toml').read_bytes()).hexdigest(),'lock_sha256':hashlib.sha256((base/driver/'Cargo.lock').read_bytes()).hexdigest(),'binary_bytes':binary.stat().st_size if binary.exists() else None,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest() if binary.exists() else None})
 print(json.dumps({'driver':driver,'exit_code':p.returncode,'wall_s':result[-1]['wall_s']}),flush=True)
 (base.parent/'driver-results-build-costs.json').write_text(json.dumps(result,indent=2)+'\n')
 if p.returncode:raise RuntimeError('Fresh build failed; retained log')
