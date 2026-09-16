"""Fail-closed TDI binding for exact scirust-hub workflow admission.

TDI owns the scientific Graph/v1 identities and authorization semantics. Hub
owns registry resolution, workflow scheduling, leases, retries, transport and
publication. This adapter consumes a Hub workflow record only after Hub has
persisted and enforced exact registry admission pins; it does not submit work,
resolve registry state, schedule nodes, mint leases/fences or publish artifacts.

The qualified Hub source is the squash merge of scirust-hub PR #54. That
revision enforces exact component version, manifest digest and capability
contract version for every workflow step before any attempt can execute, and
exposes the declarative WorkflowSpec together with those pins in WorkflowDto.
"""
from __future__ import annotations

import hashlib
import copy

import tdi_execution_graph as execution_graph
import tdi_experiment_contract as experiment
import tdi_hub_edge_contract as hub_edge

HUB_WORKFLOW_ADMISSION_BINDING_SCHEMA = 1
HUB_EXECUTION_ARTIFACT_BINDING_SCHEMA = 3
HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION = 1
HUB_WORKFLOW_MODEL_VERSION = "1.2.0"
HUB_REPOSITORY = "Memorithm/scirust-hub"
PINNED_HUB_EXACT_ADMISSION_SOURCE = "7187d72e025e0a7a88b52d9fc4675bae26581f76"

_HEX40 = hub_edge._HEX40
_HEX64 = hub_edge._HEX64
_WORKFLOW_REQUIRED_FIELDS = {
    "id",
    "name",
    "spec",
    "state",
    "admission",
    "model_version",
    "created_at",
}
_WORKFLOW_OPTIONAL_FIELDS = {
    "started_at",
    "finished_at",
    "cancel_requested_at",
    "steps",
    "failure",
}


class HubAdmissionContractError(ValueError):
    """A Hub workflow admission record or its TDI binding is invalid."""


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise HubAdmissionContractError(f"{name} has unknown or missing fields")
    return value


def _canonical_uuid(value, name):
    try:
        return hub_edge._canonical_uuid(value, name)
    except hub_edge.HubEdgeContractError as exc:
        raise HubAdmissionContractError(str(exc)) from exc


def _source_sha(value):
    if not isinstance(value, str) or _HEX40.fullmatch(value) is None:
        raise HubAdmissionContractError("Hub exact-admission source SHA must be lowercase 40-hex")
    if value != PINNED_HUB_EXACT_ADMISSION_SOURCE:
        raise HubAdmissionContractError("Hub exact-admission source is not the qualified revision")
    return value


def _canonical_workflow_record_projection(response):
    """Validate the pinned WorkflowDto surface needed for admission evidence.

    Runtime state/timestamps/step results are deliberately outside the returned
    identity projection. They remain Hub-owned operational facts and do not
    affect whether exact registry identities were admitted for the workflow.
    """
    if not isinstance(response, dict):
        raise HubAdmissionContractError("Hub workflow response must be an object")
    fields = set(response)
    missing = _WORKFLOW_REQUIRED_FIELDS - fields
    unknown = fields - (_WORKFLOW_REQUIRED_FIELDS | _WORKFLOW_OPTIONAL_FIELDS)
    if missing or unknown:
        raise HubAdmissionContractError(
            f"Hub workflow response has missing={sorted(missing)!r}, unknown={sorted(unknown)!r} fields"
        )
    workflow = _canonical_uuid(response["id"], "Hub workflow id")
    if not isinstance(response["name"], str):
        raise HubAdmissionContractError("Hub workflow name must be a string")
    if not isinstance(response["spec"], dict):
        raise HubAdmissionContractError("Hub workflow response must expose the declarative spec")
    if not isinstance(response["admission"], dict):
        raise HubAdmissionContractError("Hub workflow response must expose exact admission pins")
    if response["model_version"] != HUB_WORKFLOW_MODEL_VERSION:
        raise HubAdmissionContractError("Hub workflow model version mismatch")
    if type(response["created_at"]) is not int or response["created_at"] < 0:
        raise HubAdmissionContractError("Hub workflow created_at must be a non-negative integer")
    return {
        "workflow": workflow,
        "name": response["name"],
        "spec": response["spec"],
        "admission": response["admission"],
        "model_version": HUB_WORKFLOW_MODEL_VERSION,
    }


def _expected_admission(graph):
    value = execution_graph.canonical_graph(graph)
    return {
        "schema_version": HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION,
        "steps": {
            step["key"]: {
                "component_version": step["component_version"],
                "manifest_digest": step["component_manifest_digest"],
                "capability_contract_version": step["capability_contract_version"],
            }
            for step in value["steps"]
        },
    }


def _canonical_admission(admission, expected):
    _exact(admission, {"schema_version", "steps"}, "Hub workflow admission")
    if (
        type(admission["schema_version"]) is not int
        or admission["schema_version"] != HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION
    ):
        raise HubAdmissionContractError("unsupported Hub workflow admission schema version")
    if not isinstance(admission["steps"], dict):
        raise HubAdmissionContractError("Hub workflow admission steps must be an object")
    expected_keys = set(expected["steps"])
    actual_keys = set(admission["steps"])
    if actual_keys != expected_keys:
        raise HubAdmissionContractError(
            f"Hub workflow admission step coverage mismatch; missing={sorted(expected_keys - actual_keys)!r}, "
            f"extra={sorted(actual_keys - expected_keys)!r}"
        )
    canonical_steps = {}
    for key in sorted(expected_keys):
        pin = admission["steps"][key]
        _exact(
            pin,
            {"component_version", "manifest_digest", "capability_contract_version"},
            f"Hub workflow admission pin {key!r}",
        )
        expected_pin = expected["steps"][key]
        if pin != expected_pin:
            raise HubAdmissionContractError(f"Hub workflow admission pin {key!r} does not match TDI Graph/v1")
        canonical_steps[key] = dict(expected_pin)
    return {
        "schema_version": HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION,
        "steps": canonical_steps,
    }


def _root_artifact_digests(graph_value):
    digests = set()
    for step in graph_value["steps"]:
        for source in step["inputs"].values():
            if source["kind"] == "artifact":
                digests.add(source["sha256"])
    return digests


def _canonical_root_artifact_bindings(graph_value, bindings):
    """Prove every Graph/v1 root digest maps to the exact Hub artifact used.

    The existing G1 HubArtifactBinding/v1 is the qualified digest-translation
    evidence. G3 accepts no bare digest→UUID map because such a map cannot prove
    that the Hub artifact selected for WorkflowSpec actually has Graph/v1's
    declared portable SHA-256.
    """
    if not isinstance(bindings, dict):
        raise HubAdmissionContractError("root_artifact_bindings must be an object")
    if any(not isinstance(key, str) for key in bindings):
        raise HubAdmissionContractError("root_artifact_bindings keys must be SHA-256 strings")
    expected = _root_artifact_digests(graph_value)
    actual = set(bindings)
    if actual != expected:
        raise HubAdmissionContractError(
            f"root artifact binding coverage mismatch; missing={sorted(expected - actual)!r}, "
            f"extra={sorted(actual - expected)!r}"
        )
    canonical = {}
    artifact_ids = {}
    for digest in sorted(expected):
        if not isinstance(digest, str) or _HEX64.fullmatch(digest) is None:
            raise HubAdmissionContractError("Graph/v1 root artifact digest must be lowercase SHA-256")
        try:
            binding = hub_edge.canonical_hub_artifact_binding(bindings[digest])
        except hub_edge.HubEdgeContractError as exc:
            raise HubAdmissionContractError(str(exc)) from exc
        if binding["descriptor"]["raw_sha256"] != digest:
            raise HubAdmissionContractError(
                "root HubArtifactBinding/v1 digest does not match Graph/v1 root artifact digest"
            )
        canonical[digest] = binding
        artifact_ids[digest] = binding["hub_artifact_id"]
    return canonical, artifact_ids


def _json_equal(left, right):
    """Compare JSON with type fidelity (`1`, `1.0`, and `true` are distinct)."""
    try:
        return experiment.canonical(left) == experiment.canonical(right)
    except experiment.ExperimentContractError as exc:
        raise HubAdmissionContractError(str(exc)) from exc


def bind_exact_workflow_admission(graph, workflow_response, *, root_artifact_bindings):
    """Bind Graph/v1 to one Hub workflow admitted with exact registry pins.

    The full declarative WorkflowSpec must equal TDI's structural compilation,
    including component ids/capabilities, parameters, inputs, dependencies and
    limits. Every root artifact mapping must be proven by qualified G1
    HubArtifactBinding/v1 evidence. The separately persisted admission envelope
    must then match every Graph/v1 component version, manifest digest and
    capability-contract version.

    ``execution_authorized`` means only that the pinned Hub boundary admitted
    this exact workflow for execution. It never authorizes a TDI scientific
    stage, holdout, claim or verdict.
    """
    try:
        graph_value = execution_graph.canonical_graph(graph)
    except execution_graph.ExecutionGraphError as exc:
        raise HubAdmissionContractError(str(exc)) from exc
    root_bindings, artifact_ids = _canonical_root_artifact_bindings(
        graph_value, root_artifact_bindings
    )
    try:
        preview = execution_graph.compile_hub_workflow_preview(
            graph_value,
            artifact_bindings=artifact_ids,
        )
    except execution_graph.ExecutionGraphError as exc:
        raise HubAdmissionContractError(str(exc)) from exc
    response = _canonical_workflow_record_projection(workflow_response)
    if response["name"] != graph_value["name"]:
        raise HubAdmissionContractError("Hub workflow name does not match TDI Graph/v1")
    # Hub omits empty `after` arrays when serializing its real WorkflowDto.
    # Restore only that documented default, preserving JSON type fidelity and
    # rejecting every other semantic difference (including explicit retries).
    wire_spec = copy.deepcopy(response["spec"])
    if isinstance(wire_spec.get("steps"), list):
        for wire_step in wire_spec["steps"]:
            if isinstance(wire_step, dict) and "after" not in wire_step:
                wire_step["after"] = []
    if not _json_equal(wire_spec, preview["workflow"]):
        raise HubAdmissionContractError("Hub workflow spec does not exactly match TDI Graph/v1 compilation")
    admission = _canonical_admission(response["admission"], _expected_admission(graph_value))
    return {
        "schema": HUB_WORKFLOW_ADMISSION_BINDING_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_source_sha": PINNED_HUB_EXACT_ADMISSION_SOURCE,
        "workflow": response["workflow"],
        "workflow_model_version": HUB_WORKFLOW_MODEL_VERSION,
        "admission_schema_version": HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION,
        "graph_identity": execution_graph.graph_identity(graph_value),
        "workflow_spec_identity": _digest("tdi-hub-workflow-spec/v1", preview["workflow"]),
        "admission_identity": _digest("tdi-hub-workflow-admission/v1", admission),
        "graph": graph_value,
        "workflow_spec": preview["workflow"],
        "admission": admission,
        "root_artifact_bindings": root_bindings,
        "admitted_steps": sorted(admission["steps"]),
        "execution_authorized": True,
        "scientific_stage_authorized": False,
    }


def canonical_workflow_admission_binding(binding):
    """Recompute HubWorkflowAdmissionBinding/v1 evidence before accepting it."""
    _exact(
        binding,
        {
            "schema",
            "hub_repository",
            "hub_source_sha",
            "workflow",
            "workflow_model_version",
            "admission_schema_version",
            "graph_identity",
            "workflow_spec_identity",
            "admission_identity",
            "graph",
            "workflow_spec",
            "admission",
            "root_artifact_bindings",
            "admitted_steps",
            "execution_authorized",
            "scientific_stage_authorized",
        },
        "Hub workflow admission binding",
    )
    if (
        type(binding["schema"]) is not int
        or binding["schema"] != HUB_WORKFLOW_ADMISSION_BINDING_SCHEMA
    ):
        raise HubAdmissionContractError("unsupported Hub workflow admission binding schema")
    if binding["hub_repository"] != HUB_REPOSITORY:
        raise HubAdmissionContractError("Hub repository pin mismatch")
    source = _source_sha(binding["hub_source_sha"])
    workflow = _canonical_uuid(binding["workflow"], "Hub workflow id")
    if binding["workflow_model_version"] != HUB_WORKFLOW_MODEL_VERSION:
        raise HubAdmissionContractError("Hub workflow model version mismatch")
    if (
        type(binding["admission_schema_version"]) is not int
        or binding["admission_schema_version"] != HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION
    ):
        raise HubAdmissionContractError("Hub workflow admission schema version mismatch")

    try:
        graph_value = execution_graph.canonical_graph(binding["graph"])
    except execution_graph.ExecutionGraphError as exc:
        raise HubAdmissionContractError(str(exc)) from exc
    root_bindings, artifact_ids = _canonical_root_artifact_bindings(
        graph_value, binding["root_artifact_bindings"]
    )
    try:
        preview = execution_graph.compile_hub_workflow_preview(
            graph_value,
            artifact_bindings=artifact_ids,
        )
    except execution_graph.ExecutionGraphError as exc:
        raise HubAdmissionContractError(str(exc)) from exc
    if not _json_equal(binding["workflow_spec"], preview["workflow"]):
        raise HubAdmissionContractError("embedded Hub workflow spec does not match embedded TDI Graph/v1")
    admission = _canonical_admission(binding["admission"], _expected_admission(graph_value))

    graph_identity = execution_graph.graph_identity(graph_value)
    workflow_spec_identity = _digest("tdi-hub-workflow-spec/v1", preview["workflow"])
    admission_identity = _digest("tdi-hub-workflow-admission/v1", admission)
    expected_identities = {
        "graph_identity": graph_identity,
        "workflow_spec_identity": workflow_spec_identity,
        "admission_identity": admission_identity,
    }
    for field, expected in expected_identities.items():
        value = binding[field]
        if not isinstance(value, str) or _HEX64.fullmatch(value) is None:
            raise HubAdmissionContractError(f"{field} must be lowercase SHA-256")
        if value != expected:
            raise HubAdmissionContractError(f"{field} does not match embedded admission evidence")

    expected_steps = sorted(admission["steps"])
    if binding["admitted_steps"] != expected_steps:
        raise HubAdmissionContractError("admitted_steps does not match embedded admission evidence")
    for step in expected_steps:
        try:
            hub_edge._hub_name(step, "admitted workflow step")
        except hub_edge.HubEdgeContractError as exc:
            raise HubAdmissionContractError(str(exc)) from exc
    if binding["execution_authorized"] is not True:
        raise HubAdmissionContractError("exact Hub admission binding requires execution_authorized=true")
    if binding["scientific_stage_authorized"] is not False:
        raise HubAdmissionContractError("Hub admission evidence never authorizes a scientific stage")
    return {
        "schema": HUB_WORKFLOW_ADMISSION_BINDING_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_source_sha": source,
        "workflow": workflow,
        "workflow_model_version": HUB_WORKFLOW_MODEL_VERSION,
        "admission_schema_version": HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION,
        "graph_identity": graph_identity,
        "workflow_spec_identity": workflow_spec_identity,
        "admission_identity": admission_identity,
        "graph": graph_value,
        "workflow_spec": preview["workflow"],
        "admission": admission,
        "root_artifact_bindings": root_bindings,
        "admitted_steps": expected_steps,
        "execution_authorized": True,
        "scientific_stage_authorized": False,
    }


def workflow_admission_binding_identity(binding):
    value = canonical_workflow_admission_binding(binding)
    return _digest("tdi-hub-workflow-admission-binding/v1", value)


def bind_execution_authorized_artifact(authoritative_artifact_binding, workflow_admission_binding):
    """Combine Hub publication authority with exact workflow execution admission.

    Both pieces must name the same workflow and the publication's producing step
    must be one of the exactly admitted steps. This is still infrastructure
    evidence only; scientific stage authorization remains hard-false.
    """
    try:
        artifact_binding = hub_edge.canonical_authoritative_hub_artifact_binding(
            authoritative_artifact_binding
        )
    except hub_edge.HubEdgeContractError as exc:
        raise HubAdmissionContractError(str(exc)) from exc
    admission_binding = canonical_workflow_admission_binding(workflow_admission_binding)
    if artifact_binding["workflow"] != admission_binding["workflow"]:
        raise HubAdmissionContractError("artifact publication and workflow admission name different workflows")
    if artifact_binding["step_key"] not in admission_binding["admitted_steps"]:
        raise HubAdmissionContractError("artifact publication step is not covered by exact workflow admission")
    return {
        "schema": HUB_EXECUTION_ARTIFACT_BINDING_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_admission_source_sha": PINNED_HUB_EXACT_ADMISSION_SOURCE,
        "workflow": artifact_binding["workflow"],
        "step_key": artifact_binding["step_key"],
        "authoritative_artifact_binding_identity": hub_edge.authoritative_hub_artifact_binding_identity(
            artifact_binding
        ),
        "workflow_admission_binding_identity": workflow_admission_binding_identity(admission_binding),
        "authoritative_artifact_binding": artifact_binding,
        "workflow_admission_binding": admission_binding,
        "execution_authorized": True,
        "publication_authoritative": True,
        "scientific_stage_authorized": False,
    }


def canonical_execution_authorized_artifact_binding(binding):
    """Validate the combined G3 execution+publication evidence binding."""
    _exact(
        binding,
        {
            "schema",
            "hub_repository",
            "hub_admission_source_sha",
            "workflow",
            "step_key",
            "authoritative_artifact_binding_identity",
            "workflow_admission_binding_identity",
            "authoritative_artifact_binding",
            "workflow_admission_binding",
            "execution_authorized",
            "publication_authoritative",
            "scientific_stage_authorized",
        },
        "execution-authorized Hub artifact binding",
    )
    if binding["schema"] != HUB_EXECUTION_ARTIFACT_BINDING_SCHEMA:
        raise HubAdmissionContractError("unsupported execution-authorized artifact binding schema")
    if binding["hub_repository"] != HUB_REPOSITORY:
        raise HubAdmissionContractError("Hub repository pin mismatch")
    source = _source_sha(binding["hub_admission_source_sha"])
    try:
        artifact_binding = hub_edge.canonical_authoritative_hub_artifact_binding(
            binding["authoritative_artifact_binding"]
        )
    except hub_edge.HubEdgeContractError as exc:
        raise HubAdmissionContractError(str(exc)) from exc
    admission_binding = canonical_workflow_admission_binding(binding["workflow_admission_binding"])
    workflow = _canonical_uuid(binding["workflow"], "Hub workflow id")
    if workflow != artifact_binding["workflow"] or workflow != admission_binding["workflow"]:
        raise HubAdmissionContractError("combined Hub binding workflow mismatch")
    if binding["step_key"] != artifact_binding["step_key"]:
        raise HubAdmissionContractError("combined Hub binding step mismatch")
    if binding["step_key"] not in admission_binding["admitted_steps"]:
        raise HubAdmissionContractError("combined Hub binding step is not admitted")
    expected_artifact_identity = hub_edge.authoritative_hub_artifact_binding_identity(artifact_binding)
    expected_admission_identity = workflow_admission_binding_identity(admission_binding)
    if binding["authoritative_artifact_binding_identity"] != expected_artifact_identity:
        raise HubAdmissionContractError("authoritative artifact binding identity mismatch")
    if binding["workflow_admission_binding_identity"] != expected_admission_identity:
        raise HubAdmissionContractError("workflow admission binding identity mismatch")
    if binding["execution_authorized"] is not True:
        raise HubAdmissionContractError("combined exact-admission binding requires execution_authorized=true")
    if binding["publication_authoritative"] is not True:
        raise HubAdmissionContractError("combined binding requires authoritative publication")
    if binding["scientific_stage_authorized"] is not False:
        raise HubAdmissionContractError("combined Hub evidence never authorizes a scientific stage")
    return {
        "schema": HUB_EXECUTION_ARTIFACT_BINDING_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_admission_source_sha": source,
        "workflow": workflow,
        "step_key": artifact_binding["step_key"],
        "authoritative_artifact_binding_identity": expected_artifact_identity,
        "workflow_admission_binding_identity": expected_admission_identity,
        "authoritative_artifact_binding": artifact_binding,
        "workflow_admission_binding": admission_binding,
        "execution_authorized": True,
        "publication_authoritative": True,
        "scientific_stage_authorized": False,
    }


def execution_authorized_artifact_binding_identity(binding):
    value = canonical_execution_authorized_artifact_binding(binding)
    return _digest("tdi-hub-execution-authorized-artifact-binding/v3", value)
