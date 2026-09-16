"""Portable TDI provenance, artifact and export contracts.

This module defines scientific/evidence identities only. It does not implement a
physical content-addressed store, registry service, remote transport or export
authorization. Storage remains owned by the selected backend (for example
scirust-hub); TDI keeps a portable raw SHA-256 over the actual artifact bytes.
"""
from __future__ import annotations

import hashlib
import re
import uuid

import tdi_experiment_contract as experiment

ARTIFACT_SCHEMA = 1
PROVENANCE_SCHEMA = 1
STORAGE_BINDING_SCHEMA = 1
EXPORT_SCHEMA = 1
MAX_RECORDS = 4096
ACCESS_CLASSES = {"public", "development", "validation", "restricted-reference"}
ROLES = {"input", "result", "checkpoint", "evidence", "report", "log", "other"}
HUB_PROVIDER = "scirust-hub"
HUB_ARTIFACT_DIGEST_NAMESPACE = "scirust-hub:artifact-blob:v1"
_HEX32 = re.compile(r"[0-9a-f]{32}\Z")
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")


class ProvenanceContractError(ValueError):
    """A portable provenance/export contract is invalid."""


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise ProvenanceContractError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, max_bytes=16_384):
    if not isinstance(value, str) or not value.strip():
        raise ProvenanceContractError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > max_bytes:
        raise ProvenanceContractError(f"{name} exceeds the text bound")
    return value


def _sha256(value, name):
    if not isinstance(value, str) or _HEX64.fullmatch(value) is None:
        raise ProvenanceContractError(f"{name} must be lowercase SHA-256")
    return value


def _attempt_id(value, name):
    if not isinstance(value, str) or (
        _HEX64.fullmatch(value) is None and _HEX32.fullmatch(value) is None
    ):
        raise ProvenanceContractError(f"{name} must be a lowercase 32- or 64-hex attempt id")
    return value


def _safe_int(value, name):
    if type(value) is not int or not 0 <= value <= experiment.JSON_SAFE_INTEGER:
        raise ProvenanceContractError(
            f"{name} must be a non-negative JSON-safe integer <= {experiment.JSON_SAFE_INTEGER}"
        )
    return value


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _bounded_list(value, name, *, allow_empty=False):
    if not isinstance(value, list) or (not allow_empty and not value):
        raise ProvenanceContractError(f"{name} must be a bounded list")
    if len(value) > MAX_RECORDS:
        raise ProvenanceContractError(f"{name} exceeds the item bound")
    return value


def _producer(value):
    if value is None:
        return None
    _exact(
        value,
        {
            "experiment_id",
            "plan_id",
            "trial_id",
            "attempt_id",
            "step_identity",
            "scientific_result_id",
        },
        "artifact producer",
    )
    result = {}
    for name in ("experiment_id", "plan_id", "trial_id", "step_identity"):
        result[name] = _sha256(value[name], f"artifact producer.{name}")
    result["attempt_id"] = _attempt_id(value["attempt_id"], "artifact producer.attempt_id")
    result["scientific_result_id"] = (
        None
        if value["scientific_result_id"] is None
        else _sha256(value["scientific_result_id"], "artifact producer.scientific_result_id")
    )
    return result


def canonical_artifact(descriptor):
    """Validate PortableArtifact/v1 and return its canonical semantic record."""
    _exact(
        descriptor,
        {
            "schema",
            "name",
            "role",
            "media_type",
            "size_bytes",
            "sha256",
            "access_class",
            "producer",
        },
        "portable artifact",
    )
    if descriptor["schema"] != ARTIFACT_SCHEMA:
        raise ProvenanceContractError("unsupported portable artifact schema")
    name = _text(descriptor["name"], "artifact.name", max_bytes=512)
    role = descriptor["role"]
    if role not in ROLES:
        raise ProvenanceContractError("artifact.role is invalid")
    media_type = _text(descriptor["media_type"], "artifact.media_type", max_bytes=512)
    size = _safe_int(descriptor["size_bytes"], "artifact.size_bytes")
    sha = _sha256(descriptor["sha256"], "artifact.sha256")
    access = descriptor["access_class"]
    if access not in ACCESS_CLASSES:
        raise ProvenanceContractError("artifact.access_class is invalid")
    return {
        "schema": ARTIFACT_SCHEMA,
        "name": name,
        "role": role,
        "media_type": media_type,
        "size_bytes": size,
        "sha256": sha,
        "access_class": access,
        "producer": _producer(descriptor["producer"]),
    }


def artifact_identity(descriptor):
    return _digest("tdi-portable-artifact/v1", canonical_artifact(descriptor))


def verify_artifact_bytes(descriptor, payload):
    """Verify actual bytes against a portable artifact descriptor."""
    value = canonical_artifact(descriptor)
    if not isinstance(payload, (bytes, bytearray, memoryview)):
        raise ProvenanceContractError("artifact payload must be bytes-like")
    raw = bytes(payload)
    if len(raw) != value["size_bytes"]:
        raise ProvenanceContractError("artifact payload size mismatch")
    if hashlib.sha256(raw).hexdigest() != value["sha256"]:
        raise ProvenanceContractError("artifact payload SHA-256 mismatch")
    return artifact_identity(value)


def canonical_storage_binding(binding, descriptor):
    """Validate provider metadata without treating it as portable-byte attestation."""
    artifact = canonical_artifact(descriptor)
    _exact(
        binding,
        {
            "schema",
            "artifact_identity",
            "portable_sha256",
            "provider",
            "provider_artifact_id",
            "provider_digest_namespace",
            "provider_digest",
        },
        "storage binding",
    )
    if binding["schema"] != STORAGE_BINDING_SCHEMA:
        raise ProvenanceContractError("unsupported storage binding schema")
    expected_identity = artifact_identity(artifact)
    if _sha256(binding["artifact_identity"], "storage binding.artifact_identity") != expected_identity:
        raise ProvenanceContractError("storage binding artifact identity mismatch")
    if _sha256(binding["portable_sha256"], "storage binding.portable_sha256") != artifact["sha256"]:
        raise ProvenanceContractError("storage binding portable SHA-256 mismatch")
    provider = _text(binding["provider"], "storage binding.provider", max_bytes=256)
    provider_artifact_id = _text(
        binding["provider_artifact_id"], "storage binding.provider_artifact_id", max_bytes=512
    )
    namespace = _text(
        binding["provider_digest_namespace"], "storage binding.provider_digest_namespace", max_bytes=256
    )
    provider_digest = _text(binding["provider_digest"], "storage binding.provider_digest", max_bytes=512)
    if provider == HUB_PROVIDER:
        try:
            if str(uuid.UUID(provider_artifact_id)) != provider_artifact_id:
                raise ValueError
        except (ValueError, AttributeError) as error:
            raise ProvenanceContractError("scirust-hub artifact id must be canonical UUID") from error
        if namespace != HUB_ARTIFACT_DIGEST_NAMESPACE:
            raise ProvenanceContractError("scirust-hub artifact digest namespace mismatch")
        provider_digest = _sha256(provider_digest, "scirust-hub ContentDigest")
    return {
        "schema": STORAGE_BINDING_SCHEMA,
        "artifact_identity": expected_identity,
        "portable_sha256": artifact["sha256"],
        "provider": provider,
        "provider_artifact_id": provider_artifact_id,
        "provider_digest_namespace": namespace,
        "provider_digest": provider_digest,
    }


def _artifact_ref_list(value, name):
    result = []
    seen = set()
    for index, item in enumerate(_bounded_list(value, name, allow_empty=True)):
        _exact(item, {"artifact_identity", "sha256"}, f"{name}[{index}]")
        identity = _sha256(item["artifact_identity"], f"{name}[{index}].artifact_identity")
        sha = _sha256(item["sha256"], f"{name}[{index}].sha256")
        if identity in seen:
            raise ProvenanceContractError(f"{name} contains duplicate artifact identity")
        seen.add(identity)
        result.append({"artifact_identity": identity, "sha256": sha})
    return sorted(result, key=lambda item: item["artifact_identity"])


def _dependencies(value):
    result = []
    seen = set()
    for index, item in enumerate(_bounded_list(value, "dependencies", allow_empty=True)):
        _exact(item, {"name", "identity"}, f"dependencies[{index}]")
        name = _text(item["name"], f"dependencies[{index}].name", max_bytes=256)
        identity = _text(item["identity"], f"dependencies[{index}].identity")
        if name in seen:
            raise ProvenanceContractError("duplicate dependency name")
        seen.add(name)
        result.append({"name": name, "identity": identity})
    return sorted(result, key=lambda item: item["name"])


def canonical_provenance(record):
    """Validate ProvenanceRecord/v1. Wall-clock values are deliberately absent."""
    _exact(
        record,
        {
            "schema",
            "experiment_id",
            "plan_id",
            "trial_id",
            "attempt_id",
            "step_identity",
            "checkpoint_id",
            "scientific_result_id",
            "adapter_identity",
            "backend_identity",
            "environment_identity",
            "inputs",
            "outputs",
            "dependencies",
        },
        "provenance record",
    )
    if record["schema"] != PROVENANCE_SCHEMA:
        raise ProvenanceContractError("unsupported provenance schema")
    value = {"schema": PROVENANCE_SCHEMA}
    for name in ("experiment_id", "plan_id", "trial_id", "step_identity"):
        value[name] = _sha256(record[name], f"provenance.{name}")
    value["attempt_id"] = _attempt_id(record["attempt_id"], "provenance.attempt_id")
    for name in ("checkpoint_id", "scientific_result_id"):
        item = record[name]
        value[name] = None if item is None else _sha256(item, f"provenance.{name}")
    for name in ("adapter_identity", "backend_identity", "environment_identity"):
        value[name] = _text(record[name], f"provenance.{name}")
    value["inputs"] = _artifact_ref_list(record["inputs"], "inputs")
    value["outputs"] = _artifact_ref_list(record["outputs"], "outputs")
    value["dependencies"] = _dependencies(record["dependencies"])
    return value


def provenance_identity(record):
    return _digest("tdi-provenance/v1", canonical_provenance(record))


def _producer_matches_provenance(producer, provenance):
    if producer is None:
        return True
    return all(
        producer[name] == provenance[name]
        for name in ("experiment_id", "plan_id", "trial_id", "attempt_id", "step_identity")
    ) and producer["scientific_result_id"] == provenance["scientific_result_id"]


def canonical_export(manifest, *, allowed_access_classes):
    """Validate ExportManifest/v1 and full portable-reference closure.

    `allowed_access_classes` is mandatory caller authority. The manifest never
    authorizes itself for validation/restricted export.
    """
    if not isinstance(allowed_access_classes, (set, frozenset)) or not allowed_access_classes:
        raise ProvenanceContractError("allowed_access_classes must be an explicit non-empty set")
    if not set(allowed_access_classes) <= ACCESS_CLASSES:
        raise ProvenanceContractError("allowed_access_classes contains an unknown class")
    _exact(
        manifest,
        {"schema", "kind", "experiment_id", "plan_id", "artifacts", "provenance", "roots"},
        "export manifest",
    )
    if manifest["schema"] != EXPORT_SCHEMA or manifest["kind"] != "tdi-portable-export":
        raise ProvenanceContractError("unsupported export manifest schema/kind")
    experiment_id = _sha256(manifest["experiment_id"], "export.experiment_id")
    plan_id = _sha256(manifest["plan_id"], "export.plan_id")

    artifacts = []
    artifacts_by_id = {}
    for descriptor in _bounded_list(manifest["artifacts"], "export.artifacts", allow_empty=True):
        value = canonical_artifact(descriptor)
        identity = artifact_identity(value)
        if identity in artifacts_by_id:
            raise ProvenanceContractError("duplicate portable artifact identity")
        if value["access_class"] not in allowed_access_classes:
            raise ProvenanceContractError(
                f"artifact access class {value['access_class']!r} is not explicitly authorized for export"
            )
        artifacts_by_id[identity] = value
        artifacts.append(value)

    provenance = []
    provenance_by_id = {}
    for record in _bounded_list(manifest["provenance"], "export.provenance"):
        value = canonical_provenance(record)
        identity = provenance_identity(value)
        if identity in provenance_by_id:
            raise ProvenanceContractError("duplicate provenance identity")
        if value["experiment_id"] != experiment_id or value["plan_id"] != plan_id:
            raise ProvenanceContractError("provenance experiment/plan identity escapes export root")
        for ref in [*value["inputs"], *value["outputs"]]:
            descriptor = artifacts_by_id.get(ref["artifact_identity"])
            if descriptor is None or descriptor["sha256"] != ref["sha256"]:
                raise ProvenanceContractError("provenance artifact reference is missing or hash-mismatched")
        provenance_by_id[identity] = value
        provenance.append(value)

    roots = []
    seen_roots = set()
    for root in _bounded_list(manifest["roots"], "export.roots"):
        root = _sha256(root, "export root provenance id")
        if root in seen_roots:
            raise ProvenanceContractError("duplicate export root")
        if root not in provenance_by_id:
            raise ProvenanceContractError("export root does not name included provenance")
        seen_roots.add(root)
        roots.append(root)

    # If an artifact claims a producer, that exact producing attempt/step/result
    # must be present and must list this artifact among its outputs. External
    # artifacts whose producing lineage is intentionally outside this export use
    # producer=null instead of an unverifiable partial producer claim.
    for identity, descriptor in artifacts_by_id.items():
        producer = descriptor["producer"]
        if producer is None:
            continue
        matched = False
        for record in provenance:
            if not _producer_matches_provenance(producer, record):
                continue
            if any(
                ref["artifact_identity"] == identity and ref["sha256"] == descriptor["sha256"]
                for ref in record["outputs"]
            ):
                matched = True
                break
        if not matched:
            raise ProvenanceContractError("artifact producer claim is not closed by included provenance")

    artifacts.sort(key=artifact_identity)
    provenance.sort(key=provenance_identity)
    roots.sort()
    return {
        "schema": EXPORT_SCHEMA,
        "kind": "tdi-portable-export",
        "experiment_id": experiment_id,
        "plan_id": plan_id,
        "artifacts": artifacts,
        "provenance": provenance,
        "roots": roots,
    }


def export_identity(manifest, *, allowed_access_classes):
    return _digest(
        "tdi-portable-export/v1",
        canonical_export(manifest, allowed_access_classes=allowed_access_classes),
    )


def verify_export_payloads(manifest, payloads, *, allowed_access_classes):
    """Verify a complete portable export payload map keyed by artifact identity."""
    value = canonical_export(manifest, allowed_access_classes=allowed_access_classes)
    if not isinstance(payloads, dict):
        raise ProvenanceContractError("export payloads must be a dictionary keyed by artifact identity")
    expected = {artifact_identity(item): item for item in value["artifacts"]}
    if set(payloads) != set(expected):
        raise ProvenanceContractError("export payload set does not exactly match manifest artifacts")
    for identity, descriptor in expected.items():
        if verify_artifact_bytes(descriptor, payloads[identity]) != identity:
            raise ProvenanceContractError("export payload identity mismatch")
    return export_identity(value, allowed_access_classes=allowed_access_classes)
