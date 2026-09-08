#!/usr/bin/env python3
"""Validate the non-executing TDI-11.2 model/observation freeze contract."""

from __future__ import annotations

import json
from pathlib import Path

CONTRACT = Path("docs/tdi11.2-model-observation-freeze.json")
PREARM = Path("docs/tdi11.2-prearm.yaml")

EXPECTED_FIELDS = {
    "model_artifact_identity",
    "adapter_identity_and_version",
    "tokenizer_and_template_identity",
    "decoding_configuration",
    "prompt_serializer_version",
    "exact_observation_registry",
    "exact_observation_timing",
    "primary_pre_assertion_eligibility",
    "development_validation_population_derivation",
    "resource_accounting_contract",
    "typed_rejection_contract",
    "provenance_contract",
}
ALLOWED_FIELD_STATUS = {"unresolved_blocking", "pinned"}
ALLOWED_PROVENANCE_KINDS = {
    "git_blob",
    "git_commit",
    "sha256",
    "registry_digest",
    "protocol_revision",
}
EXPECTED_DOMAINS = ["Development", "Validation"]
EXPECTED_FORBIDDEN_INPUTS = [
    "complete_world_hidden_truth",
    "evaluator_labels",
    "hidden_difficulty",
    "future_trajectory_state",
    "alternative_arm_outcomes",
    "final_seed_material",
]
EXPECTED_FORBIDDEN_SURFACES = [
    "concrete_model_runner",
    "final_seed_list",
    "final_dataset",
    "final_runner",
    "final_result_payload",
]
FORBIDDEN_TOP_LEVEL = {
    "final_seed_list",
    "final_seed_range",
    "final_result",
    "confirmatory_result",
    "final_model_evaluation",
    "human_confirmation_token",
}


def fail(message: str) -> None:
    raise SystemExit(f"TDI-11.2 model/observation freeze ERROR: {message}")


def validate_pin_provenance(name: str, provenance: object) -> None:
    if not isinstance(provenance, dict):
        fail(f"{name} pinned value requires pin_provenance")
    if set(provenance) != {"kind", "immutable_reference"}:
        fail(f"{name} pin_provenance must contain exactly kind and immutable_reference")
    kind = provenance["kind"]
    reference = provenance["immutable_reference"]
    if kind not in ALLOWED_PROVENANCE_KINDS:
        fail(f"{name} has unsupported pin provenance kind {kind!r}")
    if not isinstance(reference, str) or not reference.strip():
        fail(f"{name} immutable_reference must be a non-empty string")
    if kind in {"git_blob", "git_commit"}:
        ref = reference.lower()
        if len(ref) != 40 or any(ch not in "0123456789abcdef" for ch in ref):
            fail(f"{name} {kind} reference must be an exact 40-hex Git object id")
    elif kind == "sha256":
        ref = reference.lower()
        if len(ref) != 64 or any(ch not in "0123456789abcdef" for ch in ref):
            fail(f"{name} sha256 reference must be an exact 64-hex digest")


def main() -> None:
    data = json.loads(CONTRACT.read_text(encoding="utf-8"))
    prearm = PREARM.read_text(encoding="utf-8")

    if data.get("schema") != "tdi11.2-model-observation-freeze-v2":
        fail("unexpected schema")
    if data.get("stage") != "TDI-11.2":
        fail("unexpected stage")

    if any(key in data for key in FORBIDDEN_TOP_LEVEL):
        fail("final/confirmatory material is forbidden in this contract")

    for gate in (
        "model_execution_authorized",
        "final_execution_authorized",
        "final_seed_material_exists",
        "final_dataset_exists",
        "final_runner_exists",
        "confirmatory_result_exists",
    ):
        if data.get(gate) is not False:
            fail(f"{gate} must remain false in the non-executing freeze")

    if data.get("allowed_domains") != EXPECTED_DOMAINS:
        fail("only Development and Validation are permitted")
    if data.get("forbidden_inputs") != EXPECTED_FORBIDDEN_INPUTS:
        fail("forbidden input contract drifted from the TDI-11.2 pre-arm")
    if data.get("forbidden_surfaces") != EXPECTED_FORBIDDEN_SURFACES:
        fail("forbidden surface contract drifted from the TDI-11.2 pre-arm")

    fields = data.get("fields")
    if not isinstance(fields, dict) or set(fields) != EXPECTED_FIELDS:
        fail("field set must exactly match the pre-arm unresolved freeze requirements")

    all_pinned = True
    for name in sorted(EXPECTED_FIELDS):
        entry = fields[name]
        if not isinstance(entry, dict) or set(entry) != {"status", "value", "pin_provenance"}:
            fail(f"{name} must contain exactly status, value and pin_provenance")
        status = entry["status"]
        value = entry["value"]
        provenance = entry["pin_provenance"]
        if status not in ALLOWED_FIELD_STATUS:
            fail(f"{name} has unsupported status {status!r}")
        if status == "unresolved_blocking":
            all_pinned = False
            if value is not None:
                fail(f"{name} unresolved value must be null")
            if provenance is not None:
                fail(f"{name} unresolved pin_provenance must be null")
        else:
            if value is None or (isinstance(value, str) and not value.strip()):
                fail(f"{name} pinned value must be explicit and non-empty")
            validate_pin_provenance(name, provenance)

    expected_scientific_status = "frozen_nonfinal" if all_pinned else "unresolved_blocking"
    if data.get("scientific_status") != expected_scientific_status:
        fail(
            "scientific_status must be frozen_nonfinal iff every required field is pinned"
        )

    # Cross-check the source pre-arm text without needing a YAML dependency.
    for field in sorted(EXPECTED_FIELDS):
        needle = f"  {field}: unresolved_blocking"
        if needle not in prearm:
            fail(f"pre-arm no longer declares required field {field}")
    for item in EXPECTED_FORBIDDEN_INPUTS:
        if f"  - {item}" not in prearm:
            fail(f"pre-arm no longer forbids input {item}")
    for item in EXPECTED_FORBIDDEN_SURFACES:
        if f"  - {item}" not in prearm:
            fail(f"pre-arm no longer forbids surface {item}")

    print(
        "TDI-11.2 model/observation freeze OK: "
        f"status={expected_scientific_status}; model execution remains unauthorized"
    )


if __name__ == "__main__":
    main()
