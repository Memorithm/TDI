#!/usr/bin/env python3
"""Fail-closed validator for the non-executing TDI-11.2 freeze contract.

This tool validates contract shape and integrity only. It never selects, loads,
or executes a model and it never authorizes final evaluation.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

REQUIRED_FIELDS = (
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
)

ALLOWED_DOMAINS = ["Development", "Validation"]
ALLOWED_FIELD_STATUSES = {"unresolved_blocking", "pinned"}


class ContractError(ValueError):
    pass


def fail(message: str) -> None:
    raise ContractError(message)


def canonical_freeze_bytes(freeze: dict[str, Any]) -> bytes:
    return json.dumps(
        freeze,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
        allow_nan=False,
    ).encode("utf-8")


def freeze_digest(freeze: dict[str, Any]) -> str:
    return hashlib.sha256(canonical_freeze_bytes(freeze)).hexdigest()


def is_nonempty_pinned_value(value: Any) -> bool:
    if value is None or isinstance(value, bool):
        return False
    if isinstance(value, str):
        return bool(value.strip())
    if isinstance(value, (list, dict)):
        return bool(value)
    if isinstance(value, (int, float)):
        return True
    return False


def validate_contract(document: Any, *, require_complete: bool) -> tuple[int, str | None]:
    if not isinstance(document, dict):
        fail("top-level contract must be a JSON object")

    expected_top = {
        "schema_version",
        "kind",
        "status",
        "model_execution_authorized",
        "final_execution_authorized",
        "allowed_domains",
        "contract_digest_sha256",
        "freeze",
    }
    actual_top = set(document)
    if actual_top != expected_top:
        fail(
            "top-level keys differ from the frozen schema: "
            f"missing={sorted(expected_top - actual_top)} "
            f"unexpected={sorted(actual_top - expected_top)}"
        )

    if document["schema_version"] != 1:
        fail("schema_version must equal 1")
    if document["kind"] != "tdi11_2_model_observation_freeze":
        fail("unexpected contract kind")
    if document["allowed_domains"] != ALLOWED_DOMAINS:
        fail("allowed_domains must be exactly Development then Validation")
    if document["final_execution_authorized"] is not False:
        fail("final execution must remain unauthorized")
    if document["model_execution_authorized"] is not False:
        fail("this freeze contract cannot itself authorize model execution")

    freeze = document["freeze"]
    if not isinstance(freeze, dict):
        fail("freeze must be a JSON object")
    expected_fields = set(REQUIRED_FIELDS)
    actual_fields = set(freeze)
    if actual_fields != expected_fields:
        fail(
            "freeze fields differ from the 12-field pre-arm contract: "
            f"missing={sorted(expected_fields - actual_fields)} "
            f"unexpected={sorted(actual_fields - expected_fields)}"
        )

    unresolved = 0
    for name in REQUIRED_FIELDS:
        entry = freeze[name]
        if not isinstance(entry, dict) or set(entry) != {"status", "value"}:
            fail(f"{name} must contain exactly status and value")
        status = entry["status"]
        if status not in ALLOWED_FIELD_STATUSES:
            fail(f"{name} has unsupported status {status!r}")
        if status == "unresolved_blocking":
            unresolved += 1
            if entry["value"] is not None:
                fail(f"{name} is unresolved but carries a value")
        elif not is_nonempty_pinned_value(entry["value"]):
            fail(f"{name} is pinned without a non-empty value")

    digest = document["contract_digest_sha256"]
    if unresolved:
        if require_complete:
            fail(f"contract remains blocked by {unresolved} unresolved field(s)")
        if document["status"] != "template_unfrozen":
            fail("an incomplete contract must have status template_unfrozen")
        if digest is not None:
            fail("an incomplete contract must not claim a frozen digest")
        return unresolved, None

    if document["status"] != "frozen_non_final":
        fail("a complete contract must have status frozen_non_final")
    expected_digest = freeze_digest(freeze)
    if digest != expected_digest:
        fail("contract_digest_sha256 does not match canonical freeze payload")
    return 0, expected_digest


def load_document(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot load {path}: {exc}")


def self_test(template: Any) -> None:
    unresolved, digest = validate_contract(template, require_complete=False)
    if unresolved != len(REQUIRED_FIELDS) or digest is not None:
        fail("template self-test did not preserve all blockers")

    armed = copy.deepcopy(template)
    armed["model_execution_authorized"] = True
    try:
        validate_contract(armed, require_complete=False)
    except ContractError:
        pass
    else:
        fail("self-test: armed pre-freeze contract was accepted")

    missing = copy.deepcopy(template)
    del missing["freeze"][REQUIRED_FIELDS[0]]
    try:
        validate_contract(missing, require_complete=False)
    except ContractError:
        pass
    else:
        fail("self-test: missing required field was accepted")

    bad_pin = copy.deepcopy(template)
    bad_pin["freeze"][REQUIRED_FIELDS[0]]["status"] = "pinned"
    try:
        validate_contract(bad_pin, require_complete=False)
    except ContractError:
        pass
    else:
        fail("self-test: empty pinned value was accepted")

    complete = copy.deepcopy(template)
    complete["status"] = "frozen_non_final"
    for name in REQUIRED_FIELDS:
        complete["freeze"][name] = {
            "status": "pinned",
            "value": {"fixture": name},
        }
    complete["contract_digest_sha256"] = freeze_digest(complete["freeze"])
    unresolved, digest = validate_contract(complete, require_complete=True)
    if unresolved != 0 or digest != complete["contract_digest_sha256"]:
        fail("self-test: complete synthetic contract failed integrity validation")

    tampered = copy.deepcopy(complete)
    tampered["freeze"][REQUIRED_FIELDS[0]]["value"] = {"fixture": "tampered"}
    try:
        validate_contract(tampered, require_complete=True)
    except ContractError:
        pass
    else:
        fail("self-test: digest mismatch was accepted")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", type=Path)
    parser.add_argument(
        "--require-complete",
        action="store_true",
        help="require all 12 fields to be pinned and digest-bound",
    )
    parser.add_argument(
        "--self-test",
        action="store_true",
        help="run deterministic negative and positive validator fixtures",
    )
    args = parser.parse_args()

    try:
        document = load_document(args.path)
        unresolved, digest = validate_contract(
            document, require_complete=args.require_complete
        )
        if args.self_test:
            self_test(document)
    except ContractError as exc:
        print(f"TDI-11.2 freeze schema ERROR: {exc}", file=sys.stderr)
        return 1

    if unresolved:
        print(f"TDI-11.2 freeze contract: BLOCKED ({unresolved}/12 unresolved)")
    else:
        print(f"TDI-11.2 freeze contract: COMPLETE ({digest})")
    if args.self_test:
        print("TDI-11.2 freeze validator self-test: PASS")
    print("TDI-11.2 model execution authorization: NOT GRANTED BY THIS CONTRACT")
    print("TDI-11.2 final execution authorization: BLOCKED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
