"""Diagnostic profiler for the existing public session smoke workload.

This does not select policies or change search/durability contracts. Instrumented
times include profiler overhead and must not be used as benchmark speed claims.
"""
import argparse
import cProfile
import hashlib
from pathlib import Path
import pstats

import tdi_optuna_benchmark as benchmark
from tdi_engine_store import atomic_json


def function_rows(stats):
    rows = []
    for (filename, line, name), (primitive, calls, own, cumulative, _) in stats.items():
        rows.append({"file": filename, "line": line, "function": name,
                     "primitive_calls": primitive, "calls": calls,
                     "self_seconds": own, "cumulative_seconds": cumulative})
    return sorted(rows, key=lambda row: (-row["self_seconds"], row["file"], row["line"]))


def profile_run(worker, output, max_seconds=600):
    output = Path(output)
    output.mkdir(exist_ok=False)
    profiler = cProfile.Profile()
    status = "incomplete"
    report_identity = None
    try:
        report = profiler.runcall(benchmark.run, output / "experiment", worker,
                                 "session-smoke", max_seconds)
        benchmark.verify_report(output / "experiment/report.json")
        report_identity = report["identity"]
        status = "complete"
    finally:
        # No pickle output or pickle input: retained diagnostics are plain JSON.
        stats = pstats.Stats(profiler)
        atomic_json(output / "profile.json", {
            "schema": "tdi-session-profile/v1", "status": status,
            "benchmark_report_identity": report_identity,
            "profile": "session-smoke", "tdi_source_commit": benchmark.git_revision(),
            "profiler_source_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
            "scope": "Python caller including wait and IO; Rust internal functions not profiled",
            "limits": ["instrumentation overhead; not an uninstrumented speed comparison",
                       "cumulative times overlap and must not be added",
                       "self times exclude child functions; IO wait is not CPU time",
                       "failure retains partial evidence and is not retried"],
            "total_self_seconds": stats.total_tt,
            "functions": function_rows(stats.stats)})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge-worker", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--max-seconds", type=int, default=600)
    args = parser.parse_args()
    if not 1 <= args.max_seconds <= 1800:
        parser.error("--max-seconds must be between 1 and 1800")
    profile_run(args.forge_worker, args.output, args.max_seconds)


if __name__ == "__main__":
    main()
