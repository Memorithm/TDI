"""Versioned Linux execution contracts for TDI Development/Validation trials.

Schema 2 preserves the qualified cgroup-v2 plan introduced by the containment
lot. Schema 3 adds ExperimentSpec/v1 and worker-response/v2 identities without
changing schema-2 semantics or authorizing a scientific stage.
"""
from pathlib import Path

import tdi_experiment_contract as experiment
import tdi_experiment_supervisor as durable
from tdi_linux_containment import ContainmentError, ResourceProfile


SCHEMA2_FIELDS = {
    "schema", "purpose", "domain", "indices", "argv", "artifacts",
    "timeout_seconds", "max_output_bytes", "max_trials", "execution",
}
SCHEMA3_FIELDS = SCHEMA2_FIELDS | {"experiment"}


def execution_profile(plan):
    """Validate and return the bound cgroup-v2 resource profile."""
    execution = plan.get("execution") if isinstance(plan, dict) else None
    if not isinstance(execution, dict) or set(execution) != {"backend", "profile"}:
        raise durable.ContractError("execution contract has unknown or missing fields")
    if execution["backend"] != "linux-cgroup-v2":
        raise durable.ContractError("execution backend must be linux-cgroup-v2")
    try:
        profile = ResourceProfile.from_json(execution["profile"])
        profile.validate_static_capabilities()
    except ContainmentError as error:
        raise durable.ContractError(str(error)) from error
    return profile


def _validate_legacy_surface(plan, root, extra_fields):
    legacy = {key: value for key, value in plan.items() if key not in extra_fields}
    legacy["schema"] = 1
    durable.validate(legacy, Path(root))


def _validate_experiment_binding(plan, profile):
    """Require the executable limits to equal the ExperimentSpec physical plan."""
    try:
        spec = experiment.validate_experiment_spec(plan["experiment"])
    except experiment.ExperimentContractError as error:
        raise durable.ContractError(str(error)) from error

    for index in plan["indices"]:
        if index > experiment.JSON_SAFE_INTEGER:
            raise durable.ContractError(
                "schema-3 trial indices must fit the language-independent JSON-safe range"
            )

    logical = spec["logical_budget"]
    if plan["max_trials"] > logical["max_trials"]:
        raise durable.ContractError("execution max_trials exceeds ExperimentSpec logical budget")

    physical = spec["physical_constraints"]
    expected = {
        "max_output_bytes": plan["max_output_bytes"],
        "memory_max_bytes": profile.memory_max_bytes,
        "swap_max_bytes": profile.swap_max_bytes,
        "cpu_quota_us": profile.cpu_quota_us,
        "cpu_period_us": profile.cpu_period_us,
        "pids_max": profile.pids_max,
        "gpu_required": profile.gpu_required,
        "gpu_memory_max_bytes": profile.gpu_memory_max_bytes,
    }
    for name, actual in expected.items():
        if physical[name] != actual:
            raise durable.ContractError(
                f"ExperimentSpec physical_constraints.{name} does not match execution plan"
            )
    if plan["timeout_seconds"] != physical["timeout_milliseconds"] / 1000:
        raise durable.ContractError(
            "ExperimentSpec timeout_milliseconds does not match execution timeout_seconds"
        )
    return spec


def validate_plan(plan, root):
    """Validate schema 2 or 3 while preserving schema-1 scientific semantics."""
    if not isinstance(plan, dict) or plan.get("schema") not in (2, 3):
        raise durable.ContractError("linux-cgroup-v2 runner requires plan schema 2 or 3")
    schema = plan["schema"]
    required = SCHEMA2_FIELDS if schema == 2 else SCHEMA3_FIELDS
    if set(plan) != required:
        raise durable.ContractError(f"unknown or missing schema-{schema} plan fields")
    _validate_legacy_surface(plan, root, {"execution", "experiment"})
    profile = execution_profile(plan)
    if schema == 3:
        _validate_experiment_binding(plan, profile)
    return plan_identity(plan)


def plan_identity(plan):
    """Return the concrete execution-plan identity for a validated plan shape."""
    if not isinstance(plan, dict) or plan.get("schema") not in (2, 3):
        raise durable.ContractError("unsupported plan identity schema")
    if plan["schema"] == 2:
        return durable.digest(durable.canonical(plan).encode())
    try:
        spec_id = experiment.experiment_plan_identity(plan["experiment"])
    except experiment.ExperimentContractError as error:
        raise durable.ContractError(str(error)) from error
    value = {"experiment_plan_id": spec_id, "execution_plan": plan}
    return durable.digest(b"tdi-execution-plan/v1\0" + experiment.canonical(value))


def experiment_identity(plan):
    """Return the question/protocol identity for schema 3, otherwise None."""
    if not isinstance(plan, dict) or plan.get("schema") != 3:
        return None
    try:
        return experiment.question_identity(plan["experiment"])
    except experiment.ExperimentContractError as error:
        raise durable.ContractError(str(error)) from error


def journal_binding(plan):
    """Bind the entire execution plan without changing legacy Start/Finish events."""
    return {"schema": 1, "indices": list(plan["indices"]), "execution_plan": plan}


def attempt_identity(plan_id, index, ordinal=0):
    """Legacy schema-2 attempt identity retained for persistent compatibility."""
    if type(index) is not int or type(ordinal) is not int or ordinal < 0:
        raise durable.ContractError("invalid attempt identity coordinates")
    return durable.digest(f"tdi-attempt/v1:{plan_id}:{index}:{ordinal}".encode())[:32]


def attempt_coordinates(plan, plan_id, index, ordinal=0):
    """Return (trial_id, attempt_id), preserving schema-2 attempt identities."""
    if plan.get("schema") == 2:
        return None, attempt_identity(plan_id, index, ordinal)
    if plan.get("schema") != 3:
        raise durable.ContractError("attempt coordinates require plan schema 2 or 3")
    try:
        trial_id = experiment.trial_identity(plan_id, plan["domain"], index)
        attempt_id = experiment.attempt_identity(
            plan_id, trial_id, plan["execution"]["backend"], ordinal,
        )
    except experiment.ExperimentContractError as error:
        raise durable.ContractError(str(error)) from error
    return trial_id, attempt_id


def recovery_marker(path):
    path = Path(path)
    if not path.exists():
        return None
    value = durable.strict_json(path.read_bytes(), max_bytes=16_384, max_depth=8,
                                max_items=64, max_string_bytes=2048)
    if not isinstance(value, dict) or value.get("schema") != 1:
        raise durable.ContractError("invalid containment recovery marker")
    return value
