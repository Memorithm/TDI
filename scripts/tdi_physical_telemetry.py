"""Linux physical measurements with explicit sensors, clocks and availability.

Read-only capacity discovery is local to the visible cgroup hierarchy. Process
measurements use wait4 for that child, not cumulative RUSAGE_CHILDREN shared by
other runs. These are trusted-process diagnostics, not a hostile-code sandbox.
"""
from __future__ import annotations

import math
import os
from pathlib import Path, PurePosixPath
import platform
import signal
import subprocess
import tempfile
import time

from tdi_engine_store import identity
import tdi_experiment_supervisor as durable


def _read(path, limit=65536):
    with Path(path).open("rb") as stream:
        raw = stream.read(limit + 1)
    if len(raw) > limit:
        raise ValueError("sensor payload exceeds bound")
    return raw.decode("ascii").strip()


def capacity_snapshot():
    """Sample Linux MemAvailable, CPU affinity and every visible cgroup-v2 limit.

    Unknown layout/sensor errors return unknown capacity. A zero computed
    capacity is unavailable. CPU slots are integer full-core budgets. No GPU or
    energy availability is inferred. Hidden ancestor limits outside a cgroup
    namespace are not observable and are reported as an explicit scope limit.
    """
    started, realtime = time.monotonic_ns(), time.time_ns()
    evidence = {"schema": 1, "sensor": "tdi-linux-capacity/v1", "sampled_monotonic_ns": str(started),
                "sampled_unix_ns": str(realtime), "clock": "time.monotonic_ns; freshness local to this boot",
                "scope": "local process affinity and visible cgroup-v2 ancestors; no physical reservation",
                "environment": {"system": platform.system(), "release": platform.release(), "machine": platform.machine()},
                "readings": [], "accelerator_memory": {"status": "unknown", "reason": "no qualified device sensor"},
                "energy": {"status": "unknown", "reason": "no qualified energy sensor"}}
    try:
        if platform.system() != "Linux":
            raise ValueError("Linux capacity sensor required")
        evidence["environment"]["boot_identity"] = identity("tdi-boot/v1", _read("/proc/sys/kernel/random/boot_id"))
        affinity = sorted(os.sched_getaffinity(0))
        cpu = len(affinity)
        evidence["environment"]["cpu_affinity"] = affinity
        memory = None
        for line in _read("/proc/meminfo").splitlines():
            if line.startswith("MemAvailable:"):
                fields = line.split()
                if len(fields) != 3 or fields[2] != "kB" or not fields[1].isdigit():
                    raise ValueError("invalid MemAvailable unit/value")
                memory = int(fields[1]) * 1024
        if memory is None:
            raise ValueError("MemAvailable is not exposed")
        evidence["readings"].append({"sensor": "/proc/meminfo:MemAvailable", "unit": "byte", "value": memory})
        cgroups = _read("/proc/self/cgroup").splitlines()
        if len(cgroups) != 1 or not cgroups[0].startswith("0::/"):
            raise ValueError("only unified cgroup-v2 is qualified")
        path = PurePosixPath(cgroups[0][3:])
        if ".." in path.parts:
            raise ValueError("unrecognized cgroup namespace path")
        mounts = [line.split() for line in _read("/proc/self/mountinfo").splitlines() if " - cgroup2 " in line]
        if len(mounts) != 1 or mounts[0][3:5] != ["/", "/sys/fs/cgroup"]:
            raise ValueError("unrecognized cgroup-v2 mount layout")
        mount = Path("/sys/fs/cgroup")
        node = mount.joinpath(*path.parts[1:])
        nodes = [node, *[p for p in node.parents if p == mount or mount in p.parents]]
        if len(nodes) > 64:
            raise ValueError("cgroup ancestor depth exceeds bound")
        for current in nodes:
            for sensor in ("memory.max", "cpu.max"):
                location = current / sensor
                # Controllers absent at the hierarchy root have no local limit;
                # absence inside a delegated subtree is not assumed unlimited.
                if not location.exists() and current == mount:
                    continue
                raw = _read(location, 128)
                if sensor == "memory.max":
                    if raw == "max":
                        evidence["readings"].append({"sensor": str(location), "unit": "byte", "value": None, "status": "unlimited-at-this-level"})
                        continue
                    if not raw.isdigit(): raise ValueError("invalid memory limit")
                    used = _read(current / "memory.current", 128)
                    if not used.isdigit(): raise ValueError("invalid memory usage")
                    free = max(0, int(raw) - int(used))
                    memory = min(memory, free)
                    evidence["readings"].append({"sensor": str(location), "unit": "byte", "limit": int(raw), "current": int(used), "available": free})
                else:
                    fields = raw.split()
                    if len(fields) != 2 or not fields[1].isdigit() or int(fields[1]) == 0:
                        raise ValueError("invalid CPU period")
                    if fields[0] != "max":
                        if not fields[0].isdigit(): raise ValueError("invalid CPU quota")
                        cpu = min(cpu, int(fields[0]) // int(fields[1]))
                    evidence["readings"].append({"sensor": str(location), "unit": "microsecond quota per period", "raw": raw})
        evidence["capacity"] = ({"status": "available", "cpu_slots": cpu, "available_memory_bytes": memory}
                                if cpu and memory else {"status": "unavailable", "reason": "zero effective CPU or memory capacity"})
    except (OSError, ValueError, AttributeError) as error:
        evidence["capacity"] = {"status": "unknown", "reason": str(error)[:256]}
    evidence["sampling_duration_ns"] = str(time.monotonic_ns() - started)
    evidence["environment_id"] = identity("tdi-local-environment/v1", evidence["environment"])
    evidence["observation_id"] = identity("tdi-capacity-observation/v1", evidence)
    return evidence


def measured_process(command, *, input_bytes=b"", timeout=10.0, max_output=1048576, env=None):
    """Run one pinned trusted process with bounded captured files and wait4 usage.

    Wall time includes launch, I/O and up to 5 ms completion polling latency.
    Child user/system CPU seconds and Linux ru_maxrss bytes are actual OS
    measurements. Unsupported platforms fail explicitly. Exit/timeout/output
    failure remains visible in the returned record; no performance is invented.
    """
    if (platform.system() != "Linux" or not hasattr(os, "wait4") or not isinstance(command, list)
            or not command or not 0 < timeout <= 3600 or not math.isfinite(timeout)
            or type(max_output) is not int or not 1 <= max_output <= 16 * 1024 * 1024
            or not isinstance(input_bytes, bytes) or len(input_bytes) > 1024 * 1024):
        raise durable.ContractError("unsupported measured-process platform or bounds")
    start = time.monotonic_ns()
    reason, usage = None, None
    with tempfile.TemporaryFile() as source, tempfile.TemporaryFile() as stdout, tempfile.TemporaryFile() as stderr:
        source.write(input_bytes); source.seek(0)
        child = subprocess.Popen(command, stdin=source, stdout=stdout, stderr=stderr,
                                 env=env or {"LANG": "C", "LC_ALL": "C"}, start_new_session=True)
        launched = time.monotonic_ns()
        try:
            while True:
                pid, status, usage = os.wait4(child.pid, os.WNOHANG)
                if pid:
                    child.returncode = os.waitstatus_to_exitcode(status)
                    break
                if time.monotonic_ns() - start > timeout * 1e9:
                    reason = "timeout"
                if os.fstat(stdout.fileno()).st_size > max_output or os.fstat(stderr.fileno()).st_size > max_output:
                    reason = "output-budget"
                if reason:
                    os.killpg(child.pid, signal.SIGKILL)
                    _, status, usage = os.wait4(child.pid, 0)
                    child.returncode = os.waitstatus_to_exitcode(status)
                    break
                time.sleep(0.005)
        finally:
            try: os.killpg(child.pid, signal.SIGKILL)
            except ProcessLookupError: pass
            if child.returncode is None:
                _, status, usage = os.wait4(child.pid, 0)
                child.returncode = os.waitstatus_to_exitcode(status)
        finish = time.monotonic_ns()
        stdout.seek(0); stderr.seek(0)
        out, err = stdout.read(max_output + 1), stderr.read(max_output + 1)
        if len(out) > max_output or len(err) > max_output:
            reason = "output-budget"
        measurements = {"schema": 1, "sensor": "python-os.wait4/linux/v1", "clock": "time.monotonic_ns",
                        "wall_ns": str(finish - start), "launch_ns": str(launched - start),
                        "completion_polling_granularity_ns": 5000000, "exit_code": child.returncode,
                        "technical_failure": reason, "user_cpu_seconds": usage.ru_utime,
                        "system_cpu_seconds": usage.ru_stime, "peak_rss_bytes": int(usage.ru_maxrss) * 1024,
                        "rss_scope": "wait4 child high-water mark; not summed concurrent descendants",
                        "stdout_bytes": os.fstat(stdout.fileno()).st_size, "stderr_bytes": os.fstat(stderr.fileno()).st_size,
                        "gpu_time": None, "accelerator_memory_bytes": None, "energy_joules": None}
        return child.returncode, out[:max_output], err[:max_output], measurements
