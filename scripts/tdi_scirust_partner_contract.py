"""Versioned, authority-free TDI -> SciRust primitive-promotion contract.

This Lot-H slice binds the qualified common ``PartnerAdapter/v1`` boundary to
an exact audited SciRust Tensor-IR representation surface. It describes a
portable candidate artifact that may be reviewed for promotion into SciRust;
it does not execute SciRust, copy SciRust implementation code into TDI, or
transfer TDI scientific-state/verdict ownership.

The adapter protocol defined here is TDI-owned interchange metadata, not a
claim that SciRust publishes a network protocol. SciRust remains authoritative
for whether a candidate is generic, correct, API-compatible, tested and suitable
for promotion into its reusable mathematical core.
"""
from __future__ import annotations

import hashlib

import tdi_artifact_contract as artifact
import tdi_experiment_contract as experiment
import tdi_partner_adapter_contract as partner

SCIRUST_PROMOTION_CONTRACT_SCHEMA = 1
SCIRUST_REPOSITORY = "Memorithm/scirust"
SCIRUST_SOURCE_SHA = "aa13f62ea829c5c42e370a32d51143f016a0dc44"
SCIRUST_TENSOR_IR_CRATE = "scirust-tensor-ir"
SCIRUST_REPRESENTATION_MODULE = "scirust-tensor-ir/src/representation.rs"
SCIRUST_REPRESENTATION_BLOB_SHA = "753bc57e1da862c9cd73a47ef9f72b981fbdd177"
SCIRUST_PUBLIC_SURFACES = (
    "PrimitiveRepresentation",
    "RepresentationPlan",
    "StorageBits",
)
SCIRUST_REPRESENTATION_SCHEMA_IDENTITY = (
    "92f258383afe68c364a9ee361999bf18db3b7e611da84c75290f5de17fb3853d"
)

# TDI-owned adapter interchange contract. The identity is frozen from the
# audited source/API descriptor documented in lot-h-scirust-primitive-boundary.md.
SCIRUST_PROTOCOL_NAME = "tdi.scirust.representation-promotion"
SCIRUST_PROTOCOL_VERSION = 1
SCIRUST_PROTOCOL_SCHEMA_IDENTITY = (
    "5fa966102003d48811e14bf17043e43a4eaaa7ed649f1d5b024a54b24ec54c62"
)
SCIRUST_HUB_CAPABILITY = "tdi.prepare"
SCIRUST_HUB_CAPABILITY_CONTRACT_VERSION = "1.0.0"
SCIRUST_CANDIDATE_MEDIA_TYPE = "application/vnd.tdi.scirust-primitive-candidate.v1+json"

_ALLOWED_DOMAINS = {"Development", "Validation"}
_ACCESS_FOR_DOMAIN = {"Development": "development", "Validation": "validation"}
_ALLOWED_KINDS = {"representation-primitive", "storage-accounting-primitive"}

_REQUIRED_PERMISSIONS = {
    "scirust_execution_qualified": False,
    "primitive_promotion_authorized": False,
    "protected_holdout_access_authorized": False,
    "scientific_stage_authorized": False,
    "scientific_verdict_authorized": False,
    "runtime_actuation_authorized": False,
}


class SciRustPartnerContractError(ValueError):
    """The TDI -> SciRust reusable-primitive boundary is invalid."""


def _digest(kind, value):
    return hashlib.sha256(
        kind.encode("ascii") + b"\0" + experiment.canonical(value)
    ).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise SciRustPartnerContractError(f"{name} has unknown or missing fields")
    return value


def _strict_version(value, expected, name):
    if type(value) is not int or value != expected:
        raise SciRustPartnerContractError(f"{name} must be the integer {expected}")
    return value


def scirust_adapter_surface():
    """Return the exact TDI projection of the audited SciRust interface."""
    return {
        "capabilities": ["scirust.tensor-ir.representation.v1"],
        "inputs": [
            {
                "name": "primitive-candidate",
                "schema": 1,
                "identity": SCIRUST_REPRESENTATION_SCHEMA_IDENTITY,
            }
        ],
        "outputs": [],
    }


def _canonical_candidate_artifact(value, domain):
    if not isinstance(value, dict):
        raise SciRustPartnerContractError("request.candidate_artifact must be an object")
    _strict_version(
        value.get("schema"), artifact.ARTIFACT_SCHEMA, "candidate artifact schema"
    )
    try:
        descriptor = artifact.canonical_artifact(value)
    except artifact.ArtifactContractError as exc:
        raise SciRustPartnerContractError(str(exc)) from exc
    if descriptor["media_type"] != SCIRUST_CANDIDATE_MEDIA_TYPE:
        raise SciRustPartnerContractError(
            f"candidate artifact media_type must be {SCIRUST_CANDIDATE_MEDIA_TYPE}"
        )
    if descriptor["access_class"] != _ACCESS_FOR_DOMAIN[domain]:
        raise SciRustPartnerContractError(
            "candidate artifact access_class must match Development/Validation domain"
        )
    return descriptor


def _canonical_scirust_metadata(value):
    _exact(
        value,
        {
            "repository",
            "source_sha",
            "tensor_ir_crate",
            "representation_module",
            "representation_blob_sha",
            "public_surfaces",
        },
        "scirust",
    )
    if value["repository"] != SCIRUST_REPOSITORY:
        raise SciRustPartnerContractError("scirust.repository does not match SciRust")
    if value["source_sha"] != SCIRUST_SOURCE_SHA:
        raise SciRustPartnerContractError(
            "scirust.source_sha does not match the audited SciRust source"
        )
    if value["tensor_ir_crate"] != SCIRUST_TENSOR_IR_CRATE:
        raise SciRustPartnerContractError("SciRust Tensor-IR crate identity drift")
    if value["representation_module"] != SCIRUST_REPRESENTATION_MODULE:
        raise SciRustPartnerContractError("SciRust representation module identity drift")
    if value["representation_blob_sha"] != SCIRUST_REPRESENTATION_BLOB_SHA:
        raise SciRustPartnerContractError("SciRust representation module blob drift")
    if (
        type(value["public_surfaces"]) is not list
        or tuple(value["public_surfaces"]) != SCIRUST_PUBLIC_SURFACES
    ):
        raise SciRustPartnerContractError("SciRust audited public surface drift")
    return {
        "repository": SCIRUST_REPOSITORY,
        "source_sha": SCIRUST_SOURCE_SHA,
        "tensor_ir_crate": SCIRUST_TENSOR_IR_CRATE,
        "representation_module": SCIRUST_REPRESENTATION_MODULE,
        "representation_blob_sha": SCIRUST_REPRESENTATION_BLOB_SHA,
        "public_surfaces": list(SCIRUST_PUBLIC_SURFACES),
    }


def _canonical_request(value):
    _exact(
        value,
        {"domain", "candidate_artifact", "primitive_kind", "promotion_scope"},
        "request",
    )
    domain = value["domain"]
    if not isinstance(domain, str) or domain not in _ALLOWED_DOMAINS:
        raise SciRustPartnerContractError(
            "request.domain must be Development or Validation"
        )
    primitive_kind = value["primitive_kind"]
    if not isinstance(primitive_kind, str) or primitive_kind not in _ALLOWED_KINDS:
        raise SciRustPartnerContractError(
            "request.primitive_kind is not an allowed reusable primitive class"
        )
    if value["promotion_scope"] != "candidate-only":
        raise SciRustPartnerContractError(
            "request.promotion_scope must remain candidate-only"
        )
    return {
        "domain": domain,
        "candidate_artifact": _canonical_candidate_artifact(
            value["candidate_artifact"], domain
        ),
        "primitive_kind": primitive_kind,
        "promotion_scope": "candidate-only",
    }


def _canonical_review(value):
    _exact(
        value,
        {
            "owner",
            "independent_scirust_validation_required",
            "implementation_copy_forbidden",
        },
        "review",
    )
    if value["owner"] != SCIRUST_REPOSITORY:
        raise SciRustPartnerContractError(
            "SciRust must remain the promotion review owner"
        )
    if value["independent_scirust_validation_required"] is not True:
        raise SciRustPartnerContractError("independent SciRust validation is mandatory")
    if value["implementation_copy_forbidden"] is not True:
        raise SciRustPartnerContractError(
            "TDI must not copy SciRust implementation ownership"
        )
    return {
        "owner": SCIRUST_REPOSITORY,
        "independent_scirust_validation_required": True,
        "implementation_copy_forbidden": True,
    }


def _canonical_permissions(value):
    _exact(value, set(_REQUIRED_PERMISSIONS), "permissions")
    if any(
        type(value[field]) is not bool or value[field] is not expected
        for field, expected in _REQUIRED_PERMISSIONS.items()
    ):
        raise SciRustPartnerContractError(
            "TDI SciRust contract grants no execution, promotion, holdout, stage, verdict, or actuation authority"
        )
    return dict(_REQUIRED_PERMISSIONS)


def canonical_scirust_promotion_contract(value):
    """Validate one authority-free reusable-primitive promotion candidate."""
    _exact(
        value,
        {"schema", "partner_step", "scirust", "request", "review", "permissions"},
        "SciRust promotion contract",
    )
    _strict_version(
        value["schema"],
        SCIRUST_PROMOTION_CONTRACT_SCHEMA,
        "SciRust promotion contract schema",
    )
    try:
        admitted_partner_step = partner.canonical_admitted_partner_step(
            value["partner_step"]
        )
    except partner.PartnerAdapterContractError as exc:
        raise SciRustPartnerContractError(str(exc)) from exc

    adapter = admitted_partner_step["adapter"]
    if (
        adapter["partner"] != "scirust"
        or adapter["repository"] != SCIRUST_REPOSITORY
    ):
        raise SciRustPartnerContractError("partner_step is not the SciRust adapter")
    if adapter["source_sha"] != SCIRUST_SOURCE_SHA:
        raise SciRustPartnerContractError(
            "SciRust adapter source does not match the audited source"
        )
    protocol = adapter["protocol"]
    if (
        protocol["name"] != SCIRUST_PROTOCOL_NAME
        or protocol["version"] != SCIRUST_PROTOCOL_VERSION
        or protocol["schema_identity"] != SCIRUST_PROTOCOL_SCHEMA_IDENTITY
    ):
        raise SciRustPartnerContractError(
            "SciRust adapter protocol does not match the qualified TDI interchange contract"
        )
    hub = adapter["hub_component"]
    if (
        hub["capability"] != SCIRUST_HUB_CAPABILITY
        or hub["capability_contract_version"]
        != SCIRUST_HUB_CAPABILITY_CONTRACT_VERSION
    ):
        raise SciRustPartnerContractError(
            "SciRust adapter must bind the non-executing preparation boundary"
        )
    for field, expected in scirust_adapter_surface().items():
        if adapter[field] != expected:
            raise SciRustPartnerContractError(
                f"SciRust adapter {field} does not match the audited source projection"
            )

    return {
        "schema": SCIRUST_PROMOTION_CONTRACT_SCHEMA,
        "partner_step": admitted_partner_step,
        "scirust": _canonical_scirust_metadata(value["scirust"]),
        "request": _canonical_request(value["request"]),
        "review": _canonical_review(value["review"]),
        "permissions": _canonical_permissions(value["permissions"]),
    }


def scirust_promotion_contract_identity(value):
    """Content identity for a validated SciRustPromotionContract/v1."""
    return _digest(
        "tdi-scirust-promotion-contract/v1",
        canonical_scirust_promotion_contract(value),
    )


def compile_scirust_promotion_request(value):
    """Compile review metadata without granting implementation/promotion authority."""
    contract = canonical_scirust_promotion_contract(value)
    candidate = contract["request"]["candidate_artifact"]
    return {
        "protocol": SCIRUST_PROTOCOL_NAME,
        "protocol_version": SCIRUST_PROTOCOL_VERSION,
        "protocol_schema_identity": SCIRUST_PROTOCOL_SCHEMA_IDENTITY,
        "scirust_source_sha": SCIRUST_SOURCE_SHA,
        "representation_module_blob_sha": SCIRUST_REPRESENTATION_BLOB_SHA,
        "public_surfaces": list(SCIRUST_PUBLIC_SURFACES),
        "primitive_kind": contract["request"]["primitive_kind"],
        "candidate": {
            "artifact_identity": artifact.artifact_identity(candidate),
            "raw_sha256": candidate["raw_sha256"],
            "size_bytes": candidate["size_bytes"],
        },
        "independent_scirust_validation_required": True,
        "scirust_execution_qualified": False,
        "primitive_promotion_authorized": False,
    }
