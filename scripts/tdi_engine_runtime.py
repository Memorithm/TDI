"""Execute explicit non-final TDI graphs through the real Hub HTTP boundary.

No scheduling or lease state lives here. The catalogue records intent, Hub
admission/publication evidence and verified payload descriptors. Ambiguous
dispatch is reconciled by GET; it is never silently repeated on another Hub.
"""
from __future__ import annotations

import copy
import hashlib
import urllib.parse

import tdi_artifact_contract as artifacts
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
import tdi_hub_admission_contract as admission
import tdi_hub_edge_contract as edge
from tdi_engine_store import identity
from tdi_hub_client import HubClientError, HubTransportUnknown, entity_id

TERMINAL = {"succeeded": "completed", "failed": "failed", "cancelled": "cancelled"}
MAX_RESULT_BYTES = 256 * 1024


def canonical_campaign(spec):
    """Validate an explicit trusted-software campaign without authorizing a series.

    The operator freezes allowed component/capability identities in the policy.
    Only Development or Validation are admitted. Each returned output has an
    explicit access class, expected JSON fields and cache eligibility. This is
    infrastructure execution; protected or concrete-model stage gates remain
    the responsibility of the separately qualified scientific adapter.
    """
    required = {"schema", "purpose", "domain", "graph", "policy", "outputs"}
    if not isinstance(spec, dict) or set(spec) != required or type(spec["schema"]) is not int or spec["schema"] != 1:
        raise durable.ContractError("invalid operational campaign schema")
    if spec["purpose"] != "development-software" or spec["domain"] not in ("Development", "Validation"):
        raise durable.ContractError("only non-final software campaigns are supported")
    graph = graphs.canonical_graph(spec["graph"])
    policy = spec["policy"]
    if (not isinstance(policy, dict) or set(policy) != {"trust", "allowed_steps"}
            or policy["trust"] != "trusted-software" or not isinstance(policy["allowed_steps"], dict)):
        raise durable.ContractError("explicit trusted-software execution policy required")
    keys = {step["key"] for step in graph["steps"]}
    if set(policy["allowed_steps"]) != keys or not isinstance(spec["outputs"], dict) or set(spec["outputs"]) != keys:
        raise durable.ContractError("execution policy and output coverage must be exact")
    output_policy = {}
    for step in graph["steps"]:
        expected_pin = {name: step[name] for name in (
            "component_id", "component_version", "component_manifest_digest",
            "capability", "capability_contract_version",
        )}
        if policy["allowed_steps"][step["key"]] != expected_pin:
            raise durable.ContractError("step is not exactly permitted by execution policy")
        outputs = spec["outputs"][step["key"]]
        if not isinstance(outputs, dict) or set(outputs) != set(step["outputs"]):
            raise durable.ContractError("step output declarations must be exact")
        output_policy[step["key"]] = {}
        for name, rule in outputs.items():
            if not isinstance(rule, dict) or set(rule) != {"media_type", "access_class", "json_fields", "cache"}:
                raise durable.ContractError("invalid output policy")
            if rule["access_class"] != spec["domain"].lower():
                raise durable.ContractError("output access must preserve campaign domain")
            if (rule["media_type"] != "application/json" or not isinstance(rule["json_fields"], dict)
                    or rule["cache"] not in ("disabled", "deterministic-data")):
                raise durable.ContractError("unsupported output validation/cache policy")
            graphs._safe_json(rule["json_fields"], "expected JSON fields")
            output_policy[step["key"]][name] = copy.deepcopy(rule)
    return dict(spec, graph=graph, policy=copy.deepcopy(policy), outputs=output_policy)


def _check_endpoint(client, record):
    if client.endpoint != record["endpoint"]:
        raise durable.ContractError("campaign is bound to a different Hub endpoint")


def _bind_record(spec, response, roots):
    return admission.bind_exact_workflow_admission(spec["graph"], response, root_artifact_bindings=roots)


def _verify_roots(client, spec, roots):
    verified, ids = admission._canonical_root_artifact_bindings(spec["graph"], roots)
    for binding in verified.values():
        descriptor = binding["descriptor"]
        if descriptor["access_class"] not in ("public", spec["domain"].lower()):
            raise durable.ContractError("root artifact access is incompatible with campaign domain")
        _, portable = client.download(binding["hub_artifact_id"], descriptor)
        if edge.bind_portable_artifact(descriptor, portable) != binding:
            raise durable.ContractError("root artifact differs from its supplied receipt")
    return verified, ids


def submit(client, store, spec, roots):
    """Submit once after durable intent, then verify exact registry admission.

    `roots` maps each graph input's raw digest to a verified Hub artifact binding.
    An ambiguous or invalid response leaves a visible submission-unknown record;
    the caller must attach the existing workflow ID before any execution.
    """
    spec = canonical_campaign(spec)
    # Validate all roots before persisting a mutation intent or sending a POST.
    verified_roots, ids = _verify_roots(client, spec, roots)
    preview = graphs.compile_hub_workflow_preview(spec["graph"], artifact_bindings=ids)
    campaign = store.create(spec, client.endpoint)
    record = store.get(campaign)
    if record["phase"] != "prepared":
        return campaign
    store.event(campaign, "root-bindings", verified_roots)
    store.transition(campaign, "prepared", "submitting")
    try:
        response = client.request("POST", "/api/v1/workflows", value={
            "schema_version": 1, "workflow": preview["workflow"],
            "admission": admission._expected_admission(spec["graph"]),
        })["workflow"]
        binding = _bind_record(spec, response, verified_roots)
    except (HubClientError, ValueError, KeyError, TypeError):
        store.transition(campaign, "submitting", "submission-unknown")
        raise HubTransportUnknown("submission needs reconciliation with its existing Hub workflow id") from None
    store.transition(campaign, "submitting", "admitted", workflow=entity_id(response["id"]),
                     admission=binding, snapshot=response)
    return campaign


def attach(client, store, campaign, workflow, roots):
    """Reconcile ambiguous submission with an existing exactly matching workflow.

    This only reads the Hub. It never creates another workflow or executes it.
    """
    record = store.get(campaign)
    _check_endpoint(client, record)
    if record["phase"] not in ("submitting", "submission-unknown"):
        raise durable.ContractError("only an ambiguous submission can be attached")
    roots, _ = _verify_roots(client, canonical_campaign(record["spec"]), roots)
    workflow = entity_id(workflow)
    response = client.request("GET", f"/api/v1/workflows/{workflow}")
    if response.get("id") != workflow:
        raise durable.ContractError("Hub returned a different workflow")
    binding = _bind_record(record["spec"], response, roots)
    store.transition(campaign, record["phase"], "admitted", workflow=workflow,
                     admission=binding, snapshot=response)
    return refresh(client, store, campaign)


def execute(client, store, campaign):
    """Execute an admitted workflow once; subsequent calls reconcile via GET only."""
    record = store.get(campaign)
    _check_endpoint(client, record)
    canonical_campaign(record["spec"])
    if record["phase"] != "admitted":
        return refresh(client, store, campaign)
    binding = admission.canonical_workflow_admission_binding(record["admission"])
    workflow = entity_id(record["workflow"])
    if binding["workflow"] != workflow:
        raise durable.ContractError("stored admission/workflow mismatch")
    store.transition(campaign, "admitted", "executing")
    try:
        client.request("POST", f"/api/v1/workflows/{workflow}/executions")
    except HubClientError:
        store.event(campaign, "execution-response-unknown", {"action": "inspect existing workflow; never redispatch"})
        raise
    return refresh(client, store, campaign)


def _repair_completed_cache(store, campaign, record):
    """Retry idempotent derived cache publication before returning completion.

    The campaign terminal transition and cache rows are intentionally separate
    durability boundaries.  A crash after the former must therefore make the
    latter recoverable from already-verified result evidence on every completed
    reconciliation path.  This helper never re-fetches Hub payloads.
    """
    if record["phase"] == "completed":
        from tdi_engine_cache import publish_completed
        publish_completed(store, campaign)
        return store.get(campaign)
    return record


def refresh(client, store, campaign):
    """Read authoritative workflow state and verify declared successful outputs.

    Failed attempts remain in the Hub snapshot and catalogue; successful outputs
    from other steps remain retrievable. A technical success is not a scientific
    beneficial verdict. Pending/running snapshots do not trigger execution.
    """
    record = store.get(campaign)
    if record["phase"] == "cancelled" and record["workflow"] is None:
        return record
    if record["phase"] == "imported":
        raise durable.ContractError("imported evidence is read-only and cannot resume execution")
    _check_endpoint(client, record)
    spec = canonical_campaign(record["spec"])
    if record["workflow"] is None:
        raise durable.ContractError("campaign has no reconciled workflow")
    workflow = entity_id(record["workflow"])
    response = client.request("GET", f"/api/v1/workflows/{workflow}")
    if response.get("id") != workflow:
        raise durable.ContractError("Hub returned a different workflow")
    binding = _bind_record(spec, response, record["admission"]["root_artifact_bindings"])
    if binding != record["admission"]:
        raise durable.ContractError("workflow admission changed")
    state, actual_steps = validated_snapshot(spec, response)
    phase = TERMINAL.get(state)
    # A terminal snapshot already committed before this refresh began has crossed
    # the evidence boundary: successful outputs were verified before the terminal
    # phase was committed. Keep terminal evidence read-only; derived cache repair is retried separately.
    # Concurrent refreshes are rechecked again below after read-side verification
    # so only one durable terminal transition can win.
    if (phase is not None and record["phase"] == phase
            and record["snapshot"] is not None):
        if durable.canonical(record["snapshot"]) == durable.canonical(response):
            return _repair_completed_cache(store, campaign, record)
        raise durable.ContractError("Hub terminal workflow snapshot changed after authoritative commit")
    # Close the window between the initial catalogue read and the Hub GET before
    # performing publication/artifact reads. Another reconciler may have already
    # committed this exact terminal snapshot while this request was in flight.
    if phase is not None:
        current = store.get(campaign)
        if current["phase"] == phase and current["snapshot"] is not None:
            if durable.canonical(current["snapshot"]) == durable.canonical(response):
                return _repair_completed_cache(store, campaign, current)
            raise durable.ContractError("Hub terminal workflow snapshot changed after authoritative commit")
        if current["phase"] in set(TERMINAL.values()) and current["phase"] != phase:
            raise durable.ContractError("local terminal campaign state conflicts with Hub workflow")
    for key in sorted(actual_steps):
        step = actual_steps[key]
        if step.get("state") == "succeeded":
            _collect_step(client, store, campaign, spec, binding, step)
    if phase is None:
        phase = "cancel-requested" if record["phase"] == "cancel-requested" else (
            "admitted" if record["phase"] == "admitted" and state in ("created", "validated") else "executing")
    if phase == "admitted" and record["phase"] == "admitted":
        return record
    # A cancellation request and the execution response can race on separate
    # catalogue connections. Re-read and retry only when another writer really
    # changed the local phase between our read and compare-and-swap. This keeps
    # refresh bounded and fail-closed while allowing the same authoritative Hub
    # terminal snapshot to reconcile from `cancel-requested`.
    terminal_phases = set(TERMINAL.values())
    for _ in range(3):
        current = store.get(campaign)
        current_phase = current["phase"]
        target_phase = phase
        if target_phase == "executing" and current_phase == "cancel-requested":
            target_phase = "cancel-requested"
        # The first terminal transition is the durable authoritative campaign
        # commit. If another reconciler won while this worker was validating the
        # same terminal phase, accept it only when it committed the exact Hub
        # snapshot this worker observed. Result verification alone is insufficient
        # because failed-attempt metadata is evidence too.
        if target_phase in terminal_phases and current_phase == target_phase:
            if (current["snapshot"] is None
                    or durable.canonical(current["snapshot"]) != durable.canonical(response)):
                raise durable.ContractError(
                    "Hub terminal workflow snapshot changed after authoritative commit"
                )
            return _repair_completed_cache(store, campaign, current)
        if current_phase in terminal_phases and current_phase != target_phase:
            raise durable.ContractError("local terminal campaign state conflicts with Hub workflow")
        try:
            store.transition(campaign, current_phase, target_phase, snapshot=response)
        except durable.ContractError:
            if store.get(campaign)["phase"] == current_phase:
                raise
            continue
        phase = target_phase
        break
    else:
        raise durable.ContractError("campaign state kept changing during Hub reconciliation")
    return _repair_completed_cache(store, campaign, store.get(campaign))


def validated_snapshot(spec, response):
    """Validate workflow state and exact successful coverage, including archives."""
    if not isinstance(response, dict):
        raise durable.ContractError("invalid Hub workflow snapshot")
    state = response.get("state")
    if not isinstance(state, str) or state not in {"created", "validated", "running", "succeeded", "failed", "cancelled"}:
        raise durable.ContractError("unknown Hub workflow state")
    steps = response.get("steps", [])
    if not isinstance(steps, list) or len(steps) > len(spec["graph"]["steps"]):
        raise durable.ContractError("invalid Hub step results")
    actual_steps = {}
    expected_steps = {step["key"]: step for step in spec["graph"]["steps"]}
    for step in steps:
        if (not isinstance(step, dict) or not isinstance(step.get("key"), str)
                or step["key"] not in expected_steps or step["key"] in actual_steps):
            raise durable.ContractError("unknown or duplicate Hub step")
        actual_steps[step["key"]] = step
    if state == "succeeded" and (set(actual_steps) != set(expected_steps)
                                 or any(s.get("state") != "succeeded" for s in steps)):
        raise durable.ContractError("workflow succeeded without complete successful steps")
    return state, actual_steps


def _collect_step(client, store, campaign, spec, binding, step):
    workflow, key = binding["workflow"], step["key"]
    publication = client.request("GET", f"/api/v1/workflows/{workflow}/steps/{key}/publication")
    attempts = step.get("attempts", [])
    winners = [a for a in attempts if a.get("id") == publication.get("attempt")]
    if (len(winners) != 1 or winners[0].get("state") != "succeeded"
            or winners[0].get("run") != step.get("run")):
        raise durable.ContractError("publication does not match the successful step attempt")
    for output, rule in sorted(spec["outputs"][key].items()):
        artifact_id = entity_id(publication.get("outputs", {}).get(output))
        # Metadata checks happen before payload reads, and declared outputs only
        # are fetched (never arbitrary diagnostics or unrelated artifacts).
        meta = client.request("GET", f"/api/v1/artifacts/{artifact_id}")
        if (meta.get("id") != artifact_id or meta.get("produced_by_run") != step["run"]
                or meta.get("media_type") != rule["media_type"]
                or type(meta.get("size")) is not int or not 0 <= meta["size"] <= MAX_RESULT_BYTES):
            raise durable.ContractError("result metadata/producer mismatch")
        raw, portable = client.download(artifact_id)
        descriptor = {"schema": 1, "name": output, "raw_sha256": hashlib.sha256(raw).hexdigest(),
                      "size_bytes": len(raw), "media_type": rule["media_type"], "access_class": rule["access_class"]}
        value = durable.strict_json(raw, max_bytes=MAX_RESULT_BYTES)
        if not isinstance(value, dict):
            raise durable.ContractError("result must be a JSON object")
        for field, expected in rule["json_fields"].items():
            if field not in value or not admission._json_equal(value[field], expected):
                raise durable.ContractError("result JSON field does not match declared expectation")
        portable_binding = edge.bind_portable_artifact(descriptor, portable)
        authoritative = edge.bind_authoritative_publication(
            portable_binding, publication, expected_workflow=workflow,
            expected_step_key=key, expected_output=output,
        )
        combined = admission.bind_execution_authorized_artifact(authoritative, binding)
        provenance = artifacts.canonical_provenance({
            "schema": 1, "artifact_identity": artifacts.artifact_identity(descriptor),
            "experiment_id": spec["graph"]["root_plan_id"], "plan_id": graphs.graph_identity(spec["graph"]),
            "trial_id": identity("tdi-hub-step/v1", {"workflow": workflow, "step": key}),
            "attempt_id": identity("tdi-hub-attempt/v1", publication["attempt"]),
            "step_key": key, "step_identity": graphs.step_identity(spec["graph"], key),
            "implementation_identity": expected_component(spec, key), "domain": spec["domain"],
            "inputs": [{"name": "graph", "identity": graphs.graph_identity(spec["graph"])}],
            "dependencies": [{"name": "admission", "identity": admission.workflow_admission_binding_identity(binding)}],
        })
        store.put_result(campaign, key, output, {
            "descriptor": descriptor, "artifact_identity": artifacts.artifact_identity(descriptor),
            "provenance": provenance, "provenance_identity": artifacts.provenance_identity(provenance),
            "binding": combined, "hub_artifact": artifact_id,
            "cache_eligible": rule["cache"] == "deterministic-data", "json": value,
        })


def expected_component(spec, key):
    """Return the exact manifest identity of a permitted graph step."""
    return spec["policy"]["allowed_steps"][key]["component_manifest_digest"]


def cancel(client, store, campaign):
    """Request Hub cancellation durably; cleanup acknowledgement comes from Hub state."""
    record = store.get(campaign)
    _check_endpoint(client, record)
    if record["phase"] == "imported":
        raise durable.ContractError("imported evidence cannot cancel its original workflow")
    if record["phase"] in TERMINAL.values():
        return record
    if record["phase"] == "prepared":
        store.transition(campaign, "prepared", "cancelled")
        return store.get(campaign)
    workflow = entity_id(record["workflow"])
    if record["phase"] == "cancel-requested" and store.has_event(campaign, "cancel-response-unknown"):
        return refresh(client, store, campaign)
    if record["phase"] != "cancel-requested":
        store.transition(campaign, record["phase"], "cancel-requested")
    try:
        client.request("POST", f"/api/v1/workflows/{workflow}/cancel")
    except HubTransportUnknown:
        store.event(campaign, "cancel-response-unknown", {
            "action": "inspect existing workflow; never re-cancel after ambiguous response",
        })
        raise
    return refresh(client, store, campaign)
