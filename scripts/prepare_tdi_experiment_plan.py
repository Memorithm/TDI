#!/usr/bin/env python3
"""Pin explicitly selected artifacts for a non-final software worker.

No repository-wide scan: historical/protected datasets must never be collected.
The operator must list all relevant inputs and use immutable storage while running.
"""
import argparse
from pathlib import Path
import tdi_experiment_supervisor as supervisor


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--worker", required=True)
    parser.add_argument("--artifact", action="append", required=True)
    parser.add_argument("--index", action="append", type=int, required=True)
    parser.add_argument("--domain", choices=["Development", "Validation"], required=True)
    parser.add_argument("--timeout", type=float, required=True)
    parser.add_argument("--output-limit", type=int, required=True)
    parser.add_argument("--plan", required=True, type=Path)
    args = parser.parse_args()
    root = args.root.resolve(strict=True)
    plan = {"schema": 1, "purpose": "development-software", "domain": args.domain,
            "indices": args.index, "argv": [args.worker],
            "artifacts": {path: supervisor.file_digest(supervisor.artifact(root, path))
                          for path in [args.worker, *args.artifact]},
            "timeout_seconds": args.timeout, "max_output_bytes": args.output_limit,
            "max_trials": len(args.index)}
    supervisor.validate(plan, root)
    # Refuse overwrite: freezing a different plan must use a new explicit path.
    with args.plan.open("x") as stream:
        stream.write(supervisor.canonical(plan) + "\n")


if __name__ == "__main__":
    main()
