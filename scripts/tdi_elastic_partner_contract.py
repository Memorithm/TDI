"""Versioned, authority-free TDI -> ElasticXxx resource-control contract.

This Lot-H slice binds the qualified common ``PartnerAdapter/v1`` boundary to
ElasticXxx's published ``elastic.hub.run@1.0.0`` process contract.  It records a
portable OperatorConfig/v1 artifact reference and the evidence shape expected
from ElasticXxx, but it deliberately does not execute ElasticXxx or reimplement
OperatorConfig validation.

ElasticXxx remains authoritative for resource semantics, trusted validation,
physical actuation, post-actuation verification, and commit/rollback.  This
contract permits only observe-only, plan-only, and dry-run intent; it rejects
``apply`` and keeps every execution/actuation/scientific authority bit false.
"""
from __future__ import annotations

import hashlib

import tdi_artifact_contract as artifact
import tdi_experiment_contract as experiment
import tdi_partner_adapter_contract as partner

ELASTIC_RESOURCE_CONTRACT_SCHEMA = 1
ELASTIC_REPOSITORY = "Memorithm/ElasticXxx"
ELASTIC_SOURCE_SHA = "50bb85ea84191c01d95e5b4e5e3c81af10e95ebd"
ELASTIC_PROTOCOL_NAME = "elastic.hub.run"
ELASTIC_PROTOCOL_VERSION = 1
ELASTIC_PROTOCOL_SEMVER = "1.0.0"
ELASTIC_PROTOCOL_SCHEMA_IDENTITY = (
    "ae9712fbca20c666012260ef8537a8ced59a2bc7a17f8c4f5e504cb5984efc24"
)
ELASTIC_OPERATOR_CONFIG_SCHEMA_VERSION = 1
ELASTIC_EVIDENCE_SCHEMA = "elastic-runtime-evidence-v1"
ELASTIC_EVIDENCE_MEDIA_TYPE = "application/vnd.elastic.runtime-evidence.v1+json"
ELASTIC_EVIDENCE_SOURCE_COMMAND = "run"
ELASTIC_HUB_CAPABILITY = "tdi.prepare"
ELASTIC_HUB_CAPABILITY_CONTRACT_VERSION = "1.0.0"
ELASTIC_CONFIG_MEDIA_TYPE = "application/json"
# Ordinary SHA-256 of the schema-owning source files at ELASTIC_SOURCE_SHA.
ELASTIC_CONFIG_SCHEMA_IDENTITY = "3830ca34d8724c4f5e1c7801472c67ba1394e235b23a6172565324b542f77d94"
ELASTIC_EVIDENCE_SCHEMA_IDENTITY = "94e214e9649df75d896a0cb46e8065e56e927a6d01106fed575da30cd159b6cb"


def elastic_adapter_surface():
    """Return fresh exact process capability and schema-owner source pins.

    This describes this contract's non-actuating projection and does not
    authorize invoking the underlying process.
    """
    return {
        "capabilities": [ELASTIC_PROTOCOL_NAME],
        "inputs": [{"name": "operator-config", "schema": 1, "identity": ELASTIC_CONFIG_SCHEMA_IDENTITY}],
        "outputs": [{"name": "runtime-evidence", "schema": 1, "identity": ELASTIC_EVIDENCE_SCHEMA_IDENTITY}],
    }

MAX_TEXT_BYTES = 512
_ALLOWED_DOMAINS = {"Development", "Validation"}
_ALLOWED_MODES = {"observe-only", "plan-only", "dry-run"}
_ACCESS_FOR_DOMAIN = {"Development": "development", "Validation": "validation"}

_REQUIRED_PERMISSIONS = {
    "elastic_execution_qualified": False,
    "physical_actuation_authorized": False,
    "protected_holdout_access_authorized": False,
    "scientific_stage_authorized": False,
    "scientific_verdict_authorized": False,
    "runtime_actuation_authorized": False,
}


class ElasticPartnerContractError(ValueError):
    """The TDI -> ElasticXxx resource-control boundary is invalid."""


def _digest(kind, value):
    return hashlib.sha256(
        kind.encode("ascii") + b"\0" + experiment.canonical(value)
    ).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise ElasticPartnerContractError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, allow_none=False):
    if value is None and allow_none:
        return None
    if not isinstance(value, str) or not value.strip():
        raise ElasticPartnerContractError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > MAX_TEXT_BYTES:
        raise ElasticPartnerContractError(f"{name} exceeds the text bound")
    if any(ord(ch) < 0x20 or ord(ch) == 0x7F for ch in value):
        raise ElasticPartnerContractError(f"{name} contains an ASCII control character")
    return value


def _strict_version(value, expected, name):
    if type(value) is not int or value != expected:
        raise ElasticPartnerContractError(f"{name} must be the integer {expected}")
    return value


def _canonical_config_artifact(value, domain):
    if not isinstance(value, dict):
        raise ElasticPartnerContractError("request.config_artifact must be an object")
    _strict_version(
        value.get("schema"), artifact.ARTIFACT_SCHEMA, "OperatorConfig artifact schema"
    )
    try:
        descriptor = artifact.canonical_artifact(value)
    except artifact.ArtifactContractError as exc:
        raise ElasticPartnerContractError(str(exc)) from exc
    if descriptor["media_type"] != ELASTIC_CONFIG_MEDIA_TYPE:
        raise ElasticPartnerContractError(
            f"OperatorConfig artifact media_type must be {ELASTIC_CONFIG_MEDIA_TYPE}"
        )
    expected_access = _ACCESS_FOR_DOMAIN[domain]
    if descriptor["access_class"] != expected_access:
        raise ElasticPartnerContractError(
            "OperatorConfig artifact access_class must match the declared Development/Validation domain"
        )
    return descriptor


def _canonical_elastic_metadata(value):
    _exact(
        value,
        {
            "repository",
            "source_sha",
            "hub_run_protocol_version",
            "operator_config_schema_version",
            "runtime_evidence_schema",
            "runtime_evidence_media_type",
            "runtime_evidence_source_command",
        },
        "elastic",
    )
    if value["repository"] != ELASTIC_REPOSITORY:
        raise ElasticPartnerContractError("elastic.repository does not match ElasticXxx")
    if value["source_sha"] != ELASTIC_SOURCE_SHA:
        raise ElasticPartnerContractError(
            "elastic.source_sha does not match the audited ElasticXxx source"
        )
    if value["hub_run_protocol_version"] != ELASTIC_PROTOCOL_SEMVER:
        raise ElasticPartnerContractError(
            "ElasticXxx Hub runtime protocol version does not match the audited contract"
        )
    _strict_version(
        value["operator_config_schema_version"],
        ELASTIC_OPERATOR_CONFIG_SCHEMA_VERSION,
        "ElasticXxx OperatorConfig schema version",
    )
    if value["runtime_evidence_schema"] != ELASTIC_EVIDENCE_SCHEMA:
        raise ElasticPartnerContractError("ElasticXxx runtime evidence schema drift")
    if value["runtime_evidence_media_type"] != ELASTIC_EVIDENCE_MEDIA_TYPE:
        raise ElasticPartnerContractError("ElasticXxx runtime evidence media type drift")
    if value["runtime_evidence_source_command"] != ELASTIC_EVIDENCE_SOURCE_COMMAND:
        raise ElasticPartnerContractError("ElasticXxx runtime evidence source command drift")
    return {
        "repository": ELASTIC_REPOSITORY,
        "source_sha": ELASTIC_SOURCE_SHA,
        "hub_run_protocol_version": ELASTIC_PROTOCOL_SEMVER,
        "operator_config_schema_version": ELASTIC_OPERATOR_CONFIG_SCHEMA_VERSION,
        "runtime_evidence_schema": ELASTIC_EVIDENCE_SCHEMA,
        "runtime_evidence_media_type": ELASTIC_EVIDENCE_MEDIA_TYPE,
        "runtime_evidence_source_command": ELASTIC_EVIDENCE_SOURCE_COMMAND,
    }


def _canonical_request(value):
    _exact(
        value,
        {"domain", "config_artifact", "requested_mode", "resource_id"},
        "request",
    )
    domain = value["domain"]
    if not isinstance(domain, str) or domain not in _ALLOWED_DOMAINS:
        raise ElasticPartnerContractError("request.domain must be Development or Validation")
    mode = value["requested_mode"]
    if not isinstance(mode, str) or mode not in _ALLOWED_MODES:
        raise ElasticPartnerContractError(
            "request.requested_mode must be observe-only, plan-only, or dry-run; apply is not authorized"
        )
    resource_id = _text(value["resource_id"], "request.resource_id", allow_none=True)
    return {
        "domain": domain,
        "config_artifact": _canonical_config_artifact(value["config_artifact"], domain),
        "requested_mode": mode,
        "resource_id": resource_id,
    }


def _canonical_expected_evidence(value):
    _exact(value, {"schema", "media_type", "source_command"}, "expected_evidence")
    if value["schema"] != ELASTIC_EVIDENCE_SCHEMA:
        raise ElasticPartnerContractError("expected evidence schema drift")
    if value["media_type"] != ELASTIC_EVIDENCE_MEDIA_TYPE:
        raise ElasticPartnerContractError("expected evidence media type drift")
    if value["source_command"] != ELASTIC_EVIDENCE_SOURCE_COMMAND:
        raise ElasticPartnerContractError("expected evidence source command drift")
    return {
        "schema": ELASTIC_EVIDENCE_SCHEMA,
        "media_type": ELASTIC_EVIDENCE_MEDIA_TYPE,
        "source_command": ELASTIC_EVIDENCE_SOURCE_COMMAND,
    }


def _canonical_permissions(value):
    _exact(value, set(_REQUIRED_PERMISSIONS), "permissions")
    if any(
        type(value[field]) is not bool or value[field] is not expected
        for field, expected in _REQUIRED_PERMISSIONS.items()
    ):
        raise ElasticPartnerContractError(
            "TDI ElasticXxx contract grants no execution, actuation, holdout, stage, or verdict authority"
        )
    return dict(_REQUIRED_PERMISSIONS)


def canonical_elastic_resource_contract(value):
    """Validate a non-executing TDI -> ElasticXxx resource-control request."""
    _exact(
        value,
        {"schema", "partner_step", "elastic", "request", "expected_evidence", "permissions"},
        "ElasticXxx resource contract",
    )
    _strict_version(
        value["schema"],
        ELASTIC_RESOURCE_CONTRACT_SCHEMA,
        "ElasticXxx resource contract schema",
    )
    try:
        admitted_partner_step = partner.canonical_admitted_partner_step(
            value["partner_step"]
        )
    except partner.PartnerAdapterContractError as exc:
        raise ElasticPartnerContractError(str(exc)) from exc

    adapter = admitted_partner_step["adapter"]
    if adapter["partner"] != "elasticxxx" or adapter["repository"] != ELASTIC_REPOSITORY:
        raise ElasticPartnerContractError("partner_step is not the ElasticXxx adapter")
    if adapter["source_sha"] != ELASTIC_SOURCE_SHA:
        raise ElasticPartnerContractError(
            "ElasticXxx adapter source does not match the audited source"
        )
    if (
        adapter["protocol"]["name"] != ELASTIC_PROTOCOL_NAME
        or adapter["protocol"]["version"] != ELASTIC_PROTOCOL_VERSION
        or adapter["protocol"]["schema_identity"] != ELASTIC_PROTOCOL_SCHEMA_IDENTITY
    ):
        raise ElasticPartnerContractError(
            "ElasticXxx adapter protocol does not match the audited hub-run contract"
        )
    if (
        adapter["hub_component"]["capability"] != ELASTIC_HUB_CAPABILITY
        or adapter["hub_component"]["capability_contract_version"]
        != ELASTIC_HUB_CAPABILITY_CONTRACT_VERSION
    ):
        raise ElasticPartnerContractError(
            "ElasticXxx adapter Hub capability does not match the non-actuating preparation boundary"
        )
    for field, expected in elastic_adapter_surface().items():
        if adapter[field] != expected:
            raise ElasticPartnerContractError(
                f"ElasticXxx adapter {field} does not match the audited process surface"
            )

    return {
        "schema": ELASTIC_RESOURCE_CONTRACT_SCHEMA,
        "partner_step": admitted_partner_step,
        "elastic": _canonical_elastic_metadata(value["elastic"]),
        "request": _canonical_request(value["request"]),
        "expected_evidence": _canonical_expected_evidence(value["expected_evidence"]),
        "permissions": _canonical_permissions(value["permissions"]),
    }


def elastic_resource_contract_identity(value):
    """Content identity for a validated ElasticResourceContract/v1."""
    return _digest(
        "tdi-elastic-resource-contract/v1", canonical_elastic_resource_contract(value)
    )


def compile_elastic_hub_run_request(value):
    """Compile an authority-free interchange envelope for later Elastic validation.

    This result is not an invocation command and cannot authorize execution.  An
    execution owner must independently retrieve the OperatorConfig bytes, verify
    the ArtifactDescriptor, parse them with the pinned ElasticXxx implementation,
    and prove that the actual config stays within the requested non-actuating
    mode before any run is attempted.
    """
    contract = canonical_elastic_resource_contract(value)
    config = contract["request"]["config_artifact"]
    return {
        "protocol": ELASTIC_PROTOCOL_NAME,
        "protocol_version": ELASTIC_PROTOCOL_SEMVER,
        "elastic_source_sha": ELASTIC_SOURCE_SHA,
        "operator_config": {
            "artifact_identity": artifact.artifact_identity(config),
            "raw_sha256": config["raw_sha256"],
            "size_bytes": config["size_bytes"],
            "schema_version": ELASTIC_OPERATOR_CONFIG_SCHEMA_VERSION,
            "declared_mode": contract["request"]["requested_mode"],
        },
        "resource_id": contract["request"]["resource_id"],
        "expected_evidence": contract["expected_evidence"],
        "independent_elastic_validation_required": True,
        "execution_qualified": False,
        "physical_actuation_authorized": False,
    }
