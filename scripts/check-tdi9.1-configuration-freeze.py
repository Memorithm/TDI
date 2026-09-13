#!/usr/bin/env python3
"""Validate the non-executing TDI-9.1 configuration freeze contract."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

DEFAULT_CONTRACT = Path("docs/tdi9.1-configuration-freeze.json")
EXPECTED_FIELDS = {
    "p1_p2_p3_difficulty_parameters",
    "c0_c1_schedules",
    "c2_c3_thresholds_and_verification_cadence",
    "permitted_observation_vector",
    "common_max_compute_memory_envelopes",
    "paired_interval_method",
    "paired_resampling_replicate_count",
    "paired_resampling_seed",
    "closed_rejection_taxonomy",
    "development_validation_population_domains",
    "future_public_entropy_source_event_encoding",
    "final_seed_derivation_contract",
    "agent_search_safe_policy_mutation_contract",
    "h9_classifier_aggregation_plumbing",
}
ALLOWED_FIELD_STATUS = {"unresolved_blocking", "pinned"}
AUTHORIZED_PINS = {
    "closed_rejection_taxonomy": "ReferenceRejectionCode",
}
PINNED_COUNT_FLOOR = 1
FORBIDDEN_TOP_LEVEL = {
    "tdi9_2_seed_list",
    "tdi9_2_result",
    "final_result",
    "confirmation_token",
    "confirmatory_result",
    "human_token",
}
PR_CITATION = re.compile(r"PR #\d+")
# TDI-9.3 Boolean policy synthesis (#195) is ACTIVE DESIGN / NON-FINAL and must
# never appear as authorizing evidence for a TDI-9.1 freeze pin.
NON_AUTHORIZING_PIN_EVIDENCE = {
    "docs/TDI-9.3-BOOLEAN-POLICY-SYNTHESIS.md",
}



def fail(message: str) -> None:
    raise SystemExit(f"TDI-9.1 configuration freeze ERROR: {message}")


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
        if item in NON_AUTHORIZING_PIN_EVIDENCE:
            fail(
                f"{name} pin evidence cites non-authorizing TDI-9.3 surface "
                f"{item} (Boolean IR does not authorize 9.1 pins)"
            )
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
    if data.get("schema") != "tdi9.1-configuration-freeze-v1":
        fail("unexpected schema identity")
    if data.get("stage") != "TDI-9.1":
        fail("contract must remain scoped to TDI-9.1")

    for flag in (
        "tdi9_2_execution_authorized",
        "final_seed_list_exists",
        "final_dataset_exists",
        "final_runner_exists",
        "final_result_payload_exists",
        "human_confirmation_token_exists",
    ):
        if data.get(flag) is not False:
            fail(f"{flag} must remain false in this non-executing contract")

    forbidden = FORBIDDEN_TOP_LEVEL.intersection(data)
    if forbidden:
        fail(f"forbidden TDI-9.2/result surface present: {sorted(forbidden)}")

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
        help="path to the TDI-9.1 configuration freeze JSON",
    )
    args = parser.parse_args()
    repo_root = Path.cwd()
    data = json.loads(args.contract.read_text(encoding="utf-8"))
    pinned, unresolved = validate_contract(data, repo_root=repo_root)
    print(f"TDI-9.1 configuration fields: {len(EXPECTED_FIELDS)}")
    print(f"TDI-9.1 unresolved blocking fields: {unresolved}")
    print(f"TDI-9.1 authorized pins: {len(AUTHORIZED_PINS)}")
    print(f"TDI-9.1 pinned-count floor: {PINNED_COUNT_FLOOR} (observed {pinned})")
    print("TDI-9.2 execution authorization: ABSENT")
    print("TDI-9.2 final seed/dataset/runner/result surface: ABSENT")
    print("TDI-9.1 configuration freeze schema: PASS")


if __name__ == "__main__":
    main()
