#!/usr/bin/env python3
"""Fail-closed regression tests for the TDI-8.1 freeze validator."""

from __future__ import annotations

import copy
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts/check-tdi8.1-configuration-freeze.py"
CONTRACT = ROOT / "docs/tdi8.1-configuration-freeze.json"


class FreezeValidatorTests(unittest.TestCase):
    def run_contract(self, data: dict) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as raw:
            path = Path(raw) / "tdi8.1-configuration-freeze.json"
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
        self.assertIn("pinned-count floor: 3", result.stdout)

    def test_invented_pin_fails_closed(self) -> None:
        data = self.base()
        data["fields"]["paired_interval_method"] = {
            "status": "pinned",
            "value": {
                "method": "invented",
                "evidence": [
                    "docs/TDI-8.1-PERCENTILE-INTERVAL-PREFLIGHT.md",
                    "PR #113",
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

    def test_dropping_authorized_pin_fails_closed(self) -> None:
        data = self.base()
        data["fields"]["degenerate_replicate_policy"] = {
            "status": "unresolved_blocking",
            "value": None,
        }
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        combined = result.stderr + result.stdout
        self.assertTrue(
            "must remain pinned" in combined or "pinned-count floor is 3" in combined,
            combined,
        )

    def test_execution_flag_cannot_be_armed(self) -> None:
        data = self.base()
        data["tdi8_2_execution_authorized"] = True
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("tdi8_2_execution_authorized must remain false", result.stderr + result.stdout)


if __name__ == "__main__":
    unittest.main()
