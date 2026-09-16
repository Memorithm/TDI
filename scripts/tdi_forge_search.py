"""Durable Forge ask/tell and actual Hub stages for a public finite fixture.

Forge owns proposal/prerequisite/budget/Pareto state. This consumer persists its
checkpoint before action, binds every attempt to exactly one Hub campaign, and
feeds only independently executed stage evidence back to Forge. Uncertain Hub
mutations retain their existing reconciliation path and never trigger new work.
"""
from __future__ import annotations

import fcntl
import hashlib
import ipaddress
from pathlib import Path
import re
import sys
import urllib.parse
import uuid

import tdi_artifact_contract as artifacts
import tdi_engine_runtime as runtime
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
import tdi_hub_edge_contract as edge
import tdi_hub_admission_contract as admission
from tdi_engine_store import identity
from tdi_forge_fixture import IMPLEMENTATIONS
from tdi_physical_telemetry import capacity_snapshot, measured_process


def pinned_file(path, expected=None):
    """Pin an explicitly trusted regular executable or evaluator source file."""
    path = Path(path)
    if path.is_symlink() or not path.is_file():
        raise durable.ContractError("expected regular pinned search deployment file")
    path = path.resolve(strict=True)
    digest = durable.file_digest(path)
    if expected is not None and digest != expected:
        raise durable.ContractError("search deployment file changed")
    return {"path": str(path), "sha256": digest}


class ForgeClient:
    """Bounded real Forge process client, with actual OS costs and byte pinning.

    The binary's source revision is declared, not a build attestation. Candidate
    execution happens through Hub and never inside this control-plane process.
    """
    def __init__(self, binary, sha256, source_commit):
        self.binary = pinned_file(binary, sha256)
        if not isinstance(source_commit, str) or not re.fullmatch("[0-9a-f]{40}", source_commit):
            raise durable.ContractError("Forge source must be an exact declared commit")
        self.source_commit = source_commit

    @property
    def binding(self):
        return dict(self.binary, source_commit=self.source_commit, repository="Memorithm/Forge",
                    protocol="forge-finite-search/v1")

    def call(self, spec, checkpoint=None, command=None):
        """Return a verified checkpoint projection and measured control costs."""
        request = {"spec": spec, "checkpoint": checkpoint, "command": command}
        raw = durable.canonical(request).encode()
        if len(raw) > 1024 * 1024:
            raise durable.ContractError("TDI Forge process request exceeds 1 MiB")
        pinned_file(self.binary["path"], self.binary["sha256"])
        code, out, _, costs = measured_process([self.binary["path"]], input_bytes=raw, timeout=30, max_output=8 * 1024 * 1024)
        pinned_file(self.binary["path"], self.binary["sha256"])
        if code != 0 or costs["technical_failure"]:
            raise ForgeError("Forge process rejected or failed", costs, hashlib.sha256(out).hexdigest())
        try:
            result = durable.strict_json(out, max_bytes=8 * 1024 * 1024, max_items=1000000)
            expected_commands = list(checkpoint["commands"]) if checkpoint else []
            if command is not None:
                expected_commands.append(command)
            external = spec["manifest"]["external_domain"]
            generation = {"upstream": external["upstream"], "allowed_candidate_dimensions": external["allowed_candidate_dimensions"],
                          "generation_sources": external["data_boundary"]["generation_sources"]}
            if (set(result) != {"schema_version", "checkpoint", "snapshot", "generation_view"}
                    or type(result["schema_version"]) is not int or result["schema_version"] != 1
                    or result["checkpoint"]["commands"] != expected_commands
                    or (checkpoint and result["checkpoint"]["spec_sha256"] != checkpoint["spec_sha256"])
                    or result["generation_view"] != generation or result["snapshot"]["scientific_verdict"] != "not-assessed"):
                raise ValueError("Forge response binding mismatch")
        except (ValueError, TypeError, KeyError) as error:
            raise ForgeError(str(error), costs, hashlib.sha256(out).hexdigest()) from None
        return result, costs


class ForgeError(durable.ContractError):
    def __init__(self, message, costs, output_sha256):
        super().__init__(message)
        self.evidence = {"error": message, "cost_measurements": costs, "stdout_sha256": output_sha256}


def prepare_fixture(client, store, forge, worker, source_commit, *, strategy="grid", seed="0"):
    """Create a non-final search; this registers no worker and executes no candidate."""
    if not re.fullmatch("[0-9a-f]{40}", source_commit):
        raise durable.ContractError("TDI fixture source must be an exact declared commit")
    try:
        local = ipaddress.ip_address(urllib.parse.urlsplit(client.endpoint).hostname).is_loopback
    except ValueError:
        local = False
    if not local:
        raise durable.ContractError("finite search fixture requires a local trusted Hub")
    wrapper = pinned_file(Path(__file__).with_name("tdi_forge_fixture.py"))
    binding = {"forge": forge.binding, "worker": pinned_file(worker), "evaluator": wrapper,
               "python": pinned_file(Path(sys.executable).resolve()), "hub_endpoint": client.endpoint,
               "telemetry": pinned_file(Path(__file__).with_name("tdi_physical_telemetry.py")),
               "support_modules": {name: pinned_file(Path(__file__).with_name(name + ".py")) for name in (
                   "tdi_forge_search", "tdi_experiment_supervisor", "tdi_execution_graph", "tdi_engine_runtime",
                   "tdi_engine_store", "tdi_hub_client", "tdi_hub_admission_contract", "tdi_hub_edge_contract", "tdi_artifact_contract")},
               "tdi_source_commit": source_commit, "environment_id": capacity_snapshot()["environment_id"]}
    semantic = {"schema": 1, "kind": "public-finite-software-search", "state_count": 4,
                "implementations": list(IMPLEMENTATIONS), "measurement_steps": 32768, "measurement_start": 3,
                "warmup": 1, "repetitions": 3, "measurements": "external-process-wall-and-wait4-peak-rss",
                "worker_sha256": binding["worker"]["sha256"], "evaluator_sha256": wrapper["sha256"]}
    manifest = {"schema_version": 1, "external_domain": {
        "schema_version": 1, "domain_id": "tdi/public-finite-software-search",
        "upstream": {"repository": "Memorithm/TDI", "commit_id": source_commit, "contract_sha256": identity("tdi-finite-search-contract/v1", semantic)},
        "allowed_candidate_dimensions": ["implementation"],
        "data_boundary": {"generation_sources": ["public-finite/development/v1"], "verification_sources": ["public-finite/validation/v1"], "final_holdout_sources": []},
        "verification": {"adapter_id": "tdi-independent-modular-oracle/v1", "adapter_sha256": wrapper["sha256"]},
        "objectives": [{"name": "process_wall", "direction": "minimize"}, {"name": "process_peak_rss", "direction": "minimize"}],
        "environment": {"fingerprint_required": True, "isolation_required": False}}}
    spec = {"schema_version": 1, "generator_version": "forge-finite-search/v1", "manifest": manifest,
            "dimensions": [{"name": "implementation", "values": list(IMPLEMENTATIONS)}], "forbidden_combinations": [],
            "objective_units": ["ns", "bytes"], "strategy": strategy, "seed": seed,
            "budget": {"max_proposals": 3, "max_stage_attempts": 12, "max_attempts_per_stage": 2,
                       "stage_timeout_ms": 15000, "max_reserved_ms": 180000}}
    response, costs = forge.call(spec)
    record = store.create_search(spec, binding, response)
    store.search_event(record["id"], "initial-control-validation", {"cost_measurements": costs})
    return record


def _forge(record):
    b = record["binding"]["forge"]
    return ForgeClient(b["path"], b["sha256"], b["source_commit"])


def _validate_deployment(record):
    binding = record["binding"]
    for key in ("worker", "evaluator", "python", "telemetry"):
        pinned_file(binding[key]["path"], binding[key]["sha256"])
    for module in binding["support_modules"].values():
        pinned_file(module["path"], module["sha256"])


def _control(store, key, operation, *, phase="running"):
    record = store.get_search(key)
    if record["phase"] == "cancelled":
        raise durable.ContractError("search was cancelled")
    command = {"request_id": "command-" + str(len(record["response"]["checkpoint"]["commands"])), "operation": operation}
    store.search_event(key, "control-intent", {"command": command})
    try:
        response, costs = _forge(record).call(record["spec"], record["response"]["checkpoint"], command)
    except ForgeError as error:
        store.search_event(key, "control-failure", error.evidence)
        raise
    return store.update_search(key, record["sequence"], phase, response,
                               {"command": command, "cost_measurements": costs, "receipt": response["snapshot"]["receipts"][-1]})


def _root(client, evidence):
    descriptor = artifacts.canonical_artifact(evidence["descriptor"])
    binding = admission.canonical_execution_authorized_artifact_binding(evidence["binding"])
    portable_binding = binding["authoritative_artifact_binding"]["artifact_binding"]
    if (descriptor["access_class"] != "validation"
            or portable_binding["descriptor"] != descriptor
            or portable_binding["hub_artifact_id"] != evidence["hub_artifact"]
            or artifacts.artifact_identity(descriptor) != evidence["artifact_identity"]
            or artifacts.provenance_identity(evidence["provenance"]) != evidence["provenance_identity"]):
        raise durable.ContractError("search stage evidence provenance mismatch")
    raw, portable = client.download(evidence["hub_artifact"], descriptor)
    if durable.strict_json(raw) != evidence["json"]:
        raise durable.ContractError("search stage artifact differs from recorded evidence")
    return edge.bind_portable_artifact(descriptor, portable)


def _stage_evidence(store, key, candidate_id, stage):
    rows = store.db.execute("SELECT campaign FROM search_stages WHERE search=? ORDER BY rowid", (key,))
    found = None
    for row in rows:
        results = store.results(row[0])
        for item in results:
            value = item["evidence"]["json"]
            if value.get("candidate_id") == candidate_id and value.get("stage") == stage:
                found = item["evidence"]
    if found is None:
        raise durable.ContractError("missing committed prerequisite stage evidence")
    return found


def _prepare_stage(client, store, record, permit):
    binding, stage = record["binding"], permit["stage"]
    _validate_deployment(record)
    proposal = next(c["proposal"] for c in record["response"]["snapshot"]["candidates"] if c["proposal"]["candidate_id"] == permit["candidate_id"])
    external = record["spec"]["manifest"]["external_domain"]
    p = {"schema": 1, "purpose": "public-finite-software-search", "stage": stage, "candidate_id": permit["candidate_id"],
         "attempt_id": permit["attempt_id"], "implementation": proposal["parameters"]["implementation"],
         "worker_sha256": binding["worker"]["sha256"], "environment_id": binding["environment_id"],
         "verification_view": {"upstream": external["upstream"], "verification": external["verification"], "verification_source": "public-finite/validation/v1"}}
    inputs, roots = {}, {}
    for label, prerequisite in ([('compiled', 'compile')] if stage == "verify" else [('compiled', 'compile'), ('verified', 'verify')] if stage == "measure" else []):
        evidence = _stage_evidence(store, record["id"], permit["candidate_id"], prerequisite)
        root = _root(client, evidence)
        digest = root["descriptor"]["raw_sha256"]
        inputs[label], roots[digest] = {"kind": "artifact", "sha256": digest}, root
    component_id = str(uuid.uuid5(uuid.NAMESPACE_URL, identity("tdi-finite-search-component/v1", {"binding": binding, "stage": stage})))
    args = [binding["evaluator"]["path"], "--worker", binding["worker"]["path"], "--parameters", "{params}", "--output", "{output:result}"]
    for name in inputs:
        args += ["--" + name, "{input:" + name + "}"]
    manifest = {"id": component_id, "name": "tdi-finite-search-" + stage, "version": "1.0.0", "kind": "tool",
                "capabilities": [{"name": "tdi.search." + stage, "contract_version": "1.0.0", "inputs": [{"name": k} for k in inputs], "outputs": [{"name": "result"}]}],
                "execution": {"type": "process", "program": binding["python"]["path"], "args": args,
                              "outputs": [{"name": "result", "path": "result.json", "media_type": "application/json", "required": True}]},
                "metadata": {"tdi.search.deployment.sha256": identity("tdi-search-deployment/v1", binding)}}
    component = client.request("POST", "/api/v1/components", value={"schema_version": 1, "manifest": manifest})["component"]
    if component["id"] != component_id:
        raise durable.ContractError("search component registration identity mismatch")
    pin = {"component_id": component["id"], "component_version": component["version"], "component_manifest_digest": component["manifest_digest"],
           "capability": "tdi.search." + stage, "capability_contract_version": "1.0.0"}
    step = dict(pin, key=stage, component_alias="search-" + stage, parameters=p, inputs=inputs, outputs=["file:result"], after=[],
                timeout_milliseconds=permit["timeout_ms"], checkpoint={"mode": "none", "input": None, "output": None})
    graph = {"schema": 1, "semantic_version": "tdi-graph/1.0.0", "name": "search-" + permit["attempt_id"][:24],
             "root_plan_id": identity("tdi-search-stage/v1", {"search": record["id"], "permit": permit}), "max_concurrency": 1, "steps": [step],
             "hub_contract": {"repository": "Memorithm/scirust-hub", "source_commit": graphs.HUB_SOURCE_COMMIT, "workflow_schema_version": 1, "workflow_model_version": "1.2.0"}}
    spec = runtime.canonical_campaign({"schema": 1, "purpose": "development-software", "domain": "Validation", "graph": graph,
                "policy": {"trust": "trusted-software", "allowed_steps": {stage: pin}},
                "outputs": {stage: {"file:result": {"media_type": "application/json", "access_class": "validation", "cache": "disabled",
                                                   "json_fields": {"schema": 1, "status": "StageRecorded", "candidate_id": permit["candidate_id"],
                                                                   "attempt_id": permit["attempt_id"], "stage": stage}}}}})
    campaign = store.bind_search_stage(record["id"], permit["attempt_id"], spec, roots, client.endpoint)
    return {"campaign": campaign, "roots": roots}


def _execute_stage(client, store, record, permit):
    """Recover a mapping or persist one before the first Hub workflow mutation."""
    _validate_deployment(record)
    mapping = store.search_stage(record["id"], permit["attempt_id"])
    if mapping is None:
        mapping = _prepare_stage(client, store, record, permit)
    campaign = store.get(mapping["campaign"])
    runtime.submit(client, store, campaign["spec"], mapping["roots"])
    if store.get_search(record["id"])["phase"] == "cancelled":
        runtime.cancel(client, store, campaign["id"])
        raise durable.ContractError("search cancelled before execution")
    result = runtime.execute(client, store, campaign["id"])
    _validate_deployment(record)
    if result["phase"] not in ("completed", "failed", "cancelled"):
        raise durable.ContractError("stage execution still requires reconciliation")
    if result["phase"] != "completed":
        return {"op": "finish", "attempt_id": permit["attempt_id"], "wall_ms": None,
                "outcome": {"kind": "failed", "reason": "hub-stage-" + result["phase"], "execution_unknown": False}}
    evidence = store.result(campaign["id"], permit["stage"], "file:result")
    _root(client, evidence)
    value = evidence["json"]
    return {"op": "finish", "attempt_id": permit["attempt_id"], "wall_ms": value["wall_ms"], "outcome": value["outcome"]}


def run_search(client, store, key, *, max_stages=128):
    """Run/resume bounded stages; a terminal search is inspected without reevaluation.

    Pausing after `max_stages` is an explicit operational boundary, not pruning.
    Technical failures pause with all reservations retained; another resume is
    an explicit retry. Unknown Hub submission requires normal `attach` first.
    """
    if type(max_stages) is not int or not 1 <= max_stages <= 128:
        raise durable.ContractError("search stage batch must be in 1..128")
    lock_path = Path(str(store.path) + ".search-" + key + ".lock")
    graphs._sha256(key, "search identity")
    if lock_path.is_symlink():
        raise durable.ContractError("symlink search lock forbidden")
    with lock_path.open("a+b") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
        record = store.get_search(key)
        if record["binding"]["hub_endpoint"] != client.endpoint:
            raise durable.ContractError("search is bound to another Hub")
        if record["phase"] in ("completed", "cancelled"):
            return record
        restored, costs = _forge(record).call(record["spec"], record["response"]["checkpoint"])
        if restored != record["response"]:
            raise durable.ContractError("stored search projection differs from Forge replay")
        record = store.update_search(key, record["sequence"], "running", restored, {"resume": True, "cost_measurements": costs})
        completed = 0
        while completed < max_stages:
            record = store.get_search(key)
            if record["phase"] == "cancelled":
                return record
            snapshot = record["response"]["snapshot"]
            permit = snapshot["active_attempt"]
            if permit is None:
                pending = next((c for c in snapshot["candidates"] if c["status"] == "pending"), None)
                if pending is None:
                    record = _control(store, key, {"op": "ask"})
                    receipt = record["response"]["snapshot"]["receipts"][-1]
                    if receipt["status"] == "rejected":
                        return store.update_search(key, record["sequence"], "completed", record["response"], {"termination": receipt["reason"]})
                    continue
                record = _control(store, key, {"op": "begin", "candidate_id": pending["proposal"]["candidate_id"], "stage": pending["next_stage"]})
                receipt = record["response"]["snapshot"]["receipts"][-1]
                if receipt["status"] != "started":
                    return store.update_search(key, record["sequence"], "paused", record["response"], {"budget_or_prerequisite_stop": receipt})
                permit = receipt["permit"]
            # The checkpoint containing this permit has committed before this call.
            operation = _execute_stage(client, store, record, permit)
            record = _control(store, key, operation)
            completed += 1
            receipt = record["response"]["snapshot"]["receipts"][-1]
            if receipt["status"] != "finished" or operation["outcome"]["kind"] == "failed":
                return store.update_search(key, record["sequence"], "paused", record["response"], {"stage_failure": operation, "receipt": receipt})
        return store.update_search(key, record["sequence"], "paused", record["response"], {"explicit_stage_batch_limit": max_stages})


def cancel_search(client, store, key):
    """Persist cancellation only after every ambiguous Hub submission is reconciled."""
    record = store.get_search(key)
    if record["binding"]["hub_endpoint"] != client.endpoint:
        raise durable.ContractError("search is bound to another Hub")
    if record["phase"] == "completed":
        return record
    rows = store.db.execute("SELECT campaign FROM search_stages WHERE search=?", (key,)).fetchall()
    campaigns = [store.get(row[0]) for row in rows]
    if any(campaign["phase"] in ("submitting", "submission-unknown") for campaign in campaigns):
        raise durable.ContractError("ambiguous Hub submission must be attached before search cancellation")
    if record["phase"] != "cancelled":
        record = store.update_search(key, record["sequence"], "cancelled", record["response"], {"cancellation_requested": True})
    for campaign in campaigns:
        if campaign["phase"] not in ("completed", "failed", "cancelled"):
            runtime.cancel(client, store, campaign["id"])
    return store.get_search(key)
