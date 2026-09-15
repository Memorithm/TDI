import copy
import json
import unittest

import tdi_experiment_contract as contract


def spec():
    return {
        "schema": 1,
        "semantic_version": "tdi-fixture/1.0.0",
        "series_id": "fixture-series",
        "stage": "development",
        "protocol_refs": [{"name": "protocol", "sha256": "a" * 64}],
        "hypothesis_ids": ["H1"],
        "metrics": [{"id": "loss", "unit": "count", "orientation": "minimize"}],
        "data": {
            "access_class": "development",
            "generator_identity": "fixture-generator/v1",
            "partition_derivation": "index-domain/v1",
        },
        "randomness": {
            "algorithm": "fixture-rng/v1",
            "seed_namespace": "fixture",
            "stream_derivation": "sha256-coordinate/v1",
        },
        "arms": [
            {"id": "reference", "role": "baseline", "implementation_identity": "ref/v1"},
            {"id": "candidate", "role": "candidate", "implementation_identity": "candidate/v1"},
        ],
        "adapter": {
            "api_version": "tdi-adapter/1",
            "implementation_identity": "fixture-adapter/v1",
            "required_capabilities": ["cancel"],
            "reproducibility_class": "exact",
        },
        "logical_budget": {
            "max_trials": 2,
            "max_steps_per_trial": 32,
            "max_observations_per_trial": 32,
        },
        "physical_constraints": {
            "timeout_milliseconds": 2000,
            "max_output_bytes": 4096,
            "memory_max_bytes": 67108864,
            "swap_max_bytes": 0,
            "cpu_quota_us": 50000,
            "cpu_period_us": 100000,
            "pids_max": 32,
            "gpu_required": False,
            "gpu_memory_max_bytes": None,
        },
        "retry": {"policy": "never", "max_attempts_per_trial": 1},
        "statistics": {
            "plan_identity": "paired-fixture/v1",
            "missing_observation_policy": "retain-and-report",
        },
        "artifacts": {
            "access_class": "development",
            "retention_policy": "retain-all-trials",
            "export_policy": "explicit-only",
        },
        "dependencies": [{"name": "tdi-ai", "identity": "fixture-source"}],
    }


def response(s, plan_id, trial_id, attempt_id):
    experiment_id = contract.question_identity(s)
    return {
        "schema": 2,
        "execution_status": "completed",
        "scientific_disposition": "evaluated",
        "experiment_id": experiment_id,
        "plan_id": plan_id,
        "trial_id": trial_id,
        "attempt_id": attempt_id,
        "backend_identity": "linux-cgroup-v2",
        "domain": "Development",
        "seed_decimal": str(2**63 - 1),
        "progress": {"completed_steps": 3, "costs": {"logical_ops": 7}},
        "artifacts": [],
        "result": {"score": 5},
        "error": None,
    }


class ExperimentContractTests(unittest.TestCase):
    def test_canonical_identity_is_independent_of_mapping_insertion_order(self):
        original = spec()
        reordered = json.loads(json.dumps(original, sort_keys=True))
        self.assertEqual(contract.question_identity(original), contract.question_identity(reordered))
        self.assertEqual(contract.experiment_plan_identity(original), contract.experiment_plan_identity(reordered))

    def test_question_identity_separates_science_from_physical_plan(self):
        original = spec()
        changed = copy.deepcopy(original)
        changed["physical_constraints"]["memory_max_bytes"] *= 2
        self.assertEqual(contract.question_identity(original), contract.question_identity(changed))
        self.assertNotEqual(contract.experiment_plan_identity(original), contract.experiment_plan_identity(changed))
        changed = copy.deepcopy(original)
        changed["metrics"][0]["unit"] = "fraction"
        self.assertNotEqual(contract.question_identity(original), contract.question_identity(changed))
        self.assertNotEqual(contract.experiment_plan_identity(original), contract.experiment_plan_identity(changed))

    def test_rng_stream_coordinate_does_not_depend_on_schedule_order(self):
        s = spec()
        plan_id = contract.experiment_plan_identity(s)
        question_id = contract.question_identity(s)
        a = contract.trial_identity(plan_id, "Development", 0)
        b = contract.trial_identity(plan_id, "Development", 1)
        forward = {
            trial: contract.stream_identity(question_id, trial, "fixture", "adapter")
            for trial in (a, b)
        }
        reverse = {
            trial: contract.stream_identity(question_id, trial, "fixture", "adapter")
            for trial in (b, a)
        }
        self.assertEqual(forward, reverse)

    def test_retry_and_decimal_seed_fail_closed(self):
        invalid = spec()
        invalid["retry"] = {"policy": "never", "max_attempts_per_trial": 2}
        with self.assertRaises(contract.ExperimentContractError):
            contract.validate_experiment_spec(invalid)
        for seed in (1, -1, "01", "+1", str(2**64)):
            with self.subTest(seed=seed):
                with self.assertRaises(contract.ExperimentContractError):
                    contract.parse_seed_decimal(seed)
        self.assertEqual(contract.parse_seed_decimal(str(2**64 - 1)), 2**64 - 1)

    def test_worker_binding_and_result_identity_exclude_operational_progress(self):
        s = spec()
        plan_id = contract.experiment_plan_identity(s)
        trial_id = contract.trial_identity(plan_id, "Development", 0)
        attempt_id = contract.attempt_identity(plan_id, trial_id, "linux-cgroup-v2", 0)
        value = response(s, plan_id, trial_id, attempt_id)
        contract.validate_worker_response_v2(
            value,
            experiment_id=contract.question_identity(s),
            plan_id=plan_id,
            trial_id=trial_id,
            attempt_id=attempt_id,
            backend_identity="linux-cgroup-v2",
            domain="Development",
            seed=2**63 - 1,
        )
        identity = contract.scientific_result_identity(value)
        value["progress"]["costs"]["logical_ops"] = 99
        self.assertEqual(identity, contract.scientific_result_identity(value))
        value["result"]["score"] = 6
        self.assertNotEqual(identity, contract.scientific_result_identity(value))

    def test_wrong_attempt_binding_is_rejected(self):
        s = spec()
        plan_id = contract.experiment_plan_identity(s)
        trial_id = contract.trial_identity(plan_id, "Development", 0)
        attempt_id = contract.attempt_identity(plan_id, trial_id, "linux-cgroup-v2", 0)
        value = response(s, plan_id, trial_id, attempt_id)
        value["attempt_id"] = "b" * 64
        with self.assertRaises(contract.ExperimentContractError):
            contract.validate_worker_response_v2(
                value,
                experiment_id=contract.question_identity(s),
                plan_id=plan_id,
                trial_id=trial_id,
                attempt_id=attempt_id,
                backend_identity="linux-cgroup-v2",
                domain="Development",
                seed=2**63 - 1,
            )


if __name__ == "__main__":
    unittest.main()
