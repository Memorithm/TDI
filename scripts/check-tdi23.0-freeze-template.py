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
    """Require one Stage-0 freeze field to remain unresolved and unpinned."""
    if not isinstance(entry, dict):
        fail(f"{name}: must be a JSON object")
    if entry.get("status") != "unresolved_blocking":
        fail(f"{name}: status must remain unresolved_blocking")
    if entry.get("value") is not None:
        fail(f"{name}: value must remain null during Stage 0")


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
    if not isinstance(document["notes"], list) or not document["notes"]:
        fail("notes must be a non-empty list")

    freeze = document["freeze"]
    if not isinstance(freeze, dict):
        fail("freeze must be a JSON object")
    if set(freeze) != set(REQUIRED_FREEZE_FIELDS):
        fail("freeze fields differ from the TDI-23.0 Stage-0 contract")

    for name in REQUIRED_FREEZE_FIELDS:
        require_unresolved_field(name, freeze[name])

    if freeze["nonlinear_boundary"].get("required_named_boundaries") != REQUIRED_BOUNDARIES:
        fail("nonlinear_boundary.required_named_boundaries drifted")
    if (
        freeze["global_reduction_preservation"].get("required_named_diagnostics")
        != REQUIRED_REDUCTION_DIAGNOSTICS
    ):
        fail("global_reduction_preservation.required_named_diagnostics drifted")
    if freeze["control_battery"].get("required_named_controls") != REQUIRED_CONTROLS:
        fail("control_battery.required_named_controls drifted")

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


def self_test(template: Any) -> None:
    """Run deterministic negative fixtures proving the validator fails closed."""
    validate_template(template)

    armed = copy.deepcopy(template)
    armed["confirmatory_execution_authorized"] = True
    try:
        validate_template(armed)
    except TemplateError:
        pass
    else:
        fail("self-test: confirmatory authorization was accepted")

    pinned = copy.deepcopy(template)
    pinned["freeze"]["dagger_semantics"]["status"] = "pinned"
    pinned["freeze"]["dagger_semantics"]["value"] = "transpose"
    try:
        validate_template(pinned)
    except TemplateError:
        pass
    else:
        fail("self-test: invented dagger pin was accepted")

    reduction_pinned = copy.deepcopy(template)
    reduction_pinned["freeze"]["global_reduction_family"]["status"] = "pinned"
    reduction_pinned["freeze"]["global_reduction_family"]["value"] = "coordinate_subspace"
    try:
        validate_template(reduction_pinned)
    except TemplateError:
        pass
    else:
        fail("self-test: invented global-reduction pin was accepted")

    reduction_softened = copy.deepcopy(template)
    reduction_softened["freeze"]["global_reduction_preservation"][
        "required_named_diagnostics"
    ] = []
    try:
        validate_template(reduction_softened)
    except TemplateError:
        pass
    else:
        fail("self-test: global-reduction diagnostics removal was accepted")

    softened = copy.deepcopy(template)
    softened["freeze"]["nonlinear_boundary"]["required_named_boundaries"] = []
    try:
        validate_template(softened)
    except TemplateError:
        pass
    else:
        fail("self-test: nonlinear boundary removal was accepted")

    missing = copy.deepcopy(template)
    del missing["freeze"]["provenance_contract"]
    try:
        validate_template(missing)
    except TemplateError:
        pass
    else:
        fail("self-test: missing provenance field was accepted")


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
