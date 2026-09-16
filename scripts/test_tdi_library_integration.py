"""Actual TableSystem/GreenBands workers, process restart codecs and Hub graphs."""
import copy
import json
import os
from pathlib import Path
import subprocess
import unittest

import test_tdi_engine_integration as operational
import tdi_experiment_supervisor as durable
from tdi_library_fixture import check_result, parameters


class LibraryIntegrationTests(unittest.TestCase):
    setUp = operational.OperationalIntegrationTests.setUp
    start_hub = operational.OperationalIntegrationTests.start_hub
    stop_hub = operational.OperationalIntegrationTests.stop_hub
    cli = operational.OperationalIntegrationTests.cli

    @classmethod
    def setUpClass(cls):
        cls.hubd = Path(os.environ["TDI_HUBD_BIN"]).resolve(strict=True)
        cls.worker = Path(os.environ["TDI_LIBRARY_WORKER"]).resolve(strict=True)

    def worker_run(self, adapter, steps, *, seed=3, plan="a" * 64, restore=None, expected=0):
        command = [str(self.worker), "--adapter", adapter, "--steps", str(steps), "--seed", str(seed), "--plan-id", plan]
        if restore is not None:
            command += ["--restore", restore]
        result = subprocess.run(command, capture_output=True, check=False, timeout=5, env={"LANG": "C"})
        self.assertEqual(expected, result.returncode, result.stderr.decode())
        return json.loads(result.stdout)

    def test_actual_hub_both_libraries_and_restored_catalogue(self):
        for adapter, domain in (("finite", "Development"), ("jacobi", "Validation")):
            with self.subTest(adapter=adapter):
                plan = self.root / (adapter + ".json")
                self.cli("library-fixture-plan", "--worker", self.worker, "--adapter", adapter,
                         "--domain", domain, "--trials", 2, "--output", plan)
                self.assertEqual(8, self.cli("validate", plan)["steps"])
                campaign = self.cli("submit", plan)["campaign"]
                state = self.cli("run", campaign)
                self.assertEqual("completed", state["phase"])
                results = self.cli("inspect", campaign)["results"]
                verified = [r["evidence"]["json"] for r in results if r["evidence"]["json"]["status"] == "Verified"]
                self.assertEqual(2, len(verified))
                self.assertTrue(all(r["split_replay_equal"] for r in verified))
                self.assertEqual({str(i | (2**63 if domain == "Validation" else 0)) for i in range(2)}, {r["seed"] for r in verified})
                self.stop_hub()
                self.start_hub()
                self.assertEqual(state["snapshot"], self.cli("resume", campaign)["snapshot"])
                bundle = self.root / (adapter + "-bundle.json")
                receipt = self.cli("export", campaign, bundle)
                self.assertEqual(8, self.cli("verify", bundle, "--expected-identity", receipt["identity"])["verified_members"])

    def test_real_process_codec_continuation_identity_and_budget_rejection(self):
        for adapter in ("finite", "jacobi"):
            with self.subTest(adapter=adapter):
                prefix = self.worker_run(adapter, 2)
                suffix = self.worker_run(adapter, 2, restore=prefix["checkpoint"])
                full = self.worker_run(adapter, 4)
                self.assertEqual(prefix["observations"] + suffix["observations"], full["observations"])
                self.assertEqual(suffix["checkpoint"], full["checkpoint"])
                self.worker_run(adapter, 2, plan="b" * 64, restore=prefix["checkpoint"], expected=21)
                self.worker_run(adapter, 2, seed=4, restore=prefix["checkpoint"], expected=21)
                self.worker_run(adapter, 63, restore=prefix["checkpoint"], expected=21)
                self.worker_run(adapter, 2, restore=prefix["checkpoint"] + "00", expected=21)
                wrong_backend = self.worker_run("finite" if adapter == "jacobi" else "jacobi", 2)
                self.worker_run(adapter, 2, restore=wrong_backend["checkpoint"], expected=21)
                p, seed = parameters(json.dumps({"schema": 1, "purpose": "development-software", "domain": "Development",
                                                "index": 3, "plan_id": "a" * 64, "adapter": adapter}))
                check_result(full, p, seed, 0, 4)
                for field, value in (("seed", "4"), ("completed_depth", True), ("checkpoint", prefix["checkpoint"]),
                                     ("observations", [[True]] * 4), ("observations", [[float("nan")]] * 4)):
                    changed = copy.deepcopy(full); changed[field] = value
                    with self.assertRaises(durable.ContractError):
                        check_result(changed, p, seed, 0, 4)


if __name__ == "__main__":
    unittest.main()
