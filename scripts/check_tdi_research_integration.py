#!/usr/bin/env python3
"""Run the complete public software integration profile; never skip missing tools.

Usage: python scripts/check_tdi_research_integration.py --report NEW_FILE.json
Required binary/source environment variables are documented in
docs/engineering/release-and-compatibility.md. No protected/model suite is
discovered. Exit 0 requires every listed test to run successfully, no skips or
expected failures, unchanged executable bytes, and a written diagnostic report.
This report is software evidence, not a release or source/build attestation.
"""
from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import sqlite3
import subprocess
import sys
import time
import unittest

MODULES = (
    "test_tdi_engine_unit", "test_tdi_engine_integration",
    "test_tdi_shared_evidence", "test_tdi_library_integration",
    "test_tdi_research_analysis", "test_tdi_analysis_integration",
    "test_tdi_resource_integration", "test_tdi_forge_integration",
    "test_tdi_sensitivity", "test_tdi_research_reporting",
    "test_tdi_attention_integration", "test_tdi_engine_benchmark",
    "test_tdi_observability_unit", "test_tdi_observability_integration",
)
BINARIES = (
    "TDI_HUBD_BIN", "TDI_DURABLE_WORKER", "TDI_LIBRARY_WORKER",
    "TDI_SCIRUST_STATS_BIN", "TDI_SEARCH_WORKER", "TDI_FORGE_WORKER",
    "TDI_ELASTIC_BIN", "TDI_ATTENTION_PROBE", "TDI_MLFLOW_BIN",
)
PINS = {
    "TDI_SCIRUST_SOURCE_COMMIT": "be7fcca3b31cedf722d71a2a56db8f6d088037cf",
    "TDI_FORGE_SOURCE_COMMIT": "28067ab0aa1d52a2260d9bb2bf35a346547a292a",
    "TDI_ELASTIC_SOURCE_COMMIT": "f8e10b1a1d05d6c22c0f56fed87252bc65d4e66a",
}


def executable_digest(variable):
    """Hash the explicitly selected executable; missing/non-executable is fatal."""
    path = Path(os.environ[variable]).resolve(strict=True)
    if not path.is_file() or not os.access(path, os.X_OK):
        raise ValueError(f"{variable}: expected an executable file")
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"sha256": digest, "size": path.stat().st_size}


def main(argv=None):
    """Execute the fixed suite and write a new JSON report, including failures.

    Report paths must be new and have an existing parent. Partial/crashed runs
    may leave an empty report; only status=passed is complete software evidence.
    Runtime, source declarations and file hashes do not authenticate provenance.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args(argv)
    sys.dont_write_bytecode = True
    os.environ["PYTHONDONTWRITEBYTECODE"] = "1"
    root = Path(__file__).resolve().parents[1]
    os.chdir(root)
    sys.path.insert(0, str(root / "scripts"))
    os.environ["PYTHONPATH"] = str(root / "scripts")
    report = {
        "schema": 1, "profile": "public-research-software-integration",
        "started_at": datetime.now(timezone.utc).isoformat(),
        "status": "failed", "hardware_execution_performed": False,
        "release_qualified": False, "source_build_attestation": False,
        "python": platform.python_version(), "platform": platform.platform(),
        "sqlite": sqlite3.sqlite_version, "modules": list(MODULES),
    }
    started = time.monotonic()
    # Exclusive creation also refuses symlinks and already-existing reports.
    with args.report.open("x", encoding="utf-8") as destination:
        try:
            if sys.version_info[:2] != (3, 12) or sys.platform != "linux":
                raise ValueError("this profile requires Linux and Python 3.12")
            if os.environ.get("TDI_REPORT_FIGURES") != "1":
                raise ValueError("TDI_REPORT_FIGURES=1 is required")
            for key, value in PINS.items():
                if os.environ.get(key) != value:
                    raise ValueError(f"{key}: source declaration differs from the candidate profile")
            revision = os.environ.get("TDI_SEARCH_SOURCE_COMMIT", "")
            if len(revision) != 40 or any(c not in "0123456789abcdef" for c in revision):
                raise ValueError("TDI_SEARCH_SOURCE_COMMIT must explicitly declare 40 lowercase hex digits")
            checkout = subprocess.run(["git", "rev-parse", "HEAD"], check=True,
                                      capture_output=True, text=True, timeout=5).stdout.strip()
            if revision != checkout:
                raise ValueError("TDI_SEARCH_SOURCE_COMMIT differs from the actual TDI checkout HEAD")
            report["checkout_head"] = checkout
            report["test_source_sha256"] = {
                name: hashlib.sha256((root / "scripts" / (name + ".py")).read_bytes()).hexdigest()
                for name in MODULES
            }
            report["runner_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
            if not Path(os.environ["TDI_NNIS_CHECKOUT"]).is_dir():
                raise ValueError("TDI_NNIS_CHECKOUT: missing exact NNIS checkout")
            report["declared_source_pins"] = dict(PINS, TDI_SEARCH_SOURCE_COMMIT=revision)
            report["executables_before"] = {key: executable_digest(key) for key in BINARIES}
            report["packages"] = {}
            for line in (root / "scripts/requirements-research-qualification.txt").read_text().splitlines():
                if not line or line.startswith("#"):
                    continue
                name, version = line.split("==")
                actual = importlib.metadata.version(name)
                if actual != version:
                    raise ValueError(f"{name}: installed version differs from the pinned qualification profile")
                report["packages"][name] = actual
            suite = unittest.defaultTestLoader.loadTestsFromNames(MODULES)
            expected_count = suite.countTestCases()
            report["collected_tests"] = expected_count
            result = unittest.TextTestRunner(verbosity=2).run(suite)
            report.update(
                tests_run=result.testsRun,
                failures=[str(test) for test, _ in result.failures],
                errors=[str(test) for test, _ in result.errors],
                skipped=[str(test) for test, _ in result.skipped],
                expected_failures=[str(test) for test, _ in result.expectedFailures],
                unexpected_successes=[str(test) for test in result.unexpectedSuccesses],
            )
            report["executables_after"] = {key: executable_digest(key) for key in BINARIES}
            if (not unittest.defaultTestLoader.errors and expected_count >= 70
                    and result.testsRun == expected_count and result.wasSuccessful()
                    and not result.skipped and not result.expectedFailures
                    and report["executables_before"] == report["executables_after"]):
                report["status"] = "passed"
        except Exception as error:
            report["preflight_or_runtime_error"] = f"{type(error).__name__}: {error}"
            print(report["preflight_or_runtime_error"], file=sys.stderr)
        finally:
            report["elapsed_seconds"] = time.monotonic() - started
            json.dump(report, destination, ensure_ascii=True, allow_nan=False, indent=2)
            destination.write("\n")
            destination.flush()
            os.fsync(destination.fileno())
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
