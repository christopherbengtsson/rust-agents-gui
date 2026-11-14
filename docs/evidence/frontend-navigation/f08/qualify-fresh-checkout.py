"""Reconstruct final tracked product changes in a fresh private source and target."""
from pathlib import Path
import hashlib,json,os,subprocess,tempfile,time
out=Path(__file__).resolve().parent
root=Path(tempfile.mkdtemp(prefix='aw-f08-final-checkout-',dir='/private/tmp'));checkout=root/'checkout';target=root/'target'
revision=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
patch=subprocess.check_output(['git','diff','--binary','HEAD'])
subprocess.run(['git','worktree','add','--detach',str(checkout),revision],check=True)
subprocess.run(['git','apply','-'],cwd=checkout,input=patch,check=True)
context={'revision':revision,'patch_sha256':hashlib.sha256(patch).hexdigest(),'checkout':str(checkout),'target':str(target),'cache':'fresh private Cargo target; shared immutable download caches; offline frozen pnpm install','commands':[]}
(out/'fresh-checkout.patch').write_bytes(patch)
try:
    for command in [['corepack','pnpm','install','--offline','--frozen-lockfile'],['corepack','pnpm','build']]:
        start=time.monotonic();print(json.dumps({'cwd':str(checkout),'CARGO_TARGET_DIR':str(target),'command':command}),flush=True)
        result=subprocess.run(command,cwd=checkout,env={**os.environ,'CARGO_TARGET_DIR':str(target)})
        context['commands'].append({'args':command,'status':result.returncode,'seconds':time.monotonic()-start})
        if result.returncode:raise SystemExit(result.returncode)
finally:
    (out/'fresh-checkout.json').write_text(json.dumps(context,indent=2)+'\n')
