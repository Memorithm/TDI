"""Real-kernel schema-3 ExperimentSpec/worker-response qualification."""
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).parent))
import tdi_experiment_supervisor as durable
import tdi_linux_containment as containment
import tdi_linux_runner as campaign
from test_tdi_experiment_contract import spec as experiment_spec

PARENT = os.environ.get("TDI_CGROUP_TEST_PARENT")


@unittest.skipUnless(PARENT, "set TDI_CGROUP_TEST_PARENT to a delegated cgroup-v2 parent")
class KernelExperimentContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.parent = Path(PARENT).resolve()
        caps = containment.doctor(cls.parent)
        if not caps.unified_v2 or not caps.writable_parent:
            raise RuntimeError(f"unqualified delegated parent: {caps.as_json()}")
        missing = {"cpu", "memory", "pids"} - set(caps.controllers_enabled)
        if missing:
            raise RuntimeError(f"controllers not enabled for child cgroups: {sorted(missing)}")

    def test_schema3_run_resume_and_semantic_drift_rejection(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            worker = root / "worker"
            worker.write_text(
                f"#!{sys.executable}\n"
                "import json,sys\n"
                "args=dict(zip(sys.argv[1::2],sys.argv[2::2]))\n"
                "seed=int(args['--tdi-seed'])\n"
                "response={\n"
                " 'schema':2,\n"
                " 'execution_status':'completed',\n"
                " 'scientific_disposition':'evaluated',\n"
                " 'experiment_id':args['--tdi-experiment-id'],\n"
                " 'plan_id':args['--tdi-plan-id'],\n"
                " 'trial_id':args['--tdi-trial-id'],\n"
                " 'attempt_id':args['--tdi-attempt-id'],\n"
                " 'backend_identity':'linux-cgroup-v2',\n"
                " 'domain':'Development',\n"
                " 'seed_decimal':str(seed),\n"
                " 'progress':{'completed_steps':1,'costs':{'logical_ops':1}},\n"
                " 'artifacts':[],\n"
                " 'result':{'value':seed+1},\n"
                " 'error':None}\n"
                "print(json.dumps(response,separators=(',',':'),sort_keys=True))\n"
            )
            worker.chmod(0o755)
            scientific = experiment_spec()
            scientific["logical_budget"]["max_trials"] = 1
            scientific["physical_constraints"].update({
                "timeout_milliseconds": 5000,
                "memory_max_bytes": 64 * 1024 * 1024,
                "cpu_quota_us": 100_000,
                "cpu_period_us": 100_000,
                "pids_max": 16,
            })
            plan = {
                "schema": 3,
                "purpose": "development-software",
                "domain": "Development",
                "indices": [0],
                "argv": ["worker"],
                "artifacts": {"worker": durable.file_digest(worker)},
                "timeout_seconds": 5,
                "max_output_bytes": 4096,
                "max_trials": 1,
                "execution": {
                    "backend": "linux-cgroup-v2",
                    "profile": {
                        "schema": 1,
                        "memory_max_bytes": 64 * 1024 * 1024,
                        "swap_max_bytes": 0,
                        "cpu_quota_us": 100_000,
                        "cpu_period_us": 100_000,
                        "pids_max": 16,
                        "trust": "trusted",
                        "gpu_required": False,
                        "gpu_memory_max_bytes": None,
                    },
                },
                "experiment": scientific,
            }
            journal = root / "campaign.db"
            recovery = root / "recovery"
            first = campaign.run(plan, root, journal, self.parent, recovery_dir=recovery)
            self.assertEqual(len(first), 1)
            result = first[0]["result"]
            self.assertEqual(result["status"], "Completed", result)
            self.assertEqual(result["response"]["result"]["value"], 1)
            self.assertEqual(result["response"]["trial_id"], result["trial_id"])
            self.assertEqual(result["response"]["attempt_id"], result["attempt_id"])
            self.assertEqual(len(result["scientific_result_id"]), 64)

            resumed = campaign.run(plan, root, journal, self.parent, recovery_dir=recovery)
            self.assertEqual(first, resumed)

            changed = copy.deepcopy(plan)
            changed["experiment"]["metrics"][0]["unit"] = "fraction"
            with self.assertRaises(durable.ContractError):
                campaign.run(changed, root, journal, self.parent, recovery_dir=recovery)


if __name__ == "__main__":
    unittest.main()
