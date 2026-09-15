#!/usr/bin/env python3
"""Internal launcher: attach this exact child to a delegated cgroup, then exec."""
import argparse
import os
from pathlib import Path
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--cgroup-procs", required=True, type=Path)
    parser.add_argument("worker_argv", nargs=argparse.REMAINDER)
    args = parser.parse_args()
    worker_argv = list(args.worker_argv)
    if worker_argv and worker_argv[0] == "--":
        worker_argv.pop(0)
    if not worker_argv or any("\0" in value for value in worker_argv):
        parser.error("worker argv must be non-empty and NUL-free")
    try:
        args.cgroup_procs.write_text(str(os.getpid()), encoding="ascii")
    except OSError as error:
        print(f"cannot attach worker to cgroup: {error}", file=sys.stderr)
        return 23
    os.execve(worker_argv[0], worker_argv, {"LANG": "C", "LC_ALL": "C"})


if __name__ == "__main__":
    raise SystemExit(main())
