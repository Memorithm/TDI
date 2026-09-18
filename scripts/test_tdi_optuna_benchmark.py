"""Methodology regressions; actual Forge/Optuna execution is a separate CI step."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest import mock

import tdi_optuna_benchmark as bench


class ComparisonTests(unittest.TestCase):
    def fixture(self, profile="smoke"):
        p = bench.protocol(profile)
        manifest = {"protocol": p, "task_bounds": {t: bench.task_bounds(t) for t in p["tasks"]}}
        runs = []
        # Synthetic records for adversarial report validation, never benchmark evidence.
        for task in p["tasks"]:
            for seed in p["seeds"]:
                for arm in p["arms"]:
                    rows, best = [], None
                    for i in range(p["evaluations_per_arm"]):
                        point = {"x": -8, "y": -8 + i}
                        value = bench.oracle(task, **point)
                        best = value if best is None else min(best, value)
                        rows.append({"trial": i, "parameters": point, "loss": value,
                                     "best_loss": best, "duplicate": False})
                    runs.append({"task": task, "seed": seed, "arm": arm, "trials": rows,
                                 "observed_adapter_ns": "1"})
        return manifest, runs

    def test_exact_objective_oracles_over_entire_domain(self):
        for task in bench.SESSION_TASKS:
            for x in bench.VALUES:
                for y in bench.VALUES:
                    self.assertEqual(bench.objective(task, x, y), bench.oracle(task, x, y))
        self.assertEqual(bench.task_bounds("shifted-bowl")["minimum"], 0)
        self.assertEqual(bench.task_bounds("coupled-ridge")["minimum"], 1)

    def test_new_session_tasks_have_baseline_headroom(self):
        p = bench.protocol("session-development")
        for task in p["tasks"]:
            if task not in p["nondiscriminating_tasks"]:
                self.assertGreater(bench.objective(task, **p["baseline"]), bench.task_bounds(task)["minimum"])
        self.assertEqual(p["nondiscriminating_tasks"], ["categorical-interaction"])
        self.assertEqual(bench.protocol("adaptive-development")["forge_source_commit"], bench.ADAPTIVE_FORGE_COMMIT)

    def test_transport_verifier_rejects_valid_but_different_trajectory(self):
        manifest, runs = self.fixture("transport-smoke")
        self.assertTrue(bench.summarize(manifest, runs)["all_paired_trajectories_equal"])
        run = next(r for r in runs if r["arm"] == "forge-tpe-session")
        row = run["trials"][-1]
        row["parameters"] = {"x": 7, "y": 7}
        row["loss"] = bench.oracle(run["task"], **row["parameters"])
        row["best_loss"] = min(run["trials"][-2]["best_loss"], row["loss"])
        with self.assertRaisesRegex(ValueError, "changed optimizer trajectory"):
            bench.summarize(manifest, runs)

    def test_closed_profile_budget_and_baseline(self):
        manifest, runs = self.fixture()
        self.assertEqual(len(bench.summarize(manifest, runs)["aggregates"]), 12)
        for mutation in ("budget", "baseline", "loss", "best", "duplicate", "domain"):
            bad = copy.deepcopy(runs)
            if mutation == "budget":
                bad[0]["trials"].pop()
            elif mutation == "baseline":
                bad[0]["trials"][0]["parameters"]["x"] = -7
            elif mutation == "loss":
                bad[0]["trials"][2]["loss"] = -100
            elif mutation == "best":
                bad[0]["trials"][2]["best_loss"] = -100
            elif mutation == "duplicate":
                bad[0]["trials"][2]["duplicate"] = True
            else:
                bad[0]["trials"][2]["parameters"]["x"] = True
            with self.subTest(mutation=mutation), self.assertRaises(ValueError):
                bench.summarize(manifest, bad)

    def test_absent_duplicate_runs_or_changed_protocol_are_rejected(self):
        manifest, runs = self.fixture()
        for bad in (runs[:-1], runs + [runs[0]]):
            with self.assertRaises(ValueError):
                bench.summarize(manifest, bad)
        changed = copy.deepcopy(manifest)
        changed["protocol"]["evaluations_per_arm"] -= 1
        with self.assertRaises(ValueError):
            bench.summarize(changed, runs)
        changed = copy.deepcopy(manifest)
        changed["task_bounds"]["double-well"]["minimum"] -= 1
        with self.assertRaises(ValueError):
            bench.summarize(changed, runs)

    def test_report_verification_rejects_forged_summary_even_with_new_hash(self):
        manifest, runs = self.fixture()
        report = {"schema": 1, "status": "complete", "manifest": manifest,
                  "manifest_identity": bench.identity("tdi-optuna-manifest/v1", manifest),
                  "raw_runs": runs, "summary": bench.summarize(manifest, runs)}
        report["identity"] = bench.identity("tdi-optuna-report/v1", report)
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "report.json"
            path.write_text(json.dumps(report))
            bench.verify_report(path)
            report.pop("identity")
            report["summary"]["aggregates"][0]["median_final_regret"] = -1
            report["identity"] = bench.identity("tdi-optuna-report/v1", report)
            path.write_text(json.dumps(report))
            with self.assertRaises(ValueError):
                bench.verify_report(path)

    def test_adaptive_profile_keeps_historical_protocol_and_stronger_reference(self):
        old = bench.protocol("development")
        new = bench.protocol("adaptive-development")
        self.assertEqual(old["forge_source_commit"], "28067ab0aa1d52a2260d9bb2bf35a346547a292a")
        self.assertEqual(old["arms"], list(bench.ARMS))
        self.assertEqual(old["tasks"], list(bench.TASKS))
        self.assertEqual(new["evaluations_per_arm"], old["evaluations_per_arm"])
        self.assertEqual(new["seeds"], old["seeds"])
        self.assertTrue(new["multivariate_tpe"]["multivariate"])
        self.assertIn("forge-tpe", new["arms"])
        self.assertGreater(bench.protocol("adaptive-smoke")["evaluations_per_arm"], 10)
        self.assertFalse(new["confirmatory"])

    def test_trial_crossing_deadline_is_not_accepted(self):
        clock = {"now": 0.0}

        class CrossingArm:
            def ask(self):
                return {"x": -8, "y": -8}

            def begin(self, stage):
                pass

            def finish(self, stage, evidence, elapsed_ns):
                if stage == "measure":
                    clock["now"] = 2.0

            def close(self):
                raise AssertionError("expired arm must not be closed as complete")

        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            arm_dir = root / "arm"
            arm_dir.mkdir()
            event_path = root / "events.jsonl"
            with event_path.open("x", encoding="utf-8") as stream, mock.patch.object(
                    bench.time, "monotonic", side_effect=lambda: clock["now"]):
                with self.assertRaises(TimeoutError):
                    bench.run_arm(
                        CrossingArm(), "shifted-bowl", 0, 1, arm_dir, stream, deadline=1.0)
            self.assertEqual("", event_path.read_text())
            self.assertTrue((arm_dir / "trial-000-measure.json").exists())

    def test_forge_timeout_never_exceeds_remaining_whole_run_budget(self):
        with mock.patch.object(bench.time, "monotonic", return_value=10.0):
            self.assertEqual(3.5, bench.forge_timeout(13.5))
            self.assertEqual(30.0, bench.forge_timeout(100.0))
            with self.assertRaises(TimeoutError):
                bench.forge_timeout(10.0)

    def test_report_crossing_deadline_is_removed_and_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "report.json"
            with mock.patch.object(bench.time, "monotonic", return_value=2.0):
                with self.assertRaises(TimeoutError):
                    bench.publish_complete_report(path, {"status": "complete"}, 1.0)
            self.assertFalse(path.exists())

    def test_real_optuna_seed_replay_and_common_first_observation(self):
        bench.check_packages()
        for name in ("optuna-random", "optuna-tpe", "optuna-tpe-multivariate", "optuna-tpe-multivariate-early"):
            trajectories = []
            for _ in range(2):
                arm = bench.OptunaArm(name, 7, bench.protocol("smoke")["tpe"])
                points = []
                for _ in range(12):
                    point = arm.ask()
                    points.append(point)
                    arm.finish("measure", {"loss": bench.objective("shifted-bowl", **point)}, 0)
                trajectories.append(points)
            self.assertEqual(trajectories[0][0], {"x": -8, "y": -8})
            self.assertEqual(trajectories[0], trajectories[1])


if __name__ == "__main__":
    unittest.main()
