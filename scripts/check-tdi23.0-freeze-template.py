#!/usr/bin/env python3
"""Fail-closed validator for the TDI-23.0 Stage-0 freeze template."""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

REQUIRED_FREEZE_FIELDS = (
    "scalar_field_contract",
    "inner_product_contract",
    "linear_map_storage_contract",
    "dagger_semantics",
    "composition_semantics",
    "attention_score_mapping",
    "global_reduction_family",
    "global_reduction_preservation",
    "nonlinear_boundary",
    "numerical_tolerance_policy",
    "development_fixture_family",
    "control_battery",
    "split_discipline",
    "provenance_contract",
)

REQUIRED_TOP_KEYS = {
    "schema_version",
    "kind",
    "status",
    "confirmatory_execution_authorized",
    "final_execution_authorized",
    "allowed_domains",
    "notes",
    "freeze",
}

ALLOWED_DOMAINS = ["Development", "Validation"]
EXPECTED_NOTES = [
    "Stage-0 template only. Every scientific field stays unresolved_blocking.",
    "Merging this template does not freeze TDI-23.0 and does not authorize comparative or confirmatory attention experiments.",
    "Global-reduction candidates remain development-only and may not be promoted as functorial, lossless, or performance-improving without later evidence.",
    "FLAT-ATTENTION integration remains downstream and independently gated.",
]
REQUIRED_BOUNDARIES = [
    "softmax_external_to_fdhilb_linear_morphisms",
    "boolean_not_silently_fdhilb",
    "f2_not_silently_fdhilb",
    "anf_not_silently_fdhilb",
    "max_plus_not_silently_fdhilb",
]
REQUIRED_REDUCTION_DIAGNOSTICS = [
    "dagger_commutation",
    "reduce_lift_residual",
    "composition_defect",
    "full_middle_space_control",
    "dropped_middle_path_control",
]
REQUIRED_CONTROLS = [
    "identity_map",
    "zero_map",
    "rectangular_map",
    "noncommuting_composition",
    "dimension_mismatch_rejection",
    "nonfinite_rejection",
    "rewrite_disabled_reference",
    "no_reduction_reference",
    "dagger_reduction_commutation",
    "full_middle_space_composition_defect_zero",
    "dropped_middle_path_composition_defect_nonzero",
    "reduce_lift_residual",
]

# Every schema-approved metadata key also has exact Stage-0 contents. This is
# deliberately stricter than a JSON-shape check: no final seeds, selected policy,
# tolerance, hidden result material, or alternate candidate may be smuggled into
# an otherwise approved key while the field still reports unresolved_blocking.
EXPECTED_FIELD_METADATA: dict[str, dict[str, Any]] = {
    "scalar_field_contract": {
        "non_authorizing_candidates": ["real_f64_stage0", "complex_f64_future_only"],
    },
    "inner_product_contract": {
        "non_authorizing_candidates": ["standard_euclidean_real"],
    },
    "linear_map_storage_contract": {
        "non_authorizing_candidates": ["row_major_dense"],
    },
    "dagger_semantics": {
        "non_authorizing_candidates": [
            "real_matrix_transpose_in_declared_orthonormal_bases"
        ],
    },
    "composition_semantics": {
        "non_authorizing_candidates": [
            "left_after_right_typed_matrix_composition"
        ],
    },
    "attention_score_mapping": {
        "non_authorizing_candidates": [
            "key_dagger_compose_query_ket",
            "batched_q_times_k_transpose_future",
        ],
    },
    "global_reduction_family": {
        "non_authorizing_candidates": [
            "objectwise_coordinate_subspace_stage0",
            "arbitrary_orthonormal_subspace_future_only",
            "learned_subspace_future_only",
        ],
    },
    "global_reduction_preservation": {
        "required_named_diagnostics": REQUIRED_REDUCTION_DIAGNOSTICS,
        "non_authorizing_candidates": [
            "coordinate_reduction_exact_dagger_commutation",
            "composition_not_assumed_preserved",
        ],
    },
    "nonlinear_boundary": {
        "required_named_boundaries": REQUIRED_BOUNDARIES,
    },
    "numerical_tolerance_policy": {
        "non_authorizing_candidates": [
            "exact_structural_checks_plus_declared_scaled_f64_tolerance"
        ],
    },
    "development_fixture_family": {
        "non_authorizing_candidates": [
            "small_integer_dense_maps",
            "basis_and_small_integer_query_key_kets",
            "small_coordinate_reduction_fixtures",
        ],
    },
    "control_battery": {
        "required_named_controls": REQUIRED_CONTROLS,
    },
    "split_discipline": {
        "non_authorizing_candidates": [
            "development_and_validation_only_no_final_surface"
        ],
    },
    "provenance_contract": {
        "non_authorizing_candidates": ["stage0_contract_version_plus_git_sha"],
    },
}


class TemplateError(ValueError):
    """Raised when the Stage-0 template violates its fail-closed contract."""


def fail(message: str) -> None:
    """Abort validation with a typed template error."""
    raise TemplateError(message)


def load_document(path: Path) -> Any:
    """Load one UTF-8 JSON template from disk."""
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot load {path}: {exc}")


def require_unresolved_field(name: str, entry: Any) -> None:
    """Require one Stage-0 freeze field to match its exact unpinned schema."""
    if not isinstance(entry, dict):
        fail(f"{name}: must be a JSON object")

    expected_metadata = EXPECTED_FIELD_METADATA[name]
    expected_keys = {"status", "value", *expected_metadata}
    if set(entry) != expected_keys:
        unexpected = sorted(set(entry) - expected_keys)
        missing = sorted(expected_keys - set(entry))
        fail(f"{name}: field keys drifted; unexpected={unexpected}, missing={missing}")
    if entry.get("status") != "unresolved_blocking":
        fail(f"{name}: status must remain unresolved_blocking")
    if entry.get("value") is not None:
        fail(f"{name}: value must remain null during Stage 0")

    for key, expected in expected_metadata.items():
        if entry.get(key) != expected:
            fail(f"{name}.{key}: contents drifted from the Stage-0 schema")


def validate_template(document: Any) -> int:
    """Validate schema, unresolved fields, boundaries, controls, and execution gates."""
    if not isinstance(document, dict):
        fail("top-level template must be a JSON object")
    if set(document) != REQUIRED_TOP_KEYS:
        fail("top-level keys differ from the TDI-23.0 Stage-0 schema")
    if document["schema_version"] != 1:
        fail("schema_version must equal 1")
    if document["kind"] != "tdi23_0_stage0_freeze":
        fail("kind must be tdi23_0_stage0_freeze")
    if document["status"] != "template_unfrozen":
        fail("status must remain template_unfrozen")
    if document["confirmatory_execution_authorized"] is not False:
        fail("confirmatory_execution_authorized must remain false")
    if document["final_execution_authorized"] is not False:
        fail("final_execution_authorized must remain false")
    if document["allowed_domains"] != ALLOWED_DOMAINS:
        fail("allowed_domains must be exactly Development then Validation")
    if document["notes"] != EXPECTED_NOTES:
        fail("notes drifted from the exact Stage-0 non-authorizing text")

    freeze = document["freeze"]
    if not isinstance(freeze, dict):
        fail("freeze must be a JSON object")
    if set(freeze) != set(REQUIRED_FREEZE_FIELDS):
        fail("freeze fields differ from the TDI-23.0 Stage-0 contract")

    for name in REQUIRED_FREEZE_FIELDS:
        require_unresolved_field(name, freeze[name])

    serialized = json.dumps(document, sort_keys=True).lower()
    forbidden_positive_claims = (
        "softmax_is_fdhilb_morphism",
        "confirmatory_execution_authorized_true",
        "final_execution_authorized_true",
        "flat_attention_integration_authorized",
        "global_reduction_is_lossless",
        "global_reduction_is_functorial",
        "global_reduction_improves_performance",
    )
    for token in forbidden_positive_claims:
        if token in serialized:
            fail(f"forbidden Stage-0 claim present: {token}")

    return len(REQUIRED_FREEZE_FIELDS)


def expect_rejected(document: Any, label: str) -> None:
    """Require one mutated self-test fixture to be rejected."""
    try:
        validate_template(document)
    except TemplateError:
        return
    fail(f"self-test: {label} was accepted")


def self_test(template: Any) -> None:
    """Run deterministic negative fixtures proving the validator fails closed."""
    validate_template(template)

    armed = copy.deepcopy(template)
    armed["confirmatory_execution_authorized"] = True
    expect_rejected(armed, "confirmatory authorization")

    pinned = copy.deepcopy(template)
    pinned["freeze"]["dagger_semantics"]["status"] = "pinned"
    pinned["freeze"]["dagger_semantics"]["value"] = "transpose"
    expect_rejected(pinned, "invented dagger pin")

    hidden_policy = copy.deepcopy(template)
    hidden_policy["freeze"]["global_reduction_family"]["selected_policy"] = (
        "learned_subspace"
    )
    expect_rejected(hidden_policy, "undeclared selected_policy key")

    hidden_final_material = copy.deepcopy(template)
    hidden_final_material["freeze"]["development_fixture_family"]["final_seed_list"] = [
        1,
        2,
        3,
    ]
    expect_rejected(hidden_final_material, "undeclared final_seed_list key")

    nested_final_material = copy.deepcopy(template)
    nested_final_material["freeze"]["development_fixture_family"][
        "non_authorizing_candidates"
    ] = {"final_seed_list": [1, 2, 3]}
    expect_rejected(nested_final_material, "final material hidden in an allowed metadata key")

    altered_candidate = copy.deepcopy(template)
    altered_candidate["freeze"]["global_reduction_family"][
        "non_authorizing_candidates"
    ].append("selected_secret_policy")
    expect_rejected(altered_candidate, "extra candidate hidden in an allowed metadata key")

    altered_notes = copy.deepcopy(template)
    altered_notes["notes"].append("final_seed_list=1,2,3")
    expect_rejected(altered_notes, "scientific material hidden in notes")

    reduction_pinned = copy.deepcopy(template)
    reduction_pinned["freeze"]["global_reduction_family"]["status"] = "pinned"
    reduction_pinned["freeze"]["global_reduction_family"]["value"] = (
        "coordinate_subspace"
    )
    expect_rejected(reduction_pinned, "invented global-reduction pin")

    reduction_softened = copy.deepcopy(template)
    reduction_softened["freeze"]["global_reduction_preservation"][
        "required_named_diagnostics"
    ] = []
    expect_rejected(reduction_softened, "global-reduction diagnostics removal")

    softened = copy.deepcopy(template)
    softened["freeze"]["nonlinear_boundary"]["required_named_boundaries"] = []
    expect_rejected(softened, "nonlinear boundary removal")

    missing = copy.deepcopy(template)
    del missing["freeze"]["provenance_contract"]
    expect_rejected(missing, "missing provenance field")


def main() -> int:
    """CLI entry point for validation and optional negative self-tests."""
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "path",
        type=Path,
        nargs="?",
        default=Path("docs/tdi23/tdi23.0-stage0-freeze.template.json"),
    )
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()

    try:
        document = load_document(args.path)
        unresolved = validate_template(document)
        if args.self_test:
            self_test(document)
    except TemplateError as exc:
        print(f"TDI-23.0 Stage-0 freeze template ERROR: {exc}", file=sys.stderr)
        return 1

    print(
        f"TDI-23.0 Stage-0 freeze template: BLOCKED "
        f"({unresolved}/{len(REQUIRED_FREEZE_FIELDS)} unresolved_blocking)"
    )
    print("TDI-23.0 confirmatory_execution_authorized: false")
    print("TDI-23.0 final_execution_authorized: false")
    if args.self_test:
        print("TDI-23.0 Stage-0 freeze template validator self-test: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
