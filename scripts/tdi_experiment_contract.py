"""Versioned TDI experiment identities and worker-response contracts.

This module separates scientific-question identity, concrete execution-plan
identity, trial/attempt identity and variable operational telemetry. It does not
authorize any scientific stage and it deliberately has no final-domain policy.
"""
from __future__ import annotations

import hashlib
import json
import re

SPEC_SCHEMA = 1
WORKER_RESPONSE_SCHEMA = 2
MAX_TEXT_BYTES = 16_384
MAX_LIST_ITEMS = 4_096
MAX_RESULT_DEPTH = 16
UINT64_MAX = (1 << 64) - 1
JSON_SAFE_INTEGER = (1 << 53) - 1
_DECIMAL_U64 = re.compile(r"(?:0|[1-9][0-9]*)\Z")
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")


class ExperimentContractError(ValueError):
    """A versioned experiment or worker-response contract is invalid."""


def canonical(value):
    """Return deterministic finite JSON bytes for identity computation."""
    try:
        return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False,
                          allow_nan=False).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise ExperimentContractError("value is not canonical finite JSON") from error


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + canonical(value)).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise ExperimentContractError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, max_bytes=MAX_TEXT_BYTES):
    if not isinstance(value, str) or not value.strip():
        raise ExperimentContractError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > max_bytes:
        raise ExperimentContractError(f"{name} exceeds the text bound")
    return value


def _string_list(value, name, *, allow_empty=False):
    if not isinstance(value, list) or (not allow_empty and not value):
        raise ExperimentContractError(f"{name} must be a bounded string list")
    if len(value) > MAX_LIST_ITEMS:
        raise ExperimentContractError(f"{name} exceeds the item bound")
    seen = set()
    for index, item in enumerate(value):
        item = _text(item, f"{name}[{index}]")
        if item in seen:
            raise ExperimentContractError(f"{name} contains a duplicate")
        seen.add(item)
    return value


def _u64(value, name, *, positive=False):
    lower = 1 if positive else 0
    if type(value) is not int or not lower <= value <= JSON_SAFE_INTEGER:
        qualifier = "positive " if positive else "non-negative "
        raise ExperimentContractError(
            f"{name} must be a {qualifier}JSON-safe integer <= {JSON_SAFE_INTEGER}"
        )
    return value


def _bool(value, name):
    if type(value) is not bool:
        raise ExperimentContractError(f"{name} must be boolean")
    return value


def _optional_u64(value, name, *, positive=False):
    return None if value is None else _u64(value, name, positive=positive)


def _sha256(value, name):
    if not isinstance(value, str) or _HEX64.fullmatch(value) is None:
        raise ExperimentContractError(f"{name} must be lowercase SHA-256")
    return value


def _records(value, name, *, allow_empty=False):
    if not isinstance(value, list) or (not allow_empty and not value):
        raise ExperimentContractError(f"{name} must be a bounded record list")
    if len(value) > MAX_LIST_ITEMS:
        raise ExperimentContractError(f"{name} exceeds the item bound")
    return value


def _validated_artifacts(value):
    """Validate and canonically order worker artifact references by name."""
    artifacts = _records(value, "artifacts", allow_empty=True)
    names = set()
    normalized = []
    for index, artifact in enumerate(artifacts):
        _exact(artifact, {"name", "sha256", "access_class"}, f"artifacts[{index}]")
        name = _text(artifact["name"], f"artifacts[{index}].name")
        _sha256(artifact["sha256"], f"artifacts[{index}].sha256")
        _text(artifact["access_class"], f"artifacts[{index}].access_class")
        if name in names:
            raise ExperimentContractError("duplicate artifact name")
        names.add(name)
        normalized.append({
            "name": name,
            "sha256": artifact["sha256"],
            "access_class": artifact["access_class"],
        })
    normalized.sort(key=lambda artifact: artifact["name"])
    return normalized


def _canonical_result_value(value, name="result", depth=0):
    """Validate result values with language-independent numeric rules.

    Worker-response/v2 intentionally excludes JSON floating-point values from
    canonical scientific identity. Encode floats as an explicitly specified
    string representation until a later schema freezes cross-language semantics.
    """
    if depth > MAX_RESULT_DEPTH:
        raise ExperimentContractError(f"{name} exceeds result nesting depth")
    if value is None or type(value) is bool:
        return
    if type(value) is int:
        if not -JSON_SAFE_INTEGER <= value <= JSON_SAFE_INTEGER:
            raise ExperimentContractError(
                f"{name} integer exceeds the language-independent JSON-safe range"
            )
        return
    if isinstance(value, float):
        raise ExperimentContractError(
            f"{name} floating-point values require an explicit encoded representation"
        )
    if isinstance(value, str):
        if len(value.encode("utf-8")) > MAX_TEXT_BYTES:
            raise ExperimentContractError(f"{name} string exceeds the text bound")
        return
    if isinstance(value, list):
        if len(value) > MAX_LIST_ITEMS:
            raise ExperimentContractError(f"{name} exceeds the item bound")
        for index, item in enumerate(value):
            _canonical_result_value(item, f"{name}[{index}]", depth + 1)
        return
    if isinstance(value, dict):
        if len(value) > MAX_LIST_ITEMS:
            raise ExperimentContractError(f"{name} exceeds the item bound")
        for key, item in value.items():
            _text(key, f"{name} key", max_bytes=256)
            _canonical_result_value(item, f"{name}.{key}", depth + 1)
        return
    raise ExperimentContractError(f"{name} contains an unsupported JSON value")


def validate_experiment_spec(spec):
    """Validate ExperimentSpec/v1 and return it unchanged.

    Scientific meaning remains owned by the series protocol. Numeric identity
    fields are restricted to the JavaScript-safe integer range; larger values
    require a later string-encoded field/schema rather than silent rounding.
    """
    _exact(spec, {
        "schema", "semantic_version", "series_id", "stage", "protocol_refs",
        "hypothesis_ids", "metrics", "data", "randomness", "arms", "adapter",
        "logical_budget", "physical_constraints", "retry", "statistics",
        "artifacts", "dependencies",
    }, "experiment spec")
    if spec["schema"] != SPEC_SCHEMA:
        raise ExperimentContractError("unsupported experiment spec schema")
    _text(spec["semantic_version"], "semantic_version", max_bytes=256)
    _text(spec["series_id"], "series_id", max_bytes=256)
    _text(spec["stage"], "stage", max_bytes=256)

    names = set()
    for index, ref in enumerate(_records(spec["protocol_refs"], "protocol_refs")):
        _exact(ref, {"name", "sha256"}, f"protocol_refs[{index}]")
        name = _text(ref["name"], f"protocol_refs[{index}].name")
        _sha256(ref["sha256"], f"protocol_refs[{index}].sha256")
        if name in names:
            raise ExperimentContractError("duplicate protocol reference name")
        names.add(name)
    _string_list(spec["hypothesis_ids"], "hypothesis_ids")

    metrics = set()
    for index, metric in enumerate(_records(spec["metrics"], "metrics")):
        _exact(metric, {"id", "unit", "orientation"}, f"metrics[{index}]")
        metric_id = _text(metric["id"], f"metrics[{index}].id")
        _text(metric["unit"], f"metrics[{index}].unit")
        if metric["orientation"] not in ("minimize", "maximize", "descriptive"):
            raise ExperimentContractError("metric orientation is invalid")
        if metric_id in metrics:
            raise ExperimentContractError("duplicate metric id")
        metrics.add(metric_id)

    data = _exact(spec["data"], {"access_class", "generator_identity", "partition_derivation"},
                  "data")
    if data["access_class"] not in ("public", "development", "validation",
                                     "restricted-reference"):
        raise ExperimentContractError("data access_class is invalid")
    _text(data["generator_identity"], "data.generator_identity")
    _text(data["partition_derivation"], "data.partition_derivation")

    randomness = _exact(spec["randomness"], {"algorithm", "seed_namespace", "stream_derivation"},
                        "randomness")
    _text(randomness["algorithm"], "randomness.algorithm")
    _text(randomness["seed_namespace"], "randomness.seed_namespace")
    _text(randomness["stream_derivation"], "randomness.stream_derivation")

    arm_ids = set()
    for index, arm in enumerate(_records(spec["arms"], "arms")):
        _exact(arm, {"id", "role", "implementation_identity"}, f"arms[{index}]")
        arm_id = _text(arm["id"], f"arms[{index}].id")
        if arm["role"] not in ("baseline", "candidate", "intervention"):
            raise ExperimentContractError("arm role is invalid")
        _text(arm["implementation_identity"], f"arms[{index}].implementation_identity")
        if arm_id in arm_ids:
            raise ExperimentContractError("duplicate arm id")
        arm_ids.add(arm_id)

    adapter = _exact(spec["adapter"], {
        "api_version", "implementation_identity", "required_capabilities",
        "reproducibility_class",
    }, "adapter")
    _text(adapter["api_version"], "adapter.api_version", max_bytes=256)
    _text(adapter["implementation_identity"], "adapter.implementation_identity")
    _string_list(adapter["required_capabilities"], "adapter.required_capabilities", allow_empty=True)
    if adapter["reproducibility_class"] not in ("exact", "numeric", "statistical"):
        raise ExperimentContractError("adapter reproducibility_class is invalid")

    budget = _exact(spec["logical_budget"], {
        "max_trials", "max_steps_per_trial", "max_observations_per_trial",
    }, "logical_budget")
    _u64(budget["max_trials"], "logical_budget.max_trials", positive=True)
    _u64(budget["max_steps_per_trial"], "logical_budget.max_steps_per_trial")
    _u64(budget["max_observations_per_trial"], "logical_budget.max_observations_per_trial")

    physical = _exact(spec["physical_constraints"], {
        "timeout_milliseconds", "max_output_bytes", "memory_max_bytes", "swap_max_bytes",
        "cpu_quota_us", "cpu_period_us", "pids_max", "gpu_required",
        "gpu_memory_max_bytes",
    }, "physical_constraints")
    _u64(physical["timeout_milliseconds"], "physical_constraints.timeout_milliseconds", positive=True)
    _u64(physical["max_output_bytes"], "physical_constraints.max_output_bytes", positive=True)
    _u64(physical["memory_max_bytes"], "physical_constraints.memory_max_bytes", positive=True)
    _u64(physical["swap_max_bytes"], "physical_constraints.swap_max_bytes")
    _u64(physical["cpu_quota_us"], "physical_constraints.cpu_quota_us", positive=True)
    _u64(physical["cpu_period_us"], "physical_constraints.cpu_period_us", positive=True)
    _u64(physical["pids_max"], "physical_constraints.pids_max", positive=True)
    _bool(physical["gpu_required"], "physical_constraints.gpu_required")
    _optional_u64(physical["gpu_memory_max_bytes"],
                  "physical_constraints.gpu_memory_max_bytes", positive=True)

    retry = _exact(spec["retry"], {"policy", "max_attempts_per_trial"}, "retry")
    if retry["policy"] not in ("never", "explicit-protocol-only"):
        raise ExperimentContractError("retry policy is invalid")
    _u64(retry["max_attempts_per_trial"], "retry.max_attempts_per_trial", positive=True)
    if retry["policy"] == "never" and retry["max_attempts_per_trial"] != 1:
        raise ExperimentContractError("retry=never requires exactly one attempt")

    statistics = _exact(spec["statistics"], {"plan_identity", "missing_observation_policy"},
                        "statistics")
    _text(statistics["plan_identity"], "statistics.plan_identity")
    _text(statistics["missing_observation_policy"], "statistics.missing_observation_policy")

    artifact_policy = _exact(spec["artifacts"],
                             {"access_class", "retention_policy", "export_policy"}, "artifacts")
    _text(artifact_policy["access_class"], "artifacts.access_class")
    _text(artifact_policy["retention_policy"], "artifacts.retention_policy")
    _text(artifact_policy["export_policy"], "artifacts.export_policy")

    dependencies = set()
    for index, dependency in enumerate(_records(spec["dependencies"], "dependencies")):
        _exact(dependency, {"name", "identity"}, f"dependencies[{index}]")
        name = _text(dependency["name"], f"dependencies[{index}].name")
        _text(dependency["identity"], f"dependencies[{index}].identity")
        if name in dependencies:
            raise ExperimentContractError("duplicate dependency name")
        dependencies.add(name)
    return spec


def question_identity(spec):
    """Return the semantic question/protocol identity."""
    validate_experiment_spec(spec)
    value = {
        "semantic_version": spec["semantic_version"], "series_id": spec["series_id"],
        "stage": spec["stage"], "protocol_refs": spec["protocol_refs"],
        "hypothesis_ids": spec["hypothesis_ids"], "metrics": spec["metrics"],
        "data": spec["data"], "randomness": spec["randomness"],
        "statistics": spec["statistics"],
    }
    return _digest("tdi-question/v1", value)


def experiment_plan_identity(spec):
    """Return the full ExperimentSpec computation-plan identity."""
    validate_experiment_spec(spec)
    return _digest("tdi-experiment-plan/v1", spec)


def trial_identity(plan_id, domain, index):
    _sha256(plan_id, "plan_id")
    if domain not in ("Development", "Validation"):
        raise ExperimentContractError("trial domain is invalid")
    _u64(index, "trial index")
    return _digest("tdi-trial/v1", {"plan_id": plan_id, "domain": domain, "index": index})


def attempt_identity(plan_id, trial_id, backend_identity, ordinal):
    _sha256(plan_id, "plan_id")
    _sha256(trial_id, "trial_id")
    _text(backend_identity, "backend_identity")
    _u64(ordinal, "attempt ordinal")
    return _digest("tdi-attempt/v2", {
        "plan_id": plan_id, "trial_id": trial_id,
        "backend_identity": backend_identity, "ordinal": ordinal,
    })


def stream_identity(question_id, trial_id, namespace, stream):
    """Return a stable RNG coordinate, not a sample or proof of independence."""
    _sha256(question_id, "question_id")
    _sha256(trial_id, "trial_id")
    _text(namespace, "stream namespace")
    _text(stream, "stream name")
    return _digest("tdi-rng-stream/v1", {
        "question_id": question_id, "trial_id": trial_id,
        "namespace": namespace, "stream": stream,
    })


def parse_seed_decimal(value):
    """Parse canonical decimal u64 transported as a JSON string."""
    if not isinstance(value, str) or _DECIMAL_U64.fullmatch(value) is None:
        raise ExperimentContractError("seed_decimal must be canonical unsigned decimal text")
    seed = int(value)
    if seed > UINT64_MAX:
        raise ExperimentContractError("seed_decimal exceeds u64")
    return seed


def validate_worker_response_v2(response, *, experiment_id, plan_id, trial_id, attempt_id,
                                backend_identity, domain, seed, max_completed_steps,
                                max_completed_observations):
    """Validate a worker response and all caller-owned identity/budget bindings."""
    _exact(response, {
        "schema", "execution_status", "scientific_disposition", "experiment_id", "plan_id",
        "trial_id", "attempt_id", "backend_identity", "domain", "seed_decimal", "progress",
        "artifacts", "result", "error",
    }, "worker response")
    if response["schema"] != WORKER_RESPONSE_SCHEMA:
        raise ExperimentContractError("unsupported worker response schema")
    if response["execution_status"] != "completed":
        raise ExperimentContractError("successful worker process must report execution_status=completed")
    if response["scientific_disposition"] not in ("evaluated", "rejected"):
        raise ExperimentContractError("scientific_disposition is invalid")
    expected = {
        "experiment_id": experiment_id, "plan_id": plan_id, "trial_id": trial_id,
        "attempt_id": attempt_id, "backend_identity": backend_identity, "domain": domain,
    }
    for name, value in expected.items():
        if response[name] != value:
            raise ExperimentContractError(f"worker response {name} mismatch")
    if parse_seed_decimal(response["seed_decimal"]) != seed:
        raise ExperimentContractError("worker response seed mismatch")

    _u64(max_completed_steps, "max_completed_steps")
    _u64(max_completed_observations, "max_completed_observations")
    progress = _exact(
        response["progress"],
        {"completed_steps", "completed_observations", "costs"},
        "progress",
    )
    _u64(progress["completed_steps"], "progress.completed_steps")
    _u64(progress["completed_observations"], "progress.completed_observations")
    if progress["completed_steps"] > max_completed_steps:
        raise ExperimentContractError("worker completed_steps exceeds ExperimentSpec logical budget")
    if progress["completed_observations"] > max_completed_observations:
        raise ExperimentContractError(
            "worker completed_observations exceeds ExperimentSpec logical budget"
        )
    if not isinstance(progress["costs"], dict) or len(progress["costs"]) > MAX_LIST_ITEMS:
        raise ExperimentContractError("progress.costs must be a bounded object")
    for name, value in progress["costs"].items():
        _text(name, "progress cost name", max_bytes=256)
        _u64(value, f"progress.costs.{name}")

    _validated_artifacts(response["artifacts"])

    if response["result"] is not None and not isinstance(response["result"], dict):
        raise ExperimentContractError("worker result must be object or null")
    if response["result"] is not None:
        _canonical_result_value(response["result"])
    if response["error"] is not None:
        error = _exact(response["error"], {"code", "message"}, "worker error")
        _text(error["code"], "worker error code", max_bytes=256)
        _text(error["message"], "worker error message", max_bytes=4096)
    if response["scientific_disposition"] == "evaluated" and response["error"] is not None:
        raise ExperimentContractError("evaluated response cannot contain an error")
    return response


def scientific_result_identity(response):
    """Hash scientific output/artifacts while excluding variable operational telemetry."""
    if not isinstance(response, dict):
        raise ExperimentContractError("worker response must be an object")
    for name in ("experiment_id", "plan_id", "trial_id", "scientific_disposition",
                 "artifacts", "result"):
        if name not in response:
            raise ExperimentContractError(f"worker response missing {name}")
    artifacts = _validated_artifacts(response["artifacts"])
    if response["result"] is not None:
        _canonical_result_value(response["result"])
    return _digest("tdi-scientific-result/v1", {
        "experiment_id": response["experiment_id"], "plan_id": response["plan_id"],
        "trial_id": response["trial_id"],
        "scientific_disposition": response["scientific_disposition"],
        "artifacts": artifacts,
        "result": response["result"],
    })
