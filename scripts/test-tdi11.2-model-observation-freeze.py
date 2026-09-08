#!/usr/bin/env python3
"""Regression tests for the TDI-11.2 freeze validator."""

from __future__ import annotations

import copy
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
VALIDATOR = ROOT / "scripts/check-tdi11.2-model-observation-freeze.py"
CONTRACT = ROOT / "docs/tdi11.2-model-observation-freeze.json"
PREARM = ROOT / "docs/tdi11.2-prearm.yaml"


class FreezeValidatorTests(unittest.TestCase):
    def run_contract(self, data: dict) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "docs").mkdir()
            (root / "scripts").mkdir()
            (root / "docs/tdi11.2-model-observation-freeze.json").write_text(
                json.dumps(data), encoding="utf-8"
            )
            (root / "docs/tdi11.2-prearm.yaml").write_text(
                PREARM.read_text(encoding="utf-8"), encoding="utf-8"
            )
            (root / "scripts/check-tdi11.2-model-observation-freeze.py").write_text(
                VALIDATOR.read_text(encoding="utf-8"), encoding="utf-8"
            )
            return subprocess.run(
                ["python3", "scripts/check-tdi11.2-model-observation-freeze.py"],
                cwd=root,
                text=True,
                capture_output=True,
                check=False,
            )

    def base(self) -> dict:
        return json.loads(CONTRACT.read_text(encoding="utf-8"))

    def test_current_unresolved_contract_is_valid(self) -> None:
        result = self.run_contract(self.base())
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    def test_pinned_value_without_provenance_fails_closed(self) -> None:
        data = self.base()
        entry = data["fields"]["prompt_serializer_version"]
        entry["status"] = "pinned"
        entry["value"] = "serializer-v1"
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires pin_provenance", result.stderr + result.stdout)

    def test_malformed_git_identity_is_rejected(self) -> None:
        data = self.base()
        entry = data["fields"]["prompt_serializer_version"]
        entry["status"] = "pinned"
        entry["value"] = "serializer-v1"
        entry["pin_provenance"] = {
            "kind": "git_commit",
            "immutable_reference": "not-a-commit",
        }
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("exact 40-hex Git object id", result.stderr + result.stdout)

    def test_valid_immutable_reference_can_pin_one_nonfinal_field(self) -> None:
        data = self.base()
        entry = data["fields"]["prompt_serializer_version"]
        entry["status"] = "pinned"
        entry["value"] = "serializer-v1"
        entry["pin_provenance"] = {
            "kind": "git_commit",
            "immutable_reference": "49bae715b4523e198237efbb8bc2f9f4500d8d47",
        }
        result = self.run_contract(data)
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)

    def test_unresolved_field_cannot_carry_provenance(self) -> None:
        data = copy.deepcopy(self.base())
        data["fields"]["prompt_serializer_version"]["pin_provenance"] = {
            "kind": "git_commit",
            "immutable_reference": "49bae715b4523e198237efbb8bc2f9f4500d8d47",
        }
        result = self.run_contract(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unresolved pin_provenance must be null", result.stderr + result.stdout)


if __name__ == "__main__":
    unittest.main()
