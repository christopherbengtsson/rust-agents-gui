"""Sequential ticket-02 evidence capture; no provider or authentication actions."""
import hashlib
import json
import pathlib
import statistics
import subprocess
import time

ROOT = pathlib.Path(__file__).resolve().parents[4]
OUT = pathlib.Path(__file__).resolve().parent
SUMMARY = {"source": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(), "checks": []}


def run(command, name, timeout=300):
    print("Running:", " ".join(command), flush=True)
    start = time.perf_counter()
    result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=timeout)
    elapsed = time.perf_counter() - start
    (OUT / (name + ".log")).write_text(result.stdout + result.stderr)
    SUMMARY["checks"].append({"command": command, "exit": result.returncode, "seconds": elapsed, "log": name + ".log"})
    (OUT / "verification.json").write_text(json.dumps(SUMMARY, indent=2) + "\n")
    print("Exit:", result.returncode, "elapsed:", round(elapsed, 3), flush=True)
    return result


# The complete gate is expected to stop at the deliberately unconverted callers.
run(["corepack", "pnpm", "verify"], "verify-complete")
run(["node", "scripts/cargo.mjs", "clippy", "--locked", "-p", "storage-checkpoint", "--all-targets", "--all-features", "--", "-D", "warnings"], "clippy-final")
build = run(["node", "scripts/cargo.mjs", "test", "--locked", "-p", "storage-checkpoint", "--lib", "--no-run", "--message-format=json-render-diagnostics"], "unit-compile-final")
if build.returncode == 0:
    messages = [json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")]
    executables = [m["executable"] for m in messages if m.get("reason") == "compiler-artifact" and m.get("executable") and m.get("profile", {}).get("test")]
    assert len(executables) == 1, executables
    executable = pathlib.Path(executables[0])
    SUMMARY["unit_executable_sha256"] = hashlib.sha256(executable.read_bytes()).hexdigest()
    timings = []
    for iteration in range(6):
        run([str(executable), "--test-threads=1"], "units-warmup" if iteration == 0 else f"units-run-{iteration}")
        if iteration:
            timings.append(SUMMARY["checks"][-1]["seconds"])
    SUMMARY["warm_units"] = {"median_seconds": statistics.median(timings), "range_seconds": [min(timings), max(timings)], "warmup_runs": 1, "measured_runs": 5, "threads": 1}
qualification = run(["node", "scripts/cargo.mjs", "build", "--locked", "-p", "storage-checkpoint", "--features", "qualification", "--bin", "qualify-storage"], "qualification-build-final")
if qualification.returncode == 0:
    disposable = "/private/tmp/aw-sqlx02-final-" + SUMMARY["source"][:7]
    result = run([str(ROOT / "target/debug/qualify-storage"), disposable], "qualification-final", timeout=60)
    if result.returncode == 0:
        (OUT / "qualification.json").write_text(json.dumps(json.loads(result.stdout), indent=2) + "\n")
run(["corepack", "pnpm", "exec", "tsc", "--noEmit"], "typescript-final")
tooling = sorted(str(p.relative_to(ROOT)) for p in (ROOT / "scripts").glob("*.test.mjs"))
run(["node", "--test", "--test-concurrency=1", *tooling], "tooling-final")
run(["node", "node_modules/vitest/vitest.mjs", "run", "--maxWorkers=1"], "ui-final")
(OUT / "verification.json").write_text(json.dumps(SUMMARY, indent=2) + "\n")
print("Final evidence captured", flush=True)
