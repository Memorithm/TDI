#!/usr/bin/env python3
"""Fail-closed tests for the freeze-progress summary emitter."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EMITTER = ROOT / "scripts/emit-tdi-freeze-progress-summary.py"
SUMMARY = ROOT / "docs/tdi-freeze-progress-summary.json"


class FreezeProgressSummaryTests(unittest.TestCase):
    def test_checked_in_summary_passes(self) -> None:
        result = subprocess.run(
            ["python3", str(EMITTER), "--check"],
            cwd=ROOT,
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertIn("3/17", result.stdout)
        self.assertIn("1/14", result.stdout)
        self.assertIn("0/12", result.stdout)
        self.assertIn("ledger digest verified", result.stdout)

    def test_summary_schema_and_scout(self) -> None:
        data = json.loads(SUMMARY.read_text(encoding="utf-8"))
        self.assertEqual(data["schema"], "tdi-freeze-progress-summary-v1")
        self.assertEqual(data["series"]["tdi8.1"]["pinned_progress"], "3/17")
        self.assertEqual(data["series"]["tdi9.1"]["pinned_progress"], "1/14")
        self.assertEqual(data["series"]["tdi11.2"]["pinned_progress"], "0/12")
        self.assertTrue(data["series"]["tdi11.2"]["ledger_digest_verified"])
        digest = data["series"]["tdi11.2"]["ledger_digest_sha256"]
        self.assertRegex(digest, r"^[0-9a-f]{64}$")
        self.assertEqual(data["scout"]["new_8_1_pins"], [])
        self.assertEqual(data["scout"]["new_9_1_pins"], [])
        self.assertEqual(data["scout"]["new_11_2_pins"], [])
        surfaces = data["series"]["tdi9.1"]["non_authorizing_surfaces"]
        self.assertIn("docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md", surfaces)
        self.assertIn("PR #195", surfaces)
        for sid in ("tdi8.1", "tdi9.1", "tdi11.2"):
            self.assertGreaterEqual(
                len(data["series"][sid]["blockers"]),
                1 if sid != "tdi11.2" else 12,
            )
            for blocker in data["series"][sid]["blockers"]:
                self.assertTrue(blocker["required_evidence_class"])
            for flag, value in data["series"][sid]["execution_flags"].items():
                self.assertIs(value, False, flag)

    def test_stale_summary_fails_check(self) -> None:
        original = SUMMARY.read_bytes()
        try:
            data = json.loads(original)
            data["scout"]["verdict"] = "stale-on-purpose"
            SUMMARY.write_text(
                json.dumps(data, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            result = subprocess.run(
                ["python3", str(EMITTER), "--check"],
                cwd=ROOT,
                text=True,
                capture_output=True,
                check=False,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("stale", result.stderr + result.stdout)
        finally:
            SUMMARY.write_bytes(original)

    def test_inventory_missing_evidence_class_fails_write(self) -> None:
        inv_path = ROOT / "docs/tdi8.1-blocker-evidence-classes.json"
        original = inv_path.read_text(encoding="utf-8")
        try:
            data = json.loads(original)
            del data["fields"]["paired_interval_method"]["required_evidence_class"]
            inv_path.write_text(json.dumps(data), encoding="utf-8")
            with tempfile.TemporaryDirectory() as raw:
                out = Path(raw) / "summary.json"
                result = subprocess.run(
                    ["python3", str(EMITTER), "--write", "--output", str(out)],
                    cwd=ROOT,
                    text=True,
                    capture_output=True,
                    check=False,
                )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("required_evidence_class", result.stderr + result.stdout)
        finally:
            inv_path.write_text(original, encoding="utf-8")


if __name__ == "__main__":
    unittest.main()
