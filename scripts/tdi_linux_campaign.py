#!/usr/bin/env python3
"""CLI for schema-2 TDI Development/Validation campaigns under cgroup v2."""
import argparse
from pathlib import Path
import signal
import sqlite3
import sys

import tdi_experiment_supervisor as durable
from tdi_linux_containment import ContainmentError
from tdi_linux_runner import exit_code_for_records, run

EXIT_CAPABILITY = 23


def emit(payload, diagnostic=None):
    print(durable.canonical(payload))
    if diagnostic:
        print(diagnostic, file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True, type=Path)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--journal", required=True, type=Path)
    parser.add_argument("--cgroup-parent", required=True, type=Path)
    parser.add_argument("--recovery-dir", type=Path)
    parser.add_argument("--anchor", type=Path)
    args = parser.parse_args()
    stopped = False

    def stop(_signum, _frame):
        nonlocal stopped
        stopped = True

    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGTERM, stop)
    try:
        plan = durable.strict_json(args.plan.read_bytes())
        records = run(plan, args.root, args.journal, args.cgroup_parent,
                      lambda: stopped, anchor_path=args.anchor,
                      recovery_dir=args.recovery_dir)
        code = exit_code_for_records(records, stopped=stopped)
        status = "cancelled" if code == durable.EXIT_CANCELLED else (
            "ok" if code == durable.EXIT_OK else "trial-failure")
        emit({"schema": 1, "operation": "run", "status": status,
              "exit_code": code, "records": records},
             None if code == durable.EXIT_OK else
             "TDI contained runner recorded a technical trial failure")
        return code
    except durable.ContractError as error:
        emit({"schema": 1, "operation": "run", "status": "contract-error",
              "exit_code": durable.EXIT_CONTRACT, "error": str(error)}, str(error))
        return durable.EXIT_CONTRACT
    except ContainmentError as error:
        emit({"schema": 1, "operation": "run", "status": "capability-error",
              "exit_code": EXIT_CAPABILITY, "error": str(error)}, str(error))
        return EXIT_CAPABILITY
    except (durable.StorageError, sqlite3.Error, OSError) as error:
        emit({"schema": 1, "operation": "run", "status": "storage-error",
              "exit_code": durable.EXIT_STORAGE, "error": str(error)}, str(error))
        return durable.EXIT_STORAGE


if __name__ == "__main__":
    raise SystemExit(main())
