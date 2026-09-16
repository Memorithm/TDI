"""Portable TDI artifact/provenance/export and exact-cache contracts.

This module defines scientific metadata and verification rules only. It never
stores payloads, schedules work, acquires leases, or authorizes a scientific
stage. Physical CAS/registry/transport ownership remains with scirust-hub.
"""
from __future__ import annotations

import hashlib
import re

import tdi_experiment_contract as experiment

ARTIFACT_SCHEMA = 1
PROVENANCE_SCHEMA = 1
EXPORT_SCHEMA = 1
CACHE_SCHEMA = 1
MAX_RECORDS = 4_096
MAX_TEXT_BYTES = 16_384
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")
_STEP_KEY = re.compile(r"[a-z0-9][a-z0-9_-]{0,63}\Z")
_ALLOWED_ACCESS = {"public", "development", "validation", "restricted-reference"}
_ALLOWED_CACHE_POLICY = {"disabled", "exact-domain"}


class ArtifactContractError(ValueError):
    """A portable artifact, provenance, export, or cache contract is invalid."""


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise ArtifactContractError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, max_bytes=MAX_TEXT_BYTES):
    if not isinstance(value, str) or not value.strip():
        raise ArtifactContractError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > max_bytes:
        raise ArtifactContractError(f"{name} exceeds the text bound")
    return value


def _sha256(value, name):
    if not isinstance(value, str) or _HEX64.fullmatch(value) is None:
        raise ArtifactContractError(f"{name} must be lowercase raw SHA-256")
    return value


def _safe_integer(value, name, *, positive=False):
    lower = 1 if positive else 0
    if type(value) is not int or not lower <= value <= experiment.JSON_SAFE_INTEGER:
        qualifier = "positive " if positive else "non-negative "
        raise ArtifactContractError(
            f"{name} must be a {qualifier}JSON-safe integer <= {experiment.JSON_SAFE_INTEGER}"
        )
    return value


def _bounded_records(value, name, *, allow_empty=True):
    if not isinstance(value, list) or (not allow_empty and not value):
        raise ArtifactContractError(f"{name} must be a bounded list")
    if len(value) > MAX_RECORDS:
        raise ArtifactContractError(f"{name} exceeds the item bound")
    return value


def _access_class(value, name="access_class"):
    if value not in _ALLOWED_ACCESS:
        raise ArtifactContractError(f"{name} is invalid")
    return value


def _step_key(value, name="step_key"):
    if not isinstance(value, str) or _STEP_KEY.fullmatch(value) is None:
        raise ArtifactContractError(
            f"{name} must match [a-z0-9][a-z0-9_-]{{0,63}}"
        )
    return value


def _canonical_named_identities(records, name):
    seen = set()
    normalized = []
    for index, record in enumerate(_bounded_records(records, name)):
        _exact(record, {"name", "identity"}, f"{name}[{index}]")
        record_name = _text(record["name"], f"{name}[{index}].name", max_bytes=256)
        identity = _sha256(record["identity"], f"{name}[{index}].identity")
        if record_name in seen:
            raise ArtifactContractError(f"duplicate {name} name")
        seen.add(record_name)
        normalized.append({"name": record_name, "identity": identity})
    normalized.sort(key=lambda item: item["name"])
    return normalized


def canonical_artifact(descriptor):
    """Validate a portable ArtifactDescriptor/v1.

    ``raw_sha256`` is the ordinary SHA-256 of payload bytes. It is deliberately
    distinct from scirust-hub's domain-separated ``ContentDigest`` and therefore
    cannot be used as a Hub CAS key without an explicit verified translation.
    """
    _exact(
        descriptor,
        {"schema", "name", "raw_sha256", "size_bytes", "media_type", "access_class"},
        "artifact descriptor",
    )
    if descriptor["schema"] != ARTIFACT_SCHEMA:
        raise ArtifactContractError("unsupported artifact descriptor schema")
    name = _text(descriptor["name"], "artifact.name", max_bytes=256)
    digest = _sha256(descriptor["raw_sha256"], "artifact.raw_sha256")
    size = _safe_integer(descriptor["size_bytes"], "artifact.size_bytes")
    media_type = _text(descriptor["media_type"], "artifact.media_type", max_bytes=512)
    access = _access_class(descriptor["access_class"], "artifact.access_class")
    return {
        "schema": ARTIFACT_SCHEMA,
        "name": name,
        "raw_sha256": digest,
        "size_bytes": size,
        "media_type": media_type,
        "access_class": access,
    }


def artifact_identity(descriptor):
    """Return TDI metadata identity for one validated portable descriptor."""
    return _digest("tdi-artifact-descriptor/v1", canonical_artifact(descriptor))


def verify_artifact_bytes(descriptor, payload):
    """Verify payload bytes against a portable descriptor without storing them."""
    value = canonical_artifact(descriptor)
    if not isinstance(payload, (bytes, bytearray, memoryview)):
        raise ArtifactContractError("artifact payload must be bytes-like")
    payload = bytes(payload)
    if len(payload) != value["size_bytes"]:
        raise ArtifactContractError("artifact payload size mismatch")
    if hashlib.sha256(payload).hexdigest() != value["raw_sha256"]:
        raise ArtifactContractError("artifact payload SHA-256 mismatch")
    return value


def canonical_provenance(record):
    """Validate ProvenanceRecord/v1 for one artifact publication."""
    _exact(
        record,
        {
            "schema", "artifact_identity", "experiment_id", "plan_id", "trial_id",
            "attempt_id", "step_key", "step_identity", "implementation_identity",
            "domain", "inputs", "dependencies",
        },
        "provenance record",
    )
    if record["schema"] != PROVENANCE_SCHEMA:
        raise ArtifactContractError("unsupported provenance schema")
    artifact_id = _sha256(record["artifact_identity"], "provenance.artifact_identity")
    experiment_id = _sha256(record["experiment_id"], "provenance.experiment_id")
    plan_id = _sha256(record["plan_id"], "provenance.plan_id")
    trial_id = _sha256(record["trial_id"], "provenance.trial_id")
    attempt_id = _sha256(record["attempt_id"], "provenance.attempt_id")
    step_key = _step_key(record["step_key"], "provenance.step_key")
    step_identity = _sha256(record["step_identity"], "provenance.step_identity")
    implementation = _text(record["implementation_identity"], "provenance.implementation_identity")
    domain = record["domain"]
    if domain not in ("Development", "Validation"):
        raise ArtifactContractError("provenance.domain is invalid")
    return {
        "schema": PROVENANCE_SCHEMA,
        "artifact_identity": artifact_id,
        "experiment_id": experiment_id,
        "plan_id": plan_id,
        "trial_id": trial_id,
        "attempt_id": attempt_id,
        "step_key": step_key,
        "step_identity": step_identity,
        "implementation_identity": implementation,
        "domain": domain,
        "inputs": _canonical_named_identities(record["inputs"], "provenance.inputs"),
        "dependencies": _canonical_named_identities(record["dependencies"], "provenance.dependencies"),
    }


def provenance_identity(record):
    """Return content identity for a validated provenance record."""
    return _digest("tdi-provenance/v1", canonical_provenance(record))


def canonical_export_manifest(manifest):
    """Validate ExportManifest/v1 and canonicalize member order.

    The manifest references portable descriptors and provenance records by TDI
    identities. It does not create a registry, CAS, transport, or publication.
    """
    _exact(manifest, {"schema", "export_id", "access_class", "members"}, "export manifest")
    if manifest["schema"] != EXPORT_SCHEMA:
        raise ArtifactContractError("unsupported export manifest schema")
    export_id = _text(manifest["export_id"], "export_id", max_bytes=256)
    access = _access_class(manifest["access_class"], "export.access_class")
    members = []
    names = set()
    for index, member in enumerate(_bounded_records(manifest["members"], "members", allow_empty=False)):
        _exact(
            member,
            {"name", "artifact_identity", "provenance_identity"},
            f"members[{index}]",
        )
        name = _text(member["name"], f"members[{index}].name", max_bytes=256)
        artifact_id = _sha256(member["artifact_identity"], f"members[{index}].artifact_identity")
        provenance_id = _sha256(member["provenance_identity"], f"members[{index}].provenance_identity")
        if name in names:
            raise ArtifactContractError("duplicate export member name")
        names.add(name)
        members.append({
            "name": name,
            "artifact_identity": artifact_id,
            "provenance_identity": provenance_id,
        })
    members.sort(key=lambda item: item["name"])
    return {"schema": EXPORT_SCHEMA, "export_id": export_id, "access_class": access, "members": members}


def export_manifest_identity(manifest):
    return _digest("tdi-export-manifest/v1", canonical_export_manifest(manifest))


def verify_export(manifest, artifacts, provenances, payloads):
    """Verify a complete export from independent descriptor/provenance/payload maps.

    Every member must resolve exactly once. Descriptor/provenance identities and
    payload bytes must agree; provenance must refer to the same artifact identity.
    Access may become more restrictive but never less restrictive than the
    manifest's declared class.
    """
    value = canonical_export_manifest(manifest)
    if not all(isinstance(mapping, dict) for mapping in (artifacts, provenances, payloads)):
        raise ArtifactContractError("export inputs must be mappings")
    expected_names = {member["name"] for member in value["members"]}
    if set(artifacts) != expected_names or set(provenances) != expected_names or set(payloads) != expected_names:
        raise ArtifactContractError("export member set mismatch")
    rank = {"public": 0, "development": 1, "validation": 2, "restricted-reference": 3}
    for member in value["members"]:
        name = member["name"]
        artifact = verify_artifact_bytes(artifacts[name], payloads[name])
        if artifact_identity(artifact) != member["artifact_identity"]:
            raise ArtifactContractError(f"export artifact identity mismatch for {name}")
        provenance = canonical_provenance(provenances[name])
        if provenance_identity(provenance) != member["provenance_identity"]:
            raise ArtifactContractError(f"export provenance identity mismatch for {name}")
        if provenance["artifact_identity"] != member["artifact_identity"]:
            raise ArtifactContractError(f"export provenance/artifact binding mismatch for {name}")
        if rank[artifact["access_class"]] < rank[value["access_class"]]:
            raise ArtifactContractError(f"export weakens artifact access class for {name}")
    return value


def canonical_cache_request(request):
    """Validate CacheRequest/v1 exact-reuse semantics.

    ``policy=disabled`` is a hard refusal. ``exact-domain`` only defines an
    identity boundary; the caller must separately possess scientific/stage
    authorization to use a cache. No lookup or storage happens here.
    """
    _exact(
        request,
        {
            "schema", "policy", "domain", "plan_id", "step_identity",
            "implementation_identity", "backend_identity", "inputs", "parameters_identity",
        },
        "cache request",
    )
    if request["schema"] != CACHE_SCHEMA:
        raise ArtifactContractError("unsupported cache request schema")
    policy = request["policy"]
    if policy not in _ALLOWED_CACHE_POLICY:
        raise ArtifactContractError("cache policy is invalid")
    if policy == "disabled":
        raise ArtifactContractError("cache reuse is disabled")
    domain = request["domain"]
    if domain not in ("Development", "Validation"):
        raise ArtifactContractError("cache domain is invalid")
    return {
        "schema": CACHE_SCHEMA,
        "policy": policy,
        "domain": domain,
        "plan_id": _sha256(request["plan_id"], "cache.plan_id"),
        "step_identity": _sha256(request["step_identity"], "cache.step_identity"),
        "implementation_identity": _text(request["implementation_identity"], "cache.implementation_identity"),
        "backend_identity": _text(request["backend_identity"], "cache.backend_identity"),
        "inputs": _canonical_named_identities(request["inputs"], "cache.inputs"),
        "parameters_identity": _sha256(request["parameters_identity"], "cache.parameters_identity"),
    }


def cache_key(request):
    """Return an exact-domain cache key; never authorizes cache use by itself."""
    return _digest("tdi-cache-key/v1", canonical_cache_request(request))


def validate_cache_reuse(entry, request, *, cache_authorized):
    """Fail closed unless a declared cache entry is an exact authorized reuse.

    ``cache_authorized`` must come from the owning scientific protocol/stage
    policy. This function deliberately refuses to infer that authorization.
    """
    if type(cache_authorized) is not bool or not cache_authorized:
        raise ArtifactContractError("cache reuse lacks caller authorization")
    expected = canonical_cache_request(request)
    _exact(
        entry,
        {"schema", "cache_key", "artifact_identity", "provenance_identity", "request"},
        "cache entry",
    )
    if entry["schema"] != CACHE_SCHEMA:
        raise ArtifactContractError("unsupported cache entry schema")
    _sha256(entry["cache_key"], "cache_entry.cache_key")
    _sha256(entry["artifact_identity"], "cache_entry.artifact_identity")
    _sha256(entry["provenance_identity"], "cache_entry.provenance_identity")
    actual_request = canonical_cache_request(entry["request"])
    expected_key = cache_key(expected)
    if entry["cache_key"] != expected_key or actual_request != expected:
        raise ArtifactContractError("cache entry request binding mismatch")
    return {
        "artifact_identity": entry["artifact_identity"],
        "provenance_identity": entry["provenance_identity"],
        "cache_key": expected_key,
    }
