#!/usr/bin/env python3
"""Fail-closed regression tests for the TDI-11.2 readiness gate."""

from __future__ import annotations

import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GATE = ROOT / "scripts/check-tdi11.2-readiness.py"
FREEZE = ROOT / "docs/tdi11.2-model-observation-freeze.json"
PREARM = ROOT / "docs/tdi11.2-prearm.yaml"
LEDGER = ROOT / "docs/TDI-11.2-UNRESOLVED-LEDGER.md"
DIGEST = ROOT / "docs/tdi11.2-model-observation-freeze.sha256"
STATUS = ROOT / "docs/TDI-11.2-STATUS.md"


class ReadinessTests(unittest.TestCase):
    def run_bundle(self, freeze: dict | None = None, *, extra_file: str | None = None, status_text: str | None = None) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as raw:
            root = Path(raw)
            (root / "docs").mkdir()
            freeze_path = root / "docs/tdi11.2-model-observation-freeze.json"
            if freeze is None:
                freeze_path.write_bytes(FREEZE.read_bytes())
            else:
                freeze_path.write_text(json.dumps(freeze, indent=2) + "\n", encoding="utf-8")
            prearm = root / "docs/tdi11.2-prearm.yaml"
            prearm.write_text(PREARM.read_text(encoding="utf-8"), encoding="utf-8")
            ledger = root / "docs/TDI-11.2-UNRESOLVED-LEDGER.md"
            ledger.write_text(LEDGER.read_text(encoding="utf-8"), encoding="utf-8")
            digest = root / "docs/tdi11.2-model-observation-freeze.sha256"
            digest.write_text(DIGEST.read_text(encoding="utf-8"), encoding="utf-8")
            status = root / "docs/TDI-11.2-STATUS.md"
            status.write_text(
                status_text if status_text is not None else STATUS.read_text(encoding="utf-8"),
                encoding="utf-8",
            )
            if extra_file:
                extra = root / extra_file
                extra.parent.mkdir(parents=True, exist_ok=True)
                extra.write_text("forbidden surface fixture\n", encoding="utf-8")
            cmd = [
                "python3",
                str(GATE),
                "--freeze",
                str(freeze_path),
                "--prearm",
                str(prearm),
                "--ledger",
                str(ledger),
                "--digest",
                str(digest),
                "--status",
                str(status),
            ]
            if extra_file is None:
                cmd.append("--skip-surface-scan")
            return subprocess.run(
                cmd,
                cwd=root if extra_file else ROOT,
                text=True,
                capture_output=True,
                check=False,
            )

    def base(self) -> dict:
        return json.loads(FREEZE.read_text(encoding="utf-8"))

    def test_current_unresolved_ledger_passes(self) -> None:
        result = self.run_bundle()
        self.assertEqual(result.returncode, 0, result.stderr + result.stdout)
        self.assertIn("unresolved fields: 12/12", result.stdout)
        self.assertIn("STATUS↔JSON pin-count cross-check: 0/12", result.stdout)

    def test_armed_execution_with_unresolved_fields_fails_closed(self) -> None:
        data = self.base()
        data["model_execution_authorized"] = True
        result = self.run_bundle(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(
            "model_execution_authorized is true while unresolved_blocking",
            result.stderr + result.stdout,
        )

    def test_invented_model_pin_fails_closed(self) -> None:
        data = self.base()
        data["fields"]["model_artifact_identity"] = {
            "status": "pinned",
            "value": "invented-model",
            "pin_provenance": {
                "kind": "git_commit",
                "immutable_reference": "ecca754dc226432faf88daee55c66b5d43308204",
            },
        }
        result = self.run_bundle(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("invented TDI-11.2 pins are forbidden", result.stderr + result.stdout)

    def test_silent_scientific_status_upgrade_fails_closed(self) -> None:
        data = self.base()
        data["scientific_status"] = "frozen_nonfinal"
        result = self.run_bundle(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("cannot be upgraded while fields remain unresolved", result.stderr + result.stdout)

    def test_digest_drift_fails_closed(self) -> None:
        data = self.base()
        data["stage"] = "TDI-11.2"
        # Touch a non-field key that still changes file bytes after rewrite? schema stays.
        # Adding an unused comment is impossible in JSON; change a boolean then restore
        # is not needed — writing the same dict via json.dumps changes key order / separators
        # enough that the raw digest will not match the sidecar.
        result = self.run_bundle(data)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("sha256 sidecar does not match", result.stderr + result.stdout)

    def test_status_pin_count_mismatch_fails_closed(self) -> None:
        bad_status = STATUS.read_text(encoding="utf-8").replace("0/12 pinned", "1/12 pinned", 1)
        self.assertIn("1/12 pinned", bad_status)
        result = self.run_bundle(status_text=bad_status)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("STATUS pin count", result.stderr + result.stdout)

    def test_forbidden_surface_fails_closed(self) -> None:
        result = self.run_bundle(extra_file="tdi-ai/tdi11.2_concrete_model_runner.py")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("forbidden TDI-11.2 surfaces present", result.stderr + result.stdout)


if __name__ == "__main__":
    unittest.main()
