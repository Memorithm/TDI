"""Versioned, authority-free TDI -> Forge scientific-search contract.

This Lot-H slice binds a qualified common ``PartnerAdapter/v1`` Forge step to
the wire shape currently accepted by Forge's ``ScientificExternalDomainManifestV1``.
TDI owns the scientific/search boundary and source identities; Forge remains the
owner of candidate search, verification, measurement, and selection.

The contract is deliberately non-executing.  Successful validation does not
authorize Forge execution, protected/final holdout access, a scientific stage,
a scientific verdict, or runtime actuation.
"""
from __future__ import annotations

import hashlib
import re

import tdi_experiment_contract as experiment
import tdi_partner_adapter_contract as partner

FORGE_SEARCH_CONTRACT_SCHEMA = 1
FORGE_REPOSITORY = "Memorithm/Forge"
# Audited Forge main at creation of this adapter slice.
FORGE_SOURCE_SHA = "8946e702e697c144c85e0fb166a21fe759cdae46"
FORGE_EXTERNAL_DOMAIN_SCHEMA_VERSION = 1
FORGE_SCIENTIFIC_DOMAIN_SCHEMA_VERSION = 1

MAX_LIST_ITEMS = 128
MAX_TEXT_BYTES = 512

_GIT_ID = re.compile(r"(?:[0-9a-f]{40}|[0-9a-f]{64})\Z")
_SHA256 = re.compile(r"[0-9a-f]{64}\Z")
_DOMAIN_ID = re.compile(r"[a-z0-9][a-z0-9._/-]{0,127}\Z")

_REQUIRED_PERMISSIONS = {
    "forge_execution_qualified": False,
    "protected_holdout_access_authorized": False,
    "scientific_stage_authorized": False,
    "scientific_verdict_authorized": False,
    "runtime_actuation_authorized": False,
}


class ForgePartnerContractError(ValueError):
    """The TDI -> Forge scientific-search boundary is invalid."""


def _digest(kind, value):
    return hashlib.sha256(
        kind.encode("ascii") + b"\0" + experiment.canonical(value)
    ).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise ForgePartnerContractError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, grammar=None):
    if not isinstance(value, str) or not value.strip():
        raise ForgePartnerContractError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > MAX_TEXT_BYTES:
        raise ForgePartnerContractError(f"{name} exceeds the text bound")
    if grammar is not None and grammar.fullmatch(value) is None:
        raise ForgePartnerContractError(f"{name} has invalid syntax")
    return value


def _sha256(value, name):
    if not isinstance(value, str) or _SHA256.fullmatch(value) is None:
        raise ForgePartnerContractError(f"{name} must be lowercase SHA-256")
    return value


def _git_id(value, name):
    if not isinstance(value, str) or _GIT_ID.fullmatch(value) is None:
        raise ForgePartnerContractError(
            f"{name} must be a lowercase 40- or 64-hex Git object id"
        )
    return value


def _strict_version(value, expected, name):
    if type(value) is not int or value != expected:
        raise ForgePartnerContractError(f"{name} must be the integer {expected}")
    return value


def _unique_text_list(values, name, *, require_nonempty=False):
    if not isinstance(values, list) or len(values) > MAX_LIST_ITEMS:
        raise ForgePartnerContractError(f"{name} must be a bounded list")
    if require_nonempty and not values:
        raise ForgePartnerContractError(f"{name} must not be empty")
    result = []
    seen = set()
    for index, value in enumerate(values):
        item = _text(value, f"{name}[{index}]")
        if item in seen:
            raise ForgePartnerContractError(f"duplicate {name} entry")
        seen.add(item)
        result.append(item)
    return result


def _canonical_upstream(value):
    _exact(value, {"repository", "commit_id", "contract_sha256"}, "upstream")
    repository = _text(value["repository"], "upstream.repository")
    parts = repository.split("/")
    if (
        len(parts) != 2
        or not all(parts)
        or any(
            not all(ch.isalnum() or ch in "._-" for ch in component)
            for component in parts
        )
    ):
        raise ForgePartnerContractError("upstream.repository must use owner/name syntax")
    return {
        "repository": repository,
        "commit_id": _git_id(value["commit_id"], "upstream.commit_id"),
        "contract_sha256": _sha256(
            value["contract_sha256"], "upstream.contract_sha256"
        ),
    }


def _canonical_data_boundary(value):
    _exact(
        value,
        {"generation_sources", "verification_sources", "final_holdout_sources"},
        "data_boundary",
    )
    generation = _unique_text_list(
        value["generation_sources"],
        "data_boundary.generation_sources",
        require_nonempty=True,
    )
    verification = _unique_text_list(
        value["verification_sources"],
        "data_boundary.verification_sources",
        require_nonempty=True,
    )
    final = _unique_text_list(
        value["final_holdout_sources"], "data_boundary.final_holdout_sources"
    )
    overlap = set(generation) & set(verification)
    if overlap:
        raise ForgePartnerContractError(
            f"development/validation source overlap: {sorted(overlap)[0]}"
        )
    forbidden = set(final)
    for name, values in (
        ("generation_sources", generation),
        ("verification_sources", verification),
    ):
        overlap = forbidden & set(values)
        if overlap:
            raise ForgePartnerContractError(
                f"final holdout source leaked into {name}: {sorted(overlap)[0]}"
            )
    return {
        "generation_sources": generation,
        "verification_sources": verification,
        "final_holdout_sources": final,
    }


def _canonical_verification(value):
    _exact(value, {"adapter_id", "adapter_sha256"}, "verification")
    return {
        "adapter_id": _text(value["adapter_id"], "verification.adapter_id"),
        "adapter_sha256": _sha256(
            value["adapter_sha256"], "verification.adapter_sha256"
        ),
    }


def _canonical_objectives(values):
    if not isinstance(values, list) or not values or len(values) > MAX_LIST_ITEMS:
        raise ForgePartnerContractError("objectives must be a non-empty bounded list")
    seen = set()
    result = []
    for index, value in enumerate(values):
        _exact(value, {"name", "direction"}, f"objectives[{index}]")
        name = _text(value["name"], f"objectives[{index}].name")
        if name in seen:
            raise ForgePartnerContractError("duplicate objective name")
        seen.add(name)
        direction = value["direction"]
        if direction not in {"minimize", "maximize"}:
            raise ForgePartnerContractError(
                f"objectives[{index}].direction must be minimize or maximize"
            )
        result.append({"name": name, "direction": direction})
    return result


def _canonical_environment(value):
    _exact(
        value,
        {"fingerprint_required", "isolation_required"},
        "environment",
    )
    for field in ("fingerprint_required", "isolation_required"):
        if type(value[field]) is not bool:
            raise ForgePartnerContractError(f"environment.{field} must be Boolean")
    return {
        "fingerprint_required": value["fingerprint_required"],
        "isolation_required": value["isolation_required"],
    }


def _canonical_permissions(value):
    _exact(value, set(_REQUIRED_PERMISSIONS), "permissions")
    if any(
        type(value[field]) is not bool or value[field] is not expected
        for field, expected in _REQUIRED_PERMISSIONS.items()
    ):
        raise ForgePartnerContractError(
            "TDI Forge search contract grants no execution, holdout, verdict, stage, or actuation authority"
        )
    return dict(_REQUIRED_PERMISSIONS)


def canonical_forge_search_contract(value):
    """Validate the leak-safe, non-executing TDI -> Forge search contract."""
    _exact(
        value,
        {
            "schema",
            "partner_step",
            "forge",
            "domain_id",
            "upstream",
            "allowed_candidate_dimensions",
            "data_boundary",
            "verification",
            "objectives",
            "environment",
            "permissions",
        },
        "Forge search contract",
    )
    _strict_version(
        value["schema"], FORGE_SEARCH_CONTRACT_SCHEMA, "Forge search contract schema"
    )

    try:
        admitted_partner_step = partner.canonical_admitted_partner_step(
            value["partner_step"]
        )
    except partner.PartnerAdapterContractError as exc:
        raise ForgePartnerContractError(str(exc)) from exc

    adapter = admitted_partner_step["adapter"]
    if adapter["partner"] != "forge" or adapter["repository"] != FORGE_REPOSITORY:
        raise ForgePartnerContractError("partner_step is not the Forge adapter")
    if adapter["source_sha"] != FORGE_SOURCE_SHA:
        raise ForgePartnerContractError(
            "Forge adapter source does not match the audited Forge source"
        )

    forge = value["forge"]
    _exact(
        forge,
        {
            "repository",
            "source_sha",
            "external_domain_schema_version",
            "scientific_domain_schema_version",
        },
        "forge",
    )
    if forge["repository"] != FORGE_REPOSITORY:
        raise ForgePartnerContractError("forge.repository does not match Forge")
    if forge["source_sha"] != FORGE_SOURCE_SHA:
        raise ForgePartnerContractError(
            "forge.source_sha does not match the audited Forge source"
        )
    _strict_version(
        forge["external_domain_schema_version"],
        FORGE_EXTERNAL_DOMAIN_SCHEMA_VERSION,
        "Forge external-domain schema version",
    )
    _strict_version(
        forge["scientific_domain_schema_version"],
        FORGE_SCIENTIFIC_DOMAIN_SCHEMA_VERSION,
        "Forge scientific-domain schema version",
    )

    domain_id = _text(value["domain_id"], "domain_id", grammar=_DOMAIN_ID)
    candidate_dimensions = _unique_text_list(
        value["allowed_candidate_dimensions"],
        "allowed_candidate_dimensions",
        require_nonempty=True,
    )

    return {
        "schema": FORGE_SEARCH_CONTRACT_SCHEMA,
        "partner_step": admitted_partner_step,
        "forge": {
            "repository": FORGE_REPOSITORY,
            "source_sha": FORGE_SOURCE_SHA,
            "external_domain_schema_version": FORGE_EXTERNAL_DOMAIN_SCHEMA_VERSION,
            "scientific_domain_schema_version": FORGE_SCIENTIFIC_DOMAIN_SCHEMA_VERSION,
        },
        "domain_id": domain_id,
        "upstream": _canonical_upstream(value["upstream"]),
        "allowed_candidate_dimensions": candidate_dimensions,
        "data_boundary": _canonical_data_boundary(value["data_boundary"]),
        "verification": _canonical_verification(value["verification"]),
        "objectives": _canonical_objectives(value["objectives"]),
        "environment": _canonical_environment(value["environment"]),
        "permissions": _canonical_permissions(value["permissions"]),
    }


def forge_search_contract_identity(value):
    """Content identity for a validated TDI ForgeSearchContract/v1."""
    return _digest(
        "tdi-forge-search-contract/v1", canonical_forge_search_contract(value)
    )


def compile_forge_scientific_domain_manifest(value):
    """Compile the validated TDI contract into Forge's v1 scientific manifest shape.

    The returned object is an interchange payload only. Forge must independently
    validate it under its pinned source before any candidate search is executed.
    """
    contract = canonical_forge_search_contract(value)
    return {
        "schema_version": FORGE_SCIENTIFIC_DOMAIN_SCHEMA_VERSION,
        "external_domain": {
            "schema_version": FORGE_EXTERNAL_DOMAIN_SCHEMA_VERSION,
            "domain_id": contract["domain_id"],
            "upstream": contract["upstream"],
            "allowed_candidate_dimensions": contract[
                "allowed_candidate_dimensions"
            ],
            "data_boundary": contract["data_boundary"],
            "verification": contract["verification"],
            "objectives": contract["objectives"],
            "environment": contract["environment"],
        },
    }
