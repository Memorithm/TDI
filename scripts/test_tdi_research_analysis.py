"""Declared unit/exclusion tests and actual shared SciRust process qualification."""
import copy
import os
from pathlib import Path
import tempfile
import unittest

import tdi_experiment_supervisor as durable
from tdi_engine_store import identity
from tdi_research_analysis import canonical_protocol, analyze
from tdi_scirust_client import SciRustStats


def fixture():
    protocol = {"schema": 1, "purpose": "exploratory-analysis", "domain": "Development", "name": "paired-example",
                "unit_kind": "task", "units": [{"id": "u0", "stratum": "small", "replicates": ["r0", "r1", "r2", "r3"]},
                                                {"id": "u1", "stratum": "small", "replicates": ["r0"]},
                                                {"id": "u2", "stratum": "small", "replicates": ["r0"]}],
                "comparisons": [{"id": "candidate-reference", "reference": "reference", "candidate": "candidate", "metric": "score", "unit": "points"}],
                "strata": ["small"], "missing": "exclude-incomplete-unit", "confidence": 0.95,
                "resamples": 1000, "seed": "731", "multiplicity": "bonferroni-family"}
    rows = []
    for unit in protocol["units"]:
        for rep in unit["replicates"]:
            for arm in ("reference", "candidate"):
                missing = unit["id"] == "u2" and arm == "candidate"
                source = identity("synthetic-analysis-fixture/v1", [unit["id"], rep, arm])
                rows.append({"unit": unit["id"], "replicate": rep, "arm": arm, "metric": "score",
                             "status": "technical-error" if missing else "observed", "reason": "worker-exit" if missing else None,
                             "value": None if missing else (10 if unit["id"] == "u1" and arm == "candidate" else 0),
                             "source": None if missing else {"campaign": source, "artifact_identity": source, "provenance_identity": source, "pointer": "/score"}})
    return protocol, {"schema": 1, "domain": "Development", "protocol_identity": identity("tdi-analysis-protocol/v1", protocol), "rows": rows}


class ProtocolTests(unittest.TestCase):
    def test_rejects_implicit_final_policies_unknown_units_and_unbounded_work(self):
        protocol, _ = fixture()
        for key, value in (("domain", "Confirmation"), ("unit_kind", "row"), ("missing", "ignore"),
                           ("multiplicity", "choose-later"), ("seed", "-1"), ("confidence", float("nan")), ("resamples", 100_001)):
            changed = dict(protocol, **{key: value})
            with self.subTest(key=key), self.assertRaises(durable.ContractError): canonical_protocol(changed)

    def test_duplicate_unplanned_nonfinite_and_reused_observations_are_rejected(self):
        protocol, data = fixture()
        variants = []
        x = copy.deepcopy(data); x["rows"].append(copy.deepcopy(x["rows"][0])); variants.append(x)
        x = copy.deepcopy(data); x["rows"][0]["value"] = float("inf"); variants.append(x)
        x = copy.deepcopy(data); x["rows"][1]["source"] = x["rows"][0]["source"]; variants.append(x)
        x = copy.deepcopy(data); x["rows"][0]["unit"] = "unknown"; variants.append(x)
        for x in variants:
            with self.assertRaises(durable.ContractError): analyze(protocol, x, None)
        x = copy.deepcopy(protocol); x["missing"] = "reject-incomplete"
        data["protocol_identity"] = identity("tdi-analysis-protocol/v1", x)
        with self.assertRaises(durable.ContractError): analyze(x, data, None)


class SharedSciRustTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        binary = Path(os.environ["TDI_SCIRUST_STATS_BIN"]).resolve(strict=True)
        cls.worker = SciRustStats(binary, durable.file_digest(binary), os.environ["TDI_SCIRUST_SOURCE_COMMIT"])

    def test_equal_unit_weighting_exclusions_family_and_deterministic_report(self):
        protocol, data = fixture()
        report = analyze(protocol, data, self.worker)
        result = report["results"][0]
        self.assertEqual((3, 2, 1), (result["expected_units"], result["included_units"], result["excluded_units"]))
        # Four zero-effect repeats in u0 do not count as four independent units.
        self.assertEqual(5, result["interval"]["estimate"])
        self.assertEqual(2, result["interval"]["units"])
        self.assertEqual(0.975, result["interval"]["confidence"])
        self.assertEqual(6, result["denominators"]["candidate"]["planned"])
        self.assertEqual(5, result["denominators"]["candidate"]["observed"])
        self.assertEqual(1, result["denominators"]["candidate"]["technical-error"])
        self.assertEqual("not-assessed", report["scientific_verdict"])
        self.assertEqual(report, analyze(protocol, data, self.worker))

    def test_insufficient_units_stays_visible_without_interval(self):
        protocol, data = fixture()
        data["rows"] = [r for r in data["rows"] if r["unit"] != "u1"]
        report = analyze(protocol, data, self.worker)
        self.assertEqual("insufficient-units", report["results"][0]["status"])
        self.assertIsNone(report["results"][0]["interval"])

    def test_process_identity_invalid_input_and_real_shared_methods(self):
        with self.assertRaises(durable.ContractError): SciRustStats(self.worker.binary, "0" * 64, self.worker.source_commit)
        with self.assertRaises(durable.ContractError): self.worker.call("holm", p_values=[-1])
        self.assertEqual([0.03, 0.06, 0.06], self.worker.call("holm", p_values=[0.01, 0.04, 0.03])["adjusted"])
        value = self.worker.call("morris", inputs=[[0], [1], [1], [0]], outputs=[0, 3, 3, 0])
        self.assertEqual([3], value["mu_star"])


if __name__ == "__main__": unittest.main()
