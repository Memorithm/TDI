#!/usr/bin/env python3
"""Validate the non-executing TDI-8.1 configuration freeze contract."""

from __future__ import annotations

import json
from pathlib import Path

CONTRACT = Path("docs/tdi8.1-configuration-freeze.json")
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
FORBIDDEN_TOP_LEVEL = {
    "tdi8_2_seed_range",
    "tdi8_2_result",
    "final_result",
    "confirmation_token",
    "confirmatory_result",
}


def fail(message: str) -> None:
    raise SystemExit(f"TDI-8.1 configuration freeze ERROR: {message}")


def main() -> None:
    data = json.loads(CONTRACT.read_text(encoding="utf-8"))

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

    print(f"TDI-8.1 configuration fields: {len(fields)}")
    print(f"TDI-8.1 unresolved blocking fields: {len(unresolved)}")
    print("TDI-8.2 execution authorization: ABSENT")
    print("TDI-8.2 result/seed/token surface: ABSENT")
    print("TDI-8.1 configuration freeze schema: PASS")


if __name__ == "__main__":
    main()
