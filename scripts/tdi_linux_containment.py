#!/usr/bin/env python3
"""Linux cgroup-v2 containment primitives for non-final TDI workers.

The module enforces process-tree CPU, memory, swap and PID limits when a writable
cgroup-v2 parent has been deliberately delegated to the supervisor. It is not a
filesystem/network sandbox and it never treats device visibility as a GPU-memory
quota. Untrusted workers therefore fail closed unless a separately qualified
sandbox boundary is supplied by a future contract.
"""
from __future__ import annotations

import argparse
import contextlib
from dataclasses import dataclass
import json
import os
from pathlib import Path
import select
import shutil
import subprocess
import sys
import time
import uuid


CGROUP_ROOT = Path("/sys/fs/cgroup")
PROFILE_SCHEMA = 1
DEFAULT_CPU_PERIOD_US = 100_000
MAX_CPU_PERIOD_US = 1_000_000
MIN_CPU_PERIOD_US = 1_000


class ContainmentError(RuntimeError):
    """The requested Linux containment contract cannot be applied or verified."""


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def _require_exact_fields(value, required, *, name):
    if not isinstance(value, dict) or set(value) != set(required):
        raise ContainmentError(f"{name} has unknown or missing fields")


def _positive_int(value, name):
    if type(value) is not int or value <= 0:
        raise ContainmentError(f"{name} must be a positive integer")
    return value


@dataclass(frozen=True)
class ResourceProfile:
    """Versioned process-tree limits; units are explicit in field names."""

    memory_max_bytes: int
    swap_max_bytes: int
    cpu_quota_us: int
    cpu_period_us: int
    pids_max: int
    trust: str
    gpu_required: bool
    gpu_memory_max_bytes: int | None

    @classmethod
    def from_json(cls, value):
        _require_exact_fields(
            value,
            {
                "schema",
                "memory_max_bytes",
                "swap_max_bytes",
                "cpu_quota_us",
                "cpu_period_us",
                "pids_max",
                "trust",
                "gpu_required",
                "gpu_memory_max_bytes",
            },
            name="resource profile",
        )
        if value["schema"] != PROFILE_SCHEMA:
            raise ContainmentError("unsupported resource profile schema")
        memory = _positive_int(value["memory_max_bytes"], "memory_max_bytes")
        swap = value["swap_max_bytes"]
        if type(swap) is not int or swap < 0:
            raise ContainmentError("swap_max_bytes must be a non-negative integer")
        quota = _positive_int(value["cpu_quota_us"], "cpu_quota_us")
        period = _positive_int(value["cpu_period_us"], "cpu_period_us")
        if not MIN_CPU_PERIOD_US <= period <= MAX_CPU_PERIOD_US:
            raise ContainmentError("cpu_period_us outside supported kernel range")
        if quota > period * 1024:
            raise ContainmentError("cpu_quota_us exceeds bounded 1024-CPU envelope")
        pids = _positive_int(value["pids_max"], "pids_max")
        if pids > 1_000_000:
            raise ContainmentError("pids_max exceeds TDI operational bound")
        if value["trust"] not in ("trusted", "untrusted"):
            raise ContainmentError("trust must be trusted or untrusted")
        if type(value["gpu_required"]) is not bool:
            raise ContainmentError("gpu_required must be boolean")
        gpu_memory = value["gpu_memory_max_bytes"]
        if gpu_memory is not None:
            _positive_int(gpu_memory, "gpu_memory_max_bytes")
        return cls(memory, swap, quota, period, pids, value["trust"],
                   value["gpu_required"], gpu_memory)

    def as_json(self):
        return {
            "schema": PROFILE_SCHEMA,
            "memory_max_bytes": self.memory_max_bytes,
            "swap_max_bytes": self.swap_max_bytes,
            "cpu_quota_us": self.cpu_quota_us,
            "cpu_period_us": self.cpu_period_us,
            "pids_max": self.pids_max,
            "trust": self.trust,
            "gpu_required": self.gpu_required,
            "gpu_memory_max_bytes": self.gpu_memory_max_bytes,
        }

    def validate_static_capabilities(self):
        if self.trust == "untrusted":
            raise ContainmentError(
                "untrusted workers require a separately qualified filesystem/network sandbox"
            )
        if self.gpu_memory_max_bytes is not None:
            raise ContainmentError(
                "GPU memory hard limits are not qualified; device visibility is not a VRAM quota"
            )
        if self.gpu_required and not _nvidia_device_visible():
            raise ContainmentError("resource profile requires a visible NVIDIA device")


def _nvidia_device_visible():
    return any(Path("/dev").glob("nvidia[0-9]*")) or Path("/proc/driver/nvidia/version").exists()


def _read_text(path):
    try:
        return path.read_text(encoding="utf-8").strip()
    except OSError as error:
        raise ContainmentError(f"cannot read {path}: {error}") from error


def _write_text(path, value):
    try:
        path.write_text(str(value), encoding="ascii")
    except OSError as error:
        raise ContainmentError(f"cannot write {path}: {error}") from error


def _flat_keys(text):
    values = {}
    for line in text.splitlines():
        fields = line.split()
        if len(fields) >= 2:
            try:
                values[fields[0]] = int(fields[1])
            except ValueError:
                values[fields[0]] = " ".join(fields[1:])
    return values


def _page_floor(value):
    page = int(os.sysconf("SC_PAGE_SIZE"))
    if value < page:
        raise ContainmentError(f"memory limit must be at least one system page ({page} bytes)")
    return (value // page) * page


@dataclass(frozen=True)
class CgroupCapabilities:
    root: str
    parent: str
    unified_v2: bool
    controllers_available: tuple[str, ...]
    controllers_enabled: tuple[str, ...]
    writable_parent: bool
    cgroup_kill: bool
    pidfd: bool
    nvidia_visible: bool
    user_namespace_tool: bool

    def as_json(self):
        return {
            "root": self.root,
            "parent": self.parent,
            "unified_v2": self.unified_v2,
            "controllers_available": list(self.controllers_available),
            "controllers_enabled": list(self.controllers_enabled),
            "writable_parent": self.writable_parent,
            "cgroup_kill": self.cgroup_kill,
            "pidfd": self.pidfd,
            "nvidia_visible": self.nvidia_visible,
            "user_namespace_tool": self.user_namespace_tool,
            "sandbox_qualified": False,
            "gpu_memory_quota_qualified": False,
        }


def doctor(parent=CGROUP_ROOT):
    parent = Path(parent).resolve()
    root = CGROUP_ROOT.resolve()
    unified = (root / "cgroup.controllers").is_file()
    available = tuple(sorted(_read_text(parent / "cgroup.controllers").split())) if (parent / "cgroup.controllers").is_file() else ()
    enabled = tuple(sorted(item.lstrip("+") for item in _read_text(parent / "cgroup.subtree_control").split())) if (parent / "cgroup.subtree_control").is_file() else ()
    writable = os.access(parent, os.W_OK) and os.access(parent / "cgroup.procs", os.W_OK)
    return CgroupCapabilities(
        root=str(root),
        parent=str(parent),
        unified_v2=unified,
        controllers_available=available,
        controllers_enabled=enabled,
        writable_parent=writable,
        cgroup_kill=(parent / "cgroup.kill").exists() or _child_has_file(parent, "cgroup.kill"),
        pidfd=hasattr(os, "pidfd_open"),
        nvidia_visible=_nvidia_device_visible(),
        user_namespace_tool=shutil.which("unshare") is not None,
    )


def _child_has_file(parent, name):
    for child in parent.iterdir() if parent.is_dir() else ():
        if child.is_dir() and (child / name).exists():
            return True
    return False


class CgroupV2Attempt:
    """One isolated cgroup-v2 process tree with verified hard limits."""

    def __init__(self, parent, attempt_id, profile: ResourceProfile):
        self.parent = Path(parent).resolve()
        self.attempt_id = attempt_id
        self.profile = profile
        if not attempt_id or any(ch not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_" for ch in attempt_id):
            raise ContainmentError("invalid attempt identifier for cgroup path")
        self.path = self.parent / f"tdi-{attempt_id}"
        self._attach_fd = None
        self._created = False

    def create(self):
        self.profile.validate_static_capabilities()
        caps = doctor(self.parent)
        if not caps.unified_v2:
            raise ContainmentError("cgroup v2 unified hierarchy is unavailable")
        if not caps.writable_parent:
            raise ContainmentError("cgroup parent is not deliberately delegated/writable")
        required = {"cpu", "memory", "pids"}
        missing = required - set(caps.controllers_available)
        if missing:
            raise ContainmentError("missing cgroup controllers: " + ",".join(sorted(missing)))
        disabled = required - set(caps.controllers_enabled)
        if disabled:
            raise ContainmentError("required controllers are not enabled for children: " + ",".join(sorted(disabled)))
        if self.path.exists():
            raise ContainmentError("attempt cgroup already exists; reconcile it before reuse")
        try:
            self.path.mkdir(mode=0o700)
        except OSError as error:
            raise ContainmentError(f"cannot create attempt cgroup: {error}") from error
        self._created = True
        try:
            for required_file in ("cgroup.procs", "cgroup.events", "cgroup.kill", "memory.max",
                                  "memory.swap.max", "memory.oom.group", "cpu.max", "pids.max"):
                if not (self.path / required_file).exists():
                    raise ContainmentError(f"required cgroup-v2 interface absent: {required_file}")
            memory = _page_floor(self.profile.memory_max_bytes)
            swap = 0 if self.profile.swap_max_bytes == 0 else _page_floor(self.profile.swap_max_bytes)
            _write_text(self.path / "memory.max", memory)
            _write_text(self.path / "memory.swap.max", swap)
            _write_text(self.path / "memory.oom.group", 1)
            _write_text(self.path / "cpu.max", f"{self.profile.cpu_quota_us} {self.profile.cpu_period_us}")
            _write_text(self.path / "pids.max", self.profile.pids_max)
            self._verify_limit("memory.max", str(memory))
            self._verify_limit("memory.swap.max", str(swap))
            self._verify_limit("memory.oom.group", "1")
            self._verify_limit("cpu.max", f"{self.profile.cpu_quota_us} {self.profile.cpu_period_us}")
            self._verify_limit("pids.max", str(self.profile.pids_max))
            self._attach_fd = os.open(self.path / "cgroup.procs", os.O_WRONLY)
            return self
        except BaseException:
            with contextlib.suppress(Exception):
                self.kill()
            with contextlib.suppress(Exception):
                self.cleanup(timeout=1.0)
            raise

    def _verify_limit(self, name, expected):
        actual = _read_text(self.path / name)
        if actual != expected:
            raise ContainmentError(f"cgroup limit verification failed for {name}: {actual!r} != {expected!r}")

    def preexec_attach(self):
        """Return the minimal child hook that attaches before worker exec()."""
        if self._attach_fd is None:
            raise ContainmentError("attempt cgroup has not been created")
        fd = self._attach_fd
        def attach():
            os.write(fd, str(os.getpid()).encode("ascii"))
        return attach

    def close_attach_fd(self):
        if self._attach_fd is not None:
            os.close(self._attach_fd)
            self._attach_fd = None

    def kill(self):
        if self.path.exists():
            _write_text(self.path / "cgroup.kill", "1")

    def is_populated(self):
        events = _flat_keys(_read_text(self.path / "cgroup.events"))
        return events.get("populated") == 1

    def metrics(self):
        values = {"backend": "linux-cgroup-v2", "attempt_cgroup": self.path.name}
        for name in ("memory.current", "memory.peak", "memory.swap.current", "pids.current", "pids.peak"):
            path = self.path / name
            if path.exists():
                raw = _read_text(path)
                values[name.replace(".", "_")] = int(raw) if raw.isdigit() else raw
        for name in ("cpu.stat", "memory.events", "pids.events", "cgroup.events"):
            path = self.path / name
            if path.exists():
                values[name.replace(".", "_")] = _flat_keys(_read_text(path))
        values["effective_limits"] = {
            "memory_max_bytes": int(_read_text(self.path / "memory.max")),
            "swap_max_bytes": int(_read_text(self.path / "memory.swap.max")),
            "cpu_max": _read_text(self.path / "cpu.max"),
            "pids_max": int(_read_text(self.path / "pids.max")),
        }
        return values

    def cleanup(self, timeout=5.0):
        self.close_attach_fd()
        if not self.path.exists():
            return
        deadline = time.monotonic() + timeout
        while self.is_populated() and time.monotonic() < deadline:
            time.sleep(0.01)
        if self.is_populated():
            raise ContainmentError("attempt cgroup remains populated after cleanup deadline")
        try:
            self.path.rmdir()
        except OSError as error:
            raise ContainmentError(f"cannot remove empty attempt cgroup: {error}") from error
        self._created = False

    @classmethod
    def reconcile_existing(cls, parent, attempt_id, timeout=5.0):
        """Kill and remove a stale attempt tree before recording interruption.

        The caller must already know that this attempt is not allowed to continue.
        ``cgroup.kill`` provides tree-wide kill semantics protected against
        concurrent forks; failure to verify an empty cgroup is fail-closed.
        """
        path = Path(parent).resolve() / f"tdi-{attempt_id}"
        if not path.exists():
            return {"present": False, "killed": False}
        if not (path / "cgroup.kill").exists() or not (path / "cgroup.events").exists():
            raise ContainmentError("stale attempt lacks required cgroup recovery interfaces")
        _write_text(path / "cgroup.kill", "1")
        deadline = time.monotonic() + timeout
        while _flat_keys(_read_text(path / "cgroup.events")).get("populated") == 1 and time.monotonic() < deadline:
            time.sleep(0.01)
        if _flat_keys(_read_text(path / "cgroup.events")).get("populated") == 1:
            raise ContainmentError("stale attempt remains populated after recovery deadline")
        try:
            path.rmdir()
        except OSError as error:
            raise ContainmentError(f"cannot remove recovered attempt cgroup: {error}") from error
        return {"present": True, "killed": True}


@dataclass
class ReaperHandle:
    process: subprocess.Popen
    control_fd: int

    def release(self, timeout=5.0):
        with contextlib.suppress(OSError):
            os.write(self.control_fd, b"release\n")
        with contextlib.suppress(OSError):
            os.close(self.control_fd)
        try:
            code = self.process.wait(timeout=timeout)
        except subprocess.TimeoutExpired as error:
            self.process.kill()
            self.process.wait()
            raise ContainmentError("containment reaper did not stop") from error
        if code != 0:
            raise ContainmentError(f"containment reaper exited with {code}")


def spawn_reaper(cgroup_path, marker_path):
    """Start an independent pidfd-bound recovery owner for supervisor death."""
    if not hasattr(os, "pidfd_open"):
        raise ContainmentError("pidfd_open is required for PID-reuse-safe recovery")
    pidfd = os.pidfd_open(os.getpid(), 0)
    read_fd, write_fd = os.pipe()
    try:
        process = subprocess.Popen(
            [sys.executable, str(Path(__file__).resolve()), "_reaper",
             "--pidfd", str(pidfd), "--control-fd", str(read_fd),
             "--cgroup", str(Path(cgroup_path).resolve()),
             "--marker", str(Path(marker_path).resolve())],
            stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            close_fds=True,
            pass_fds=(pidfd, read_fd),
            start_new_session=True,
            env={"LANG": "C", "LC_ALL": "C"},
        )
    except BaseException:
        os.close(read_fd)
        os.close(write_fd)
        os.close(pidfd)
        raise
    os.close(read_fd)
    os.close(pidfd)
    return ReaperHandle(process, write_fd)


def _atomic_marker(path, payload):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temp = path.with_name(f".{path.name}.{os.getpid()}.{uuid.uuid4().hex}.tmp")
    with temp.open("x", encoding="utf-8") as stream:
        stream.write(canonical(payload) + "\n")
        stream.flush()
        os.fsync(stream.fileno())
    os.replace(temp, path)
    fd = os.open(path.parent, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def reaper_main(pidfd, control_fd, cgroup_path, marker_path):
    poller = select.poll()
    poller.register(pidfd, select.POLLIN | select.POLLHUP | select.POLLERR)
    poller.register(control_fd, select.POLLIN | select.POLLHUP | select.POLLERR)
    while True:
        for fd, _events in poller.poll():
            if fd == control_fd:
                data = os.read(control_fd, 64)
                if data.startswith(b"release"):
                    return 0
                if not data:
                    # The supervisor disappeared without a release token.
                    fd = pidfd
                else:
                    continue
            if fd == pidfd:
                cgroup = Path(cgroup_path)
                outcome = "already-gone"
                error = None
                if cgroup.exists():
                    try:
                        _write_text(cgroup / "cgroup.kill", "1")
                        deadline = time.monotonic() + 5.0
                        while cgroup.exists() and _flat_keys(_read_text(cgroup / "cgroup.events")).get("populated") == 1 and time.monotonic() < deadline:
                            time.sleep(0.02)
                        outcome = "killed" if cgroup.exists() else "gone"
                    except Exception as exc:  # marker must retain recovery failure
                        outcome = "error"
                        error = str(exc)
                payload = {"schema": 1, "kind": "supervisor-death-recovery",
                           "cgroup": cgroup.name, "outcome": outcome,
                           "wall_time_ns": time.time_ns()}
                if error is not None:
                    payload["error"] = error[:1024]
                try:
                    _atomic_marker(marker_path, payload)
                except Exception:
                    return 74
                return 0 if outcome != "error" else 75


def parse_profile_file(path):
    try:
        raw = Path(path).read_text(encoding="utf-8")
        value = json.loads(raw)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ContainmentError(f"invalid resource profile file: {error}") from error
    return ResourceProfile.from_json(value)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    doctor_parser = sub.add_parser("doctor", help="report capabilities without launching a worker")
    doctor_parser.add_argument("--parent", type=Path, default=CGROUP_ROOT)
    profile_parser = sub.add_parser("validate-profile", help="validate a resource profile without execution")
    profile_parser.add_argument("profile", type=Path)
    internal = sub.add_parser("_reaper")
    internal.add_argument("--pidfd", type=int, required=True)
    internal.add_argument("--control-fd", type=int, required=True)
    internal.add_argument("--cgroup", type=Path, required=True)
    internal.add_argument("--marker", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "doctor":
            print(canonical({"schema": 1, "operation": "doctor", "capabilities": doctor(args.parent).as_json()}))
            return 0
        if args.command == "validate-profile":
            profile = parse_profile_file(args.profile)
            profile.validate_static_capabilities()
            print(canonical({"schema": 1, "operation": "validate-profile", "status": "ok", "profile": profile.as_json()}))
            return 0
        if args.command == "_reaper":
            return reaper_main(args.pidfd, args.control_fd, args.cgroup, args.marker)
    except ContainmentError as error:
        print(canonical({"schema": 1, "operation": args.command, "status": "error", "error": str(error)}))
        print(str(error), file=sys.stderr)
        return 21
    raise AssertionError("unreachable command")


if __name__ == "__main__":
    raise SystemExit(main())
