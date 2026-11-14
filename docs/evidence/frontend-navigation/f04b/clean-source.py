import hashlib, json, os, shutil, subprocess, tempfile, time
from pathlib import Path
source = Path.cwd()
private = Path(tempfile.mkdtemp(prefix='f04b-clean-', dir='/private/tmp'))
checkout = private / 'checkout'
checkout.mkdir()
subprocess.run(['git', 'checkout-index', '--all', '--prefix=' + str(checkout) + '/'], check=True)
(checkout / 'node_modules').symlink_to(source / 'node_modules', target_is_directory=True)
expected = {p.name: p.read_bytes() for p in (checkout / 'src/generated/contracts').iterdir()}
env = {**os.environ, 'CARGO_TARGET_DIR': str(private / 'target')}
try:
    print('Indexed source tree:', subprocess.check_output(['git','write-tree'],text=True).strip(), flush=True)
    print('Private target:', env['CARGO_TARGET_DIR'], '; fresh preparation; shared installed Node dependencies/download caches', flush=True)
    for mode, expected_exit in [('--check',0), ('--check',1), ('--write',0), ('--check',0)]:
        if expected_exit == 1: shutil.rmtree(checkout / 'src/generated/contracts')
        command = ['node','scripts/prepare-generated.mjs',mode]
        start = time.monotonic()
        result = subprocess.run(command,cwd=checkout,env=env,stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
        print('$', ' '.join(command), '\n', result.stdout.decode(), '\nexit', result.returncode, '; wall',time.monotonic()-start, flush=True)
        assert result.returncode == expected_exit
    actual = {p.name: p.read_bytes() for p in (checkout / 'src/generated/contracts').iterdir()}
    assert expected == actual
    print('Clean indexed source: committed outputs pass; missing outputs fail; explicit write reproduces every artifact byte.',flush=True)
    print(json.dumps({name:hashlib.sha256(data).hexdigest() for name,data in actual.items()},indent=2),flush=True)
finally:
    shutil.rmtree(private)
