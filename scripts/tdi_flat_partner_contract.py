"""Versioned, authority-free TDI -> FLAT-ATTENTION qualification contract.

This Lot-H slice binds the qualified common PartnerAdapter/v1 boundary to an
exact audited FLAT-ATTENTION API / Boolean-front-end surface.  It describes
candidate qualification metadata only.  It does not invoke FLAT, stage K/V,
select a GPU backend, open a holdout, or transfer attention semantics into TDI.
"""
from __future__ import annotations

import hashlib

import tdi_artifact_contract as artifact
import tdi_experiment_contract as experiment
import tdi_partner_adapter_contract as partner

FLAT_QUALIFICATION_CONTRACT_SCHEMA = 1
FLAT_REPOSITORY = "Memorithm/FLAT-ATTENTION"
FLAT_SOURCE_SHA = "1d5ac64cc87c5dd526e04527c3bb4b78ba0add33"
FLAT_API_MODULE = "src/api.rs"
FLAT_API_BLOB_SHA = "ffc5eb912afd7599e3e4b326877d0f8617bc45dd"
FLAT_API_VERSION = 1
FLAT_BOOLEAN_MASK_MODULE = "src/boolean_attention_mask.rs"
FLAT_BOOLEAN_MASK_BLOB_SHA = "732e028251ad02e1f60633e9caa45023484fac24"
FLAT_BOOLEAN_MASK_SCHEMA = 1
FLAT_BOOLEAN_SIGNATURE_MODULE = "src/boolean_attention_signature.rs"
FLAT_BOOLEAN_SIGNATURE_BLOB_SHA = "6c03c9c414c7fb333bb2047a4187b78934709a74"
FLAT_BOOLEAN_SIGNATURE_SCHEMA = 1

# These are TDI adapter-schema pins derived from the audited FLAT source. They
# are not claimed to be FLAT-published digests.
FLAT_API_SCHEMA_IDENTITY = "e6f23e26b2ffb878de42b09873566f94204383a9b4e8af664d6c3c3c5f599f90"
FLAT_BOOLEAN_MASK_SCHEMA_IDENTITY = "9a5224386cb02cd45200288005c06628cb4967d8709d16e5d6c138ebf5823527"
FLAT_BOOLEAN_SIGNATURE_SCHEMA_IDENTITY = "1e669b85f5dea70fdc6883110482caa24b95180f04666fe1b432998b791abe45"
FLAT_PROTOCOL_NAME = "tdi.flat-attention.qualification"
FLAT_PROTOCOL_VERSION = 1
FLAT_PROTOCOL_SCHEMA_IDENTITY = "60d1ff3a1f648257dc15f95c2b877881504cf3be9d33838158ce5880d7b2d5eb"
FLAT_HUB_CAPABILITY = "tdi.prepare"
FLAT_HUB_CAPABILITY_CONTRACT_VERSION = "1.0.0"
FLAT_CANDIDATE_MEDIA_TYPE = "application/vnd.tdi.flat-qualification-candidate.v1+json"

_ALLOWED_DOMAINS = {"Development", "Validation"}
_ACCESS_FOR_DOMAIN = {"Development": "development", "Validation": "validation"}
_ALLOWED_SCOPES = {"api-contract", "boolean-front-end-contract"}
_REQUIRED_PERMISSIONS = {
    "flat_execution_qualified": False,
    "real_device_performance_qualified": False,
    "boolean_front_end_speedup_qualified": False,
    "protected_holdout_access_authorized": False,
    "scientific_stage_authorized": False,
    "scientific_verdict_authorized": False,
    "runtime_actuation_authorized": False,
}


class FlatPartnerContractError(ValueError):
    """The TDI -> FLAT qualification boundary is invalid."""


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise FlatPartnerContractError(f"{name} has unknown or missing fields")
    return value


def _strict_version(value, expected, name):
    if type(value) is not int or value != expected:
        raise FlatPartnerContractError(f"{name} must be the integer {expected}")
    return value


def flat_adapter_surface():
    """Return the exact TDI projection of the audited FLAT surface."""
    return {
        "capabilities": ["flat.api.v1", "flat.boolean-front-end.v1"],
        "inputs": [
            {"name": "attention-request", "schema": 1, "identity": FLAT_API_SCHEMA_IDENTITY},
            {"name": "boolean-mask", "schema": 1, "identity": FLAT_BOOLEAN_MASK_SCHEMA_IDENTITY},
            {"name": "boolean-signature", "schema": 1, "identity": FLAT_BOOLEAN_SIGNATURE_SCHEMA_IDENTITY},
        ],
        "outputs": [],
    }


def _canonical_flat_metadata(value):
    _exact(
        value,
        {
            "repository",
            "source_sha",
            "api_module",
            "api_blob_sha",
            "api_version",
            "boolean_mask_module",
            "boolean_mask_blob_sha",
            "boolean_mask_schema",
            "boolean_signature_module",
            "boolean_signature_blob_sha",
            "boolean_signature_schema",
        },
        "flat",
    )
    expected = {
        "repository": FLAT_REPOSITORY,
        "source_sha": FLAT_SOURCE_SHA,
        "api_module": FLAT_API_MODULE,
        "api_blob_sha": FLAT_API_BLOB_SHA,
        "api_version": FLAT_API_VERSION,
        "boolean_mask_module": FLAT_BOOLEAN_MASK_MODULE,
        "boolean_mask_blob_sha": FLAT_BOOLEAN_MASK_BLOB_SHA,
        "boolean_mask_schema": FLAT_BOOLEAN_MASK_SCHEMA,
        "boolean_signature_module": FLAT_BOOLEAN_SIGNATURE_MODULE,
        "boolean_signature_blob_sha": FLAT_BOOLEAN_SIGNATURE_BLOB_SHA,
        "boolean_signature_schema": FLAT_BOOLEAN_SIGNATURE_SCHEMA,
    }
    for field, expected_value in expected.items():
        if field.endswith("version") or field.endswith("schema"):
            _strict_version(value[field], expected_value, f"flat.{field}")
        elif value[field] != expected_value:
            raise FlatPartnerContractError(f"flat.{field} does not match the audited FLAT source")
    return dict(expected)


def _canonical_candidate_artifact(value, domain):
    if not isinstance(value, dict):
        raise FlatPartnerContractError("request.candidate_artifact must be an object")
    _strict_version(value.get("schema"), artifact.ARTIFACT_SCHEMA, "candidate artifact schema")
    try:
        descriptor = artifact.canonical_artifact(value)
    except artifact.ArtifactContractError as exc:
        raise FlatPartnerContractError(str(exc)) from exc
    if descriptor["media_type"] != FLAT_CANDIDATE_MEDIA_TYPE:
        raise FlatPartnerContractError(f"candidate artifact media_type must be {FLAT_CANDIDATE_MEDIA_TYPE}")
    if descriptor["access_class"] != _ACCESS_FOR_DOMAIN[domain]:
        raise FlatPartnerContractError("candidate artifact access_class must match Development/Validation domain")
    return descriptor


def _canonical_request(value):
    _exact(value, {"domain", "qualification_scope", "candidate_artifact"}, "request")
    domain = value["domain"]
    if not isinstance(domain, str) or domain not in _ALLOWED_DOMAINS:
        raise FlatPartnerContractError("request.domain must be Development or Validation")
    scope = value["qualification_scope"]
    if not isinstance(scope, str) or scope not in _ALLOWED_SCOPES:
        raise FlatPartnerContractError("request.qualification_scope is not an admitted FLAT contract scope")
    return {
        "domain": domain,
        "qualification_scope": scope,
        "candidate_artifact": _canonical_candidate_artifact(value["candidate_artifact"], domain),
    }


def _canonical_review(value):
    _exact(
        value,
        {"owner", "independent_flat_validation_required", "dense_reference_required", "real_device_claims_require_evidence"},
        "review",
    )
    if value["owner"] != FLAT_REPOSITORY:
        raise FlatPartnerContractError("FLAT-ATTENTION must remain the qualification owner")
    for field in (
        "independent_flat_validation_required",
        "dense_reference_required",
        "real_device_claims_require_evidence",
    ):
        if value[field] is not True:
            raise FlatPartnerContractError(f"review.{field} must remain true")
    return {
        "owner": FLAT_REPOSITORY,
        "independent_flat_validation_required": True,
        "dense_reference_required": True,
        "real_device_claims_require_evidence": True,
    }


def _canonical_permissions(value):
    _exact(value, set(_REQUIRED_PERMISSIONS), "permissions")
    if any(type(value[field]) is not bool or value[field] is not expected for field, expected in _REQUIRED_PERMISSIONS.items()):
        raise FlatPartnerContractError("TDI FLAT contract grants no execution, performance, holdout, stage, verdict, or actuation authority")
    return dict(_REQUIRED_PERMISSIONS)


def canonical_flat_qualification_contract(value):
    """Validate one non-executing FLAT qualification candidate."""
    _exact(value, {"schema", "partner_step", "flat", "request", "review", "permissions"}, "FLAT qualification contract")
    _strict_version(value["schema"], FLAT_QUALIFICATION_CONTRACT_SCHEMA, "FLAT qualification contract schema")
    try:
        admitted_partner_step = partner.canonical_admitted_partner_step(value["partner_step"])
    except partner.PartnerAdapterContractError as exc:
        raise FlatPartnerContractError(str(exc)) from exc
    adapter = admitted_partner_step["adapter"]
    if adapter["partner"] != "flat-attention" or adapter["repository"] != FLAT_REPOSITORY:
        raise FlatPartnerContractError("partner_step is not the FLAT-ATTENTION adapter")
    if adapter["source_sha"] != FLAT_SOURCE_SHA:
        raise FlatPartnerContractError("FLAT adapter source does not match the audited source")
    protocol = adapter["protocol"]
    if (
        protocol["name"] != FLAT_PROTOCOL_NAME
        or protocol["version"] != FLAT_PROTOCOL_VERSION
        or protocol["schema_identity"] != FLAT_PROTOCOL_SCHEMA_IDENTITY
    ):
        raise FlatPartnerContractError("FLAT adapter protocol does not match the qualified TDI interchange contract")
    hub = adapter["hub_component"]
    if hub["capability"] != FLAT_HUB_CAPABILITY or hub["capability_contract_version"] != FLAT_HUB_CAPABILITY_CONTRACT_VERSION:
        raise FlatPartnerContractError("FLAT adapter must bind the non-executing preparation boundary")
    for field, expected in flat_adapter_surface().items():
        if adapter[field] != expected:
            raise FlatPartnerContractError(f"FLAT adapter {field} does not match the audited source projection")
    return {
        "schema": FLAT_QUALIFICATION_CONTRACT_SCHEMA,
        "partner_step": admitted_partner_step,
        "flat": _canonical_flat_metadata(value["flat"]),
        "request": _canonical_request(value["request"]),
        "review": _canonical_review(value["review"]),
        "permissions": _canonical_permissions(value["permissions"]),
    }


def flat_qualification_contract_identity(value):
    return _digest("tdi-flat-qualification-contract/v1", canonical_flat_qualification_contract(value))


def compile_flat_qualification_request(value):
    """Compile review metadata without granting FLAT execution authority."""
    contract = canonical_flat_qualification_contract(value)
    candidate = contract["request"]["candidate_artifact"]
    return {
        "protocol": FLAT_PROTOCOL_NAME,
        "protocol_version": FLAT_PROTOCOL_VERSION,
        "protocol_schema_identity": FLAT_PROTOCOL_SCHEMA_IDENTITY,
        "flat_source_sha": FLAT_SOURCE_SHA,
        "api_version": FLAT_API_VERSION,
        "boolean_mask_schema": FLAT_BOOLEAN_MASK_SCHEMA,
        "boolean_signature_schema": FLAT_BOOLEAN_SIGNATURE_SCHEMA,
        "qualification_scope": contract["request"]["qualification_scope"],
        "candidate": {
            "artifact_identity": artifact.artifact_identity(candidate),
            "raw_sha256": candidate["raw_sha256"],
            "size_bytes": candidate["size_bytes"],
        },
        "independent_flat_validation_required": True,
        "dense_reference_required": True,
        "real_device_claims_require_evidence": True,
        "flat_execution_qualified": False,
        "boolean_front_end_speedup_qualified": False,
    }
