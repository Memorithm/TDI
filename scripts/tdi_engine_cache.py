"""Opt-in exact deterministic-data reuse with durable source evidence.

Hits are references to prior execution, never fresh trials or timing samples.
The complete graph/policy and immediate input artifact identities are bound.
Backend manifests must pin every relevant environment dependency; the operator
is responsible for qualifying the explicit deterministic-data declaration.
"""
from __future__ import annotations

import tdi_artifact_contract as artifacts
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
import tdi_hub_admission_contract as admission
from tdi_engine_store import identity


def cache_request(store, record, step_key, output):
    """Derive exact reuse coordinates from a completed permitted output."""
    if record["phase"] != "completed":
        raise durable.ContractError("only completed source campaigns can populate cache")
    spec = record["spec"]
    steps = {step["key"]: step for step in spec["graph"]["steps"]}
    if step_key not in steps or output not in spec["outputs"][step_key]:
        raise durable.ContractError("unknown cache output")
    if spec["outputs"][step_key][output]["cache"] != "deterministic-data":
        raise durable.ContractError("output policy disables cache")
    step = steps[step_key]
    inputs = [{"name": "campaign-policy", "identity": record["id"]},
              {"name": "output", "identity": identity("tdi-cache-output/v1", output)}]
    for name, binding in sorted(step["inputs"].items()):
        if binding["kind"] == "artifact":
            artifact = record["admission"]["root_artifact_bindings"][binding["sha256"]]["artifact_identity"]
        else:
            artifact = store.result(record["id"], binding["step"], binding["output"])["artifact_identity"]
        inputs.append({"name": "input:" + name, "identity": artifact})
    return artifacts.canonical_cache_request({
        "schema": 1, "policy": "exact-domain", "domain": spec["domain"],
        "plan_id": spec["graph"]["root_plan_id"], "step_identity": graphs.step_identity(spec["graph"], step_key),
        "implementation_identity": step["component_manifest_digest"],
        "backend_identity": step["component_manifest_digest"], "inputs": inputs,
        "parameters_identity": identity("tdi-cache-parameters/v1", step["parameters"]),
    })


def publish_completed(store, campaign):
    """Index eligible outputs after completion; no failed/rejected timing reuse."""
    record = store.get(campaign)
    for step, outputs in sorted(record["spec"]["outputs"].items()):
        for output, policy in sorted(outputs.items()):
            if policy["cache"] != "deterministic-data":
                continue
            request = cache_request(store, record, step, output)
            evidence = store.result(campaign, step, output)
            entry = {"schema": 1, "cache_key": artifacts.cache_key(request), "request": request,
                     "artifact_identity": evidence["artifact_identity"],
                     "provenance_identity": evidence["provenance_identity"]}
            store.cache_put(entry, request, campaign, step, output, authorized=True)


def lookup(client, store, request, *, authorized=False):
    """Return verified bytes and original lineage on a hit; never schedule work."""
    found = store.cache_get(request, authorized=authorized)
    if found is None:
        return {"cache_hit": False, "new_execution": False}
    record = store.get(found["campaign"])
    if record["endpoint"] != client.endpoint:
        raise durable.ContractError("cache source is bound to another Hub")
    if cache_request(store, record, found["step"], found["output"]) != request:
        raise durable.ContractError("cache index differs from current source evidence")
    evidence = store.result(found["campaign"], found["step"], found["output"])
    entry = found["entry"]
    if (evidence["artifact_identity"] != entry["artifact_identity"]
            or evidence["provenance_identity"] != entry["provenance_identity"]):
        raise durable.ContractError("cache evidence identity mismatch")
    raw, portable = client.download(evidence["hub_artifact"], evidence["descriptor"])
    if not admission._json_equal(durable.strict_json(raw), evidence["json"]):
        raise durable.ContractError("cache JSON projection mismatch")
    return {"cache_hit": True, "new_execution": False, "fresh_timing_measurement": False,
            "source_campaign": found["campaign"], "source_step": found["step"],
            "source_output": found["output"], "entry": entry, "evidence": evidence,
            "portable_digest": portable}
