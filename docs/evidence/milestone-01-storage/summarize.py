#!/usr/bin/env python3
"""Aggregate retained measurements without treating run quantiles as pooled data."""
import collections
import hashlib
import json
from pathlib import Path
import statistics

ROOT = Path(__file__).resolve().parent


def summary(values):
    return {"n": len(values), "median": statistics.median(values), "min": min(values), "max": max(values)}


def main():
    groups = collections.defaultdict(list)
    inputs = []
    failures = []
    for name in ["driver-results-screen.jsonl", "driver-results-finalists.jsonl", "driver-results-scale.jsonl"]:
        path = ROOT / name
        if not path.exists():
            continue
        inputs.append(path)
        for line in path.read_text().splitlines():
            row = json.loads(line)
            if "exit_code" not in row:
                continue
            if row.get("exit_code") != 0:
                failures.append(row)
                continue
            result = row.get("result", {})
            if "events_per_s" not in result:
                continue
            key = (row.get("driver"), row.get("phase"), row.get("label"))
            if row.get("repeat") == 0 and (row.get("phase") == "finalists" or str(row.get("label")).startswith("history-")):
                continue
            groups[key].append(result)
    output = {"meaning": "median/min/max of run statistics; exclude repeat0 warmup for finalists/history; screening and active sweep are single samples; per-run p99 medians are not pooled percentiles", "driver": [], "failed_runtime_commands": failures}
    scalar = ["events_per_s", "committed_events", "rejected_transactions", "reader_errors", "elapsed_s", "writer_drain_ms", "writer_close_ms", "sampled_main_wal_shm_bytes", "closed_bytes"]
    nested = ["queue", "service", "history", "reader_history", "batch_formation", "event_durable"]
    for (driver, phase, label), rows in sorted(groups.items()):
        entry = {"driver": driver, "phase": phase, "label": label, "runs": len(rows)}
        for key in scalar:
            values = [row[key] for row in rows if isinstance(row.get(key), (int, float))]
            if values:
                entry[key] = summary(values)
        for key in nested:
            values = [row[key]["p99_ms"] for row in rows if "p99_ms" in row.get(key, {})]
            if values:
                entry[key + "_p99_ms"] = summary(values)
        entry["errors"] = [error for row in rows for error in row.get("errors", [])]
        output["driver"].append(entry)
    native = ROOT / "native-results.json"
    if native.exists():
        inputs.append(native)
        rows = json.loads(native.read_text())
        output["main_native_crash_passes"] = sum(row["pass"] for row in rows["crashes"])
        maintenance = ROOT / "native-results-maintenance.json"
        if maintenance.exists():
            inputs.append(maintenance)
            output["maintenance_native_crash_passes"] = sum(row["pass"] for row in json.loads(maintenance.read_text())["crashes"])
        output["native_crash_passes"] = output["main_native_crash_passes"] + output.get("maintenance_native_crash_passes", 0)
        for section, field, metrics in [("layouts", "mode", ["write_ms", "payload_read_verify_ms", "bytes"]), ("transcripts", "mode", ["write_ms", "three_pages_ms", "bytes"]), ("cache", "split", ["rebuild_ms", "database_pages_bytes"])]:
            output[section] = []
            for value in sorted(set(row[field] for row in rows[section])):
                selected = [row for row in rows[section] if row[field] == value and not row["warmup"]]
                output[section].append({field: value, **{metric: summary([row[metric] for row in selected]) for metric in metrics}})
    output["input_sha256"] = {path.name: hashlib.sha256(path.read_bytes()).hexdigest() for path in inputs}
    print(json.dumps(output, indent=2))


if __name__ == "__main__":
    main()
