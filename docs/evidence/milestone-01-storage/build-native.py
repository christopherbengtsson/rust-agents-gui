#!/usr/bin/env python3
"""Build the retained 3.54.1 source into a private probe library, outside the repo."""
import argparse
import glob
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument("--source")
args = parser.parse_args()
sources = [args.source] if args.source else glob.glob(str(Path.home() / ".cargo/registry/src/*/libsqlite3-sys-0.40.0/sqlite3/sqlite3.c"))
if len(sources) != 1:
    raise SystemExit("Supply --source pointing at the locked sys 0.40.0 amalgamation")
source = Path(sources[0]).resolve()
if '#define SQLITE_VERSION        "3.54.1"' not in source.read_text():
    raise SystemExit("Unexpected amalgamation version")
os.umask(0o077)
root = Path(tempfile.mkdtemp(prefix="storage-native-06a-"))
command = ["clang", "-dynamiclib", "-O2", "-DSQLITE_THREADSAFE=1", str(source), "-o", str(root / "libsqlite3.dylib")]
start = time.perf_counter()
result = subprocess.run(command, capture_output=True, text=True, timeout=180)
record = {"source": str(source), "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(), "command": command, "exit_code": result.returncode, "wall_seconds": time.perf_counter() - start, "stdout": result.stdout, "stderr": result.stderr, "private_root": str(root), "compiler": subprocess.check_output(["clang", "--version"], text=True), "qualification": "same SQLite source as app, distinct probe compile options; not the app artifact"}
if result.returncode == 0:
    record["library_sha256"] = hashlib.sha256((root / "libsqlite3.dylib").read_bytes()).hexdigest()
print(json.dumps(record, indent=2))
raise SystemExit(result.returncode)
