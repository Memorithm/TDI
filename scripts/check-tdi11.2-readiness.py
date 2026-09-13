#!/usr/bin/env python3
"""Always-on TDI-11.2 readiness / fail-closed execution gate.

Fails if model execution is authorized while any freeze field remains
unresolved_blocking, if any field is invented as pinned, if a forbidden
surface appears, or if the content-addressed unresolved ledger drifts.

This gate never selects, loads, or executes a model.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path

DEFAULT_FREEZE = Path("docs/tdi11.2-model-observation-freeze.json")
DEFAULT_PREARM = Path("docs/tdi11.2-prearm.yaml")
DEFAULT_LEDGER = Path("docs/TDI-11.2-UNRESOLVED-LEDGER.md")
DEFAULT_DIGEST = Path("docs/tdi11.2-model-observation-freeze.sha256")
DEFAULT_STATUS = Path("docs/TDI-11.2-STATUS.md")

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

EXPECTED_FORBIDDEN_SURFACES = [
    "concrete_model_runner",
    "final_seed_list",
    "final_dataset",
    "final_runner",
    "final_result_payload",
]

HEX64 = re.compile(r"\b[0-9a-f]{64}\b")


class ReadinessError(ValueError):
    pass


def fail(message: str) -> None:
    raise ReadinessError(message)


def file_digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_readiness(
    freeze: dict,
    *,
    prearm_text: str,
    ledger_text: str,
    digest_text: str,
    status_text: str,
    freeze_digest: str,
) -> None:
    if freeze.get("schema") != "tdi11.2-model-observation-freeze-v2":
        fail("unexpected freeze schema")
    if freeze.get("stage") != "TDI-11.2":
        fail("unexpected freeze stage")

    fields = freeze.get("fields")
    if not isinstance(fields, dict) or set(fields) != set(REQUIRED_FIELDS):
        fail("freeze field set must remain the 12-field pre-arm registry")

    unresolved = []
    invented = []
    for name in REQUIRED_FIELDS:
        entry = fields[name]
        if not isinstance(entry, dict):
            fail(f"{name} must be an object")
        status = entry.get("status")
        value = entry.get("value")
        if status == "unresolved_blocking":
            if value is not None:
                fail(f"{name} is unresolved but carries a value")
            unresolved.append(name)
        elif status == "pinned":
            invented.append(name)
        else:
            fail(f"{name} has unsupported status {status!r}")

    if invented:
        fail(
            "invented TDI-11.2 pins are forbidden (no model/adapter/tokenizer "
            f"values have been reviewed): {', '.join(invented)}"
        )
    if len(unresolved) != 12:
        fail(f"expected 12 unresolved_blocking fields, found {len(unresolved)}")

    authorized = freeze.get("model_execution_authorized")
    final_authorized = freeze.get("final_execution_authorized")
    if authorized is True and unresolved:
        fail(
            "model_execution_authorized is true while unresolved_blocking "
            f"fields remain: {', '.join(unresolved)}"
        )
    if authorized is not False:
        fail("model_execution_authorized must remain false at this pre-arm stage")
    if final_authorized is not False:
        fail("final_execution_authorized must remain false")

    if freeze.get("scientific_status") != "unresolved_blocking":
        fail("scientific_status cannot be upgraded while fields remain unresolved")

    for flag in (
        "final_seed_material_exists",
        "final_dataset_exists",
        "final_runner_exists",
        "confirmatory_result_exists",
    ):
        if freeze.get(flag) is not False:
            fail(f"{flag} must remain false")

    if freeze.get("forbidden_surfaces") != EXPECTED_FORBIDDEN_SURFACES:
        fail("forbidden_surfaces drifted from the pre-arm contract")

    for field in REQUIRED_FIELDS:
        needle = f"  {field}: unresolved_blocking"
        if needle not in prearm_text:
            fail(f"pre-arm no longer declares required field {field}")
    if "model_execution_authorized: false" not in prearm_text:
        fail("pre-arm lost model_execution_authorized: false")
    if "model_execution_authorized: true" in prearm_text:
        fail("pre-arm armed model execution")
    if "final_execution_authorized: true" in prearm_text:
        fail("pre-arm armed final execution")

    if freeze_digest not in digest_text:
        fail("sha256 sidecar does not match the current unresolved freeze file")
    if freeze_digest not in ledger_text:
        fail("unresolved ledger does not content-address the current freeze file")
    if "NOT A PIN" not in ledger_text and "not a pin" not in ledger_text.lower():
        fail("unresolved ledger must declare it is not a pin / not a freeze")
    for field in REQUIRED_FIELDS:
        if field not in ledger_text:
            fail(f"unresolved ledger omitted field {field}")

    if "MODEL EXECUTION NOT AUTHORIZED" not in status_text:
        fail("STATUS lost the model-execution-not-authorized marker")
    if re.search(r"^- Status:.*\*\*(frozen|complete|authorized)", status_text, re.M):
        fail("STATUS silently upgraded while TDI-11.2 remains pre-arm")


def find_forbidden_surfaces(root: Path) -> list[str]:
    matches: list[str] = []
    search_roots = [
        root / "tdi-ai",
        root / "tdi-bench",
        root / "scripts",
        root / "docs",
        root / ".github" / "workflows",
    ]
    allowed_suffixes = {
        "scripts/check-tdi11.2-prearm.sh",
        "scripts/check-tdi11.2-readiness.sh",
        "scripts/check-tdi11.2-readiness.py",
        "scripts/check-tdi11.2-freeze-schema.py",
        "scripts/check-tdi11.2-model-observation-freeze.py",
        "scripts/test-tdi11.2-model-observation-freeze.py",
        "scripts/test-tdi11.2-readiness.py",
        "docs/TDI-11.2-STATUS.md",
        "docs/TDI-11.2-IMPLEMENTATION-GATE.md",
        "docs/TDI-11.2-PROSPECTIVE-INSTRUMENTATION-PREARM.md",
        "docs/TDI-11.2-UNRESOLVED-LEDGER.md",
        "docs/tdi11.2-prearm.yaml",
        "docs/tdi11.2-model-observation-freeze.json",
        "docs/tdi11.2-model-observation-freeze.template.json",
        "docs/tdi11.2-model-observation-freeze.sha256",
        ".github/workflows/tdi11.2-prearm.yml",
        ".github/workflows/tdi11.2-readiness.yml",
        ".github/workflows/tdi11-model-observation-freeze.yml",
    }
    patterns = (
        "tdi11.2",
        "tdi11_2",
    )
    surface_needles = ("runner", "final", "seed_list", "seed-list", "dataset")
    for base in search_roots:
        if not base.exists():
            continue
        for path in base.rglob("*"):
            if not path.is_file():
                continue
            rel = path.relative_to(root).as_posix()
            if rel in allowed_suffixes:
                continue
            name = path.name.lower()
            if not any(p in name for p in patterns):
                continue
            if any(n in name for n in surface_needles):
                matches.append(rel)
    return sorted(matches)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--freeze", type=Path, default=DEFAULT_FREEZE)
    parser.add_argument("--prearm", type=Path, default=DEFAULT_PREARM)
    parser.add_argument("--ledger", type=Path, default=DEFAULT_LEDGER)
    parser.add_argument("--digest", type=Path, default=DEFAULT_DIGEST)
    parser.add_argument("--status", type=Path, default=DEFAULT_STATUS)
    parser.add_argument(
        "--skip-surface-scan",
        action="store_true",
        help="skip repository forbidden-surface scan (unit tests only)",
    )
    args = parser.parse_args()

    try:
        freeze = json.loads(args.freeze.read_text(encoding="utf-8"))
        digest = file_digest(args.freeze)
        validate_readiness(
            freeze,
            prearm_text=args.prearm.read_text(encoding="utf-8"),
            ledger_text=args.ledger.read_text(encoding="utf-8"),
            digest_text=args.digest.read_text(encoding="utf-8"),
            status_text=args.status.read_text(encoding="utf-8"),
            freeze_digest=digest,
        )
        if not args.skip_surface_scan:
            forbidden = find_forbidden_surfaces(Path.cwd())
            if forbidden:
                fail("forbidden TDI-11.2 surfaces present: " + ", ".join(forbidden))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"TDI-11.2 readiness ERROR: cannot load inputs: {exc}", flush=True)
        return 1
    except ReadinessError as exc:
        print(f"TDI-11.2 readiness ERROR: {exc}", flush=True)
        return 1

    print(f"TDI-11.2 unresolved ledger digest: {digest}")
    print("TDI-11.2 unresolved fields: 12/12")
    print("TDI-11.2 invented pins: NONE")
    print("TDI-11.2 model_execution_authorized: false")
    print("TDI-11.2 forbidden surfaces: ABSENT")
    print("TDI-11.2 readiness integrity gate: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
