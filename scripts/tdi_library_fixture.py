#!/usr/bin/env python3
"""Bounded Hub recipe exercising finite/branched-RNG/Jacobi adapters and codecs.

Each trial runs a two-step prefix, restores it in a new process for two more
steps, runs four uninterrupted steps, then compares both paths and an independent
oracle. Hub owns processes, dependencies and artifact publication. Only these
public, fixed Development/Validation software fixtures are accepted.
"""
from __future__ import annotations

import argparse
import math
from pathlib import Path
import struct
import subprocess
import sys
import uuid

import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_runtime import canonical_campaign
from tdi_engine_store import atomic_json, identity


def prepare_library_fixture(client, worker, *, adapter, domain="Development", trials=2):
    """Register pinned local trusted software and return a four-stage trial DAG."""
    if (adapter not in ("finite", "branch-rng", "jacobi") or domain not in ("Development", "Validation")
            or type(trials) is not int or not 1 <= trials <= 8):
        raise durable.ContractError("library fixture requires a known adapter and 1..8 non-final trials")
    worker = Path(worker)
    if worker.is_symlink() or not worker.is_file():
        raise durable.ContractError("expected regular Rust library worker")
    worker = worker.resolve(strict=True)
    wrapper, python = Path(__file__).resolve(), Path(sys.executable).resolve()
    hashes = {"worker": durable.file_digest(worker), "wrapper": durable.file_digest(wrapper),
              "python": durable.file_digest(python)}
    root_plan = identity("tdi-library-fixture/v1", dict(hashes, adapter=adapter, domain=domain, trials=trials))
    dependencies = {"prefix": [], "resume": ["prefix"], "full": [], "verify": ["prefix", "resume", "full"]}
    components = {}
    for mode, inputs in dependencies.items():
        component_id = str(uuid.uuid5(uuid.NAMESPACE_URL, root_plan + mode + str(worker) + str(wrapper)))
        args = [str(wrapper), "--mode", mode, "--parameters", "{params}", "--output", "{output:result}"]
        for name in inputs:
            args += ["--" + name, "{input:" + name + "}"]
        if mode != "verify":
            args += ["--worker", str(worker), "--worker-sha256", hashes["worker"]]
        manifest = {"id": component_id, "name": "tdi-library-" + mode, "version": "1.0.0", "kind": "tool",
                    "capabilities": [{"name": "tdi.library." + mode, "contract_version": "1.0.0",
                                      "inputs": [{"name": name} for name in inputs], "outputs": [{"name": "result"}]}],
                    "execution": {"type": "process", "program": str(python), "args": args,
                                  "outputs": [{"name": "result", "path": "result.json", "media_type": "application/json", "required": True}]},
                    "metadata": {"tdi." + name + ".sha256": digest for name, digest in hashes.items()}}
        component = client.request("POST", "/api/v1/components", value={"schema_version": 1, "manifest": manifest})["component"]
        if component["id"] != component_id:
            raise durable.ContractError("registered library component identity mismatch")
        components[mode] = component
    steps, policy, outputs = [], {}, {}
    for index in range(trials):
        for mode, inputs in dependencies.items():
            component, key = components[mode], f"{mode}-{index}"
            pin = {"component_id": component["id"], "component_version": component["version"],
                   "component_manifest_digest": component["manifest_digest"],
                   "capability": "tdi.library." + mode, "capability_contract_version": "1.0.0"}
            steps.append(dict(pin, key=key, component_alias="library-" + mode,
                              parameters={"schema": 1, "purpose": "development-software", "domain": domain,
                                          "adapter": adapter, "index": index, "plan_id": root_plan},
                              inputs={name: {"kind": "step", "step": f"{name}-{index}", "output": "file:result"} for name in inputs},
                              outputs=["file:result"], after=[f"{name}-{index}" for name in inputs],
                              timeout_milliseconds=15000,
                              checkpoint={"mode": "none", "input": None, "output": None} if mode == "verify" else
                                         {"mode": "exact", "input": "prefix" if mode == "resume" else None, "output": "file:result"}))
            policy[key] = pin
            outputs[key] = {"file:result": {"media_type": "application/json", "access_class": domain.lower(),
                            "json_fields": {"status": "Verified" if mode == "verify" else "Evaluated",
                                            "plan_id": root_plan, "adapter": adapter,
                                            "seed": str(index | (2**63 if domain == "Validation" else 0))},
                            "cache": "deterministic-data"}}
    graph = {"schema": 1, "semantic_version": "tdi-graph/1.0.0", "name": "tdi-library-" + root_plan[:24],
             "root_plan_id": root_plan, "max_concurrency": min(trials * 2, 4), "steps": steps,
             "hub_contract": {"repository": "Memorithm/scirust-hub", "source_commit": graphs.HUB_SOURCE_COMMIT,
                              "workflow_schema_version": 1, "workflow_model_version": "1.2.0"}}
    return canonical_campaign({"schema": 1, "purpose": "development-software", "domain": domain,
                               "graph": graph, "policy": {"trust": "trusted-software", "allowed_steps": policy}, "outputs": outputs})


def parameters(raw):
    """Validate the entire closed fixture schema before opening inputs or workers."""
    p = durable.strict_json(raw, max_bytes=16384)
    if (not isinstance(p, dict) or set(p) != {"schema", "purpose", "domain", "adapter", "index", "plan_id"}
            or type(p["schema"]) is not int or p["schema"] != 1 or p["purpose"] != "development-software"
            or p["domain"] not in ("Development", "Validation") or p["adapter"] not in ("finite", "branch-rng", "jacobi")
            or type(p["index"]) is not int or not 0 <= p["index"] < 8):
        raise durable.ContractError("invalid library fixture parameters")
    graphs._sha256(p["plan_id"], "library plan")
    return p, p["index"] | (2**63 if p["domain"] == "Validation" else 0)


def read_result(path):
    if path is None or path.is_symlink() or not path.is_file():
        raise durable.ContractError("expected regular library result")
    with path.open("rb") as stream:
        return durable.strict_json(stream.read(65537), max_bytes=65536)


def branch_rng_oracle(seed, stop):
    """Independent integer oracle for the bounded branched finite fixture."""
    state = seed % 4
    rng_state = seed ^ 0x9E3779B97F4A7C15
    values = []
    for _depth in range(1, stop + 1):
        rng_state = (rng_state * 6364136223846793005 + 1442695040888963407) & ((1 << 64) - 1)
        successors = sorted(((state + 1) % 4, (state + 2) % 4))
        state = successors[rng_state & 1]
        values.append([float(state)])
    return values, state, rng_state


def check_result(result, p, seed, start, stop):
    """Check every observation and the complete codec bytes with a separate oracle."""
    if (not isinstance(result, dict) or set(result) != {"schema", "status", "adapter", "plan_id", "seed", "completed_depth", "observations", "checkpoint"}
            or type(result["schema"]) is not int or result["schema"] != 1 or result["status"] != "Evaluated"
            or result["adapter"] != p["adapter"] or result["plan_id"] != p["plan_id"] or result["seed"] != str(seed)
            or type(result["completed_depth"]) is not int or result["completed_depth"] != stop):
        raise durable.ContractError("library result identity/shape mismatch")
    if p["adapter"] == "finite":
        expected = [[float((seed % 4 + depth) % 4)] for depth in range(start + 1, stop + 1)]
    elif p["adapter"] == "branch-rng":
        trace, _, _ = branch_rng_oracle(seed, stop)
        expected = trace[start:stop]
    else:
        expected = [[(4 + depth / 4) / ((4 + depth / 4)**2 - 1)] * 2
                    for depth in range(start + 1, stop + 1)]
    rows = result["observations"]
    if not isinstance(rows, list) or len(rows) != len(expected):
        raise durable.ContractError("library observation population mismatch")
    for row, oracle in zip(rows, expected):
        if (not isinstance(row, list) or len(row) != len(oracle)
                or any(type(x) not in (float, int) or not math.isfinite(x)
                       or not math.isclose(x, y, rel_tol=0, abs_tol=0 if p["adapter"] == "finite" else 2e-15)
                       for x, y in zip(row, oracle))):
            raise durable.ContractError("independent library oracle rejected observation")
    encoded = result["checkpoint"]
    prefix = p["plan_id"] + f"{seed:016x}"
    if (not isinstance(encoded, str) or not encoded.startswith(prefix) or len(encoded) > len(prefix) + 1024
            or any(c not in "0123456789abcdef" for c in encoded) or len(encoded) % 2):
        raise durable.ContractError("checkpoint identity/encoding mismatch")
    raw = bytes.fromhex(encoded[len(prefix):])
    if p["adapter"] == "finite":
        if raw != b"TDIFCP1\0" + bytes([(seed % 4 + stop) % 4, stop]):
            raise durable.ContractError("finite checkpoint oracle mismatch")
    elif p["adapter"] == "branch-rng":
        _, state, rng_state = branch_rng_oracle(seed, stop)
        expected_checkpoint = b"TDIBRP1\0" + bytes([state, stop]) + struct.pack("<Q", rng_state)
        if raw != expected_checkpoint:
            raise durable.ContractError("branched RNG checkpoint oracle mismatch")
    else:
        if len(raw) != 59 or raw[:11] != b"TDIJCP1\0" + bytes([2, stop, 2]):
            raise durable.ContractError("Jacobi checkpoint shape mismatch")
        fields = struct.unpack("<6d", raw[11:])
        diagonal = (4 + stop / 4) / ((4 + stop / 4)**2 - 1)
        oracle = [stop / 4, 4, 4, 1, diagonal, diagonal]
        if any(not math.isfinite(x) or not math.isclose(x, y, rel_tol=0, abs_tol=2e-15) for x, y in zip(fields, oracle)):
            raise durable.ContractError("Jacobi checkpoint oracle mismatch")
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("prefix", "resume", "full", "verify"), required=True)
    parser.add_argument("--parameters", required=True)
    parser.add_argument("--output", type=Path, required=True)
    for name in ("prefix", "resume", "full", "worker"):
        parser.add_argument("--" + name, type=Path)
    parser.add_argument("--worker-sha256")
    args = parser.parse_args()
    try:
        p, seed = parameters(args.parameters)
        if args.mode == "verify":
            prefix = check_result(read_result(args.prefix), p, seed, 0, 2)
            resumed = check_result(read_result(args.resume), p, seed, 2, 4)
            full = check_result(read_result(args.full), p, seed, 0, 4)
            if prefix["observations"] + resumed["observations"] != full["observations"] or resumed["checkpoint"] != full["checkpoint"]:
                raise durable.ContractError("split and uninterrupted execution disagree")
            result = {"schema": 1, "status": "Verified", "plan_id": p["plan_id"], "seed": str(seed), "adapter": p["adapter"],
                      "oracle": ("finite-cycle-modular/v1" if p["adapter"] == "finite" else
                                 "finite-branch-lcg/v1" if p["adapter"] == "branch-rng" else "two-row-inverse/v1"),
                      "absolute_tolerance": 0 if p["adapter"] in ("finite", "branch-rng") else 2e-15,
                      "split_replay_equal": True, "input_sha256": {n: durable.file_digest(getattr(args, n)) for n in ("prefix", "resume", "full")}}
        else:
            if not args.worker or args.worker.is_symlink() or durable.file_digest(args.worker) != args.worker_sha256:
                raise durable.ContractError("Rust library binary identity mismatch")
            command = [str(args.worker), "--adapter", p["adapter"], "--seed", str(seed), "--plan-id", p["plan_id"],
                       "--steps", "4" if args.mode == "full" else "2"]
            if args.mode == "resume":
                previous = check_result(read_result(args.prefix), p, seed, 0, 2)
                command += ["--restore", previous["checkpoint"]]
            completed = subprocess.run(command, capture_output=True, timeout=10, check=False,
                                       env={"LANG": "C", "LC_ALL": "C"}, stdin=subprocess.DEVNULL)
            if completed.returncode or durable.file_digest(args.worker) != args.worker_sha256:
                raise durable.ContractError("Rust library execution/identity failed")
            result = check_result(durable.strict_json(completed.stdout, max_bytes=65536), p, seed,
                                  2 if args.mode == "resume" else 0, 2 if args.mode == "prefix" else 4)
        atomic_json(args.output, result)
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(durable.canonical({"schema": 1, "status": "library-fixture-error", "error": str(error)}), file=sys.stderr)
        return durable.EXIT_CONTRACT


if __name__ == "__main__":
    raise SystemExit(main())
