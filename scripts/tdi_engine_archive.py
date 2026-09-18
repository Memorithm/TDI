"""Portable, bounded, independently verifiable TDI evidence bundles.

Archives contain only declared result artifacts. They are not executable
backups of components or datasets. Hashes prove internal integrity; an external
trusted archive identity is needed to detect complete malicious replacement.
"""
from __future__ import annotations

import base64
import binascii

import tdi_artifact_contract as artifacts
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
import tdi_hub_admission_contract as admission
import tdi_engine_runtime as runtime
from tdi_engine_store import identity, atomic_json, reject_restricted_reference_metadata

MAX_ARCHIVE_BYTES = 16 * 1024 * 1024
MAX_MEMBERS = 4096


def _expected_provenance(spec, bound, source, step_key, artifact_identity):
    """Recompute the exact lineage emitted by the operational collector."""
    graph_identity = graphs.graph_identity(spec["graph"])
    return artifacts.canonical_provenance({
        "schema": 1,
        "artifact_identity": artifact_identity,
        "experiment_id": spec["graph"]["root_plan_id"],
        "plan_id": graph_identity,
        "trial_id": identity(
            "tdi-hub-step/v1", {"workflow": bound["workflow"], "step": step_key}
        ),
        "attempt_id": identity("tdi-hub-attempt/v1", source["attempt"]),
        "step_key": step_key,
        "step_identity": graphs.step_identity(spec["graph"], step_key),
        "implementation_identity": runtime.expected_component(spec, step_key),
        "domain": spec["domain"],
        "inputs": [{"name": "graph", "identity": graph_identity}],
        "dependencies": [{
            "name": "admission",
            "identity": admission.workflow_admission_binding_identity(bound),
        }],
    })


def verify_bundle(bundle, *, expected_identity=None):
    """Verify a complete archive and return decoded payloads without network I/O.

    Example: ``verify_bundle(bundle, expected_identity=trusted_receipt)``.
    Declared descriptors, provenance, admission, publication, JSON contents and
    successful-step coverage must all agree. Protected payloads are rejected.
    """
    if (not isinstance(bundle, dict) or set(bundle) != {"schema", "kind", "campaign", "members", "identity"}
            or type(bundle["schema"]) is not int or bundle["schema"] != 1 or bundle["kind"] != "tdi-evidence-bundle"):
        raise durable.ContractError("invalid evidence bundle schema")
    body = {k: v for k, v in bundle.items() if k != "identity"}
    found = identity("tdi-evidence-bundle/v1", body)
    if bundle["identity"] != found or (expected_identity is not None and found != expected_identity):
        raise durable.ContractError("bundle identity mismatch")
    if len(durable.canonical(bundle).encode()) > MAX_ARCHIVE_BYTES:
        raise durable.ContractError("bundle byte budget exceeded")
    record = bundle["campaign"]
    required = {"id", "spec", "endpoint", "phase", "workflow", "admission", "snapshot", "created_ns"}
    if not isinstance(record, dict) or set(record) != required:
        raise durable.ContractError("invalid archived campaign record")
    spec = runtime.canonical_campaign(record["spec"])
    if (identity("tdi-operational-campaign/v1", spec) != record["id"]
            or record["phase"] not in ("completed", "failed", "cancelled", "imported")
            or type(record["created_ns"]) is not int or record["created_ns"] < 0):
        raise durable.ContractError("archived campaign identity/state mismatch")
    bound = admission.canonical_workflow_admission_binding(record["admission"])
    if bound["graph"] != spec["graph"] or bound["workflow"] != record["workflow"]:
        raise durable.ContractError("archived admission mismatch")
    if runtime._bind_record(spec, record["snapshot"], bound["root_artifact_bindings"]) != bound:
        raise durable.ContractError("archived workflow snapshot mismatch")
    state, steps = runtime.validated_snapshot(spec, record["snapshot"])
    if state not in runtime.TERMINAL or record["phase"] not in ("imported", runtime.TERMINAL[state]):
        raise durable.ContractError("archive requires a matching terminal workflow")
    members = bundle["members"]
    if not isinstance(members, list) or len(members) > MAX_MEMBERS:
        raise durable.ContractError("bundle member budget exceeded")
    successful = {key for key, step in steps.items() if step.get("state") == "succeeded"}
    expected = {(step, name) for step in successful for name in spec["outputs"][step]}
    observed, payloads = set(), {}
    for member in members:
        if not isinstance(member, dict) or set(member) != {"step", "output", "evidence", "base64"}:
            raise durable.ContractError("invalid bundle member")
        if not isinstance(member["step"], str) or not isinstance(member["output"], str):
            raise durable.ContractError("invalid bundle member labels")
        key = (member["step"], member["output"])
        if key not in expected or key in observed:
            raise durable.ContractError("unknown or duplicate bundle member")
        observed.add(key)
        evidence = member["evidence"]
        if (not isinstance(evidence, dict) or set(evidence) != {"descriptor", "artifact_identity", "provenance", "provenance_identity", "binding", "hub_artifact", "cache_eligible", "json"}
                or type(evidence["cache_eligible"]) is not bool):
            raise durable.ContractError("invalid result evidence schema")
        descriptor = artifacts.canonical_artifact(evidence["descriptor"])
        if (descriptor["access_class"] != spec["domain"].lower()
                or descriptor["size_bytes"] > runtime.MAX_RESULT_BYTES):
            raise durable.ContractError("bundle member access mismatch")
        try:
            raw = base64.b64decode(member["base64"], validate=True)
        except (binascii.Error, ValueError, TypeError):
            raise durable.ContractError("invalid artifact base64") from None
        artifacts.verify_artifact_bytes(descriptor, raw)
        if artifacts.artifact_identity(descriptor) != evidence["artifact_identity"]:
            raise durable.ContractError("archived artifact identity mismatch")
        provenance = artifacts.canonical_provenance(evidence["provenance"])
        if (artifacts.provenance_identity(provenance) != evidence["provenance_identity"]
                or provenance["artifact_identity"] != evidence["artifact_identity"]):
            raise durable.ContractError("archived provenance mismatch")
        binding = admission.canonical_execution_authorized_artifact_binding(evidence["binding"])
        source = binding["authoritative_artifact_binding"]
        if (binding["workflow_admission_binding"] != bound or binding["step_key"] != member["step"]
                or source["output_label"] != member["output"] or source["hub_artifact_id"] != evidence["hub_artifact"]
                or source["artifact_binding"]["descriptor"] != descriptor):
            raise durable.ContractError("archived output publication mismatch")
        step = steps[member["step"]]
        attempts = step.get("attempts", [])
        if not isinstance(attempts, list):
            raise durable.ContractError("archived attempts must be a list")
        winners = [a for a in attempts if isinstance(a, dict) and a.get("id") == source["attempt"]]
        if (len(winners) != 1 or winners[0].get("state") != "succeeded"
                or winners[0].get("run") != step.get("run")):
            raise durable.ContractError("archived publication attempt mismatch")
        rule = spec["outputs"][member["step"]][member["output"]]
        if (descriptor["name"] != member["output"] or descriptor["media_type"] != rule["media_type"]
                or evidence["cache_eligible"] != (rule["cache"] == "deterministic-data")):
            raise durable.ContractError("archived output policy mismatch")
        if provenance != _expected_provenance(
            spec, bound, source, member["step"], evidence["artifact_identity"]
        ):
            raise durable.ContractError("archived provenance policy mismatch")
        if not admission._json_equal(durable.strict_json(raw), evidence["json"]):
            raise durable.ContractError("archived JSON projection mismatch")
        if not isinstance(evidence["json"], dict) or any(
                field not in evidence["json"] or not admission._json_equal(evidence["json"][field], expected_value)
                for field, expected_value in rule["json_fields"].items()):
            raise durable.ContractError("archived output expectation mismatch")
        payloads[key] = raw
    if observed != expected:
        raise durable.ContractError("incomplete evidence bundle")
    return payloads


def export_bundle(client, store, campaign, destination):
    """Download only selected result references and atomically publish an archive."""
    record = store.get(campaign)
    if record["endpoint"] != client.endpoint:
        raise durable.ContractError("export Hub endpoint mismatch")
    if record["phase"] not in ("completed", "failed", "cancelled", "imported"):
        raise durable.ContractError("export requires a terminal or imported campaign")
    members = []
    used = 0
    offset = 0
    while True:
        page = store.results(campaign, after=offset)
        if not page:
            break
        for result in page:
            evidence = result["evidence"]
            location = store.location(campaign, evidence["artifact_identity"])
            hub_id = location["hub_artifact_id"] if location else evidence["hub_artifact"]
            raw, _ = client.download(hub_id, evidence["descriptor"])
            item = dict(result, base64=base64.b64encode(raw).decode("ascii"))
            used += len(durable.canonical(item).encode())
            if used > MAX_ARCHIVE_BYTES or len(members) >= MAX_MEMBERS:
                raise durable.ContractError("bundle budget exceeded")
            members.append(item)
        offset += len(page)
    bundle = {"schema": 1, "kind": "tdi-evidence-bundle", "campaign": record, "members": members}
    bundle["identity"] = identity("tdi-evidence-bundle/v1", bundle)
    verify_bundle(bundle)
    atomic_json(destination, bundle)
    return {"identity": bundle["identity"], "members": len(members)}


def restore_bundle(client, store, bundle, *, expected_identity=None):
    """Restore permitted payloads to Hub and import original provenance unchanged.

    New Hub UUIDs are separate transfer receipts. A failed transfer does not
    create a completed catalogue import; already uploaded immutable blobs may
    remain unreferenced in Hub. Import never starts scientific execution.
    """
    payloads = verify_bundle(bundle, expected_identity=expected_identity)
    # Reject tagged restricted metadata before the first external Hub mutation.
    # Store-level rejection remains a second line of defence for direct callers.
    reject_restricted_reference_metadata(bundle["campaign"], "bundle restore")
    locations, results = {}, []
    for member in bundle["members"]:
        evidence = member["evidence"]
        artifact = evidence["artifact_identity"]
        if artifact not in locations:
            locations[artifact] = client.upload(evidence["descriptor"], payloads[(member["step"], member["output"])])
        results.append({key: member[key] for key in ("step", "output", "evidence")})
    return store.restore(bundle["campaign"], results, locations, client.endpoint)