"""Alternate reference/candidate shape validation on actual session records.

No optimizer or statistical-superiority claim: timings cover only the shape
walk, not JSON decoding, fsync, search or the complete adapter.
"""
import argparse
import hashlib
import json
from pathlib import Path
import statistics
import time

from test_json_shape_fastpath import reference
from tdi_experiment_supervisor import _validate_json_shape, strict_json


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--session-directory", type=Path, required=True)
    args = parser.parse_args()
    root = args.session_directory
    files = sorted((root / "session-commands").glob("*.json"))[:20]
    if len(files) != 20:
        parser.error("at least twenty session command records are required")
    files += [root / "session-open.json", root / "forge-final.json"]
    values, inputs = [], []
    for path in files:
        with path.open("rb") as stream:
            raw = stream.read(8 * 1024 * 1024 + 1)
        values.append(strict_json(raw, max_bytes=8 * 1024 * 1024, max_items=1000000))
        inputs.append({"file": str(path.relative_to(root)), "sha256": hashlib.sha256(raw).hexdigest()})
    times = {"reference": [], "candidate": []}
    functions = (("reference", reference), ("candidate", _validate_json_shape))
    for turn in range(10):
        for name, fn in (functions if turn % 2 == 0 else functions[::-1]):
            started = time.perf_counter_ns()
            for _ in range(30):
                for value in values:
                    fn(value, 64, 1000000, 1000000)
            times[name].append(time.perf_counter_ns() - started)
    print(json.dumps({"schema": "tdi-json-shape-microbenchmark/v1", "inputs": inputs,
                      "rounds": 10, "repetitions": 30, "nanoseconds": times,
                      "median_paired_reference_over_candidate": statistics.median(
                          a / b for a, b in zip(times["reference"], times["candidate"])),
                      "limits": "shared-host shape-validation only; no end-to-end or optimizer speed claim"}))


if __name__ == "__main__":
    main()
