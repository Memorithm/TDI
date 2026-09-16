#!/usr/bin/env python3
"""Real-library TDI fixture adapter and reproducible Hub campaign preparation.

The run mode invokes the compiled `tdi-ai` durable_worker example; verification
checks its finite counter/shift oracle independently. This fixture is not a
model runner or a scientific confirmation. Installed code/binaries are trusted
and immutable for the execution lifetime; the Hub owns outer process cleanup.
"""
from __future__ import annotations

import argparse
import hashlib
from pathlib import Path
import subprocess
import sys
import uuid

import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_runtime import canonical_campaign
from tdi_engine_store import atomic_json, identity


def prepare_fixture(client, worker, *, domain="Development", trials=3):
    """Register actual pinned deployment paths and return a runnable TDI graph.

    `worker` must be the locally built Rust durable_worker. This explicitly
    selected fixture registers components; registration does not execute them.
    The returned graph contains no final stage, protected data or search input.
    """
    if domain not in ("Development", "Validation") or type(trials) is not int or not 1 <= trials <= 32:
        raise durable.ContractError("fixture requires 1..32 non-final trials")
    worker = Path(worker).resolve(strict=True)
    wrapper = Path(__file__).resolve()
    worker_hash = durable.file_digest(worker)
    wrapper_hash = durable.file_digest(wrapper)
    python_hash = durable.file_digest(Path(sys.executable).resolve())
    root_plan = identity("tdi-counter-fixture/v1", {"domain": domain, "trials": trials,
                                                   "worker": worker_hash, "wrapper": wrapper_hash,
                                                   "python": python_hash})
    components = {}
    for mode in ("run", "verify"):
        component_id = str(uuid.uuid5(uuid.NAMESPACE_URL, root_plan + mode + str(worker) + str(wrapper)))
        args = [str(wrapper), "--mode", mode, "--parameters", "{params}", "--output", "{output:result}"]
        if mode == "run":
            args += ["--worker", str(worker), "--worker-sha256", worker_hash]
        else:
            args += ["--input", "{input:result}"]
        manifest = {
            "id": component_id, "name": "tdi-counter-" + mode, "version": "1.0.0", "kind": "tool",
            "capabilities": [{"name": "tdi.fixture." + mode, "contract_version": "1.0.0",
                              "inputs": [] if mode == "run" else [{"name": "result"}],
                              "outputs": [{"name": "result"}]}],
            "execution": {"type": "process", "program": str(Path(sys.executable).resolve()), "args": args,
                          "outputs": [{"name": "result", "path": "result.json", "media_type": "application/json", "required": True}]},
            "metadata": {"tdi.worker.sha256": worker_hash, "tdi.adapter.sha256": wrapper_hash,
                         "tdi.python.sha256": python_hash, "tdi.scope": "non-final-counter-fixture"},
        }
        result = client.request("POST", "/api/v1/components", value={"schema_version": 1, "manifest": manifest})
        if result["component"]["id"] != component_id:
            raise durable.ContractError("registered fixture component identity mismatch")
        components[mode] = result["component"]
    steps, policy, outputs = [], {}, {}
    for index in range(trials):
        for mode in ("run", "verify"):
            component = components[mode]
            key = f"{mode}-{index}"
            pin = {"component_id": component["id"], "component_version": component["version"],
                   "component_manifest_digest": component["manifest_digest"],
                   "capability": "tdi.fixture." + mode, "capability_contract_version": "1.0.0"}
            steps.append(dict(pin, key=key, component_alias="counter-" + mode,
                              parameters={"schema": 1, "purpose": "development-software", "domain": domain,
                                          "index": index, "plan_id": root_plan},
                              inputs={} if mode == "run" else {"result": {"kind": "step", "step": f"run-{index}", "output": "file:result"}},
                              outputs=["file:result"], after=[] if mode == "run" else [f"run-{index}"],
                              timeout_milliseconds=15000, checkpoint={"mode": "none", "input": None, "output": None}))
            policy[key] = pin
            outputs[key] = {"file:result": {"media_type": "application/json", "access_class": domain.lower(),
                            "json_fields": {"status": "Evaluated" if mode == "run" else "Verified", "plan_id": root_plan,
                                            "seed": str(index | (2**63 if domain == "Validation" else 0))},
                            "cache": "deterministic-data"}}
    graph = {"schema": 1, "semantic_version": "tdi-graph/1.0.0", "name": "tdi-counter-" + root_plan[:24],
             "root_plan_id": root_plan, "max_concurrency": min(trials, 4), "steps": steps,
             "hub_contract": {"repository": "Memorithm/scirust-hub", "source_commit": graphs.HUB_SOURCE_COMMIT,
                              "workflow_schema_version": 1, "workflow_model_version": "1.2.0"}}
    return canonical_campaign({"schema": 1, "purpose": "development-software", "domain": domain,
                               "graph": graph, "policy": {"trust": "trusted-software", "allowed_steps": policy},
                               "outputs": outputs})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--mode", choices=("run", "verify"), required=True)
    parser.add_argument("--parameters", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--input", type=Path)
    parser.add_argument("--worker", type=Path)
    parser.add_argument("--worker-sha256")
    args = parser.parse_args()
    try:
        params = durable.strict_json(args.parameters, max_bytes=16384)
        if (not isinstance(params, dict) or set(params) != {"schema", "purpose", "domain", "index", "plan_id"}
                or type(params["schema"]) is not int or params["schema"] != 1
                or params["purpose"] != "development-software" or params["domain"] not in ("Development", "Validation")
                or type(params["index"]) is not int or not 0 <= params["index"] < 32):
            raise durable.ContractError("invalid non-final counter fixture parameters")
        graphs._sha256(params["plan_id"], "fixture plan")
        seed = params["index"] | (2**63 if params["domain"] == "Validation" else 0)
        if args.mode == "run":
            if not args.worker or args.worker.is_symlink() or durable.file_digest(args.worker) != args.worker_sha256:
                raise durable.ContractError("Rust fixture binary identity mismatch")
            completed = subprocess.run([str(args.worker), "--tdi-seed", str(seed), "--tdi-plan-id", params["plan_id"]],
                                       capture_output=True, timeout=10, check=False,
                                       env={"LANG": "C", "LC_ALL": "C"}, stdin=subprocess.DEVNULL)
            if completed.returncode != 0:
                raise durable.ContractError("Rust fixture failed")
            if durable.file_digest(args.worker) != args.worker_sha256:
                raise durable.ContractError("Rust fixture binary changed")
            result = durable.strict_json(completed.stdout, max_bytes=65536)
            if result.get("seed") != seed or result.get("plan_id") != params["plan_id"] or result.get("status") != "Evaluated":
                raise durable.ContractError("Rust fixture response mismatch")
            result["seed"] = str(seed)
        else:
            if not args.input or args.input.is_symlink():
                raise durable.ContractError("verification input required")
            with args.input.open("rb") as stream:
                result = durable.strict_json(stream.read(65537), max_bytes=65536)
            if (result.get("seed") != str(seed) or result.get("plan_id") != params["plan_id"]
                    or result.get("status") != "Evaluated" or result.get("scores") != [2, 2, 2, 2]):
                raise durable.ContractError("independent counter-shift oracle rejected result")
            result = {"status": "Verified", "seed": str(seed), "plan_id": params["plan_id"],
                      "oracle": "counter-shift-distance/v1", "input_sha256": durable.file_digest(args.input)}
        atomic_json(args.output, result)
        return 0
    except (ValueError, OSError, subprocess.SubprocessError) as error:
        print(durable.canonical({"schema": 1, "status": "fixture-error", "error": str(error)}), file=sys.stderr)
        return durable.EXIT_CONTRACT


if __name__ == "__main__":
    raise SystemExit(main())
