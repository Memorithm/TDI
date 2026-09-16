"""Independent SALib numerical references and actual Hub/process integration."""
import copy
import json
import math
import os
from pathlib import Path
import unittest

import numpy as np
from SALib.analyze import morris, sobol
from SALib.test_functions import Ishigami

import tdi_engine_runtime as runtime
import tdi_experiment_supervisor as durable
from tdi_engine_store import EngineStore, atomic_json, identity
from tdi_scirust_client import SciRustStats
import tdi_sensitivity as sensitivity
import tdi_sensitivity_fixture as fixture
import test_tdi_engine_integration as hub_fixture


def protocol(method="morris", samples=16, *, ishigami=False):
    return {"schema": 1, "purpose": "exploratory-sensitivity", "domain": "Development", "name": "public-analytic-control",
            "method": method, "factors": [{"name": f"x{j}", "unit": "dimensionless", "lower": -math.pi if ishigami else 0,
                "upper": math.pi if ishigami else 1, "baseline": .5 if method == "ablation" else None,
                "intervention": 0 if method == "ablation" else None} for j in range(3)],
            "output_unit": "dimensionless", "samples": samples, "seed": "0", "max_evaluations": 4096,
            "missing": "reject-incomplete", "assumptions": ["deterministic-response", "declared-one-factor-interventions" if method == "ablation" else "independent-uniform-factors"]}


def worker():
    path = Path(os.environ["TDI_SCIRUST_STATS_BIN"]).resolve(strict=True)
    return SciRustStats(path, durable.file_digest(path), os.environ["TDI_SCIRUST_SOURCE_COMMIT"])


def observations(plan, ys):
    # Synthetic numerical-reference assertions, never represented as Hub runs.
    return {"schema": 1, "plan_identity": plan["identity"], "cost_measurements": [],
            "rows": [{"id": row["id"], "ordinal": row["ordinal"], "value": float(y),
                      "source": {"campaign": identity("synthetic-reference", 0), "step": "numerical-oracle", "output": "synthetic",
                          "artifact_identity": identity("synthetic-reference-artifact", i), "provenance_identity": identity("synthetic-reference-provenance", i), "pointer": "/rows/0/value"}}
                     for i, (row, y) in enumerate(zip(plan["rows"], ys))]}


class SensitivityTests(unittest.TestCase):
    def test_morris_sobol_independent_references_and_deterministic_regeneration(self):
        numerical = worker()
        for method, n in (("morris", 32), ("sobol", 512)):
            p = protocol(method, n, ishigami=True)
            plan = sensitivity.make_plan(p)
            self.assertEqual(plan, sensitivity.make_plan(p))
            xs = np.array([r["values"] for r in plan["rows"]])
            ys = Ishigami.evaluate(xs)
            for x, y in zip(xs.tolist(), ys):
                self.assertAlmostEqual(float(y), fixture.evaluate("ishigami", x), delta=5e-12)
            report = sensitivity.analyze(plan, observations(plan, ys), numerical)
            problem = {"num_vars": 3, "names": ["x0", "x1", "x2"], "bounds": [[0, 1]] * 3}
            if method == "morris":
                normalized = np.array([r["normalized"] for r in plan["rows"]])
                ref = morris.analyze(problem, normalized, ys, num_levels=4, num_resamples=100, seed=1)
                for field in ("mu", "mu_star", "sigma"):
                    np.testing.assert_allclose(report["result"][field], ref[field], atol=5e-12, rtol=5e-12)
            else:
                ref = sobol.analyze(problem, ys, calc_second_order=False, num_resamples=100, seed=1)
                np.testing.assert_allclose(report["result"]["first"], ref["S1"], atol=5e-12, rtol=5e-12)
                np.testing.assert_allclose(report["result"]["total"], ref["ST"], atol=5e-12, rtol=5e-12)
            self.assertIsNone(report["confidence_intervals"])
            self.assertEqual("not-assessed", report["scientific_verdict"])

    def test_additive_known_shares_and_declared_ablation(self):
        plan = sensitivity.make_plan(protocol("sobol", 512))
        ys = np.array([r["values"] for r in plan["rows"]]) @ np.array([1., 2., 3.])
        result = sensitivity.analyze(plan, observations(plan, ys), worker())["result"]
        np.testing.assert_allclose(result["first"], np.array([1, 4, 9]) / 14, atol=.015, rtol=0)
        np.testing.assert_allclose(result["total"], np.array([1, 4, 9]) / 14, atol=.015, rtol=0)
        plan = sensitivity.make_plan(protocol("ablation", 1))
        ys = [sum((i + 1) * x for i, x in enumerate(r["values"])) for r in plan["rows"]]
        result = sensitivity.analyze(plan, observations(plan, ys), None)["result"]
        self.assertEqual([-.5, -1., -1.5], result["effects"])

    def test_limits_domains_nonfinite_unknown_fields_and_predeclared_interventions(self):
        base = protocol()
        invalid = [dict(base, domain="Final"), dict(base, samples=True), dict(base, samples=129), dict(base, seed="00"),
                   dict(base, seed=str(2**64)), dict(base, missing="drop"), dict(base, max_evaluations=3), dict(base, unknown=1),
                   dict(base, method="sobol", samples=17), dict(base, assumptions=[])]
        for replacement in (float("nan"), float("inf"), True, 2**64):
            p = copy.deepcopy(base); p["factors"][0]["upper"] = replacement; invalid.append(p)
        for p in invalid:
            with self.subTest(p=p), self.assertRaises(durable.ContractError): sensitivity.make_plan(p)
        p = protocol("ablation", 1); p["factors"][0]["intervention"] = .5
        with self.assertRaises(durable.ContractError): sensitivity.make_plan(p)
        p = protocol("sobol", 2048)
        with self.assertRaises(durable.ContractError): sensitivity.make_plan(p)

    def test_missing_duplicate_permuted_and_tampered_results_are_rejected(self):
        plan = sensitivity.make_plan(protocol("morris", 2))
        obs = observations(plan, range(len(plan["rows"])))
        variants = []
        bad = copy.deepcopy(obs); bad["rows"].pop(); variants.append(bad)
        bad = copy.deepcopy(obs); bad["rows"].reverse(); variants.append(bad)
        bad = copy.deepcopy(obs); bad["rows"][1]["source"] = bad["rows"][0]["source"]; variants.append(bad)
        bad = copy.deepcopy(obs); bad["rows"][0]["value"] = None; variants.append(bad)
        for bad in variants:
            with self.assertRaises(durable.ContractError): sensitivity.analyze(plan, bad, None)
        tampered = copy.deepcopy(plan); tampered["rows"][0]["values"][0] += .1
        with self.assertRaises(durable.ContractError): sensitivity.validate_plan(tampered)
        constant = sensitivity.make_plan(protocol("sobol", 16))
        with self.assertRaises(durable.ContractError):
            sensitivity.analyze(constant, observations(constant, [1.] * len(constant["rows"])), worker())

    def test_real_hub_batched_sampling_cli_reports_and_faults(self):
        hub_fixture.OperationalIntegrationTests.setUpClass()
        hub = hub_fixture.OperationalIntegrationTests(); self.addCleanup(hub.doCleanups); hub.setUp()
        stats = worker()
        for method, n in (("morris", 4), ("sobol", 16), ("ablation", 1)):
            with self.subTest(method=method):
                p = protocol(method, n)
                paths = {name: hub.root / (method + "-" + name + ".json") for name in ("protocol", "plan", "campaign", "report")}
                atomic_json(paths["protocol"], p)
                hub.cli("sensitivity-plan", "--protocol", paths["protocol"], "--output", paths["plan"])
                plan = json.loads(paths["plan"].read_text())
                hub.cli("sensitivity-fixture-plan", "--plan", paths["plan"], "--function", "additive", "--output", paths["campaign"])
                if method == "morris":
                    tampered_path = hub.root / "morris-coordinate-tampered-campaign.json"
                    tampered_spec = json.loads(paths["campaign"].read_text())
                    row = tampered_spec["graph"]["steps"][0]["parameters"]["rows"][0]
                    row["values"][0] = repr(float(row["values"][0]) + .125)
                    atomic_json(tampered_path, tampered_spec)
                    tampered_key = hub.cli("submit", tampered_path)["campaign"]
                    self.assertEqual("completed", hub.cli("run", tampered_key)["phase"])
                    with EngineStore(hub.catalogue, readonly=True) as store:
                        tampered_selections = [{"campaign": tampered_key, "step": s["key"], "output": "file:result"}
                                               for s in store.get(tampered_key)["spec"]["graph"]["steps"]]
                        with self.assertRaises(durable.ContractError):
                            sensitivity.collect(store, plan, tampered_selections)
                key = hub.cli("submit", paths["campaign"])["campaign"]
                with EngineStore(hub.catalogue, readonly=True) as store:
                    selections = [{"campaign": key, "step": s["key"], "output": "file:result"} for s in store.get(key)["spec"]["graph"]["steps"]]
                    with self.assertRaises(durable.ContractError): sensitivity.collect(store, plan, selections)
                self.assertEqual("completed", hub.cli("run", key)["phase"])
                with EngineStore(hub.catalogue, readonly=True) as store:
                    obs = sensitivity.collect(store, plan, list(reversed(selections)))
                    self.assertEqual([r["id"] for r in plan["rows"]], [r["id"] for r in obs["rows"]])
                    self.assertTrue(all(c["measurements"]["wall_ns"] > 0 for c in obs["cost_measurements"]))
                    for planned, observed in zip(plan["rows"], obs["rows"]):
                        expected = sum((j + 1) * x for j, x in enumerate(planned["values"]))
                        self.assertAlmostEqual(expected, observed["value"], delta=1e-13)
                    with self.assertRaises(durable.ContractError): sensitivity.collect(store, plan, selections + selections)
                extra = [] if method == "ablation" else ["--worker", stats.binary, "--worker-sha256", stats.sha256, "--source-commit", stats.source_commit]
                result = hub.cli("sensitivity-analyze", "--plan", paths["plan"], "--campaign", key, "--output", paths["report"], *extra)
                self.assertEqual("not-assessed", result["scientific_verdict"])
                report = json.loads(paths["report"].read_text())
                self.assertEqual(len(plan["rows"]), report["included_rows"])
                hub.cli("sensitivity-analyze", "--plan", paths["plan"], "--campaign", key, "--output", paths["report"], *extra, expected_code=22)


if __name__ == "__main__": unittest.main()
