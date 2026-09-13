#!/usr/bin/env python3
"""Emit / verify the machine-readable TDI freeze-progress summary artifact.

Reads the 8.1 / 9.1 / 11.2 freeze contracts and blocker evidence-class
inventories, verifies status agreement, verifies the TDI-11.2 ledger digest,
and writes (or checks) docs/tdi-freeze-progress-summary.json.

This script never invents scientific freeze values and never arms execution.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SUMMARY_PATH = ROOT / "docs" / "tdi-freeze-progress-summary.json"
SCHEMA = "tdi-freeze-progress-summary-v1"

SERIES = (
    {
        "id": "tdi8.1",
        "stage": "TDI-8.1",
        "freeze": ROOT / "docs" / "tdi8.1-configuration-freeze.json",
        "inventory": ROOT / "docs" / "tdi8.1-blocker-evidence-classes.json",
        "status_doc": "docs/TDI-8.1-STATUS.md",
        "plan_doc": "docs/TDI-8.1-FREEZE-RESOLUTION-PLAN.md",
        "execution_flag": "tdi8_2_execution_authorized",
        "pin_floor": 3,
    },
    {
        "id": "tdi9.1",
        "stage": "TDI-9.1",
        "freeze": ROOT / "docs" / "tdi9.1-configuration-freeze.json",
        "inventory": ROOT / "docs" / "tdi9.1-blocker-evidence-classes.json",
        "status_doc": "docs/TDI-9.1-STATUS.md",
        "plan_doc": "docs/TDI-9.1-FREEZE-RESOLUTION-PLAN.md",
        "execution_flag": "tdi9_2_execution_authorized",
        "pin_floor": 1,
    },
    {
        "id": "tdi11.2",
        "stage": "TDI-11.2",
        "freeze": ROOT / "docs" / "tdi11.2-model-observation-freeze.json",
        "inventory": ROOT / "docs" / "tdi11.2-blocker-evidence-classes.json",
        "status_doc": "docs/TDI-11.2-STATUS.md",
        "plan_doc": "docs/TDI-11.2-UNRESOLVED-LEDGER.md",
        "execution_flag": "model_execution_authorized",
        "pin_floor": 0,
        "extra_execution_flags": ("final_execution_authorized",),
        "ledger": ROOT / "docs" / "TDI-11.2-UNRESOLVED-LEDGER.md",
        "digest_sidecar": ROOT / "docs" / "tdi11.2-model-observation-freeze.sha256",
    },
)


class SummaryError(ValueError):
    pass


def fail(message: str) -> None:
    raise SummaryError(message)


def load_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"cannot load {path}: {exc}")
    if not isinstance(data, dict):
        fail(f"{path} must contain a JSON object")
    return data


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def series_block(cfg: dict[str, Any]) -> dict[str, Any]:
    freeze = load_json(cfg["freeze"])
    inventory = load_json(cfg["inventory"])
    freeze_fields = freeze.get("fields")
    inv_fields = inventory.get("fields")
    if not isinstance(freeze_fields, dict) or not isinstance(inv_fields, dict):
        fail(f"{cfg['id']}: freeze/inventory fields must be objects")
    if set(freeze_fields) != set(inv_fields):
        missing = sorted(set(freeze_fields) - set(inv_fields))
        extra = sorted(set(inv_fields) - set(freeze_fields))
        fail(
            f"{cfg['id']}: inventory field set disagrees with freeze "
            f"(missing={missing}, extra={extra})"
        )
    if inventory.get("stage") != cfg["stage"]:
        fail(f"{cfg['id']}: inventory stage mismatch")

    pinned: list[str] = []
    unresolved: list[str] = []
    blockers: list[dict[str, Any]] = []
    pins: list[dict[str, Any]] = []

    for name in sorted(freeze_fields):
        frec = freeze_fields[name]
        irec = inv_fields[name]
        if not isinstance(frec, dict) or not isinstance(irec, dict):
            fail(f"{cfg['id']}.{name}: records must be objects")
        fstatus = frec.get("status")
        istatus = irec.get("status")
        if fstatus != istatus:
            fail(
                f"{cfg['id']}.{name}: inventory status {istatus!r} disagrees "
                f"with freeze status {fstatus!r}"
            )
        evidence_class = irec.get("required_evidence_class")
        if not isinstance(evidence_class, str) or not evidence_class.strip():
            fail(f"{cfg['id']}.{name}: missing required_evidence_class")
        definition = irec.get("evidence_class_definition")
        if not isinstance(definition, str) or not definition.strip():
            fail(f"{cfg['id']}.{name}: missing evidence_class_definition")

        if fstatus == "pinned":
            pinned.append(name)
            pins.append(
                {
                    "field": name,
                    "required_evidence_class": evidence_class,
                    "citations": irec.get("citations") or [],
                }
            )
        elif fstatus == "unresolved_blocking":
            unresolved.append(name)
            blockers.append(
                {
                    "field": name,
                    "required_evidence_class": evidence_class,
                    "blocking_until": irec.get("blocking_until"),
                    "non_authorizing_note": irec.get("non_authorizing_note"),
                }
            )
        else:
            fail(f"{cfg['id']}.{name}: unsupported status {fstatus!r}")

    total = len(freeze_fields)
    pinned_count = len(pinned)
    if pinned_count < cfg["pin_floor"]:
        fail(
            f"{cfg['id']}: pinned-count floor is {cfg['pin_floor']}; "
            f"found {pinned_count}"
        )

    exec_flag = cfg["execution_flag"]
    if freeze.get(exec_flag) is not False:
        fail(f"{cfg['id']}: {exec_flag} must remain false")
    for extra in cfg.get("extra_execution_flags", ()):
        if freeze.get(extra) is not False:
            fail(f"{cfg['id']}: {extra} must remain false")

    scientific_status = freeze.get("scientific_status")
    expected_scientific = (
        "unresolved_blocking" if unresolved else "frozen_nonfinal"
    )
    # TDI-11.2 stays unresolved_blocking while pre-arm (all unresolved).
    if scientific_status != expected_scientific and not (
        cfg["id"] == "tdi11.2" and scientific_status == "unresolved_blocking"
    ):
        fail(
            f"{cfg['id']}: scientific_status {scientific_status!r} disagrees "
            f"with field progress (expected {expected_scientific!r})"
        )

    authorized_pin_count = inventory.get("authorized_pin_count")
    if authorized_pin_count != pinned_count:
        fail(
            f"{cfg['id']}: inventory authorized_pin_count "
            f"{authorized_pin_count!r} disagrees with observed pinned "
            f"{pinned_count}"
        )

    block: dict[str, Any] = {
        "stage": cfg["stage"],
        "freeze_contract": str(cfg["freeze"].relative_to(ROOT)),
        "inventory": str(cfg["inventory"].relative_to(ROOT)),
        "status_doc": cfg["status_doc"],
        "plan_doc": cfg["plan_doc"],
        "scientific_status": scientific_status,
        "pinned_count": pinned_count,
        "unresolved_count": len(unresolved),
        "field_total": total,
        "pinned_progress": f"{pinned_count}/{total}",
        "pinned_count_floor": cfg["pin_floor"],
        "execution_flags": {exec_flag: False},
        "pins": pins,
        "blockers": blockers,
        "non_authorizing_surfaces": inventory.get("non_authorizing_surfaces")
        or [],
        "holdout_boundary": inventory.get("holdout_boundary") or {},
    }
    for extra in cfg.get("extra_execution_flags", ()):
        block["execution_flags"][extra] = False

    if "ledger" in cfg:
        digest = sha256_file(cfg["freeze"])
        sidecar = cfg["digest_sidecar"].read_text(encoding="utf-8")
        ledger = cfg["ledger"].read_text(encoding="utf-8")
        if digest not in sidecar:
            fail("TDI-11.2 digest sidecar does not match freeze file")
        if digest not in ledger:
            fail("TDI-11.2 unresolved ledger does not content-address freeze file")
        if "NOT A PIN" not in ledger and "not a pin" not in ledger.lower():
            fail("TDI-11.2 ledger must declare it is not a pin")
        block["ledger_digest_sha256"] = digest
        block["ledger_digest_verified"] = True
        block["unresolved_ledger"] = str(cfg["ledger"].relative_to(ROOT))
        block["digest_sidecar"] = str(cfg["digest_sidecar"].relative_to(ROOT))

    return block


def build_summary() -> dict[str, Any]:
    series_out: dict[str, Any] = {}
    for cfg in SERIES:
        series_out[cfg["id"]] = series_block(cfg)

    # Cross-series scout verdict (no invented pins this slice).
    scout = {
        "after_merge": "TDI-11.2 registry/timing/H11-A / #218 (6ec59c1)",
        "new_8_1_pins": [],
        "new_9_1_pins": [],
        "new_11_2_pins": [],
        "verdict": (
            "Post-#218 diversification lands TDI-11.2 non-executing "
            "Development/Validation population-derivation scaffolding "
            "(exact authorized Dev/Val domain taxonomy, caller-supplied "
            "stratum/seed-space contract, deterministic domain-separated "
            "seed-commitment framing, Final/holdout leakage guards; 0x08xx "
            "codes). Distinct from #213/#216/#218. Invents no TDI-8.1 / 9.1 / "
            "11.2 freeze pins. Existing authorized pins stay 3/17, 1/14, 0/12. "
            "Execution flags remain hard-false. No holdout 7.2/8.2/9.2 contact."
        ),
    }

    return {
        "schema": SCHEMA,
        "generated_by": "scripts/emit-tdi-freeze-progress-summary.py",
        "purpose": (
            "Machine-readable freeze progress + blocker evidence-class summary "
            "for TDI-8.1 / TDI-9.1 / TDI-11.2. CI-verified; not a pin source."
        ),
        "scout": scout,
        "series": series_out,
        "tdi10_note": (
            "TDI-10.x through 10.20 / TDI-12.0 Stage-0 remain orthogonal. "
            "TDI-11.2 population-derivation scaffolding and prior "
            "registry/timing/H11-A / accounting / rejection scaffolds invent "
            "no freeze pins from operator lemmas or from TDI-11.1 unlimited "
            "envelopes."
        ),
    }


def canonical_bytes(data: dict[str, Any]) -> bytes:
    return (json.dumps(data, indent=2, sort_keys=True) + "\n").encode("utf-8")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--check",
        action="store_true",
        help="verify docs/tdi-freeze-progress-summary.json matches emission",
    )
    parser.add_argument(
        "--write",
        action="store_true",
        help="write docs/tdi-freeze-progress-summary.json",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=SUMMARY_PATH,
        help="summary artifact path",
    )
    args = parser.parse_args(argv)

    if not args.check and not args.write:
        args.write = True

    try:
        summary = build_summary()
        payload = canonical_bytes(summary)
        if args.write:
            args.output.parent.mkdir(parents=True, exist_ok=True)
            args.output.write_bytes(payload)
            print(f"wrote {args.output.relative_to(ROOT)}")
        if args.check:
            if not args.output.is_file():
                fail(f"missing summary artifact: {args.output}")
            existing = args.output.read_bytes()
            if existing != payload:
                fail(
                    "tdi-freeze-progress-summary.json is stale; re-run "
                    "scripts/emit-tdi-freeze-progress-summary.py --write"
                )
            print("tdi-freeze-progress-summary.json: UP TO DATE")
    except SummaryError as exc:
        print(f"TDI freeze-progress summary ERROR: {exc}", file=sys.stderr)
        return 1

    # Human-readable rollup for CI logs.
    for sid, block in summary["series"].items():
        print(
            f"{sid}: {block['pinned_progress']} pinned; "
            f"scientific_status={block['scientific_status']}; "
            f"execution_flags={block['execution_flags']}"
        )
        if block.get("ledger_digest_verified"):
            print(
                f"{sid}: ledger digest verified "
                f"{block['ledger_digest_sha256']}"
            )
    print("TDI freeze-progress summary: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
