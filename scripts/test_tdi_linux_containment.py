import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from unittest import mock

MODULE = Path(__file__).with_name("tdi_linux_containment.py")
spec = importlib.util.spec_from_file_location("containment", MODULE)
containment = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = containment
spec.loader.exec_module(containment)


class ContainmentTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def profile(self, **changes):
        value = {
            "schema": 1,
            "memory_max_bytes": 64 * 1024 * 1024,
            "swap_max_bytes": 0,
            "cpu_quota_us": 50_000,
            "cpu_period_us": 100_000,
            "pids_max": 32,
            "trust": "trusted",
            "gpu_required": False,
            "gpu_memory_max_bytes": None,
        }
        value.update(changes)
        return containment.ResourceProfile.from_json(value)

    def test_profile_is_strict_bounded_and_fail_closed_for_untrusted_or_vram_quota(self):
        self.assertEqual(self.profile().pids_max, 32)
        for changes in [
            {"cpu_period_us": 999},
            {"pids_max": 0},
            {"memory_max_bytes": 0},
            {"swap_max_bytes": -1},
            {"trust": "mystery"},
        ]:
            with self.assertRaises(containment.ContainmentError):
                self.profile(**changes)
        with self.assertRaises(containment.ContainmentError):
            self.profile(trust="untrusted").validate_static_capabilities()
        with self.assertRaises(containment.ContainmentError):
            self.profile(gpu_memory_max_bytes=1024).validate_static_capabilities()

    def _fake_parent(self):
        parent = self.root / "cg"
        parent.mkdir()
        (parent / "cgroup.controllers").write_text("cpu memory pids\n")
        (parent / "cgroup.subtree_control").write_text("cpu memory pids\n")
        (parent / "cgroup.procs").write_text("")
        return parent

    def _populate_child_interfaces(self, path):
        files = {
            "cgroup.procs": "",
            "cgroup.events": "populated 0\nfrozen 0\n",
            "cgroup.kill": "",
            "memory.max": "max\n",
            "memory.swap.max": "max\n",
            "memory.oom.group": "0\n",
            "memory.current": "0\n",
            "memory.peak": "0\n",
            "memory.swap.current": "0\n",
            "memory.events": "oom 0\noom_kill 0\n",
            "cpu.max": "max 100000\n",
            "cpu.stat": "usage_usec 0\n",
            "pids.max": "max\n",
            "pids.current": "0\n",
            "pids.peak": "0\n",
            "pids.events": "max 0\n",
        }
        for name, content in files.items():
            (path / name).write_text(content)

    def test_fake_cgroup_applies_and_verifies_all_declared_limits(self):
        parent = self._fake_parent()
        attempt = containment.CgroupV2Attempt(parent, "abc123", self.profile())
        real_mkdir = Path.mkdir
        def materializing_mkdir(path, *args, **kwargs):
            result = real_mkdir(path, *args, **kwargs)
            if path == attempt.path:
                self._populate_child_interfaces(path)
            return result
        with mock.patch.object(containment, "CGROUP_ROOT", parent), \
             mock.patch.object(Path, "mkdir", materializing_mkdir):
            attempt.create()
        try:
            limits = attempt.metrics()["effective_limits"]
            page = os.sysconf("SC_PAGE_SIZE")
            self.assertEqual(limits["memory_max_bytes"], (64 * 1024 * 1024 // page) * page)
            self.assertEqual(limits["swap_max_bytes"], 0)
            self.assertEqual(limits["cpu_max"], "50000 100000")
            self.assertEqual(limits["pids_max"], 32)
            attach = attempt.preexec_attach()
            attach()
            self.assertEqual((attempt.path / "cgroup.procs").read_text(), str(os.getpid()))
            attempt.kill()
            self.assertEqual((attempt.path / "cgroup.kill").read_text(), "1")
        finally:
            (attempt.path / "cgroup.events").write_text("populated 0\nfrozen 0\n")
            real_rmdir = Path.rmdir
            def kernel_rmdir(path):
                if path == attempt.path:
                    for child in list(path.iterdir()):
                        child.unlink()
                return real_rmdir(path)
            with mock.patch.object(Path, "rmdir", kernel_rmdir):
                attempt.cleanup()

    def test_missing_or_disabled_controller_is_rejected_before_worker(self):
        parent = self._fake_parent()
        (parent / "cgroup.subtree_control").write_text("cpu pids\n")
        with mock.patch.object(containment, "CGROUP_ROOT", parent):
            attempt = containment.CgroupV2Attempt(parent, "missing-memory", self.profile())
            with self.assertRaisesRegex(containment.ContainmentError, "not enabled"):
                attempt.create()
        self.assertFalse((parent / "tdi-missing-memory").exists())

    @unittest.skipUnless(hasattr(os, "pidfd_open"), "Linux pidfd required")
    def test_reaper_uses_pidfd_and_kills_fake_cgroup_after_owner_death(self):
        cgroup = self.root / "orphan"
        cgroup.mkdir()
        (cgroup / "cgroup.kill").write_text("")
        (cgroup / "cgroup.events").write_text("populated 0\n")
        marker = self.root / "recovery" / "marker.json"
        owner = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"])
        pidfd = os.pidfd_open(owner.pid, 0)
        read_fd, write_fd = os.pipe()
        try:
            reaper = subprocess.Popen(
                [sys.executable, str(MODULE), "_reaper",
                 "--pidfd", str(pidfd), "--control-fd", str(read_fd),
                 "--cgroup", str(cgroup), "--marker", str(marker)],
                pass_fds=(pidfd, read_fd), close_fds=True,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            )
            os.close(read_fd)
            read_fd = None
            owner.kill()
            owner.wait(timeout=5)
            code = reaper.wait(timeout=5)
            stderr = reaper.stderr.read()
            reaper.stdout.close()
            reaper.stderr.close()
            self.assertEqual(code, 0, stderr)
            self.assertEqual((cgroup / "cgroup.kill").read_text(), "1")
            record = json.loads(marker.read_text())
            self.assertEqual(record["kind"], "supervisor-death-recovery")
            self.assertEqual(record["outcome"], "killed")
        finally:
            if owner.poll() is None:
                owner.kill(); owner.wait()
            if read_fd is not None:
                os.close(read_fd)
            os.close(write_fd)
            os.close(pidfd)

    def test_doctor_is_read_only_structured_and_does_not_claim_sandbox_or_gpu_quota(self):
        completed = subprocess.run(
            [sys.executable, str(MODULE), "doctor"], capture_output=True, text=True, check=False)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        data = json.loads(completed.stdout)
        self.assertEqual(data["operation"], "doctor")
        capabilities = data["capabilities"]
        self.assertFalse(capabilities["sandbox_qualified"])
        self.assertFalse(capabilities["gpu_memory_quota_qualified"])


if __name__ == "__main__":
    unittest.main()
