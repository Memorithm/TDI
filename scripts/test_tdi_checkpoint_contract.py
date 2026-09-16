import copy
import unittest

import tdi_checkpoint_contract as checkpoint

SHA = lambda c: c * 64


def manifest():
    return {
        "schema": 1,
        "experiment_id": SHA("a"),
        "plan_id": SHA("b"),
        "trial_id": SHA("c"),
        "step_key": "evaluate",
        "step_identity": SHA("d"),
        "adapter_identity": "fixture-adapter/v1",
        "backend_identity": "linux-cgroup-v2",
        "seed_decimal": str(2**63 + 7),
        "checkpoint_ordinal": 2,
        "budgets": {"max_steps": 4, "max_observations": 8},
        "progress": {"completed_steps": 4, "completed_observations": 8},
        "rng_streams": [
            {"name": "noise", "stream_id": SHA("e"), "counter_decimal": "9"},
            {"name": "task", "stream_id": SHA("f"), "counter_decimal": "5"},
        ],
        "input_artifacts": [
            {"name": "dataset", "sha256": SHA("1")},
            {"name": "config", "sha256": SHA("2")},
        ],
        "state": {
            "sha256": SHA("3"),
            "media_type": "application/vnd.tdi.checkpoint.v1+bin",
            "size_bytes": 128,
        },
    }


class Tests(unittest.TestCase):
    def test_identity_is_order_independent_for_named_sets(self):
        a = manifest()
        b = copy.deepcopy(a)
        b["rng_streams"].reverse()
        b["input_artifacts"].reverse()
        self.assertEqual(checkpoint.checkpoint_identity(a), checkpoint.checkpoint_identity(b))

    def test_state_counter_or_budget_changes_identity(self):
        a = manifest()
        base = checkpoint.checkpoint_identity(a)
        b = copy.deepcopy(a)
        b["state"]["sha256"] = SHA("4")
        self.assertNotEqual(base, checkpoint.checkpoint_identity(b))
        b = copy.deepcopy(a)
        b["rng_streams"][0]["counter_decimal"] = "10"
        self.assertNotEqual(base, checkpoint.checkpoint_identity(b))
        b = copy.deepcopy(a)
        b["budgets"]["max_steps"] = 5
        self.assertNotEqual(base, checkpoint.checkpoint_identity(b))

    def test_exact_resume_accepts_only_bound_lineage(self):
        a = manifest()
        checkpoint.validate_resume(
            a,
            experiment_id=SHA("a"),
            plan_id=SHA("b"),
            trial_id=SHA("c"),
            step_key="evaluate",
            step_identity=SHA("d"),
            adapter_identity="fixture-adapter/v1",
            backend_identity="linux-cgroup-v2",
            seed=2**63 + 7,
            input_artifacts={"config": SHA("2"), "dataset": SHA("1")},
            rng_stream_ids={"task": SHA("f"), "noise": SHA("e")},
            max_steps=4,
            max_observations=8,
        )
        bad = copy.deepcopy(a)
        bad["plan_id"] = SHA("9")
        with self.assertRaises(checkpoint.CheckpointContractError):
            checkpoint.validate_resume(
                bad,
                experiment_id=SHA("a"),
                plan_id=SHA("b"),
                trial_id=SHA("c"),
                step_key="evaluate",
                step_identity=SHA("d"),
                adapter_identity="fixture-adapter/v1",
                backend_identity="linux-cgroup-v2",
                seed=2**63 + 7,
                input_artifacts={"config": SHA("2"), "dataset": SHA("1")},
                rng_stream_ids={"task": SHA("f"), "noise": SHA("e")},
                max_steps=4,
                max_observations=8,
            )

    def test_resume_rejects_progress_beyond_frozen_budget(self):
        a = manifest()
        a["progress"]["completed_steps"] = 5
        with self.assertRaises(checkpoint.CheckpointContractError):
            checkpoint.validate_resume(
                a,
                experiment_id=SHA("a"),
                plan_id=SHA("b"),
                trial_id=SHA("c"),
                step_key="evaluate",
                step_identity=SHA("d"),
                adapter_identity="fixture-adapter/v1",
                backend_identity="linux-cgroup-v2",
                seed=2**63 + 7,
                input_artifacts={"config": SHA("2"), "dataset": SHA("1")},
                rng_stream_ids={"task": SHA("f"), "noise": SHA("e")},
                max_steps=4,
                max_observations=8,
            )
        a = manifest()
        a["progress"]["completed_observations"] = 9
        with self.assertRaises(checkpoint.CheckpointContractError):
            checkpoint.validate_resume(
                a,
                experiment_id=SHA("a"),
                plan_id=SHA("b"),
                trial_id=SHA("c"),
                step_key="evaluate",
                step_identity=SHA("d"),
                adapter_identity="fixture-adapter/v1",
                backend_identity="linux-cgroup-v2",
                seed=2**63 + 7,
                input_artifacts={"config": SHA("2"), "dataset": SHA("1")},
                rng_stream_ids={"task": SHA("f"), "noise": SHA("e")},
                max_steps=4,
                max_observations=8,
            )

    def test_resume_rejects_frozen_budget_reinterpretation(self):
        a = manifest()
        wider = copy.deepcopy(a)
        wider["budgets"] = {"max_steps": 5, "max_observations": 9}
        with self.assertRaises(checkpoint.CheckpointContractError):
            checkpoint.validate_resume(
                wider,
                experiment_id=SHA("a"),
                plan_id=SHA("b"),
                trial_id=SHA("c"),
                step_key="evaluate",
                step_identity=SHA("d"),
                adapter_identity="fixture-adapter/v1",
                backend_identity="linux-cgroup-v2",
                seed=2**63 + 7,
                input_artifacts={"config": SHA("2"), "dataset": SHA("1")},
                rng_stream_ids={"task": SHA("f"), "noise": SHA("e")},
                max_steps=4,
                max_observations=8,
            )

    def test_resume_rejects_input_or_stream_drift(self):
        a = manifest()
        kwargs = dict(
            experiment_id=SHA("a"),
            plan_id=SHA("b"),
            trial_id=SHA("c"),
            step_key="evaluate",
            step_identity=SHA("d"),
            adapter_identity="fixture-adapter/v1",
            backend_identity="linux-cgroup-v2",
            seed=2**63 + 7,
            input_artifacts={"config": SHA("2"), "dataset": SHA("1")},
            rng_stream_ids={"task": SHA("f"), "noise": SHA("e")},
            max_steps=4,
            max_observations=8,
        )
        bad = copy.deepcopy(a)
        bad["input_artifacts"][0]["sha256"] = SHA("7")
        with self.assertRaises(checkpoint.CheckpointContractError):
            checkpoint.validate_resume(bad, **kwargs)
        bad = copy.deepcopy(a)
        bad["rng_streams"][0]["stream_id"] = SHA("7")
        with self.assertRaises(checkpoint.CheckpointContractError):
            checkpoint.validate_resume(bad, **kwargs)


if __name__ == "__main__":
    unittest.main()
