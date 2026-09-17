#!/usr/bin/env python3
"""Trusted evaluator for the finite search fixture; never a candidate proposer.

Only three fixed, precompiled implementation choices are accepted. Verification
owns its independent modular oracle. Candidate processes receive only an input
state, horizon and implementation selection. Their output contains no fitness.
Physical measurements are taken by this external evaluator with Linux wait4.
"""
from __future__ import annotations

import argparse
import math
from pathlib import Path
import statistics
import sys
import time

import tdi_experiment_supervisor as durable
import tdi_execution_graph as graphs
from tdi_engine_store import atomic_json, identity
from tdi_physical_telemetry import capacity_snapshot, measured_process

IMPLEMENTATIONS = ("table", "modular", "incorrect")


def read_record(path):
    if path is None or path.is_symlink() or not path.is_file():
        raise durable.ContractError("missing regular stage artifact")
    with path.open("rb") as stream:
        return durable.strict_json(stream.read(65537), max_bytes=65536)


def validate_parameters(raw):
    """Close the non-final finite fixture surface before any process/input read."""
    p = durable.strict_json(raw, max_bytes=16384)
    required = {"schema", "purpose", "stage", "candidate_id", "attempt_id", "implementation",
                "worker_sha256", "environment_id", "verification_view"}
    if (not isinstance(p, dict) or set(p) != required or type(p["schema"]) is not int or p["schema"] != 1
            or p["purpose"] != "public-finite-software-search" or p["stage"] not in ("compile", "verify", "measure")
            or p["implementation"] not in IMPLEMENTATIONS):
        raise durable.ContractError("unsupported finite search stage parameters")
    for field in ("candidate_id", "attempt_id", "worker_sha256", "environment_id"):
        graphs._sha256(p[field], field)
    view = p["verification_view"]
    if (not isinstance(view, dict) or set(view) != {"upstream", "verification", "verification_source"}
            or view["verification_source"] != "public-finite/validation/v1"
            or view["verification"]["adapter_id"] != "tdi-independent-modular-oracle/v1"
            or view["verification"]["adapter_sha256"] != durable.file_digest(Path(__file__))):
        raise durable.ContractError("independent verifier identity mismatch")
    return p


def materialization(p):
    return {"schema": 1, "protocol": "tdi-precompiled-finite/v1", "candidate_id": p["candidate_id"],
            "implementation": p["implementation"], "worker_sha256": p["worker_sha256"]}


def check_compiled(record, p):
    value = materialization(p)
    artifact = identity("tdi-finite-materialization/v1", value)
    if (record.get("status") != "StageRecorded" or record.get("stage") != "compile"
            or record.get("candidate_id") != p["candidate_id"] or record.get("materialization") != value
            or record.get("outcome") != {"kind": "compiled", "artifact_sha256": artifact,
                                          "materialization": "precompiled-configuration"}):
        raise durable.ContractError("compiled configuration identity mismatch")
    return artifact


def invoke(worker, p, start, steps):
    """Run the real binary with only public problem inputs and measure externally."""
    if durable.file_digest(worker) != p["worker_sha256"]:
        raise durable.ContractError("candidate binary changed before invocation")
    command = [str(worker), "--implementation", p["implementation"], "--start", str(start), "--steps", str(steps)]
    code, raw, _, costs = measured_process(command, timeout=2, max_output=4096)
    if durable.file_digest(worker) != p["worker_sha256"]:
        costs["technical_failure"] = "worker-identity-drift"
    try:
        value = durable.strict_json(raw, max_bytes=4096)
        well_formed = (isinstance(value, dict) and set(value) == {"schema", "state"}
                       and type(value["schema"]) is int and value["schema"] == 1
                       and type(value["state"]) is int and 0 <= value["state"] < 4)
    except ValueError:
        value, well_formed = None, False
    return code == 0 and not costs["technical_failure"] and well_formed, value, costs


def evaluate(p, worker, compiled=None, verified=None):
    """Execute one stage; incorrect outputs produce negative evidence, never metrics."""
    started = time.monotonic_ns()
    worker = Path(worker)
    if worker.is_symlink() or not worker.is_file() or durable.file_digest(worker) != p["worker_sha256"]:
        raise durable.ContractError("untrusted or changed finite worker")
    observation = capacity_snapshot()
    if observation["environment_id"] != p["environment_id"]:
        raise durable.ContractError("executor environment changed")
    result = {"schema": 1, "status": "StageRecorded", "stage": p["stage"], "candidate_id": p["candidate_id"],
              "attempt_id": p["attempt_id"], "environment_id": p["environment_id"], "cost_measurements": []}
    if p["stage"] == "compile":
        value = materialization(p)
        result["materialization"] = value
        result["outcome"] = {"kind": "compiled", "artifact_sha256": identity("tdi-finite-materialization/v1", value),
                             "materialization": "precompiled-configuration"}
    else:
        artifact = check_compiled(compiled, p)
        if p["stage"] == "verify":
            checks = []
            # Public finite software fixtures; no preregistered research population.
            for start in range(4):
                for steps in (0, 1, 2, 3, 7, 128):
                    valid, value, costs = invoke(worker, p, start, steps)
                    correct = valid and value["state"] == (start + steps) % 4
                    checks.append({"start": start, "steps": steps, "well_formed": valid, "correct": correct})
                    result["cost_measurements"].append(dict(costs, phase="verification", start=start, steps=steps))
                    if not valid:
                        break
                if not checks[-1]["well_formed"]:
                    break
            evidence = dict(p["verification_view"], candidate_id=p["candidate_id"], passed=all(x["correct"] for x in checks),
                            environment_fingerprint=p["environment_id"])
            evidence["evidence_id"] = identity("tdi-finite-verification/v1", {"artifact": artifact, "evidence": evidence, "checks": checks})
            result["checks"] = checks
            result["planned_checks"] = 24
            result["outcome"] = ({"kind": "verified", "artifact_sha256": artifact, "evidence": evidence}
                                 if all(x["well_formed"] for x in checks) else
                                 {"kind": "failed", "reason": "candidate-process-contract-failure", "execution_unknown": False})
        else:
            if (not isinstance(verified, dict) or verified.get("status") != "StageRecorded"
                    or verified.get("stage") != "verify" or verified.get("candidate_id") != p["candidate_id"]
                    or verified.get("environment_id") != p["environment_id"]):
                raise durable.ContractError("no matching independent verification artifact")
            proof = verified.get("outcome", {})
            evidence = proof.get("evidence", {})
            if (proof.get("kind") != "verified" or proof.get("artifact_sha256") != artifact or evidence.get("passed") is not True
                    or any(evidence.get(k) != v for k, v in p["verification_view"].items())):
                raise durable.ContractError("independent verification did not authorize measurement")
            checked = dict(evidence); evidence_id = checked.pop("evidence_id", None)
            if identity("tdi-finite-verification/v1", {"artifact": artifact, "evidence": checked, "checks": verified.get("checks")}) != evidence_id:
                raise durable.ContractError("verification evidence digest mismatch")
            for repeat in range(4):
                valid, value, costs = invoke(worker, p, 3, 32768)
                result["cost_measurements"].append(dict(costs, phase="warmup" if repeat == 0 else "measurement", repetition=repeat))
                if not valid or value["state"] != 3:
                    result["outcome"] = {"kind": "failed", "reason": "measurement-process-or-output-failure", "execution_unknown": False}
                    result["wall_ms"] = math.ceil((time.monotonic_ns() - started) / 1000000)
                    result["scientific_verdict"] = "not-assessed"
                    return result
            measured = result["cost_measurements"][1:]
            metrics = [{"name": "process_wall", "unit": "ns", "value": float(statistics.median(int(x["wall_ns"]) for x in measured))},
                       {"name": "process_peak_rss", "unit": "bytes", "value": float(max(x["peak_rss_bytes"] for x in measured))}]
            measurement = {"metrics": metrics, "cost_measurements": result["cost_measurements"], "environment_id": p["environment_id"],
                           "artifact_sha256": artifact, "verification_evidence_id": evidence_id}
            result["outcome"] = {"kind": "measured", "artifact_sha256": artifact, "verification_evidence_id": evidence_id,
                                 "evidence_sha256": identity("tdi-finite-measurement/v1", measurement),
                                 "environment_id": p["environment_id"], "metrics": metrics}
    result["wall_ms"] = math.ceil((time.monotonic_ns() - started) / 1000000)
    result["scientific_verdict"] = "not-assessed"
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parameters", required=True)
    parser.add_argument("--worker", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--compiled", type=Path)
    parser.add_argument("--verified", type=Path)
    args = parser.parse_args()
    try:
        p = validate_parameters(args.parameters)
        result = evaluate(p, args.worker, read_record(args.compiled) if args.compiled else None,
                          read_record(args.verified) if args.verified else None)
        atomic_json(args.output, result)
        return 0
    except (ValueError, OSError) as error:
        print(durable.canonical({"schema": 1, "status": "search-stage-error", "error": str(error)}), file=sys.stderr)
        return durable.EXIT_TRIAL_FAILURE


if __name__ == "__main__":
    raise SystemExit(main())
