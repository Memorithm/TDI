import json
from pathlib import Path
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS))
import tdi_experiment_supervisor as durable
import tdi_linux_contract as contract


class ContractTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        worker = self.root / "worker"
        worker.write_text("#!/bin/sh\nexit 0\n")
        worker.chmod(0o755)
        self.plan = {
            "schema": 2, "purpose": "development-software", "domain": "Development",
            "indices": [0, 1], "argv": ["worker"],
            "artifacts": {"worker": durable.file_digest(worker)},
            "timeout_seconds": 2, "max_output_bytes": 4096, "max_trials": 2,
            "execution": {"backend": "linux-cgroup-v2", "profile": {
                "schema": 1, "memory_max_bytes": 64 * 1024 * 1024,
                "swap_max_bytes": 0, "cpu_quota_us": 50_000,
                "cpu_period_us": 100_000, "pids_max": 32, "trust": "trusted",
                "gpu_required": False, "gpu_memory_max_bytes": None}},
        }

    def test_schema2_identity_includes_resource_profile(self):
        first = contract.validate_plan(self.plan, self.root)
        changed = json.loads(json.dumps(self.plan))
        changed["execution"]["profile"]["pids_max"] = 31
        second = contract.validate_plan(changed, self.root)
        self.assertNotEqual(first, second)
        self.assertEqual(contract.attempt_identity(first, 0), contract.attempt_identity(first, 0))
        self.assertNotEqual(contract.attempt_identity(first, 0), contract.attempt_identity(first, 1))
        binding = contract.journal_binding(self.plan)
        self.assertEqual(binding["execution_plan"], self.plan)

    def test_schema1_and_untrusted_profiles_fail_closed(self):
        legacy = dict(self.plan); legacy.pop("execution"); legacy["schema"] = 1
        with self.assertRaises(durable.ContractError):
            contract.validate_plan(legacy, self.root)
        untrusted = json.loads(json.dumps(self.plan))
        untrusted["execution"]["profile"]["trust"] = "untrusted"
        with self.assertRaisesRegex(durable.ContractError, "separately qualified"):
            contract.validate_plan(untrusted, self.root)


if __name__ == "__main__":
    unittest.main()
