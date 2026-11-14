import json, subprocess, sys, time
from pathlib import Path
label, *command = sys.argv[1:]
start = time.monotonic()
result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
elapsed = time.monotonic() - start
directory = Path("docs/evidence/frontend-navigation/f04b")
(directory / (label + ".log")).write_bytes(result.stdout)
p = directory / "checks.json"
checks = json.loads(p.read_text()) if p.exists() else []
checks.append(dict(label=label, command=command, exitStatus=result.returncode, wallSeconds=elapsed))
p.write_text(json.dumps(checks, indent=2) + "\n")
print(result.stdout.decode(errors="replace"))
print(f"{label}: exit {result.returncode}; {elapsed:.6f} s")
sys.exit(result.returncode)
