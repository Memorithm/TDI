#!/usr/bin/env python3
"""Public analytic fixture batches executed by Hub; no model or final dataset.

The component evaluates at most 32 points in a fresh trusted Python process.
Only additive and Ishigami functions are supported. Measurements cover the
batch calculation, with process peak RSS separately scoped to this child.
"""
from __future__ import annotations

import argparse
import math
from pathlib import Path
import resource
import sys
import time
import uuid

import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_store import atomic_json, identity
from tdi_engine_runtime import canonical_campaign

MODULES = ("tdi_sensitivity_fixture.py", "tdi_experiment_supervisor.py", "tdi_engine_store.py",
           "tdi_engine_runtime.py", "tdi_execution_graph.py", "tdi_artifact_contract.py",
           "tdi_experiment_contract.py", "tdi_hub_edge_contract.py", "tdi_hub_admission_contract.py")


def evaluate(function, values):
    """Evaluate two bounded software controls, never a scientific model."""
    if not isinstance(values, list) or not 1 <= len(values) <= 8 or any(type(x) not in (float, int) or not math.isfinite(x) or abs(x) > 100 for x in values):
        raise durable.ContractError("analytic fixture requires 1..8 bounded finite coordinates")
    if function == "additive":
        return math.fsum((j + 1) * x for j, x in enumerate(values))
    if function == "ishigami" and len(values) == 3 and all(abs(x) <= math.pi for x in values):
        return math.sin(values[0]) + 7 * math.sin(values[1])**2 + .1 * values[2]**4 * math.sin(values[0])
    raise durable.ContractError("unsupported public analytic fixture or domain")


def prepare(client, plan, function):
    """Register the local pinned batch evaluator and produce an ordinary Hub DAG.

    No workflow is submitted here. The standard submit/run/attach/resume/cancel
    CLI owns execution and ambiguous-submission recovery. Cache is disabled;
    repeated design points remain separately evaluated observations.
    """
    from tdi_sensitivity import validate_plan
    plan = validate_plan(plan)
    if function not in ("additive", "ishigami"):
        raise durable.ContractError("unsupported fixture")
    if plan["protocol"]["output_unit"] != "dimensionless":
        raise durable.ContractError("analytic fixture output is dimensionless")
    for f in plan["protocol"]["factors"]:
        if f["unit"] != "dimensionless" or abs(f["lower"]) > 100 or abs(f["upper"]) > 100:
            raise durable.ContractError("analytic fixture factor domain/unit mismatch")
    if function == "ishigami" and (len(plan["protocol"]["factors"]) != 3 or any(abs(f[bound]) > math.pi for f in plan["protocol"]["factors"] for bound in ("lower", "upper"))):
        raise durable.ContractError("Ishigami requires three factors within [-pi,pi]")
    wrapper = Path(__file__).resolve()
    python = Path(sys.executable).resolve()
    deployment = {name: durable.file_digest(wrapper.with_name(name)) for name in MODULES}
    deployment["python"] = durable.file_digest(python)
    root = identity("tdi-sensitivity-fixture/v1", {"plan": plan["identity"], "function": function,
                    "deployment": deployment, "python": str(python), "wrapper": str(wrapper)})
    component_id = str(uuid.uuid5(uuid.NAMESPACE_URL, root))
    manifest = {"id": component_id, "name": "tdi-public-sensitivity", "version": "1.0.0", "kind": "tool",
                "capabilities": [{"name": "tdi.sensitivity.batch", "contract_version": "1.0.0", "inputs": [], "outputs": [{"name": "result"}]}],
                "execution": {"type": "process", "program": str(python),
                    "args": [str(wrapper), "--parameters", "{params}", "--output", "{output:result}"],
                    "outputs": [{"name": "result", "path": "result.json", "media_type": "application/json", "required": True}]},
                "metadata": {"tdi.scope": "public-non-final-analytic-fixture", "tdi.adapter.sha256": deployment[wrapper.name], "tdi.python.sha256": deployment["python"]}}
    component = client.request("POST", "/api/v1/components", value={"schema_version": 1, "manifest": manifest})["component"]
    if component["id"] != component_id:
        raise durable.ContractError("analytic component identity mismatch")
    pin = {"component_id": component_id, "component_version": component["version"], "component_manifest_digest": component["manifest_digest"],
           "capability": "tdi.sensitivity.batch", "capability_contract_version": "1.0.0"}
    steps, policy, outputs = [], {}, {}
    domain = plan["protocol"]["domain"]
    for start in range(0, len(plan["rows"]), 32):
        key = "batch-" + str(start // 32)
        rows = [{"id": r["id"], "ordinal": r["ordinal"], "values": [repr(x) for x in r["values"]]} for r in plan["rows"][start:start + 32]]
        params = {"schema": 1, "purpose": "public-analytic-sensitivity", "domain": domain, "plan_id": root,
                  "plan_identity": plan["identity"], "seed": plan["protocol"]["seed"], "function": function,
                  "output_unit": "dimensionless", "deployment": deployment, "rows": rows}
        steps.append(dict(pin, key=key, component_alias="sensitivity", parameters=params, inputs={}, outputs=["file:result"],
                          after=[], timeout_milliseconds=10000, checkpoint={"mode": "none", "input": None, "output": None}))
        policy[key] = pin
        outputs[key] = {"file:result": {"media_type": "application/json", "access_class": domain.lower(),
                       "json_fields": {"status": "SensitivityEvaluated", "plan_id": root, "plan_identity": plan["identity"], "seed": plan["protocol"]["seed"]}, "cache": "disabled"}}
    graph = {"schema": 1, "semantic_version": "tdi-graph/1.0.0", "name": "sensitivity-" + root[:24],
             "root_plan_id": root, "max_concurrency": min(4, len(steps)), "steps": steps,
             "hub_contract": {"repository": graphs.HUB_REPOSITORY, "source_commit": graphs.HUB_SOURCE_COMMIT,
                              "workflow_schema_version": 1, "workflow_model_version": "1.2.0"}}
    return canonical_campaign({"schema": 1, "purpose": "development-software", "domain": domain, "graph": graph,
                               "policy": {"trust": "trusted-software", "allowed_steps": policy}, "outputs": outputs})


def run(params):
    """Evaluate a closed batch after checking its exact deployment identity."""
    if (not isinstance(params, dict) or set(params) != {"schema", "purpose", "domain", "plan_id", "plan_identity", "seed", "function", "output_unit", "deployment", "rows"}
            or type(params["schema"]) is not int or params["schema"] != 1
            or params["purpose"] != "public-analytic-sensitivity" or params["domain"] not in ("Development", "Validation")
            or params["output_unit"] != "dimensionless"):
        raise durable.ContractError("invalid analytic batch")
    for k in ("plan_id", "plan_identity"):
        graphs._sha256(params[k], k)
    if not isinstance(params["seed"], str) or not params["seed"].isascii() or not params["seed"].isdigit() or len(params["seed"]) > 20 or str(int(params["seed"])) != params["seed"] or int(params["seed"]) > 2**64 - 1:
        raise durable.ContractError("invalid batch seed")
    expected = {name: durable.file_digest(Path(__file__).with_name(name)) for name in MODULES}
    expected["python"] = durable.file_digest(Path(sys.executable).resolve())
    if params["deployment"] != expected:
        raise durable.ContractError("analytic deployment changed")
    rows = params["rows"]
    if not isinstance(rows, list) or not 1 <= len(rows) <= 32:
        raise durable.ContractError("batch exceeds 32 points")
    seen = set(); result = []
    started, cpu = time.perf_counter_ns(), time.process_time_ns()
    for row in rows:
        if (not isinstance(row, dict) or set(row) != {"id", "ordinal", "values"} or type(row["ordinal"]) is not int
                or not 0 <= row["ordinal"] < 4096 or row["ordinal"] in seen
                or not isinstance(row["values"], list) or not 1 <= len(row["values"]) <= 8
                or any(not isinstance(x, str) or len(x) > 32 for x in row["values"])):
            raise durable.ContractError("invalid analytic row")
        graphs._sha256(row["id"], "row identity"); seen.add(row["ordinal"])
        result.append({"id": row["id"], "ordinal": row["ordinal"], "value": evaluate(params["function"], [float(x) for x in row["values"]])})
    costs = {"wall_ns": time.perf_counter_ns() - started, "cpu_ns": time.process_time_ns() - cpu,
             "sensor": "Python perf_counter_ns/process_time_ns", "scope": "batch computation; excludes interpreter startup, imports and identity checks",
             "process_peak_rss_bytes": resource.getrusage(resource.RUSAGE_SELF).ru_maxrss * 1024 if sys.platform == "linux" else None,
             "rss_scope": "whole child lifetime Linux getrusage; bytes", "gpu": None, "energy": None}
    return {"schema": 1, "status": "SensitivityEvaluated", "plan_id": params["plan_id"], "plan_identity": params["plan_identity"],
            "seed": params["seed"], "function": params["function"], "output_unit": params["output_unit"], "rows": result, "cost_measurements": costs}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--parameters", required=True); parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        atomic_json(args.output, run(durable.strict_json(args.parameters, max_bytes=16384)))
        return 0
    except (ValueError, TypeError, OSError) as error:
        print(durable.canonical({"schema": 1, "status": "analytic-error", "error": str(error)}), file=sys.stderr)
        return durable.EXIT_CONTRACT


if __name__ == "__main__":
    raise SystemExit(main())
