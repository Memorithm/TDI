"""Versioned checkpoint identity and exact-resume validation for TDI engine trials.

Checkpoint manifests bind scientific/execution identity, frozen logical budgets,
causal progress, RNG stream coordinates, immutable input artifacts and one
content-addressed state artifact. This module never authorizes a scientific stage
and never decides retry policy.
"""
from __future__ import annotations

import hashlib
import re

import tdi_experiment_contract as experiment

CHECKPOINT_SCHEMA = 1
MAX_RECORDS = 4_096
MAX_TEXT_BYTES = 16_384
_KEY = re.compile(r"[a-z0-9][a-z0-9_-]{0,63}\Z")
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")


class CheckpointContractError(ValueError):
    """A checkpoint manifest or exact-resume request violates the contract."""


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise CheckpointContractError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, max_bytes=MAX_TEXT_BYTES):
    if not isinstance(value, str) or not value.strip():
        raise CheckpointContractError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > max_bytes:
        raise CheckpointContractError(f"{name} exceeds the text bound")
    return value


def _sha256(value, name):
    if not isinstance(value, str) or _HEX64.fullmatch(value) is None:
        raise CheckpointContractError(f"{name} must be lowercase SHA-256")
    return value


def _safe_integer(value, name, *, positive=False):
    lower = 1 if positive else 0
    if type(value) is not int or not lower <= value <= experiment.JSON_SAFE_INTEGER:
        qualifier = "positive " if positive else "non-negative "
        raise CheckpointContractError(
            f"{name} must be a {qualifier}JSON-safe integer <= {experiment.JSON_SAFE_INTEGER}"
        )
    return value


def _step_key(value, name="step_key"):
    if not isinstance(value, str) or _KEY.fullmatch(value) is None:
        raise CheckpointContractError(
            f"{name} must match [a-z0-9][a-z0-9_-]{{0,63}}"
        )
    return value


def _parse_seed(value):
    try:
        return experiment.parse_seed_decimal(value)
    except experiment.ExperimentContractError as error:
        raise CheckpointContractError(str(error)) from error


def _parse_u64_decimal(value, name):
    try:
        return experiment.parse_seed_decimal(value)
    except experiment.ExperimentContractError as error:
        raise CheckpointContractError(f"{name}: {error}") from error


def _canonical_inputs(records):
    if not isinstance(records, list) or len(records) > MAX_RECORDS:
        raise CheckpointContractError("input_artifacts must be a bounded list")
    seen = set()
    normalized = []
    for index, record in enumerate(records):
        _exact(record, {"name", "sha256"}, f"input_artifacts[{index}]")
        name = _text(record["name"], f"input_artifacts[{index}].name", max_bytes=256)
        digest = _sha256(record["sha256"], f"input_artifacts[{index}].sha256")
        if name in seen:
            raise CheckpointContractError("duplicate checkpoint input artifact name")
        seen.add(name)
        normalized.append({"name": name, "sha256": digest})
    normalized.sort(key=lambda item: item["name"])
    return normalized


def _canonical_streams(records):
    if not isinstance(records, list) or len(records) > MAX_RECORDS:
        raise CheckpointContractError("rng_streams must be a bounded list")
    seen = set()
    normalized = []
    for index, record in enumerate(records):
        _exact(record, {"name", "stream_id", "counter_decimal"}, f"rng_streams[{index}]")
        name = _text(record["name"], f"rng_streams[{index}].name", max_bytes=256)
        stream_id = _sha256(record["stream_id"], f"rng_streams[{index}].stream_id")
        _parse_u64_decimal(record["counter_decimal"], f"rng_streams[{index}].counter_decimal")
        if name in seen:
            raise CheckpointContractError("duplicate checkpoint RNG stream name")
        seen.add(name)
        normalized.append({
            "name": name,
            "stream_id": stream_id,
            "counter_decimal": record["counter_decimal"],
        })
    normalized.sort(key=lambda item: item["name"])
    return normalized


def canonical_checkpoint(manifest):
    """Validate and canonicalize a CheckpointManifest/v1 for identity hashing.

    Frozen step/observation budgets are carried by the manifest itself so a
    resume boundary cannot silently reinterpret progress under wider caller
    limits. The owning engine still has to supply the same plan-derived budgets
    to ``validate_resume``.
    """
    _exact(
        manifest,
        {
            "schema",
            "experiment_id",
            "plan_id",
            "trial_id",
            "step_key",
            "step_identity",
            "adapter_identity",
            "backend_identity",
            "seed_decimal",
            "checkpoint_ordinal",
            "budgets",
            "progress",
            "rng_streams",
            "input_artifacts",
            "state",
        },
        "checkpoint manifest",
    )
    if manifest["schema"] != CHECKPOINT_SCHEMA:
        raise CheckpointContractError("unsupported checkpoint schema")
    _sha256(manifest["experiment_id"], "experiment_id")
    _sha256(manifest["plan_id"], "plan_id")
    _sha256(manifest["trial_id"], "trial_id")
    _step_key(manifest["step_key"])
    _sha256(manifest["step_identity"], "step_identity")
    _text(manifest["adapter_identity"], "adapter_identity")
    _text(manifest["backend_identity"], "backend_identity")
    _parse_seed(manifest["seed_decimal"])
    _safe_integer(manifest["checkpoint_ordinal"], "checkpoint_ordinal")

    budgets = _exact(
        manifest["budgets"],
        {"max_steps", "max_observations"},
        "checkpoint budgets",
    )
    _safe_integer(budgets["max_steps"], "budgets.max_steps")
    _safe_integer(budgets["max_observations"], "budgets.max_observations")

    progress = _exact(
        manifest["progress"],
        {"completed_steps", "completed_observations"},
        "checkpoint progress",
    )
    _safe_integer(progress["completed_steps"], "progress.completed_steps")
    _safe_integer(progress["completed_observations"], "progress.completed_observations")
    if progress["completed_steps"] > budgets["max_steps"]:
        raise CheckpointContractError("checkpoint completed_steps exceeds embedded frozen budget")
    if progress["completed_observations"] > budgets["max_observations"]:
        raise CheckpointContractError(
            "checkpoint completed_observations exceeds embedded frozen budget"
        )

    state = _exact(manifest["state"], {"sha256", "media_type", "size_bytes"}, "checkpoint state")
    _sha256(state["sha256"], "state.sha256")
    _text(state["media_type"], "state.media_type", max_bytes=512)
    _safe_integer(state["size_bytes"], "state.size_bytes", positive=True)

    return {
        "schema": CHECKPOINT_SCHEMA,
        "experiment_id": manifest["experiment_id"],
        "plan_id": manifest["plan_id"],
        "trial_id": manifest["trial_id"],
        "step_key": manifest["step_key"],
        "step_identity": manifest["step_identity"],
        "adapter_identity": manifest["adapter_identity"],
        "backend_identity": manifest["backend_identity"],
        "seed_decimal": manifest["seed_decimal"],
        "checkpoint_ordinal": manifest["checkpoint_ordinal"],
        "budgets": {
            "max_steps": budgets["max_steps"],
            "max_observations": budgets["max_observations"],
        },
        "progress": {
            "completed_steps": progress["completed_steps"],
            "completed_observations": progress["completed_observations"],
        },
        "rng_streams": _canonical_streams(manifest["rng_streams"]),
        "input_artifacts": _canonical_inputs(manifest["input_artifacts"]),
        "state": {
            "sha256": state["sha256"],
            "media_type": state["media_type"],
            "size_bytes": state["size_bytes"],
        },
    }


def checkpoint_identity(manifest):
    """Return content identity for the complete validated checkpoint manifest."""
    return _digest("tdi-checkpoint/v1", canonical_checkpoint(manifest))


def validate_resume(
    manifest,
    *,
    experiment_id,
    plan_id,
    trial_id,
    step_key,
    step_identity,
    adapter_identity,
    backend_identity,
    seed,
    input_artifacts,
    rng_stream_ids,
    max_steps,
    max_observations,
):
    """Fail closed unless a checkpoint is an exact resume of the declared step.

    RNG counters may advance and state bytes naturally differ between checkpoints,
    but stream identities, immutable inputs, frozen budgets and every caller-owned
    identity must match exactly. The function does not authorize a retry or a
    scientific stage.
    """
    value = canonical_checkpoint(manifest)
    expected = {
        "experiment_id": experiment_id,
        "plan_id": plan_id,
        "trial_id": trial_id,
        "step_key": step_key,
        "step_identity": step_identity,
        "adapter_identity": adapter_identity,
        "backend_identity": backend_identity,
    }
    _sha256(experiment_id, "expected experiment_id")
    _sha256(plan_id, "expected plan_id")
    _sha256(trial_id, "expected trial_id")
    _step_key(step_key, "expected step_key")
    _sha256(step_identity, "expected step_identity")
    _text(adapter_identity, "expected adapter_identity")
    _text(backend_identity, "expected backend_identity")
    for name, expected_value in expected.items():
        if value[name] != expected_value:
            raise CheckpointContractError(f"checkpoint {name} mismatch")

    if type(seed) is not int or seed < 0 or seed > experiment.UINT64_MAX:
        raise CheckpointContractError("expected seed must be u64")
    if _parse_seed(value["seed_decimal"]) != seed:
        raise CheckpointContractError("checkpoint seed mismatch")

    _safe_integer(max_steps, "max_steps")
    _safe_integer(max_observations, "max_observations")
    expected_budgets = {"max_steps": max_steps, "max_observations": max_observations}
    if value["budgets"] != expected_budgets:
        raise CheckpointContractError("checkpoint frozen budget binding mismatch")
    if value["progress"]["completed_steps"] > max_steps:
        raise CheckpointContractError("checkpoint completed_steps exceeds frozen budget")
    if value["progress"]["completed_observations"] > max_observations:
        raise CheckpointContractError("checkpoint completed_observations exceeds frozen budget")

    if not isinstance(input_artifacts, dict) or len(input_artifacts) > MAX_RECORDS:
        raise CheckpointContractError("expected input_artifacts must be a bounded mapping")
    expected_inputs = _canonical_inputs([
        {"name": name, "sha256": digest} for name, digest in input_artifacts.items()
    ])
    if value["input_artifacts"] != expected_inputs:
        raise CheckpointContractError("checkpoint input artifact binding mismatch")

    if not isinstance(rng_stream_ids, dict) or len(rng_stream_ids) > MAX_RECORDS:
        raise CheckpointContractError("expected rng_stream_ids must be a bounded mapping")
    for name, stream_id in rng_stream_ids.items():
        _text(name, "expected RNG stream name", max_bytes=256)
        _sha256(stream_id, "expected RNG stream id")
    expected_streams = sorted(rng_stream_ids.items())
    actual_streams = [(item["name"], item["stream_id"]) for item in value["rng_streams"]]
    if actual_streams != expected_streams:
        raise CheckpointContractError("checkpoint RNG stream binding mismatch")
    return value
