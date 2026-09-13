#!/usr/bin/env python3
"""Fail-closed regression tests for the TDI-9.1 freeze validator."""

from __future__ import annotations

import copy
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts/check-tdi9.1-configuration-freeze.py"
CONTRACT = ROOT / "docs/tdi9.1-configuration-freeze.json"


class FreezeValidatorTests(unittest.TestCase):
    def run_contract(self, data: dict) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "tdi9.1-configuration-freeze.json"
            path.write_text(json.dumps(data), encoding="utf-8")
            return subprocess.run(
                ["python3", str(VALIDATOR), "--contract", str(path)],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

    def base(self) -> dict:
        return json.loads(CONTRACT.read_text(encoding="utf-8"))

    def test_current_contract_passes(self) -> None:
        result = self.run_contract(self.base())
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertIn("pinned-count floor: 1", result.stdout)

    def test_invented_pin_fails_closed(self) -> None:
        data = self.base()
        data["fields"]["permitted_observation_vector"] = {
            "status": "pinned",
            "value": {
                "vector": "invented",
                "evidence": [
                    "docs/TDI-9.1-REFERENCE-POLICIES.md",
                    "PR #129",
                ],
            },
        }
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not an authorized evidence-backed pin", result.stderr + result.stdout)

    def test_missing_evidence_block_fails_closed(self) -> None:
        data = self.base()
        value = copy.deepcopy(data["fields"]["closed_rejection_taxonomy"]["value"])
        del value["evidence"]
        data["fields"]["closed_rejection_taxonomy"]["value"] = value
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing a non-empty evidence list", result.stderr + result.stdout)

    def test_silent_scientific_status_upgrade_fails_closed(self) -> None:
        data = self.base()
        data["scientific_status"] = "frozen_nonfinal"
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unresolved_blocking until every field is pinned", result.stderr + result.stdout)

    def test_tdi93_does_not_authorize_a_9_1_pin(self) -> None:
        data = self.base()
        data["fields"]["agent_search_safe_policy_mutation_contract"] = {
            "status": "pinned",
            "value": {
                "note": "TDI-9.3 Boolean IR is not a 9.1 freeze",
                "evidence": [
                    "docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md",
                    "PR #195",
                ],
            },
        }
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not an authorized evidence-backed pin", result.stderr + result.stdout)

    def test_execution_flag_cannot_be_armed(self) -> None:
        data = self.base()
        data["tdi9_2_execution_authorized"] = True
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("tdi9_2_execution_authorized must remain false", result.stderr + result.stdout)


if __name__ == "__main__":
    unittest.main()
