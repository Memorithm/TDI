"""Versioned, authority-free TDI -> NNIS hardware-qualification contract.

This Lot-H slice binds the common PartnerAdapter/v1 boundary to an exact audited
NNIS CUDA-Rust-SIMT qualification surface.  It describes candidate evidence and
review requirements only.  It does not invoke NNIS, execute a CUDA kernel,
select a device, authorize production routing, or convert an unresolved NNIS
qualification manifest into a qualified result.
"""
from __future__ import annotations

import hashlib

import tdi_artifact_contract as artifact
import tdi_experiment_contract as experiment
import tdi_partner_adapter_contract as partner

NNIS_QUALIFICATION_CONTRACT_SCHEMA = 1
NNIS_REPOSITORY = "Memorithm/NNIS"
NNIS_SOURCE_SHA = "5436736002834dd6dd7d5ace8c1c044b47aed18f"
NNIS_RUST_MSRV = "1.77"
NNIS_MANIFEST_PATH = "docs/cuda-rust-simt-qualification.json"
NNIS_MANIFEST_BLOB_SHA = "a777fa47f8fe4af068ef6a14c808d91e95161f58"
NNIS_MANIFEST_SCHEMA = "nnis-cuda-rust-simt-qualification-v1"
NNIS_MANIFEST_STATUS = "unresolved_blocking"
NNIS_FRONTEND_CONTRACT = "CUDA_RUST_SIMT_PTX"
NNIS_VALIDATOR_PATH = "scripts/validate_cuda_rust_simt_qualification.py"
NNIS_VALIDATOR_BLOB_SHA = "ef7af3f8cc840b6f3a8053e8e5b33bf38d29f143"
NNIS_EVIDENCE_SCHEMA = "nnis-cuda-rust-simt-evidence-v1"
NNIS_PROTOCOL_NAME = "tdi.nnis.hardware-qualification"
NNIS_PROTOCOL_VERSION = 1
NNIS_HUB_CAPABILITY = "tdi.prepare"
NNIS_HUB_CAPABILITY_CONTRACT_VERSION = "1.0.0"
NNIS_CANDIDATE_MEDIA_TYPE = "application/vnd.tdi.nnis-hardware-qualification-candidate.v1+json"

_ALLOWED_DOMAINS = {"Development", "Validation"}
_ACCESS_FOR_DOMAIN = {"Development": "development", "Validation": "validation"}
_ALLOWED_SCOPES = {"cuda-rust-simt-contract", "cuda-rust-simt-evidence-review"}
_REQUIRED_PERMISSIONS = {
    "nnis_execution_qualified": False,
    "hardware_qualification_authorized": False,
    "device_performance_qualified": False,
    "performance_claim_authorized": False,
    "production_routing_authorized": False,
    "protected_holdout_access_authorized": False,
    "scientific_stage_authorized": False,
    "scientific_verdict_authorized": False,
    "runtime_actuation_authorized": False,
}


class NnisPartnerContractError(ValueError):
    """The TDI -> NNIS qualification boundary is invalid."""


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _derived_schema_identity(kind, upstream_schema):
    """Derive a TDI adapter identity from exact audited NNIS source coordinates.

    These identities are TDI-owned interchange metadata.  They are not claimed
    to be digests published by NNIS.
    """
    return _digest(
        "tdi-nnis-adapter-schema/v1",
        {
            "kind": kind,
            "repository": NNIS_REPOSITORY,
            "source_sha": NNIS_SOURCE_SHA,
            "manifest_blob_sha": NNIS_MANIFEST_BLOB_SHA,
            "validator_blob_sha": NNIS_VALIDATOR_BLOB_SHA,
            "upstream_schema": upstream_schema,
        },
    )


NNIS_MANIFEST_SCHEMA_IDENTITY = _derived_schema_identity("qualification-manifest", NNIS_MANIFEST_SCHEMA)
NNIS_EVIDENCE_SCHEMA_IDENTITY = _derived_schema_identity("evidence-bundle", NNIS_EVIDENCE_SCHEMA)
NNIS_PROTOCOL_SCHEMA_IDENTITY = _derived_schema_identity("protocol", NNIS_PROTOCOL_NAME)


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise NnisPartnerContractError(f"{name} has unknown or missing fields")
    return value


def _strict_version(value, expected, name):
    if type(value) is not int or value != expected:
        raise NnisPartnerContractError(f"{name} must be the integer {expected}")
    return value


def nnis_adapter_surface():
    """Return the exact TDI projection of the audited NNIS qualification surface."""
    return {
        "capabilities": ["nnis.cuda-rust-simt-qualification.v1"],
        "inputs": [
            {"name": "evidence-bundle", "schema": 1, "identity": NNIS_EVIDENCE_SCHEMA_IDENTITY},
            {"name": "qualification-manifest", "schema": 1, "identity": NNIS_MANIFEST_SCHEMA_IDENTITY},
        ],
        "outputs": [],
    }


def _canonical_nnis_metadata(value):
    _exact(
        value,
        {
            "repository",
            "source_sha",
            "rust_msrv",
            "manifest_path",
            "manifest_blob_sha",
            "manifest_schema",
            "manifest_status",
            "frontend_contract",
            "validator_path",
            "validator_blob_sha",
            "evidence_schema",
        },
        "nnis",
    )
    expected = {
        "repository": NNIS_REPOSITORY,
        "source_sha": NNIS_SOURCE_SHA,
        "rust_msrv": NNIS_RUST_MSRV,
        "manifest_path": NNIS_MANIFEST_PATH,
        "manifest_blob_sha": NNIS_MANIFEST_BLOB_SHA,
        "manifest_schema": NNIS_MANIFEST_SCHEMA,
        "manifest_status": NNIS_MANIFEST_STATUS,
        "frontend_contract": NNIS_FRONTEND_CONTRACT,
        "validator_path": NNIS_VALIDATOR_PATH,
        "validator_blob_sha": NNIS_VALIDATOR_BLOB_SHA,
        "evidence_schema": NNIS_EVIDENCE_SCHEMA,
    }
    for field, expected_value in expected.items():
        if value[field] != expected_value:
            raise NnisPartnerContractError(f"nnis.{field} does not match the audited NNIS source")
    return dict(expected)


def _canonical_candidate_artifact(value, domain):
    if not isinstance(value, dict):
        raise NnisPartnerContractError("request.candidate_artifact must be an object")
    _strict_version(value.get("schema"), artifact.ARTIFACT_SCHEMA, "candidate artifact schema")
    try:
        descriptor = artifact.canonical_artifact(value)
    except artifact.ArtifactContractError as exc:
        raise NnisPartnerContractError(str(exc)) from exc
    if descriptor["media_type"] != NNIS_CANDIDATE_MEDIA_TYPE:
        raise NnisPartnerContractError(
            f"candidate artifact media_type must be {NNIS_CANDIDATE_MEDIA_TYPE}"
        )
    if descriptor["access_class"] != _ACCESS_FOR_DOMAIN[domain]:
        raise NnisPartnerContractError(
            "candidate artifact access_class must match Development/Validation domain"
        )
    return descriptor


def _canonical_request(value):
    _exact(value, {"domain", "qualification_scope", "candidate_artifact"}, "request")
    domain = value["domain"]
    if not isinstance(domain, str) or domain not in _ALLOWED_DOMAINS:
        raise NnisPartnerContractError("request.domain must be Development or Validation")
    scope = value["qualification_scope"]
    if not isinstance(scope, str) or scope not in _ALLOWED_SCOPES:
        raise NnisPartnerContractError("request.qualification_scope is not an admitted NNIS contract scope")
    return {
        "domain": domain,
        "qualification_scope": scope,
        "candidate_artifact": _canonical_candidate_artifact(value["candidate_artifact"], domain),
    }


def _canonical_review(value):
    _exact(
        value,
        {
            "owner",
            "independent_nnis_validation_required",
            "exact_nnis_revision_required",
            "exact_device_cuda_identity_required",
            "correction_before_performance_required",
            "real_device_claims_require_evidence",
        },
        "review",
    )
    if value["owner"] != NNIS_REPOSITORY:
        raise NnisPartnerContractError("NNIS must remain the hardware-qualification owner")
    for field in (
        "independent_nnis_validation_required",
        "exact_nnis_revision_required",
        "exact_device_cuda_identity_required",
        "correction_before_performance_required",
        "real_device_claims_require_evidence",
    ):
        if value[field] is not True:
            raise NnisPartnerContractError(f"review.{field} must remain true")
    return {
        "owner": NNIS_REPOSITORY,
        "independent_nnis_validation_required": True,
        "exact_nnis_revision_required": True,
        "exact_device_cuda_identity_required": True,
        "correction_before_performance_required": True,
        "real_device_claims_require_evidence": True,
    }


def _canonical_permissions(value):
    _exact(value, set(_REQUIRED_PERMISSIONS), "permissions")
    if any(
        type(value[field]) is not bool or value[field] is not expected
        for field, expected in _REQUIRED_PERMISSIONS.items()
    ):
        raise NnisPartnerContractError(
            "TDI NNIS contract grants no execution, hardware qualification, performance, routing, holdout, stage, verdict, or actuation authority"
        )
    return dict(_REQUIRED_PERMISSIONS)


def canonical_nnis_qualification_contract(value):
    """Validate one non-executing NNIS hardware-qualification candidate."""
    _exact(value, {"schema", "partner_step", "nnis", "request", "review", "permissions"}, "NNIS qualification contract")
    _strict_version(value["schema"], NNIS_QUALIFICATION_CONTRACT_SCHEMA, "NNIS qualification contract schema")
    try:
        admitted_partner_step = partner.canonical_admitted_partner_step(value["partner_step"])
    except partner.PartnerAdapterContractError as exc:
        raise NnisPartnerContractError(str(exc)) from exc
    adapter = admitted_partner_step["adapter"]
    if adapter["partner"] != "nnis" or adapter["repository"] != NNIS_REPOSITORY:
        raise NnisPartnerContractError("partner_step is not the NNIS adapter")
    if adapter["source_sha"] != NNIS_SOURCE_SHA:
        raise NnisPartnerContractError("NNIS adapter source does not match the audited source")
    protocol = adapter["protocol"]
    if (
        protocol["name"] != NNIS_PROTOCOL_NAME
        or protocol["version"] != NNIS_PROTOCOL_VERSION
        or protocol["schema_identity"] != NNIS_PROTOCOL_SCHEMA_IDENTITY
    ):
        raise NnisPartnerContractError("NNIS adapter protocol does not match the qualified TDI interchange contract")
    hub = adapter["hub_component"]
    if (
        hub["capability"] != NNIS_HUB_CAPABILITY
        or hub["capability_contract_version"] != NNIS_HUB_CAPABILITY_CONTRACT_VERSION
    ):
        raise NnisPartnerContractError("NNIS adapter must bind the non-executing preparation boundary")
    for field, expected in nnis_adapter_surface().items():
        if adapter[field] != expected:
            raise NnisPartnerContractError(f"NNIS adapter {field} does not match the audited source projection")
    return {
        "schema": NNIS_QUALIFICATION_CONTRACT_SCHEMA,
        "partner_step": admitted_partner_step,
        "nnis": _canonical_nnis_metadata(value["nnis"]),
        "request": _canonical_request(value["request"]),
        "review": _canonical_review(value["review"]),
        "permissions": _canonical_permissions(value["permissions"]),
    }


def nnis_qualification_contract_identity(value):
    return _digest("tdi-nnis-qualification-contract/v1", canonical_nnis_qualification_contract(value))


def compile_nnis_qualification_request(value):
    """Compile review metadata without qualifying hardware or performance."""
    contract = canonical_nnis_qualification_contract(value)
    candidate = contract["request"]["candidate_artifact"]
    return {
        "protocol": NNIS_PROTOCOL_NAME,
        "protocol_version": NNIS_PROTOCOL_VERSION,
        "protocol_schema_identity": NNIS_PROTOCOL_SCHEMA_IDENTITY,
        "nnis_source_sha": NNIS_SOURCE_SHA,
        "nnis_rust_msrv": NNIS_RUST_MSRV,
        "upstream_manifest_schema": NNIS_MANIFEST_SCHEMA,
        "upstream_manifest_status": NNIS_MANIFEST_STATUS,
        "frontend_contract": NNIS_FRONTEND_CONTRACT,
        "qualification_scope": contract["request"]["qualification_scope"],
        "candidate": {
            "artifact_identity": artifact.artifact_identity(candidate),
            "raw_sha256": candidate["raw_sha256"],
            "size_bytes": candidate["size_bytes"],
        },
        "independent_nnis_validation_required": True,
        "exact_nnis_revision_required": True,
        "exact_device_cuda_identity_required": True,
        "correction_before_performance_required": True,
        "real_device_claims_require_evidence": True,
        "upstream_qualification_resolved": False,
        "nnis_execution_qualified": False,
        "hardware_qualification_authorized": False,
        "device_performance_qualified": False,
        "performance_claim_authorized": False,
        "production_routing_authorized": False,
    }
