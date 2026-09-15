#!/usr/bin/env python3
"""Pin explicitly selected artifacts for a non-final software worker.

No repository-wide scan: historical/protected datasets must never be collected.
The operator must list all relevant inputs and use immutable storage while running.
Schema 1 preserves the legacy process-group path. Schema 2 binds a cgroup-v2
resource profile. Schema 3 additionally embeds a validated ExperimentSpec/v1;
the delegated cgroup path remains deployment state supplied at execution time.
"""
import argparse
from pathlib import Path
import tdi_experiment_supervisor as supervisor
import tdi_linux_contract as contained


def _resource_profile(args):
    return {
        "schema": 1,
        "memory_max_bytes": args.memory_max_bytes,
        "swap_max_bytes": args.swap_max_bytes,
        "cpu_quota_us": args.cpu_quota_us,
        "cpu_period_us": args.cpu_period_us,
        "pids_max": args.pids_max,
        "trust": args.trust,
        "gpu_required": args.gpu_required,
        "gpu_memory_max_bytes": args.gpu_memory_max_bytes,
    }


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
    parser.add_argument("--containment", choices=["legacy", "cgroup-v2"], default="legacy")
    parser.add_argument(
        "--experiment-spec",
        type=Path,
        help="ExperimentSpec/v1 JSON; requires cgroup-v2 and creates plan schema 3",
    )
    parser.add_argument("--memory-max-bytes", type=int)
    parser.add_argument("--swap-max-bytes", type=int, default=0)
    parser.add_argument("--cpu-quota-us", type=int)
    parser.add_argument("--cpu-period-us", type=int, default=100_000)
    parser.add_argument("--pids-max", type=int)
    parser.add_argument("--trust", choices=["trusted", "untrusted"], default="trusted")
    parser.add_argument("--gpu-required", action="store_true")
    parser.add_argument("--gpu-memory-max-bytes", type=int)
    args = parser.parse_args()
    if args.experiment_spec is not None and args.containment != "cgroup-v2":
        parser.error("--experiment-spec requires --containment cgroup-v2")

    root = args.root.resolve(strict=True)
    selected = list(dict.fromkeys([args.worker, *args.artifact]))
    plan = {
        "schema": 1,
        "purpose": "development-software",
        "domain": args.domain,
        "indices": args.index,
        "argv": [args.worker],
        "artifacts": {path: supervisor.file_digest(supervisor.artifact(root, path))
                      for path in selected},
        "timeout_seconds": args.timeout,
        "max_output_bytes": args.output_limit,
        "max_trials": len(args.index),
    }
    if args.containment == "cgroup-v2":
        missing = [name for name, value in (
            ("--memory-max-bytes", args.memory_max_bytes),
            ("--cpu-quota-us", args.cpu_quota_us),
            ("--pids-max", args.pids_max),
        ) if value is None]
        if missing:
            parser.error("cgroup-v2 containment requires " + ", ".join(missing))
        plan["schema"] = 2
        plan["execution"] = {"backend": "linux-cgroup-v2", "profile": _resource_profile(args)}
        if args.experiment_spec is not None:
            plan["schema"] = 3
            plan["experiment"] = supervisor.strict_json(args.experiment_spec.read_bytes())

    if plan["schema"] in (2, 3):
        contained.validate_plan(plan, root)
    else:
        supervisor.validate(plan, root)
    # Refuse overwrite: freezing a different plan must use a new explicit path.
    with args.plan.open("x", encoding="utf-8") as stream:
        stream.write(supervisor.canonical(plan) + "\n")


if __name__ == "__main__":
    main()
