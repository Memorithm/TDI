#!/usr/bin/env python3
"""Validate the non-executing TDI-8.1 configuration freeze contract."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

DEFAULT_CONTRACT = Path("docs/tdi8.1-configuration-freeze.json")
EXPECTED_FIELDS = {
    "recurrent_dimension_and_parameters",
    "readout_coordinates",
    "a2_capacity_and_projection",
    "a3_vsa_width_role_seed_and_fusion",
    "a3_event_store_read_cleanup_policy",
    "matched_dynamic_memory_budget",
    "short_medium_long_numeric_horizons",
    "late_retrieval_deficit_definition",
    "intervention_sites_and_recovery_observable",
    "closed_rejection_taxonomy",
    "paired_interval_method",
    "paired_resampling_replicate_count",
    "paired_resampling_seed",
    "degenerate_replicate_policy",
    "development_population_domain_and_sample_count",
    "validation_population_domain_and_sample_count",
    "final_population_domain_and_sample_count",
}
ALLOWED_FIELD_STATUS = {"unresolved_blocking", "pinned"}
AUTHORIZED_PINS = {
    "a3_event_store_read_cleanup_policy": "tdi8.1-a3-qualified-adapter-v1",
    "closed_rejection_taxonomy": "SymbolicRejectionCode",
    "degenerate_replicate_policy": "tdi8.1-reject-zero-baseline-bootstrap-replicates-v1",
}
PINNED_COUNT_FLOOR = 3
FORBIDDEN_TOP_LEVEL = {
    "tdi8_2_seed_range",
    "tdi8_2_result",
    "final_result",
    "confirmation_token",
    "confirmatory_result",
}
PR_CITATION = re.compile(r"PR #\d+")


def fail(message: str) -> None:
    raise SystemExit(f"TDI-8.1 configuration freeze ERROR: {message}")


def require_pin_evidence(name: str, value: object, *, repo_root: Path) -> None:
    if not isinstance(value, dict):
        fail(f"{name} pin value must be an object carrying an evidence block")
    evidence = value.get("evidence")
    if not isinstance(evidence, list) or not evidence:
        fail(f"{name} pin missing a non-empty evidence list")
    has_pr = False
    has_file = False
    for item in evidence:
        if not isinstance(item, str) or not item.strip():
            fail(f"{name} evidence items must be non-empty strings")
        if PR_CITATION.search(item):
            has_pr = True
        candidate = repo_root / item
        if candidate.is_file() and candidate.stat().st_size > 0:
            has_file = True
    if not has_pr:
        fail(f"{name} pin evidence must cite at least one PR")
    if not has_file:
        fail(f"{name} pin evidence must cite at least one existing in-repo file")


def validate_contract(data: dict, *, repo_root: Path) -> tuple[int, int]:
    if data.get("schema") != "tdi8.1-configuration-freeze-v1":
        fail("unexpected schema identity")
    if data.get("stage") != "TDI-8.1":
        fail("contract must remain scoped to TDI-8.1")

    for flag in (
        "tdi8_2_execution_authorized",
        "final_holdout_exists",
        "confirmatory_runner_exists",
        "human_confirmation_token_exists",
    ):
        if data.get(flag) is not False:
            fail(f"{flag} must remain false in this non-executing contract")

    forbidden = FORBIDDEN_TOP_LEVEL.intersection(data)
    if forbidden:
        fail(f"forbidden TDI-8.2/result surface present: {sorted(forbidden)}")

    fields = data.get("fields")
    if not isinstance(fields, dict):
        fail("fields must be an object")
    if set(fields) != EXPECTED_FIELDS:
        missing = sorted(EXPECTED_FIELDS.difference(fields))
        extra = sorted(set(fields).difference(EXPECTED_FIELDS))
        fail(f"field registry mismatch; missing={missing}, extra={extra}")

    unresolved = []
    pinned = []
    for name, record in fields.items():
        if not isinstance(record, dict) or set(record) != {"status", "value"}:
            fail(f"{name} must contain exactly status and value")
        status = record["status"]
        value = record["value"]
        if status not in ALLOWED_FIELD_STATUS:
            fail(f"{name} has invalid status {status!r}")
        if status == "unresolved_blocking":
            if value is not None:
                fail(f"{name} is unresolved but carries a value")
            unresolved.append(name)
        elif value is None or value == "" or value == {} or value == []:
            fail(f"{name} is pinned without a non-empty explicit value")
        else:
            pinned.append(name)

    expected_scientific_status = "unresolved_blocking" if unresolved else "frozen_nonfinal"
    if data.get("scientific_status") != expected_scientific_status:
        fail(
            "scientific_status must be unresolved_blocking until every field is pinned, "
            "then frozen_nonfinal"
        )

    if len(pinned) < PINNED_COUNT_FLOOR:
        fail(
            f"pinned-count floor is {PINNED_COUNT_FLOOR}; found {len(pinned)}"
        )

    for name, record in fields.items():
        if name in AUTHORIZED_PINS:
            if record["status"] != "pinned":
                fail(f"{name} is an authorized pin and must remain pinned")
            blob = json.dumps(record["value"], sort_keys=True)
            marker = AUTHORIZED_PINS[name]
            if marker not in blob:
                fail(f"{name} pin missing required marker {marker!r}")
            require_pin_evidence(name, record["value"], repo_root=repo_root)
        elif record["status"] != "unresolved_blocking":
            fail(
                f"{name} is not an authorized evidence-backed pin and must "
                "remain unresolved_blocking"
            )

    return len(pinned), len(unresolved)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--contract",
        type=Path,
        default=DEFAULT_CONTRACT,
        help="path to the TDI-8.1 configuration freeze JSON",
    )
    args = parser.parse_args()
    repo_root = Path.cwd()
    data = json.loads(args.contract.read_text(encoding="utf-8"))
    pinned, unresolved = validate_contract(data, repo_root=repo_root)
    print(f"TDI-8.1 configuration fields: {len(EXPECTED_FIELDS)}")
    print(f"TDI-8.1 unresolved blocking fields: {unresolved}")
    print(f"TDI-8.1 authorized pins: {len(AUTHORIZED_PINS)}")
    print(f"TDI-8.1 pinned-count floor: {PINNED_COUNT_FLOOR} (observed {pinned})")
    print("TDI-8.2 execution authorization: ABSENT")
    print("TDI-8.2 result/seed/token surface: ABSENT")
    print("TDI-8.1 configuration freeze schema: PASS")


if __name__ == "__main__":
    main()
