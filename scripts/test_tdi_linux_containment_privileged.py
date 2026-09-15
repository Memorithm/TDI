"""Real-kernel cgroup-v2 qualification.

This suite is skipped unless CI or an operator supplies TDI_CGROUP_TEST_PARENT as
a deliberately delegated empty cgroup-v2 parent with cpu,memory,pids enabled.
It is separate from unit tests so a mock never counts as kernel qualification.
"""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import time
import unittest
import uuid

MODULE = Path(__file__).with_name("tdi_linux_containment.py")
spec = importlib.util.spec_from_file_location("containment_priv", MODULE)
containment = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = containment
spec.loader.exec_module(containment)

PARENT = os.environ.get("TDI_CGROUP_TEST_PARENT")


@unittest.skipUnless(PARENT, "set TDI_CGROUP_TEST_PARENT to a delegated cgroup-v2 parent")
class KernelContainmentTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.parent = Path(PARENT).resolve()
        caps = containment.doctor(cls.parent)
        if not caps.unified_v2 or not caps.writable_parent:
            raise RuntimeError(f"unqualified delegated parent: {caps.as_json()}")
        missing = {"cpu", "memory", "pids"} - set(caps.controllers_enabled)
        if missing:
            raise RuntimeError(f"controllers not enabled for child cgroups: {sorted(missing)}")

    def profile(self, *, memory=64 * 1024 * 1024, swap=0, quota=100_000,
                period=100_000, pids=32):
        return containment.ResourceProfile.from_json({
            "schema": 1,
            "memory_max_bytes": memory,
            "swap_max_bytes": swap,
            "cpu_quota_us": quota,
            "cpu_period_us": period,
            "pids_max": pids,
            "trust": "trusted",
            "gpu_required": False,
            "gpu_memory_max_bytes": None,
        })

    def attempt(self, profile):
        return containment.CgroupV2Attempt(
            self.parent, f"kernel-{uuid.uuid4().hex[:20]}", profile).create()

    def launch(self, attempt, code):
        process = subprocess.Popen(
            [sys.executable, "-c", code], stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
            start_new_session=True, preexec_fn=attempt.preexec_attach(),
            env={"LANG": "C", "LC_ALL": "C"},
        )
        attempt.close_attach_fd()
        return process

    def cleanup_attempt(self, attempt):
        if attempt.path.exists():
            if attempt.is_populated():
                attempt.kill()
            attempt.cleanup(timeout=5.0)

    def test_memory_hard_limit_is_observed_by_kernel(self):
        attempt = self.attempt(self.profile(memory=32 * 1024 * 1024, pids=16))
        try:
            process = self.launch(attempt, "x=bytearray(256*1024*1024); print(len(x))")
            stdout, stderr = process.communicate(timeout=10)
            self.assertNotEqual(process.returncode, 0, (stdout, stderr))
            events = attempt.metrics().get("memory_events", {})
            self.assertGreater(events.get("max", 0) + events.get("oom", 0) + events.get("oom_kill", 0), 0)
        finally:
            self.cleanup_attempt(attempt)

    def test_pid_hard_limit_rejects_fork_growth(self):
        attempt = self.attempt(self.profile(memory=128 * 1024 * 1024, pids=8))
        code = r'''
import json,os,time
children=[]; error=None
try:
    for _ in range(64):
        pid=os.fork()
        if pid==0:
            time.sleep(30); os._exit(0)
        children.append(pid)
except OSError as exc:
    error=exc.errno
finally:
    for pid in children:
        try: os.kill(pid,9)
        except ProcessLookupError: pass
    for pid in children:
        try: os.waitpid(pid,0)
        except ChildProcessError: pass
print(json.dumps({'children':len(children),'errno':error}))
'''
        try:
            process = self.launch(attempt, code)
            stdout, stderr = process.communicate(timeout=10)
            self.assertEqual(process.returncode, 0, stderr)
            observed = json.loads(stdout)
            self.assertIsNotNone(observed["errno"])
            self.assertLess(observed["children"], 64)
            self.assertGreater(attempt.metrics().get("pids_events", {}).get("max", 0), 0)
        finally:
            self.cleanup_attempt(attempt)

    def test_cpu_bandwidth_limit_produces_throttling_evidence(self):
        attempt = self.attempt(self.profile(memory=64 * 1024 * 1024, quota=10_000, period=100_000, pids=8))
        code = "import time\nend=time.monotonic()+1.5\nx=0\nwhile time.monotonic()<end: x+=1\nprint(x)"
        try:
            process = self.launch(attempt, code)
            stdout, stderr = process.communicate(timeout=10)
            self.assertEqual(process.returncode, 0, stderr)
            cpu = attempt.metrics().get("cpu_stat", {})
            self.assertGreater(cpu.get("nr_throttled", 0), 0, cpu)
            self.assertGreater(cpu.get("throttled_usec", 0), 0, cpu)
        finally:
            self.cleanup_attempt(attempt)

    def test_cgroup_kill_cleans_descendants_without_killing_sibling_attempt(self):
        a = self.attempt(self.profile(memory=64 * 1024 * 1024, pids=16))
        b = self.attempt(self.profile(memory=64 * 1024 * 1024, pids=16))
        code = "import os,time\npid=os.fork()\nif pid==0: time.sleep(30); os._exit(0)\ntime.sleep(30)"
        pa = pb = None
        try:
            pa = self.launch(a, code)
            pb = self.launch(b, code)
            deadline = time.monotonic() + 3
            while (not a.is_populated() or not b.is_populated()) and time.monotonic() < deadline:
                time.sleep(0.02)
            self.assertTrue(a.is_populated())
            self.assertTrue(b.is_populated())
            a.kill()
            pa.wait(timeout=5)
            self.assertIsNone(pb.poll(), "killing one attempt must not kill its sibling")
            self.assertTrue(b.is_populated())
            b.kill()
            pb.wait(timeout=5)
        finally:
            for proc in (pa, pb):
                if proc is not None:
                    if proc.poll() is None:
                        proc.kill(); proc.wait()
                    if proc.stdout: proc.stdout.close()
                    if proc.stderr: proc.stderr.close()
            self.cleanup_attempt(a)
            self.cleanup_attempt(b)


if __name__ == "__main__":
    unittest.main()
