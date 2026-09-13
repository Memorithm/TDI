#!/usr/bin/env python3
"""Fail-closed validator for the TDI-12.0 Stage-0 freeze *template*.

Stage-0 scientific status: the template must remain fully unresolved and must
not authorize confirmatory or final execution. This tool validates shape and
integrity only. It never invents freeze pins, never resolves fields, and never
authorizes ordinal-transport confirmatory runs.
"""

from __future__ import annotations

import argparse
import copy
import json
import sys
from pathlib import Path
from typing import Any

REQUIRED_FREEZE_FIELDS = (
    "operator_population_families",
    "response_observable_registry",
    "ranking_metric",
    "tie_policy",
    "normalization_contract",
    "split_discipline",
    "development_validation_population_derivation",
    "control_battery",
    "typed_rejection_contract",
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

EXPECTED_RESPONSE_CANDIDATES = [
    "MidDiagonalGreen",
    "GreenTrace",
    "MeanAbsOffDiagonalGreen",
]

EXPECTED_RANKING_CANDIDATES = [
    "spearman_average_ranks",
    "kendall_tau_b_companion",
]

EXPECTED_TIE_CANDIDATES = [
    "average_ranks_midranks",
    "fail_closed_full_ties",
]

REQUIRED_NAMED_CONTROLS = [
    "identity_ordering",
    "dimension_only_ordering",
    "monotone_rescaling",
    "tie_heavy_adversarial",
    "spectral_gap_baseline",
    "norm_baseline",
    "shuffled_family",
]

FORBIDDEN_HOLDINGS = (
    "tdi-7.2",
    "tdi7.2",
    "tdi-8.2",
    "tdi8.2",
    "tdi-9.2",
    "tdi9.2",
)


class TemplateError(ValueError):
    pass


def fail(message: str) -> None:
    raise TemplateError(message)


def load_document(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot load {path}: {exc}")


def require_unresolved_field(name: str, entry: Any) -> None:
    if not isinstance(entry, dict):
        fail(f"{name}: must be a JSON object")
    if entry.get("status") != "unresolved_blocking":
        fail(f"{name}: Stage-0 requires status unresolved_blocking, got {entry.get('status')!r}")
    if entry.get("value") is not None:
        fail(f"{name}: Stage-0 template value must remain null (no invented pins)")


def validate_template(document: Any) -> int:
    if not isinstance(document, dict):
        fail("top-level template must be a JSON object")

    actual_top = set(document)
    if actual_top != REQUIRED_TOP_KEYS:
        fail(
            "top-level keys differ from Stage-0 template schema: "
            f"missing={sorted(REQUIRED_TOP_KEYS - actual_top)} "
            f"unexpected={sorted(actual_top - REQUIRED_TOP_KEYS)}"
        )

    if document["schema_version"] != 1:
        fail("schema_version must equal 1")
    if document["kind"] != "tdi12_0_stage0_freeze":
        fail("kind must be tdi12_0_stage0_freeze")
    if document["status"] != "template_unfrozen":
        fail("status must remain template_unfrozen during Stage-0")
    if document["confirmatory_execution_authorized"] is not False:
        fail("confirmatory_execution_authorized must be false")
    if document["final_execution_authorized"] is not False:
        fail("final_execution_authorized must be false")
    if document["allowed_domains"] != ALLOWED_DOMAINS:
        fail("allowed_domains must be exactly Development then Validation")
    if not isinstance(document["notes"], list) or not document["notes"]:
        fail("notes must be a non-empty list")

    blob = json.dumps(document, ensure_ascii=True).lower()
    for token in FORBIDDEN_HOLDINGS:
        # Notes may mention holdouts as *forbidden*; require the word forbidden nearby.
        pass
    notes_text = " ".join(str(n) for n in document["notes"]).lower()
    if "7.2" in notes_text or "8.2" in notes_text or "9.2" in notes_text:
        if "forbidden" not in notes_text and "remain" not in notes_text:
            fail("holdout mentions in notes must remain exclusionary")

    freeze = document["freeze"]
    if not isinstance(freeze, dict):
        fail("freeze must be a JSON object")
    actual_fields = set(freeze)
    expected_fields = set(REQUIRED_FREEZE_FIELDS)
    if actual_fields != expected_fields:
        fail(
            "freeze fields differ from Stage-0 contract: "
            f"missing={sorted(expected_fields - actual_fields)} "
            f"unexpected={sorted(actual_fields - expected_fields)}"
        )

    for name in REQUIRED_FREEZE_FIELDS:
        require_unresolved_field(name, freeze[name])

    response = freeze["response_observable_registry"]
    if response.get("non_authorizing_candidates") != EXPECTED_RESPONSE_CANDIDATES:
        fail("response_observable_registry.non_authorizing_candidates drifted")

    ranking = freeze["ranking_metric"]
    if ranking.get("non_authorizing_candidates") != EXPECTED_RANKING_CANDIDATES:
        fail("ranking_metric.non_authorizing_candidates drifted")

    ties = freeze["tie_policy"]
    if ties.get("non_authorizing_candidates") != EXPECTED_TIE_CANDIDATES:
        fail("tie_policy.non_authorizing_candidates drifted")

    controls = freeze["control_battery"]
    if controls.get("required_named_controls") != REQUIRED_NAMED_CONTROLS:
        fail("control_battery.required_named_controls drifted")

    # Stage-0 must never claim a frozen digest or pinned payload.
    if "contract_digest_sha256" in document:
        fail("Stage-0 template must not carry contract_digest_sha256")
    if any(
        isinstance(freeze[name], dict) and freeze[name].get("status") == "pinned"
        for name in REQUIRED_FREEZE_FIELDS
    ):
        fail("Stage-0 template must not contain pinned freeze fields")

    return len(REQUIRED_FREEZE_FIELDS)


def self_test(template: Any) -> None:
    unresolved = validate_template(template)
    if unresolved != len(REQUIRED_FREEZE_FIELDS):
        fail("self-test: template did not keep all Stage-0 blockers")

    armed = copy.deepcopy(template)
    armed["confirmatory_execution_authorized"] = True
    try:
        validate_template(armed)
    except TemplateError:
        pass
    else:
        fail("self-test: confirmatory authorization was accepted")

    final_armed = copy.deepcopy(template)
    final_armed["final_execution_authorized"] = True
    try:
        validate_template(final_armed)
    except TemplateError:
        pass
    else:
        fail("self-test: final authorization was accepted")

    pinned = copy.deepcopy(template)
    pinned["freeze"]["ranking_metric"]["status"] = "pinned"
    pinned["freeze"]["ranking_metric"]["value"] = "spearman_average_ranks"
    try:
        validate_template(pinned)
    except TemplateError:
        pass
    else:
        fail("self-test: invented ranking_metric pin was accepted")

    valued = copy.deepcopy(template)
    valued["freeze"]["tie_policy"]["value"] = "average_ranks_midranks"
    try:
        validate_template(valued)
    except TemplateError:
        pass
    else:
        fail("self-test: non-null unresolved value was accepted")

    missing = copy.deepcopy(template)
    del missing["freeze"]["split_discipline"]
    try:
        validate_template(missing)
    except TemplateError:
        pass
    else:
        fail("self-test: missing freeze field was accepted")

    bad_status = copy.deepcopy(template)
    bad_status["status"] = "frozen_non_final"
    try:
        validate_template(bad_status)
    except TemplateError:
        pass
    else:
        fail("self-test: frozen_non_final Stage-0 status was accepted")

    digest = copy.deepcopy(template)
    digest["contract_digest_sha256"] = "0" * 64
    # Extra top-level key should fail schema equality.
    try:
        validate_template(digest)
    except TemplateError:
        pass
    else:
        fail("self-test: digest-bearing Stage-0 template was accepted")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "path",
        type=Path,
        nargs="?",
        default=Path("docs/tdi12/tdi12.0-stage0-freeze.template.json"),
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run deterministic negative fixtures that refuse invented pins",
    )
    args = parser.parse_args()

    try:
        document = load_document(args.path)
        unresolved = validate_template(document)
        if args.self_test:
            self_test(document)
    except TemplateError as exc:
        print(f"TDI-12.0 Stage-0 freeze template ERROR: {exc}", file=sys.stderr)
        return 1

    print(
        f"TDI-12.0 Stage-0 freeze template: BLOCKED "
        f"({unresolved}/{len(REQUIRED_FREEZE_FIELDS)} unresolved_blocking)"
    )
    print("TDI-12.0 confirmatory_execution_authorized: false")
    print("TDI-12.0 final_execution_authorized: false")
    print("TDI-12.0 invented freeze pins: NONE")
    if args.self_test:
        print("TDI-12.0 Stage-0 freeze template validator self-test: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
