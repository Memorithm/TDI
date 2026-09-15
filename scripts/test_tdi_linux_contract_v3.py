import copy
from pathlib import Path
import tempfile
import unittest

import tdi_experiment_contract as experiment
import tdi_experiment_supervisor as durable
import tdi_linux_contract as contained
import tdi_linux_runner as runner
from test_tdi_experiment_contract import spec as experiment_spec


class LinuxContractV3Tests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.worker = self.root / "worker"
        self.worker.write_text("#!/bin/sh\nexit 0\n")
        self.worker.chmod(0o755)
        self.plan = {
            "schema": 3,
            "purpose": "development-software",
            "domain": "Development",
            "indices": [0, 1],
            "argv": ["worker"],
            "artifacts": {"worker": durable.file_digest(self.worker)},
            "timeout_seconds": 2,
            "max_output_bytes": 4096,
            "max_trials": 2,
            "execution": {
                "backend": "linux-cgroup-v2",
                "profile": {
                    "schema": 1,
                    "memory_max_bytes": 67108864,
                    "swap_max_bytes": 0,
                    "cpu_quota_us": 50000,
                    "cpu_period_us": 100000,
                    "pids_max": 32,
                    "trust": "trusted",
                    "gpu_required": False,
                    "gpu_memory_max_bytes": None,
                },
            },
            "experiment": experiment_spec(),
        }

    def test_schema3_plan_and_attempt_identities_are_stable(self):
        plan_id = contained.validate_plan(self.plan, self.root)
        self.assertEqual(plan_id, contained.validate_plan(copy.deepcopy(self.plan), self.root))
        trial0, attempt0 = contained.attempt_coordinates(self.plan, plan_id, 0)
        trial1, attempt1 = contained.attempt_coordinates(self.plan, plan_id, 1)
        self.assertNotEqual(trial0, trial1)
        self.assertNotEqual(attempt0, attempt1)
        self.assertEqual((trial0, attempt0), contained.attempt_coordinates(self.plan, plan_id, 0))
        self.assertEqual(contained.experiment_identity(self.plan),
                         experiment.question_identity(self.plan["experiment"]))

    def test_schema3_timeout_representation_is_canonicalized_for_binding(self):
        integer_plan_id = contained.validate_plan(self.plan, self.root)
        float_plan = copy.deepcopy(self.plan)
        float_plan["timeout_seconds"] = 2.0
        self.assertEqual(integer_plan_id, contained.validate_plan(float_plan, self.root))
        self.assertEqual(contained.journal_binding(self.plan),
                         contained.journal_binding(float_plan))
        self.assertEqual(contained.journal_binding(self.plan)["execution_plan"]["timeout_seconds"],
                         "2000ms")

        fractional = copy.deepcopy(self.plan)
        fractional["timeout_seconds"] = 1.5
        fractional["experiment"]["physical_constraints"]["timeout_milliseconds"] = 1500
        contained.validate_plan(fractional, self.root)
        self.assertEqual(
            contained.journal_binding(fractional)["execution_plan"]["timeout_seconds"],
            "1500ms",
        )

    def test_physical_change_requires_spec_change_but_not_question_change(self):
        original_plan_id = contained.validate_plan(self.plan, self.root)
        original_question = contained.experiment_identity(self.plan)
        mismatched = copy.deepcopy(self.plan)
        mismatched["execution"]["profile"]["memory_max_bytes"] *= 2
        with self.assertRaises(durable.ContractError):
            contained.validate_plan(mismatched, self.root)
        matched = copy.deepcopy(mismatched)
        matched["experiment"]["physical_constraints"]["memory_max_bytes"] *= 2
        changed_plan_id = contained.validate_plan(matched, self.root)
        self.assertNotEqual(original_plan_id, changed_plan_id)
        self.assertEqual(original_question, contained.experiment_identity(matched))

    def test_semantic_metric_change_changes_question_and_execution_plan(self):
        original_plan_id = contained.validate_plan(self.plan, self.root)
        original_question = contained.experiment_identity(self.plan)
        changed = copy.deepcopy(self.plan)
        changed["experiment"]["metrics"][0]["unit"] = "fraction"
        self.assertNotEqual(original_plan_id, contained.validate_plan(changed, self.root))
        self.assertNotEqual(original_question, contained.experiment_identity(changed))

    def test_schema2_attempt_identity_remains_legacy_compatible(self):
        legacy = {key: value for key, value in self.plan.items() if key != "experiment"}
        legacy["schema"] = 2
        plan_id = contained.validate_plan(legacy, self.root)
        trial_id, attempt_id = contained.attempt_coordinates(legacy, plan_id, 0)
        self.assertIsNone(trial_id)
        self.assertEqual(attempt_id, contained.attempt_identity(plan_id, 0))
        self.assertEqual(len(attempt_id), 32)

    def test_worker_v2_arguments_and_response_binding(self):
        plan_id = contained.validate_plan(self.plan, self.root)
        trial_id, attempt_id = contained.attempt_coordinates(self.plan, plan_id, 0)
        argv = runner._worker_argv(self.plan, self.root, 0, plan_id, trial_id, attempt_id)
        self.assertIn("--tdi-worker-protocol", argv)
        self.assertIn("--tdi-experiment-id", argv)
        self.assertIn(trial_id, argv)
        self.assertIn(attempt_id, argv)
        response = {
            "schema": 2,
            "execution_status": "completed",
            "scientific_disposition": "evaluated",
            "experiment_id": contained.experiment_identity(self.plan),
            "plan_id": plan_id,
            "trial_id": trial_id,
            "attempt_id": attempt_id,
            "backend_identity": "linux-cgroup-v2",
            "domain": "Development",
            "seed_decimal": "0",
            "progress": {
                "completed_steps": 1,
                "completed_observations": 1,
                "costs": {"logical_ops": 3},
            },
            "artifacts": [],
            "result": {"value": 4},
            "error": None,
        }
        result_id = runner._validate_worker_response(
            self.plan,
            response,
            plan_id=plan_id,
            trial_id=trial_id,
            attempt_id=attempt_id,
            seed=0,
        )
        self.assertEqual(result_id, experiment.scientific_result_identity(response))
        response["trial_id"] = "0" * 64
        with self.assertRaises(durable.ContractError):
            runner._validate_worker_response(
                self.plan,
                response,
                plan_id=plan_id,
                trial_id=trial_id,
                attempt_id=attempt_id,
                seed=0,
            )


if __name__ == "__main__":
    unittest.main()
