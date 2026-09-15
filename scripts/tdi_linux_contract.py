"""Schema-2 Linux execution contract layered over TDI's schema-1 science plan."""
from pathlib import Path

import tdi_experiment_supervisor as durable
from tdi_linux_containment import ContainmentError, ResourceProfile


def execution_profile(plan):
    """Validate and return the bound cgroup-v2 resource profile."""
    execution = plan.get("execution") if isinstance(plan, dict) else None
    if not isinstance(execution, dict) or set(execution) != {"backend", "profile"}:
        raise durable.ContractError("schema-2 execution contract has unknown or missing fields")
    if execution["backend"] != "linux-cgroup-v2":
        raise durable.ContractError("schema-2 execution backend must be linux-cgroup-v2")
    try:
        profile = ResourceProfile.from_json(execution["profile"])
        profile.validate_static_capabilities()
    except ContainmentError as error:
        raise durable.ContractError(str(error)) from error
    return profile


def validate_plan(plan, root):
    """Validate schema 2 while preserving qualified schema-1 scientific semantics."""
    if not isinstance(plan, dict) or plan.get("schema") != 2:
        raise durable.ContractError("linux-cgroup-v2 runner requires plan schema 2")
    required = {
        "schema", "purpose", "domain", "indices", "argv", "artifacts",
        "timeout_seconds", "max_output_bytes", "max_trials", "execution",
    }
    if set(plan) != required:
        raise durable.ContractError("unknown or missing schema-2 plan fields")
    legacy = {key: value for key, value in plan.items() if key != "execution"}
    legacy["schema"] = 1
    durable.validate(legacy, Path(root))
    execution_profile(plan)
    return durable.digest(durable.canonical(plan).encode())


def journal_binding(plan):
    """Bind the entire schema-2 plan without changing legacy Start/Finish events."""
    return {"schema": 1, "indices": list(plan["indices"]), "execution_plan": plan}


def attempt_identity(plan_id, index, ordinal=0):
    if type(index) is not int or type(ordinal) is not int or ordinal < 0:
        raise durable.ContractError("invalid attempt identity coordinates")
    return durable.digest(f"tdi-attempt/v1:{plan_id}:{index}:{ordinal}".encode())[:32]


def recovery_marker(path):
    path = Path(path)
    if not path.exists():
        return None
    value = durable.strict_json(path.read_bytes(), max_bytes=16_384, max_depth=8,
                                max_items=64, max_string_bytes=2048)
    if not isinstance(value, dict) or value.get("schema") != 1:
        raise durable.ContractError("invalid containment recovery marker")
    return value
