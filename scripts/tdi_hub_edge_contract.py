"""Fail-closed TDI↔scirust-hub edge contracts.

This module verifies portable artifact identity and authoritative publication
records exposed by pinned, qualified scirust-hub revisions. It does not perform
HTTP transport, schedule work, issue or renew leases, create fencing
generations, publish outputs, or authorize a scientific stage. Those generic
orchestration/transport responsibilities remain owned by scirust-hub.

``HubArtifactBinding/v1`` remains intentionally non-authoritative. The v2
binding added after Hub PRs #50/#51 may mark one artifact publication as
authoritative only after matching a qualified Hub ``PublicationFence/v1``
response to the exact v1 artifact binding. Even then
``execution_authorized`` remains ``False`` until a separate qualified edge can
enforce TDI's exact component/capability pins at Hub execution admission.
"""
from __future__ import annotations

import hashlib
import re
import uuid

import tdi_artifact_contract as artifact
import tdi_experiment_contract as experiment

HUB_EDGE_SCHEMA = 1
HUB_AUTHORITATIVE_EDGE_SCHEMA = 2
HUB_PUBLICATION_FENCE_SCHEMA = 1
HUB_REPOSITORY = "Memorithm/scirust-hub"
# scirust-hub PR #47 squash merge. That exact revision qualified the portable
# digest endpoint, same-pass Hub-digest/size re-verification, and bounded
# portable-digest scan concurrency before TDI consumes the bridge contract.
PINNED_HUB_PORTABLE_DIGEST_SOURCE = "9f666225b186fbca6160dd34068aea1c9af57040"
# scirust-hub PR #51 squash merge. It exposes the Hub-owned authoritative
# workflow-step publication through the authenticated inspect API, after PR #50
# wired PublicationFenceRepository into the real orchestrator path.
PINNED_HUB_AUTHORITATIVE_PUBLICATION_SOURCE = "a98f77dc52caa30d055d62acc84f77b176b9bc9b"
# Hub default limits allow 16 declared files, while run outcomes can additionally
# publish the two built-in stream labels stdout/stderr.
HUB_MAX_AUTHORITATIVE_OUTPUTS = 18
_UINT64_MAX = (1 << 64) - 1
_HEX40 = re.compile(r"[0-9a-f]{40}\Z")
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")
_HUB_NAME = re.compile(r"[a-z0-9][a-z0-9_-]{0,63}\Z")
_DECIMAL_U64 = re.compile(r"[1-9][0-9]{0,19}\Z")


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


def _u64(value, name, *, nonzero=False):
    lower = 1 if nonzero else 0
    if type(value) is not int or not lower <= value <= _UINT64_MAX:
        qualifier = "positive " if nonzero else ""
        raise HubEdgeContractError(f"{name} must be a {qualifier}unsigned 64-bit integer")
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


def _hub_name(value, name):
    if not isinstance(value, str) or _HUB_NAME.fullmatch(value) is None:
        raise HubEdgeContractError(f"{name} must match [a-z0-9][a-z0-9_-]{{0,63}}")
    return value


def _canonical_decimal_u64(value, name):
    if not isinstance(value, str) or _DECIMAL_U64.fullmatch(value) is None:
        raise HubEdgeContractError(f"{name} must be a canonical positive decimal u64 string")
    parsed = int(value)
    if parsed > _UINT64_MAX:
        raise HubEdgeContractError(f"{name} exceeds unsigned 64-bit range")
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

    The returned v1 binding is intentionally non-authoritative for execution
    and publication. Authoritative publication is represented separately by the
    v2 binding so existing consumers cannot gain authority by reinterpretation.
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
        raise HubEdgeContractError("HubArtifactBinding/v1 is permanently non-authoritative")
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


def canonical_authoritative_publication_response(response):
    """Validate the read-only Hub PR #51 authoritative publication response.

    The endpoint returns Hub-owned publication authority. TDI validates the
    response semantically but does not authenticate or implement its transport.
    ``generation`` is accepted across the complete Hub u64 domain and is later
    normalized to a decimal string before entering TDI canonical JSON identity.
    """
    _exact(
        response,
        {"schema_version", "workflow", "step_key", "attempt", "generation", "outputs"},
        "Hub authoritative publication response",
    )
    if type(response["schema_version"]) is not int or response["schema_version"] != HUB_PUBLICATION_FENCE_SCHEMA:
        raise HubEdgeContractError("unsupported Hub publication fence schema")
    workflow = _canonical_uuid(response["workflow"], "Hub workflow id")
    step_key = _hub_name(response["step_key"], "Hub workflow step key")
    attempt = _canonical_uuid(response["attempt"], "Hub attempt id")
    generation = _u64(response["generation"], "Hub publication generation", nonzero=True)
    outputs = response["outputs"]
    if not isinstance(outputs, dict):
        raise HubEdgeContractError("Hub authoritative publication outputs must be an object")
    if len(outputs) > HUB_MAX_AUTHORITATIVE_OUTPUTS:
        raise HubEdgeContractError(
            f"Hub authoritative publication outputs exceed pinned bound {HUB_MAX_AUTHORITATIVE_OUTPUTS}"
        )
    canonical_outputs = {}
    for name, artifact_id in sorted(outputs.items()):
        label = _hub_name(name, "Hub publication output label")
        canonical_outputs[label] = _canonical_uuid(artifact_id, f"Hub publication output {label!r} artifact id")
    return {
        "schema_version": HUB_PUBLICATION_FENCE_SCHEMA,
        "workflow": workflow,
        "step_key": step_key,
        "attempt": attempt,
        "generation": generation,
        "outputs": canonical_outputs,
    }


def bind_authoritative_publication(
    artifact_binding,
    publication_response,
    *,
    expected_workflow,
    expected_step_key,
    expected_output,
):
    """Bind one v1 portable artifact to a qualified authoritative Hub publication.

    The caller supplies the expected workflow/step/output lineage explicitly so
    a valid publication from the wrong execution cannot be silently accepted.
    The selected publication artifact id must equal the Hub artifact id already
    tied to TDI's portable descriptor by ``HubArtifactBinding/v1``.

    This raises publication authority only. Exact component/capability admission
    is still not enforced by Hub WorkflowSpec/v1, so execution remains
    unauthorized.
    """
    artifact_binding = canonical_hub_artifact_binding(artifact_binding)
    publication = canonical_authoritative_publication_response(publication_response)
    expected_workflow = _canonical_uuid(expected_workflow, "expected Hub workflow id")
    expected_step_key = _hub_name(expected_step_key, "expected Hub workflow step key")
    expected_output = _hub_name(expected_output, "expected Hub output label")
    if publication["workflow"] != expected_workflow:
        raise HubEdgeContractError("authoritative publication workflow does not match expected lineage")
    if publication["step_key"] != expected_step_key:
        raise HubEdgeContractError("authoritative publication step does not match expected lineage")
    if expected_output not in publication["outputs"]:
        raise HubEdgeContractError("authoritative publication does not contain the expected output")
    if publication["outputs"][expected_output] != artifact_binding["hub_artifact_id"]:
        raise HubEdgeContractError("authoritative publication output does not match the bound Hub artifact")
    return {
        "schema": HUB_AUTHORITATIVE_EDGE_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_portable_digest_source_sha": PINNED_HUB_PORTABLE_DIGEST_SOURCE,
        "hub_publication_source_sha": PINNED_HUB_AUTHORITATIVE_PUBLICATION_SOURCE,
        "publication_schema_version": HUB_PUBLICATION_FENCE_SCHEMA,
        "workflow": publication["workflow"],
        "step_key": publication["step_key"],
        "attempt": publication["attempt"],
        "generation": str(publication["generation"]),
        "output_label": expected_output,
        "hub_artifact_id": artifact_binding["hub_artifact_id"],
        "artifact_binding_identity": hub_artifact_binding_identity(artifact_binding),
        "artifact_binding": artifact_binding,
        "execution_authorized": False,
        "publication_authoritative": True,
    }


def canonical_authoritative_hub_artifact_binding(binding):
    """Validate HubAuthoritativeArtifactBinding/v2 without granting execution."""
    _exact(
        binding,
        {
            "schema",
            "hub_repository",
            "hub_portable_digest_source_sha",
            "hub_publication_source_sha",
            "publication_schema_version",
            "workflow",
            "step_key",
            "attempt",
            "generation",
            "output_label",
            "hub_artifact_id",
            "artifact_binding_identity",
            "artifact_binding",
            "execution_authorized",
            "publication_authoritative",
        },
        "authoritative Hub artifact binding",
    )
    if binding["schema"] != HUB_AUTHORITATIVE_EDGE_SCHEMA:
        raise HubEdgeContractError("unsupported authoritative Hub artifact binding schema")
    if binding["hub_repository"] != HUB_REPOSITORY:
        raise HubEdgeContractError("Hub repository pin mismatch")
    portable_source = _hex(
        binding["hub_portable_digest_source_sha"], _HEX40, "Hub portable digest source SHA"
    )
    if portable_source != PINNED_HUB_PORTABLE_DIGEST_SOURCE:
        raise HubEdgeContractError("Hub portable digest source is not the qualified revision")
    publication_source = _hex(
        binding["hub_publication_source_sha"], _HEX40, "Hub publication source SHA"
    )
    if publication_source != PINNED_HUB_AUTHORITATIVE_PUBLICATION_SOURCE:
        raise HubEdgeContractError("Hub publication source is not the qualified authoritative revision")
    if type(binding["publication_schema_version"]) is not int or binding["publication_schema_version"] != HUB_PUBLICATION_FENCE_SCHEMA:
        raise HubEdgeContractError("unsupported Hub publication fence schema")
    workflow = _canonical_uuid(binding["workflow"], "Hub workflow id")
    step_key = _hub_name(binding["step_key"], "Hub workflow step key")
    attempt = _canonical_uuid(binding["attempt"], "Hub attempt id")
    generation = _canonical_decimal_u64(binding["generation"], "Hub publication generation")
    output_label = _hub_name(binding["output_label"], "Hub publication output label")
    hub_artifact_id = _canonical_uuid(binding["hub_artifact_id"], "Hub artifact id")
    artifact_binding = canonical_hub_artifact_binding(binding["artifact_binding"])
    artifact_binding_identity = _hex(
        binding["artifact_binding_identity"], _HEX64, "Hub artifact binding identity"
    )
    if artifact_binding_identity != hub_artifact_binding_identity(artifact_binding):
        raise HubEdgeContractError("authoritative binding portable artifact identity mismatch")
    if artifact_binding["hub_artifact_id"] != hub_artifact_id:
        raise HubEdgeContractError("authoritative binding Hub artifact id mismatch")
    if binding["execution_authorized"] is not False:
        raise HubEdgeContractError("Hub edge execution remains unauthorized until registry-pin qualification")
    if binding["publication_authoritative"] is not True:
        raise HubEdgeContractError("v2 binding requires qualified authoritative Hub publication")
    return {
        "schema": HUB_AUTHORITATIVE_EDGE_SCHEMA,
        "hub_repository": HUB_REPOSITORY,
        "hub_portable_digest_source_sha": portable_source,
        "hub_publication_source_sha": publication_source,
        "publication_schema_version": HUB_PUBLICATION_FENCE_SCHEMA,
        "workflow": workflow,
        "step_key": step_key,
        "attempt": attempt,
        "generation": generation,
        "output_label": output_label,
        "hub_artifact_id": hub_artifact_id,
        "artifact_binding_identity": artifact_binding_identity,
        "artifact_binding": artifact_binding,
        "execution_authorized": False,
        "publication_authoritative": True,
    }


def authoritative_hub_artifact_binding_identity(binding):
    """Return the TDI identity of one qualified authoritative Hub lineage binding."""
    value = canonical_authoritative_hub_artifact_binding(binding)
    return _binding_digest("tdi-hub-authoritative-artifact-binding/v2", value)
