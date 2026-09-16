"""Versioned, fail-closed contracts for TDI cross-repository partner adapters.

This module is the Lot-H common adapter boundary. It records which qualified
partner repository/source/protocol a TDI Graph/v1 step intends to use and binds
that descriptor to an exact Hub-admitted workflow step. It does not invoke the
partner, schedule work, read protected holdouts, authorize a scientific stage,
publish a scientific verdict, or grant runtime actuation authority.

Partner-specific payload semantics remain owned by their repositories and must
be qualified in later Lot-H slices before ``partner_execution_qualified`` can
ever become true.
"""
from __future__ import annotations

import hashlib
import re

import tdi_artifact_contract as artifact
import tdi_execution_graph as execution_graph
import tdi_experiment_contract as experiment
import tdi_hub_admission_contract as admission
import tdi_hub_edge_contract as hub_edge

PARTNER_ADAPTER_SCHEMA = 1
ADMITTED_PARTNER_STEP_SCHEMA = 1
MAX_CAPABILITIES = 128
MAX_CONTRACTS = 128
MAX_TEXT_BYTES = 512

_HEX40 = re.compile(r"[0-9a-f]{40}\Z")
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")
_CAPABILITY = re.compile(r"[a-z0-9][a-z0-9._:/-]{0,127}\Z")
_CONTRACT_NAME = re.compile(r"[A-Za-z0-9][A-Za-z0-9._:/+-]{0,127}\Z")

_PARTNERS = {
    "elasticxxx": {"repository": "Memorithm/ElasticXxx", "role": "resource-control"},
    "forge": {"repository": "Memorithm/Forge", "role": "candidate-search"},
    "scirust": {"repository": "Memorithm/scirust", "role": "math-primitives"},
    "flat-attention": {"repository": "Memorithm/FLAT-ATTENTION", "role": "attention-execution"},
    "nnis": {"repository": "Memorithm/NNIS", "role": "hardware-qualification"},
}

_REQUIRED_PERMISSIONS = {
    "read_protected_holdout": False,
    "authorize_scientific_stage": False,
    "publish_scientific_verdict": False,
    "actuate_runtime": False,
}


class PartnerAdapterContractError(ValueError):
    """A partner adapter descriptor or admitted-step binding is invalid."""


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise PartnerAdapterContractError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, max_bytes=MAX_TEXT_BYTES):
    if not isinstance(value, str) or not value.strip():
        raise PartnerAdapterContractError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > max_bytes:
        raise PartnerAdapterContractError(f"{name} exceeds the text bound")
    return value


def _sha40(value, name):
    if not isinstance(value, str) or _HEX40.fullmatch(value) is None:
        raise PartnerAdapterContractError(f"{name} must be lowercase 40-hex")
    return value


def _sha256(value, name):
    if not isinstance(value, str) or _HEX64.fullmatch(value) is None:
        raise PartnerAdapterContractError(f"{name} must be lowercase SHA-256")
    return value


def _safe_positive_integer(value, name):
    if type(value) is not int or not 1 <= value <= experiment.JSON_SAFE_INTEGER:
        raise PartnerAdapterContractError(
            f"{name} must be a positive JSON-safe integer <= {experiment.JSON_SAFE_INTEGER}"
        )
    return value


def _graph_field(validator, value, name):
    """Reuse Graph/v1's pinned Hub grammar and normalize its error type."""
    try:
        return validator(value, name)
    except execution_graph.ExecutionGraphError as exc:
        raise PartnerAdapterContractError(str(exc)) from exc


def _canonical_protocol(value):
    _exact(value, {"name", "version", "schema_identity"}, "adapter protocol")
    name = _text(value["name"], "adapter protocol name")
    if _CONTRACT_NAME.fullmatch(name) is None:
        raise PartnerAdapterContractError("adapter protocol name has invalid syntax")
    return {
        "name": name,
        "version": _safe_positive_integer(value["version"], "adapter protocol version"),
        "schema_identity": _sha256(value["schema_identity"], "adapter protocol schema_identity"),
    }


def _canonical_named_contracts(records, name):
    if not isinstance(records, list) or len(records) > MAX_CONTRACTS:
        raise PartnerAdapterContractError(f"{name} must be a bounded list")
    seen = set()
    result = []
    for index, record in enumerate(records):
        _exact(record, {"name", "schema", "identity"}, f"{name}[{index}]")
        contract_name = _text(record["name"], f"{name}[{index}].name")
        if _CONTRACT_NAME.fullmatch(contract_name) is None:
            raise PartnerAdapterContractError(f"{name}[{index}].name has invalid syntax")
        if contract_name in seen:
            raise PartnerAdapterContractError(f"duplicate {name} name")
        seen.add(contract_name)
        result.append(
            {
                "name": contract_name,
                "schema": _safe_positive_integer(record["schema"], f"{name}[{index}].schema"),
                "identity": _sha256(record["identity"], f"{name}[{index}].identity"),
            }
        )
    result.sort(key=lambda item: item["name"])
    return result


def _canonical_capabilities(values):
    if not isinstance(values, list) or len(values) > MAX_CAPABILITIES:
        raise PartnerAdapterContractError("capabilities must be a bounded list")
    result = []
    seen = set()
    for index, value in enumerate(values):
        if not isinstance(value, str) or _CAPABILITY.fullmatch(value) is None:
            raise PartnerAdapterContractError(f"capabilities[{index}] has invalid syntax")
        if value in seen:
            raise PartnerAdapterContractError("duplicate capability")
        seen.add(value)
        result.append(value)
    result.sort()
    return result


def _canonical_hub_component(value):
    _exact(
        value,
        {"component_id", "component_version", "manifest_digest", "capability", "capability_contract_version"},
        "Hub component binding",
    )
    return {
        "component_id": _graph_field(execution_graph._hub_uuid, value["component_id"], "Hub component id"),
        "component_version": _graph_field(execution_graph._version, value["component_version"], "Hub component version"),
        "manifest_digest": _graph_field(execution_graph._sha256, value["manifest_digest"], "Hub component manifest_digest"),
        "capability": _graph_field(execution_graph._capability, value["capability"], "Hub component capability"),
        "capability_contract_version": _graph_field(
            execution_graph._version, value["capability_contract_version"], "Hub capability contract version"
        ),
    }


def canonical_partner_adapter(descriptor):
    """Validate and canonicalize PartnerAdapter/v1 without granting authority."""
    _exact(
        descriptor,
        {"schema", "partner", "repository", "source_sha", "role", "protocol", "hub_component", "capabilities", "inputs", "outputs", "permissions"},
        "partner adapter",
    )
    if type(descriptor["schema"]) is not int or descriptor["schema"] != PARTNER_ADAPTER_SCHEMA:
        raise PartnerAdapterContractError("unsupported partner adapter schema")
    partner_name = descriptor["partner"]
    if partner_name not in _PARTNERS:
        raise PartnerAdapterContractError("unsupported TDI partner")
    profile = _PARTNERS[partner_name]
    if descriptor["repository"] != profile["repository"]:
        raise PartnerAdapterContractError("partner repository does not match the declared partner")
    if descriptor["role"] != profile["role"]:
        raise PartnerAdapterContractError("partner role does not match the declared partner")
    permissions = descriptor["permissions"]
    _exact(permissions, set(_REQUIRED_PERMISSIONS), "partner adapter permissions")
    if any(
        type(permissions[field]) is not bool or permissions[field] is not required
        for field, required in _REQUIRED_PERMISSIONS.items()
    ):
        raise PartnerAdapterContractError("common partner adapter contract grants no authority")
    return {
        "schema": PARTNER_ADAPTER_SCHEMA,
        "partner": partner_name,
        "repository": profile["repository"],
        "source_sha": _sha40(descriptor["source_sha"], "partner source_sha"),
        "role": profile["role"],
        "protocol": _canonical_protocol(descriptor["protocol"]),
        "hub_component": _canonical_hub_component(descriptor["hub_component"]),
        "capabilities": _canonical_capabilities(descriptor["capabilities"]),
        "inputs": _canonical_named_contracts(descriptor["inputs"], "inputs"),
        "outputs": _canonical_named_contracts(descriptor["outputs"], "outputs"),
        "permissions": dict(_REQUIRED_PERMISSIONS),
    }


def partner_adapter_identity(descriptor):
    return _digest("tdi-partner-adapter/v1", canonical_partner_adapter(descriptor))


def _graph_step_for_key(admission_binding, step_key):
    if not isinstance(step_key, str):
        raise PartnerAdapterContractError("step_key must be a string")
    for step in admission_binding["graph"]["steps"]:
        if step["key"] == step_key:
            return step
    raise PartnerAdapterContractError("step_key is not present in admitted Graph/v1")


def _require_integer_version(container, key, expected, name):
    if not isinstance(container, dict):
        raise PartnerAdapterContractError(f"{name} container must be an object")
    value = container.get(key)
    if type(value) is not int or value != expected:
        raise PartnerAdapterContractError(f"{name} must be the integer {expected}")


def _require_type_faithful_g3_versions(binding):
    """Reject bool-as-int aliases before qualified G1/G3 validators canonicalize them."""
    if not isinstance(binding, dict):
        raise PartnerAdapterContractError("workflow_admission_binding must be an object")
    _require_integer_version(
        binding, "schema", admission.HUB_WORKFLOW_ADMISSION_BINDING_SCHEMA, "workflow admission binding schema"
    )
    _require_integer_version(
        binding,
        "admission_schema_version",
        admission.HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION,
        "workflow admission schema version",
    )
    graph = binding.get("graph")
    _require_integer_version(graph, "schema", execution_graph.GRAPH_SCHEMA, "embedded Graph/v1 schema")
    hub_contract = graph.get("hub_contract") if isinstance(graph, dict) else None
    _require_integer_version(
        hub_contract,
        "workflow_schema_version",
        execution_graph.HUB_WORKFLOW_SCHEMA_VERSION,
        "embedded Graph/v1 Hub workflow schema version",
    )
    workflow_spec = binding.get("workflow_spec")
    _require_integer_version(
        workflow_spec,
        "schema_version",
        execution_graph.HUB_WORKFLOW_SCHEMA_VERSION,
        "embedded Hub WorkflowSpec schema version",
    )
    embedded_admission = binding.get("admission")
    _require_integer_version(
        embedded_admission,
        "schema_version",
        admission.HUB_WORKFLOW_ADMISSION_SCHEMA_VERSION,
        "embedded Hub admission schema version",
    )
    root_bindings = binding.get("root_artifact_bindings")
    if not isinstance(root_bindings, dict):
        raise PartnerAdapterContractError("root_artifact_bindings must be an object")
    for digest, root_binding in root_bindings.items():
        _require_integer_version(
            root_binding,
            "schema",
            hub_edge.HUB_EDGE_SCHEMA,
            f"root artifact binding {digest!r} schema",
        )
        descriptor = root_binding.get("descriptor") if isinstance(root_binding, dict) else None
        _require_integer_version(
            descriptor,
            "schema",
            artifact.ARTIFACT_SCHEMA,
            f"root artifact descriptor {digest!r} schema",
        )


def bind_admitted_partner_step(descriptor, workflow_admission_binding, *, step_key):
    """Bind PartnerAdapter/v1 to one exact G3-admitted Graph/v1 step."""
    adapter = canonical_partner_adapter(descriptor)
    _require_type_faithful_g3_versions(workflow_admission_binding)
    try:
        admitted = admission.canonical_workflow_admission_binding(workflow_admission_binding)
    except admission.HubAdmissionContractError as exc:
        raise PartnerAdapterContractError(str(exc)) from exc
    if step_key not in admitted["admitted_steps"]:
        raise PartnerAdapterContractError("step_key is not covered by exact Hub admission")
    step = _graph_step_for_key(admitted, step_key)
    expected_component = {
        "component_id": step["component_id"],
        "component_version": step["component_version"],
        "manifest_digest": step["component_manifest_digest"],
        "capability": step["capability"],
        "capability_contract_version": step["capability_contract_version"],
    }
    if experiment.canonical(adapter["hub_component"]) != experiment.canonical(expected_component):
        raise PartnerAdapterContractError("partner Hub component does not match admitted Graph/v1 step")
    return {
        "schema": ADMITTED_PARTNER_STEP_SCHEMA,
        "workflow": admitted["workflow"],
        "step_key": step_key,
        "graph_identity": admitted["graph_identity"],
        "workflow_admission_binding_identity": admission.workflow_admission_binding_identity(admitted),
        "partner_adapter_identity": partner_adapter_identity(adapter),
        "adapter": adapter,
        "workflow_admission_binding": admitted,
        "workflow_execution_admitted": True,
        "partner_contract_bound": True,
        "partner_execution_qualified": False,
        "protected_holdout_access_authorized": False,
        "scientific_stage_authorized": False,
        "scientific_verdict_authorized": False,
        "runtime_actuation_authorized": False,
    }


def canonical_admitted_partner_step(binding):
    """Recompute AdmittedPartnerStep/v1 identities from embedded evidence."""
    _exact(
        binding,
        {
            "schema", "workflow", "step_key", "graph_identity", "workflow_admission_binding_identity",
            "partner_adapter_identity", "adapter", "workflow_admission_binding", "workflow_execution_admitted",
            "partner_contract_bound", "partner_execution_qualified", "protected_holdout_access_authorized",
            "scientific_stage_authorized", "scientific_verdict_authorized", "runtime_actuation_authorized",
        },
        "admitted partner step",
    )
    if type(binding["schema"]) is not int or binding["schema"] != ADMITTED_PARTNER_STEP_SCHEMA:
        raise PartnerAdapterContractError("unsupported admitted partner step schema")
    expected = bind_admitted_partner_step(
        binding["adapter"], binding["workflow_admission_binding"], step_key=binding["step_key"]
    )
    for field in ("workflow", "graph_identity", "workflow_admission_binding_identity", "partner_adapter_identity"):
        if binding[field] != expected[field]:
            raise PartnerAdapterContractError(f"{field} does not match embedded evidence")
    for field in (
        "workflow_execution_admitted", "partner_contract_bound", "partner_execution_qualified",
        "protected_holdout_access_authorized", "scientific_stage_authorized", "scientific_verdict_authorized",
        "runtime_actuation_authorized",
    ):
        if binding[field] is not expected[field]:
            raise PartnerAdapterContractError(f"{field} violates the common partner boundary")
    return expected


def admitted_partner_step_identity(binding):
    return _digest("tdi-admitted-partner-step/v1", canonical_admitted_partner_step(binding))
