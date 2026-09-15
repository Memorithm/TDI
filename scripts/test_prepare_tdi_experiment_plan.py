import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("prepare_tdi_experiment_plan.py")


class PreparePlanTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.worker = self.root / "worker"
        self.worker.write_text("#!/bin/sh\nexit 0\n")
        self.worker.chmod(0o755)
        self.input = self.root / "input.txt"
        self.input.write_text("fixture\n")

    def base_args(self, output):
        return [
            sys.executable, str(SCRIPT), "--root", str(self.root),
            "--worker", "worker", "--artifact", "input.txt",
            "--index", "0", "--index", "2", "--domain", "Development",
            "--timeout", "2", "--output-limit", "4096", "--plan", str(output),
        ]

    def test_legacy_mode_preserves_schema1(self):
        output = self.root / "legacy.json"
        completed = subprocess.run(self.base_args(output), text=True, capture_output=True, check=False)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        plan = json.loads(output.read_text())
        self.assertEqual(plan["schema"], 1)
        self.assertNotIn("execution", plan)

    def test_cgroup_mode_binds_resource_profile_into_schema2_plan(self):
        output = self.root / "cgroup.json"
        args = self.base_args(output) + [
            "--containment", "cgroup-v2", "--memory-max-bytes", str(64 * 1024 * 1024),
            "--cpu-quota-us", "50000", "--pids-max", "32",
        ]
        completed = subprocess.run(args, text=True, capture_output=True, check=False)
        self.assertEqual(completed.returncode, 0, completed.stderr)
        plan = json.loads(output.read_text())
        self.assertEqual(plan["schema"], 2)
        self.assertEqual(plan["execution"]["backend"], "linux-cgroup-v2")
        profile = plan["execution"]["profile"]
        self.assertEqual(profile["memory_max_bytes"], 64 * 1024 * 1024)
        self.assertEqual(profile["cpu_quota_us"], 50000)
        self.assertEqual(profile["pids_max"], 32)

    def test_cgroup_mode_requires_hard_limits_and_never_overwrites_plan(self):
        output = self.root / "plan.json"
        incomplete = subprocess.run(self.base_args(output) + ["--containment", "cgroup-v2"],
                                    text=True, capture_output=True, check=False)
        self.assertNotEqual(incomplete.returncode, 0)
        self.assertFalse(output.exists())
        first = subprocess.run(self.base_args(output), text=True, capture_output=True, check=False)
        self.assertEqual(first.returncode, 0, first.stderr)
        original = output.read_bytes()
        second = subprocess.run(self.base_args(output), text=True, capture_output=True, check=False)
        self.assertNotEqual(second.returncode, 0)
        self.assertEqual(output.read_bytes(), original)


if __name__ == "__main__":
    unittest.main()
