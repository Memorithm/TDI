"""Fail-closed TDI↔scirust-hub edge contracts.

This module verifies the portable artifact identity bridge exposed by a pinned,
qualified scirust-hub revision. It does not perform HTTP transport, schedule
work, issue or renew leases, create fencing generations, publish authoritative
outputs, or authorize any scientific stage. Those orchestration/transport
responsibilities remain owned by scirust-hub.

The current v1 bridge deliberately emits ``execution_authorized=False`` and
``publication_authoritative=False``. TDI must keep those values false until a
separate Hub-owned durable publication-fencing contract is qualified and the
TDI edge is explicitly advanced by a later reviewed change.
"""
from __future__ import annotations

import hashlib
import re
import uuid

import tdi_artifact_contract as artifact
import tdi_experiment_contract as experiment

HUB_EDGE_SCHEMA = 1
HUB_REPOSITORY = "Memorithm/scirust-hub"
# scirust-hub PR #47 squash merge. That exact revision qualified the portable
# digest endpoint, same-pass Hub-digest/size re-verification, and bounded
# portable-digest scan concurrency before TDI consumes the bridge contract.
PINNED_HUB_PORTABLE_DIGEST_SOURCE = "9f666225b186fbca6160dd34068aea1c9af57040"
_HEX40 = re.compile(r"[0-9a-f]{40}\Z")
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")


class HubEdgeContractError(ValueError):
    """A TDI↔Hub edge value is malformed, drifted, or unauthorized."""


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise HubEdgeContractError(f"{name} has unknown or missing fields")
    return value


def _hex(value, pattern, name):
    if not isinstance(value, str) or pattern.fullmatch(value) is None:
        raise HubEdgeContractError(f"{name} has invalid lowercase hexadecimal form")
    return value


def _safe_size(value, name):
    if type(value) is not int or not 0 <= value <= experiment.JSON_SAFE_INTEGER:
        raise HubEdgeContractError(
            f"{name} must be a non-negative JSON-safe integer <= {experiment.JSON_SAFE_INTEGER}"
        )
    return value


def _canonical_uuid(value, name):
    if not isinstance(value, str):
        raise HubEdgeContractError(f"{name} must be a canonical UUID string")
    try:
        parsed = uuid.UUID(value)
    except (ValueError, AttributeError) as exc:
        raise HubEdgeContractError(f"{name} must be a canonical UUID string") from exc
    if str(parsed) != value:
        raise HubEdgeContractError(f"{name} must use canonical lowercase UUID form")
    return value


def _binding_digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def canonical_portable_digest_response(response):
    """Validate the scirust-hub PR #47 portable-digest response shape.

    ``hub_digest`` remains an opaque Hub domain-separated ContentDigest. TDI
    records it for translation provenance but never treats it as the portable
    payload SHA-256 used by :mod:`tdi_artifact_contract`.
    """
    _exact(response, {"id", "hub_digest", "raw_sha256", "size"}, "Hub portable digest response")
    return {
        "id": _canonical_uuid(response["id"], "Hub portable digest id"),
        "hub_digest": _hex(response["hub_digest"], _HEX64, "Hub content digest"),
        "raw_sha256": _hex(response["raw_sha256"], _HEX64, "Hub raw SHA-256"),
        "size": _safe_size(response["size"], "Hub artifact size"),
    }


def bind_portable_artifact(descriptor, response):
    """Bind a TDI ArtifactDescriptor/v1 to one qualified Hub artifact response.

    The raw payload digest and exact byte count must agree. This function does
    not fetch the response and therefore does not authenticate HTTP transport;
    callers must obtain it from the pinned qualified Hub service through their
    deployment's authenticated channel.

    The returned binding is intentionally non-authoritative for execution and
    publication. A later Hub-owned durable fencing generation is required before
    TDI may advance those booleans.
    """
    descriptor = artifact.canonical_artifact(descriptor)
    response = canonical_portable_digest_response(response)
    if response["raw_sha256"] != descriptor["raw_sha256"]:
        raise HubEdgeContractError("Hub portable raw SHA-256 does not match TDI artifact descriptor")
    if response["size"] != descriptor["size_bytes"]:
        raise HubEdgeContractError("Hub portable artifact size does not match TDI artifact descriptor")
    return {
        "schema": HUB_EDGE_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_source_sha": PINNED_HUB_PORTABLE_DIGEST_SOURCE,
        "hub_artifact_id": response["id"],
        "hub_digest": response["hub_digest"],
        "artifact_identity": artifact.artifact_identity(descriptor),
        "descriptor": descriptor,
        "execution_authorized": False,
        "publication_authoritative": False,
    }


def canonical_hub_artifact_binding(binding):
    """Validate HubArtifactBinding/v1 and its fail-closed authority boundary."""
    _exact(
        binding,
        {
            "schema",
            "hub_repository",
            "hub_source_sha",
            "hub_artifact_id",
            "hub_digest",
            "artifact_identity",
            "descriptor",
            "execution_authorized",
            "publication_authoritative",
        },
        "Hub artifact binding",
    )
    if binding["schema"] != HUB_EDGE_SCHEMA:
        raise HubEdgeContractError("unsupported Hub artifact binding schema")
    if binding["hub_repository"] != HUB_REPOSITORY:
        raise HubEdgeContractError("Hub repository pin mismatch")
    source = _hex(binding["hub_source_sha"], _HEX40, "Hub source SHA")
    if source != PINNED_HUB_PORTABLE_DIGEST_SOURCE:
        raise HubEdgeContractError("Hub source SHA is not the qualified portable-digest revision")
    hub_artifact_id = _canonical_uuid(binding["hub_artifact_id"], "Hub artifact id")
    hub_digest = _hex(binding["hub_digest"], _HEX64, "Hub content digest")
    descriptor = artifact.canonical_artifact(binding["descriptor"])
    artifact_identity = _hex(binding["artifact_identity"], _HEX64, "TDI artifact identity")
    if artifact_identity != artifact.artifact_identity(descriptor):
        raise HubEdgeContractError("Hub binding TDI artifact identity mismatch")
    if binding["execution_authorized"] is not False:
        raise HubEdgeContractError("Hub edge execution remains unauthorized until later qualification")
    if binding["publication_authoritative"] is not False:
        raise HubEdgeContractError("Hub edge publication remains non-authoritative without durable fencing")
    return {
        "schema": HUB_EDGE_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_source_sha": source,
        "hub_artifact_id": hub_artifact_id,
        "hub_digest": hub_digest,
        "artifact_identity": artifact_identity,
        "descriptor": descriptor,
        "execution_authorized": False,
        "publication_authoritative": False,
    }


def hub_artifact_binding_identity(binding):
    """Return a stable TDI identity for a validated non-authoritative Hub binding."""
    value = canonical_hub_artifact_binding(binding)
    return _binding_digest("tdi-hub-artifact-binding/v1", value)
