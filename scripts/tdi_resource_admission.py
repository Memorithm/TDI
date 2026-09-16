"""Local Hub admission through Elastic's real verified concurrency controller.

This optional profile changes only graph max_concurrency. It requires a local
Hub deployment executing local process components in the same resource scope.
It is not remote placement, an OS quota, or a physical memory reservation.
"""
from __future__ import annotations

import copy
import hashlib
import fcntl
import ipaddress
from pathlib import Path
import re
import time
import urllib.parse

import tdi_engine_runtime as runtime
import tdi_experiment_supervisor as durable
from tdi_engine_store import identity
from tdi_physical_telemetry import capacity_snapshot, measured_process


def _policy(value):
    if not isinstance(value, dict) or set(value) != {"max_concurrency", "memory_bytes_per_trial", "reserve_memory_bytes", "max_age_milliseconds"}:
        raise durable.ContractError("invalid resource admission policy fields")
    bounds = {"max_concurrency": (1, 256), "memory_bytes_per_trial": (1, 2**64 - 1),
              "reserve_memory_bytes": (0, 2**64 - 1), "max_age_milliseconds": (1, 60000)}
    for key, (lo, hi) in bounds.items():
        if type(value[key]) is not int or not lo <= value[key] <= hi:
            raise durable.ContractError("invalid resource admission policy units/bounds")
    return value


def _report(out, request, code, costs):
    report = durable.strict_json(out, max_bytes=65536)
    required = {"schema_version", "request", "status", "reason", "proposed_width", "previous_width", "final_width",
                "committed", "rolled_back", "verification", "events"}
    if (not isinstance(report, dict) or set(report) != required or type(report["schema_version"]) is not int or report["schema_version"] != 1
            or durable.canonical(report["request"]) != durable.canonical(request) or report["status"] not in ("admitted", "rejected")
            or type(report["committed"]) not in (bool, type(None)) or type(report["rolled_back"]) not in (bool, type(None))
            or not isinstance(report["reason"], str) or not 1 <= len(report["reason"]) <= 2048
            or not isinstance(report["events"], list) or len(report["events"]) > 128
            or any(not isinstance(event, str) or len(event) > 2048 for event in report["events"])
            or type(report["previous_width"]) is not int or report["previous_width"] != request["max_concurrency"]
            or type(report["final_width"]) is not int or not 1 <= report["final_width"] <= request["max_concurrency"]):
        raise durable.ContractError("Elastic admission report shape/binding mismatch")
    admitted = report["status"] == "admitted"
    if (costs["technical_failure"] or code != (0 if admitted else 2)
            or (report["committed"] is True) != admitted or (admitted and (report["rolled_back"] is not False or type(report["proposed_width"]) is not int
            or report["proposed_width"] != report["final_width"] or report["verification"] != "Pass" or not report["events"]))):
        raise durable.ContractError("Elastic admission outcome/verification mismatch")
    return report


def elastic_decision(spec, worker, worker_sha256, source_commit, policy):
    """Sample real local capacity and retain actual Elastic process costs/output."""
    _policy(policy)
    worker = Path(worker)
    if (worker.is_symlink() or not worker.is_file() or not re.fullmatch(r"[0-9a-f]{40}", source_commit)
            or durable.file_digest(worker) != worker_sha256):
        raise durable.ContractError("Elastic executable or declared source identity mismatch")
    snapshot = capacity_snapshot()
    request = dict(policy, schema_version=1, plan_id=spec["graph"]["root_plan_id"],
                   max_concurrency=min(policy["max_concurrency"], spec["graph"]["max_concurrency"]),
                   observation={"observation_id": snapshot["observation_id"], "sensor": snapshot["sensor"],
                                "environment_id": snapshot["environment_id"],
                                "age_milliseconds": (time.monotonic_ns() - int(snapshot["sampled_monotonic_ns"])) // 1000000,
                                "capacity": snapshot["capacity"]})
    code, out, _, costs = measured_process([str(worker.resolve()), "admit-capacity"],
                                           input_bytes=durable.canonical(request).encode(), timeout=10, max_output=65536)
    failure = None
    try:
        if durable.file_digest(worker) != worker_sha256:
            raise durable.ContractError("Elastic executable changed during admission")
        report = _report(out, request, code, costs)
    except ValueError as error:
        report, failure = None, str(error)
    evidence = {"schema": 1, "scope": "local Hub process admission only", "observation": snapshot,
                "elastic": {"source_commit": source_commit, "binary_sha256": worker_sha256, "request": request, "report": report,
                            "protocol_failure": failure, "stdout_sha256": hashlib.sha256(out).hexdigest()},
                "cost_measurements": costs, "policy": policy,
                "original_spec_identity": identity("tdi-operational-campaign/v1", spec),
                "physical_ram_reserved": False, "scientific_verdict": "not-assessed"}
    evidence["identity"] = identity("tdi-elastic-admission/v1", evidence)
    return evidence


def _fresh(evidence):
    age = time.monotonic_ns() - int(evidence["observation"]["sampled_monotonic_ns"])
    return 0 <= age <= evidence["policy"]["max_age_milliseconds"] * 1000000


def execute_local_admitted(client, store, spec, roots, worker, worker_sha256, source_commit, policy):
    """Serialize admission for one original graph without blocking cancellation."""
    spec = runtime.canonical_campaign(spec)
    original = identity("tdi-operational-campaign/v1", spec)
    path = Path(str(store.path) + ".resource-" + original + ".lock")
    if path.is_symlink():
        raise durable.ContractError("symlink resource admission lock is forbidden")
    with path.open("a+b") as lock:
        try:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise durable.ContractError("resource admission for this graph is already active") from None
        return _execute_local_admitted(client, store, spec, roots, worker, worker_sha256, source_commit, policy)


def _execute_local_admitted(client, store, spec, roots, worker, worker_sha256, source_commit, policy):
    """Admit and execute with durable recovery intent and no ambiguous redispatch.

    On retry an existing target keeps its exact width. If current resources no
    longer admit that width, execution stops; cancel/replan explicitly. Already
    dispatched attempts are only reconciled. No workload parameters are changed.
    """
    spec = runtime.canonical_campaign(spec)
    _policy(policy)
    try:
        local = ipaddress.ip_address(urllib.parse.urlsplit(client.endpoint).hostname).is_loopback
    except ValueError:
        local = False
    if not local or len(spec["graph"]["steps"]) > 256:
        raise durable.ContractError("resource admission requires a local Hub and at most 256 process steps")
    original = store.create(spec, client.endpoint)
    binding = {"policy": policy, "elastic_binary_sha256": worker_sha256, "elastic_source_commit": source_commit}
    row = store.db.execute("SELECT payload FROM events WHERE campaign=? AND kind='resource-dispatch' ORDER BY sequence DESC LIMIT 1", (original,)).fetchone()
    existing = durable.strict_json(row[0]) if row else None
    if existing and existing["binding"] != binding:
        raise durable.ContractError("resource policy/executable changed; existing dispatch must be reconciled")
    target = store.get(existing["campaign"] if existing else original)
    if target["phase"] not in ("prepared", "admitted"):
        return dict(runtime.execute(client, store, target["id"]), resource_reconciled=True)
    # Pin the actual local process component surfaces before sampling freshness.
    seen = set()
    for step in target["spec"]["graph"]["steps"]:
        if step["component_id"] in seen:
            continue
        seen.add(step["component_id"])
        component = client.request("GET", "/api/v1/components/" + step["component_id"])
        execution = component.get("execution") or {}
        if (component.get("manifest_digest") != step["component_manifest_digest"] or execution.get("type") != "process"
                or not Path(execution.get("program", "")).is_file()):
            raise durable.ContractError("resource admission requires the pinned local process deployment")
    evidence = elastic_decision(target["spec"], worker, worker_sha256, source_commit, policy)
    report = evidence["elastic"]["report"]
    store.event(original, "resource-admission", evidence)
    if report is None:
        return {"status": "resource-rejected", "campaign": original, "reason": "elastic-protocol-failure",
                "admission_identity": evidence["identity"], "cost_measurements": evidence["cost_measurements"]}
    if report["status"] != "admitted" or not _fresh(evidence):
        return {"status": "resource-rejected", "campaign": original, "reason": report["reason"] if _fresh(evidence) else "stale-before-dispatch",
                "admission_identity": evidence["identity"], "cost_measurements": evidence["cost_measurements"]}
    admitted = copy.deepcopy(target["spec"])
    width = report["final_width"]
    if (existing or target["phase"] == "admitted") and width != admitted["graph"]["max_concurrency"]:
        return {"status": "resource-rejected", "campaign": target["id"], "reason": "existing-workflow-width-no-longer-admissible",
                "admission_identity": evidence["identity"]}
    admitted["graph"]["max_concurrency"] = width
    admitted = runtime.canonical_campaign(admitted)
    campaign = store.create(admitted, client.endpoint)
    # Persist the target link before the first Hub mutation, including when both
    # identities coincide. A lost reply never causes another graph to be created.
    if not existing:
        store.event(original, "resource-dispatch", {"campaign": campaign, "binding": binding, "admission_identity": evidence["identity"]})
    if campaign != original:
        store.event(campaign, "resource-admission", evidence)
    runtime.submit(client, store, admitted, roots)
    if not _fresh(evidence):
        store.event(campaign, "resource-dispatch-deferred", {"reason": "stale-after-Hub-admission"})
        return {"status": "resource-rejected", "campaign": campaign, "reason": "stale-after-Hub-admission"}
    result = runtime.execute(client, store, campaign)
    actual = result.get("snapshot", {}).get("spec", {}).get("max_concurrency")
    if actual != width:
        raise durable.ContractError("Hub did not preserve the admitted concurrency width")
    store.event(campaign, "resource-width-verified", {"max_concurrency": actual, "admission_identity": evidence["identity"]})
    return dict(result, admission_identity=evidence["identity"], cost_measurements=evidence["cost_measurements"])
