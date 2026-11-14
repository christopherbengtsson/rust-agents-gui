"""Record explicit F08 commands without overwriting an earlier attempt."""
from pathlib import Path
import json
import subprocess
import sys
import time

out = Path(__file__).parent
name, *command = sys.argv[1:]
start = time.perf_counter()
with (out / f'{name}.log').open('x') as log:
    result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
record = {'name': name, 'command': command, 'status': result.returncode,
          'elapsed_seconds': time.perf_counter() - start}
with (out / 'checks.jsonl').open('a') as log:
    log.write(json.dumps(record) + '\n')
print(json.dumps(record), flush=True)
raise SystemExit(result.returncode)
