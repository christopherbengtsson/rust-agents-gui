"""Record each explicitly selected F06 verification command and its result."""
from pathlib import Path
import json, subprocess, sys, time
out = Path(__file__).parent
name, *command = sys.argv[1:]
start = time.perf_counter()
with (out / f'{name}.log').open('w') as log:
    result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
record = {'name': name, 'command': command, 'status': result.returncode, 'elapsed_seconds': time.perf_counter()-start}
with (out/'checks.jsonl').open('a') as log:
    log.write(json.dumps(record)+'\n')
print(json.dumps(record))
raise SystemExit(result.returncode)
