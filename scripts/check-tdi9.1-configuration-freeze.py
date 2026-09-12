#!/usr/bin/env python3
"""Validate the non-executing TDI-9.1 configuration freeze contract."""

from __future__ import annotations

import json
from pathlib import Path

CONTRACT = Path("docs/tdi9.1-configuration-freeze.json")
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
FORBIDDEN_TOP_LEVEL = {
    "tdi9_2_seed_list",
    "tdi9_2_result",
    "final_result",
    "confirmation_token",
    "confirmatory_result",
    "human_token",
}


def fail(message: str) -> None:
    raise SystemExit(f"TDI-9.1 configuration freeze ERROR: {message}")


def main() -> None:
    data = json.loads(CONTRACT.read_text(encoding="utf-8"))

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

    expected_scientific_status = "unresolved_blocking" if unresolved else "frozen_nonfinal"
    if data.get("scientific_status") != expected_scientific_status:
        fail(
            "scientific_status must be unresolved_blocking until every field is pinned, "
            "then frozen_nonfinal"
        )

    for name, record in fields.items():
        if name in AUTHORIZED_PINS:
            if record["status"] != "pinned":
                fail(f"{name} is an authorized pin and must remain pinned")
            blob = json.dumps(record["value"], sort_keys=True)
            marker = AUTHORIZED_PINS[name]
            if marker not in blob:
                fail(f"{name} pin missing required marker {marker!r}")
        elif record["status"] != "unresolved_blocking":
            fail(
                f"{name} is not an authorized evidence-backed pin and must "
                "remain unresolved_blocking"
            )

    print(f"TDI-9.1 configuration fields: {len(fields)}")
    print(f"TDI-9.1 unresolved blocking fields: {len(unresolved)}")
    print(f"TDI-9.1 authorized pins: {len(AUTHORIZED_PINS)}")
    print("TDI-9.2 execution authorization: ABSENT")
    print("TDI-9.2 final seed/dataset/runner/result surface: ABSENT")
    print("TDI-9.1 configuration freeze schema: PASS")


if __name__ == "__main__":
    main()
