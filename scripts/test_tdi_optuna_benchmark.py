"""Methodology regressions; actual Forge/Optuna execution is a separate CI step."""
import copy
import json
from pathlib import Path
import tempfile
import unittest

import tdi_optuna_benchmark as bench


class ComparisonTests(unittest.TestCase):
    def fixture(self):
        p = bench.protocol("smoke")
        manifest = {"protocol": p, "task_bounds": {t: bench.task_bounds(t) for t in bench.TASKS}}
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
        for task in bench.TASKS:
            for x in bench.VALUES:
                for y in bench.VALUES:
                    self.assertEqual(bench.objective(task, x, y), bench.oracle(task, x, y))
        self.assertEqual(bench.task_bounds("shifted-bowl")["minimum"], 0)
        self.assertEqual(bench.task_bounds("coupled-ridge")["minimum"], 1)

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

    def test_real_optuna_seed_replay_and_common_first_observation(self):
        bench.check_packages()
        for name in ("optuna-random", "optuna-tpe"):
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
