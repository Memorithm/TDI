"""Declarative TDI execution-graph contract and thin SciRust Hub workflow compiler.

TDI owns scientific step/checkpoint semantics. Generic DAG scheduling, retries,
leases and remote execution remain owned by scirust-hub. Graph v1 is declared in
a topological list so TDI does not implement a second general-purpose scheduler.
"""
from __future__ import annotations

import copy
import hashlib
import json
import re
import uuid

import tdi_experiment_contract as experiment

GRAPH_SCHEMA = 1
HUB_REPOSITORY = "Memorithm/scirust-hub"
HUB_SOURCE_COMMIT = "4bf6186841e1ea70ed15cd84faf33de9b48429cd"
HUB_WORKFLOW_SCHEMA_VERSION = 1
HUB_WORKFLOW_MODEL_VERSION = "1.2.0"
MAX_STEPS = 1024
MAX_CONCURRENCY = 64
MAX_INPUTS = 32
MAX_OUTPUTS = 256
MAX_DEPTH = 16
MAX_PARAMS_BYTES = 16 * 1024
MAX_TIMEOUT_MS = 60 * 60 * 1000
_KEY = re.compile(r"[a-z0-9][a-z0-9_-]{0,63}\Z")
_HEX40 = re.compile(r"[0-9a-f]{40}\Z")
_HEX64 = re.compile(r"[0-9a-f]{64}\Z")
_CAP_SEGMENT = re.compile(r"[a-z][a-z0-9_]{0,63}\Z")


class ExecutionGraphError(ValueError):
    """An execution graph or Hub compilation binding is invalid."""


def _digest(kind, value):
    return hashlib.sha256(kind.encode("ascii") + b"\0" + experiment.canonical(value)).hexdigest()


def _exact(value, fields, name):
    if not isinstance(value, dict) or set(value) != set(fields):
        raise ExecutionGraphError(f"{name} has unknown or missing fields")
    return value


def _text(value, name, *, max_bytes=16_384):
    if not isinstance(value, str) or not value.strip():
        raise ExecutionGraphError(f"{name} must be a non-empty string")
    if len(value.encode("utf-8")) > max_bytes:
        raise ExecutionGraphError(f"{name} exceeds the text bound")
    return value


def _key(value, name):
    if not isinstance(value, str) or _KEY.fullmatch(value) is None:
        raise ExecutionGraphError(f"{name} must match [a-z0-9][a-z0-9_-]{{0,63}}")
    return value


def _capability(value, name):
    value = _text(value, name, max_bytes=128)
    segments = value.split(".")
    if not segments or any(_CAP_SEGMENT.fullmatch(segment) is None for segment in segments):
        raise ExecutionGraphError(
            f"{name} must use Hub CapabilityName grammar [a-z][a-z0-9_]* dot-separated"
        )
    return value


def _version(value, name):
    value = _text(value, name, max_bytes=64)
    core, sep, prerelease = value.partition("-")
    numbers = core.split(".")
    if len(numbers) != 3:
        raise ExecutionGraphError(f"{name} must have exactly three numeric components")
    for number in numbers:
        if not number or not number.isascii() or not number.isdigit():
            raise ExecutionGraphError(f"{name} has an invalid numeric component")
        if len(number) > 1 and number.startswith("0"):
            raise ExecutionGraphError(f"{name} numeric components must not have leading zeros")
    if sep:
        if not prerelease or any(
            not (ch.isascii() and (ch.isalnum() or ch in ".-")) for ch in prerelease
        ):
            raise ExecutionGraphError(f"{name} has an invalid prerelease")
    return value


def _sha256(value, name):
    if not isinstance(value, str) or _HEX64.fullmatch(value) is None:
        raise ExecutionGraphError(f"{name} must be lowercase SHA-256")
    return value


def _output_name(value, name):
    value = _text(value, name, max_bytes=128)
    if any(ch.isspace() or ord(ch) < 32 or ord(ch) == 127 for ch in value):
        raise ExecutionGraphError(f"{name} must not contain whitespace/control characters")
    return value


def _hub_uuid(value, name):
    if not isinstance(value, str):
        raise ExecutionGraphError(f"{name} must be a canonical UUID string")
    try:
        parsed = uuid.UUID(value)
    except (ValueError, AttributeError) as error:
        raise ExecutionGraphError(f"{name} must be a canonical UUID string") from error
    if str(parsed) != value:
        raise ExecutionGraphError(f"{name} must be lowercase hyphenated UUID")
    return value


def _safe_integer(value, name, *, positive=False):
    lower = 1 if positive else 0
    if type(value) is not int or not lower <= value <= experiment.JSON_SAFE_INTEGER:
        qualifier = "positive " if positive else "non-negative "
        raise ExecutionGraphError(
            f"{name} must be a {qualifier}JSON-safe integer <= {experiment.JSON_SAFE_INTEGER}"
        )
    return value


def _safe_json(value, name="parameters", depth=0):
    if depth > MAX_DEPTH:
        raise ExecutionGraphError(f"{name} exceeds nesting depth")
    if value is None or type(value) is bool:
        return
    if type(value) is int:
        if not -experiment.JSON_SAFE_INTEGER <= value <= experiment.JSON_SAFE_INTEGER:
            raise ExecutionGraphError(f"{name} integer exceeds JSON-safe range")
        return
    if isinstance(value, float):
        raise ExecutionGraphError(f"{name} floating-point values require explicit string encoding")
    if isinstance(value, str):
        _text(value, name)
        return
    if isinstance(value, list):
        if len(value) > experiment.MAX_LIST_ITEMS:
            raise ExecutionGraphError(f"{name} exceeds list bound")
        for index, item in enumerate(value):
            _safe_json(item, f"{name}[{index}]", depth + 1)
        return
    if isinstance(value, dict):
        if len(value) > experiment.MAX_LIST_ITEMS:
            raise ExecutionGraphError(f"{name} exceeds object bound")
        for key, item in value.items():
            _text(key, f"{name} key", max_bytes=256)
            _safe_json(item, f"{name}.{key}", depth + 1)
        return
    raise ExecutionGraphError(f"{name} contains unsupported value")


def _validate_hub_contract(value):
    _exact(
        value,
        {"repository", "source_commit", "workflow_schema_version", "workflow_model_version"},
        "hub_contract",
    )
    if value["repository"] != HUB_REPOSITORY:
        raise ExecutionGraphError("hub_contract.repository must be Memorithm/scirust-hub")
    if not isinstance(value["source_commit"], str) or _HEX40.fullmatch(value["source_commit"]) is None:
        raise ExecutionGraphError("hub_contract.source_commit must be a lowercase 40-hex Git commit")
    if value["source_commit"] != HUB_SOURCE_COMMIT:
        raise ExecutionGraphError("unsupported Hub source commit for Graph/v1 compiler")
    if value["workflow_schema_version"] != HUB_WORKFLOW_SCHEMA_VERSION:
        raise ExecutionGraphError("unsupported Hub workflow schema version")
    if value["workflow_model_version"] != HUB_WORKFLOW_MODEL_VERSION:
        raise ExecutionGraphError("unsupported Hub workflow model version")
    return value


def _normalize_source(source, *, input_name, seen_steps, outputs_by_step):
    if not isinstance(source, dict) or source.get("kind") not in ("artifact", "step"):
        raise ExecutionGraphError(f"input {input_name!r} has invalid source kind")
    if source["kind"] == "artifact":
        _exact(source, {"kind", "sha256"}, f"input {input_name!r}")
        _sha256(source["sha256"], f"input {input_name!r}.sha256")
        return {"kind": "artifact", "sha256": source["sha256"]}
    _exact(source, {"kind", "step", "output"}, f"input {input_name!r}")
    dep = _key(source["step"], f"input {input_name!r}.step")
    output = _output_name(source["output"], f"input {input_name!r}.output")
    if dep not in seen_steps:
        raise ExecutionGraphError(
            f"input {input_name!r} must reference an earlier graph step; {dep!r} is not available"
        )
    if output not in outputs_by_step[dep]:
        raise ExecutionGraphError(
            f"input {input_name!r} references undeclared output {output!r} of step {dep!r}"
        )
    return {"kind": "step", "step": dep, "output": output}


def _normalize_checkpoint(value, *, input_names, outputs):
    _exact(value, {"mode", "input", "output"}, "checkpoint policy")
    if value["mode"] == "none":
        if value["input"] is not None or value["output"] is not None:
            raise ExecutionGraphError("checkpoint mode none requires null input/output")
        return {"mode": "none", "input": None, "output": None}
    if value["mode"] != "exact":
        raise ExecutionGraphError("checkpoint mode must be none or exact")
    if value["input"] is not None:
        _text(value["input"], "checkpoint.input", max_bytes=128)
        if value["input"] not in input_names:
            raise ExecutionGraphError("checkpoint.input must name a declared step input")
    output = _output_name(value["output"], "checkpoint.output")
    if output not in outputs:
        raise ExecutionGraphError("checkpoint.output must name a declared step output")
    return {"mode": "exact", "input": value["input"], "output": output}


def canonical_graph(graph):
    """Validate Graph/v1 and return canonical semantic data.

    Steps are intentionally an ordered topological declaration: any dependency
    must reference an earlier step. That gives acyclicity without embedding a
    second generic DAG scheduler inside TDI.
    """
    _exact(
        graph,
        {
            "schema",
            "semantic_version",
            "name",
            "root_plan_id",
            "hub_contract",
            "max_concurrency",
            "steps",
        },
        "execution graph",
    )
    if graph["schema"] != GRAPH_SCHEMA:
        raise ExecutionGraphError("unsupported execution graph schema")
    _text(graph["semantic_version"], "semantic_version", max_bytes=256)
    _text(graph["name"], "name", max_bytes=128)
    _sha256(graph["root_plan_id"], "root_plan_id")
    _validate_hub_contract(graph["hub_contract"])
    _safe_integer(graph["max_concurrency"], "max_concurrency", positive=True)
    if graph["max_concurrency"] > MAX_CONCURRENCY:
        raise ExecutionGraphError(f"max_concurrency exceeds Hub bound {MAX_CONCURRENCY}")
    if not isinstance(graph["steps"], list) or not 0 < len(graph["steps"]) <= MAX_STEPS:
        raise ExecutionGraphError("steps must contain 1..=1024 entries")

    normalized_steps = []
    seen_steps = set()
    outputs_by_step = {}
    component_pins = {}
    for index, raw in enumerate(graph["steps"]):
        _exact(
            raw,
            {
                "key",
                "component_alias",
                "component_id",
                "component_version",
                "component_manifest_digest",
                "capability",
                "capability_contract_version",
                "parameters",
                "inputs",
                "outputs",
                "after",
                "timeout_milliseconds",
                "checkpoint",
            },
            f"steps[{index}]",
        )
        key = _key(raw["key"], f"steps[{index}].key")
        if key in seen_steps:
            raise ExecutionGraphError("duplicate graph step key")
        component_alias = _key(raw["component_alias"], f"steps[{index}].component_alias")
        component_id = _hub_uuid(raw["component_id"], f"steps[{index}].component_id")
        component_version = _version(raw["component_version"], f"steps[{index}].component_version")
        component_manifest_digest = _sha256(
            raw["component_manifest_digest"], f"steps[{index}].component_manifest_digest"
        )
        capability = _capability(raw["capability"], f"steps[{index}].capability")
        capability_contract_version = _version(
            raw["capability_contract_version"], f"steps[{index}].capability_contract_version"
        )
        pin = (component_id, component_version, component_manifest_digest)
        if component_alias in component_pins and component_pins[component_alias] != pin:
            raise ExecutionGraphError("one component alias cannot resolve to multiple component pins")
        component_pins[component_alias] = pin
        _safe_json(raw["parameters"], f"steps[{index}].parameters")
        parameter_bytes = json.dumps(
            raw["parameters"], sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False
        ).encode("utf-8")
        if len(parameter_bytes) > MAX_PARAMS_BYTES:
            raise ExecutionGraphError(
                f"steps[{index}].parameters exceed pinned Hub {MAX_PARAMS_BYTES}-byte limit"
            )
        if not isinstance(raw["inputs"], dict) or len(raw["inputs"]) > MAX_INPUTS:
            raise ExecutionGraphError("step inputs must be a bounded object")
        normalized_inputs = {}
        for input_name, source in raw["inputs"].items():
            _key(input_name, f"steps[{index}] input name")
            normalized_inputs[input_name] = _normalize_source(
                source,
                input_name=input_name,
                seen_steps=seen_steps,
                outputs_by_step=outputs_by_step,
            )

        if not isinstance(raw["outputs"], list) or len(raw["outputs"]) > MAX_OUTPUTS:
            raise ExecutionGraphError("step outputs must be a bounded list")
        outputs = []
        output_seen = set()
        for output in raw["outputs"]:
            output = _output_name(output, f"steps[{index}] output")
            if output in output_seen:
                raise ExecutionGraphError("duplicate step output")
            output_seen.add(output)
            outputs.append(output)
        outputs.sort()

        if not isinstance(raw["after"], list):
            raise ExecutionGraphError("step after must be a list")
        after = []
        after_seen = set()
        for dep in raw["after"]:
            dep = _key(dep, f"steps[{index}].after")
            if dep not in seen_steps:
                raise ExecutionGraphError(
                    f"step {key!r} after dependency {dep!r} must be declared earlier"
                )
            if dep in after_seen:
                raise ExecutionGraphError("duplicate after dependency")
            after_seen.add(dep)
            after.append(dep)
        after.sort()

        _safe_integer(raw["timeout_milliseconds"], "timeout_milliseconds", positive=True)
        if raw["timeout_milliseconds"] > MAX_TIMEOUT_MS:
            raise ExecutionGraphError(
                f"timeout_milliseconds exceeds pinned Hub limit {MAX_TIMEOUT_MS}"
            )
        checkpoint = _normalize_checkpoint(
            raw["checkpoint"], input_names=set(normalized_inputs), outputs=set(outputs)
        )
        normalized_steps.append({
            "key": key,
            "component_alias": component_alias,
            "component_id": component_id,
            "component_version": component_version,
            "component_manifest_digest": component_manifest_digest,
            "capability": capability,
            "capability_contract_version": capability_contract_version,
            "parameters": copy.deepcopy(raw["parameters"]),
            "inputs": normalized_inputs,
            "outputs": outputs,
            "after": after,
            "timeout_milliseconds": raw["timeout_milliseconds"],
            "checkpoint": checkpoint,
        })
        seen_steps.add(key)
        outputs_by_step[key] = set(outputs)

    return {
        "schema": GRAPH_SCHEMA,
        "semantic_version": graph["semantic_version"],
        "name": graph["name"],
        "root_plan_id": graph["root_plan_id"],
        "hub_contract": copy.deepcopy(graph["hub_contract"]),
        "max_concurrency": graph["max_concurrency"],
        "steps": normalized_steps,
    }


def graph_identity(graph):
    return _digest("tdi-execution-graph/v1", canonical_graph(graph))


def step_identity(graph, key):
    value = canonical_graph(graph)
    _key(key, "step identity key")
    for step in value["steps"]:
        if step["key"] == key:
            return _digest(
                "tdi-execution-step/v1",
                {"root_plan_id": value["root_plan_id"], "step": step},
            )
    raise ExecutionGraphError(f"unknown graph step {key!r}")


def compile_hub_workflow_preview(graph, *, artifact_bindings):
    """Compile Graph/v1 into a non-executable Hub WorkflowSpec preview.

    Hub WorkflowSpec/v1 does not atomically bind component version/manifest or
    capability contract version at normal submission. Graph/v1 therefore embeds
    ComponentId itself in every canonical step and this adapter emits only a
    structural preview plus the exact pins a future Hub edge must enforce.
    """
    value = canonical_graph(graph)
    if not isinstance(artifact_bindings, dict):
        raise ExecutionGraphError("Hub artifact bindings must be a dictionary")
    steps = []
    component_pins = {}
    capability_pins = []
    for step in value["steps"]:
        component = step["component_id"]
        component_pins[step["component_alias"]] = {
            "alias": step["component_alias"],
            "component_id": component,
            "component_version": step["component_version"],
            "manifest_digest": step["component_manifest_digest"],
        }
        capability_pins.append({
            "step_key": step["key"],
            "component_alias": step["component_alias"],
            "component_id": component,
            "capability": step["capability"],
            "contract_version": step["capability_contract_version"],
        })
        inputs = {}
        for name, source in step["inputs"].items():
            if source["kind"] == "artifact":
                artifact = artifact_bindings.get(source["sha256"])
                if artifact is None:
                    raise ExecutionGraphError(
                        f"missing Hub artifact binding for digest {source['sha256']}"
                    )
                _hub_uuid(artifact, f"Hub artifact binding for {source['sha256']}")
                inputs[name] = {"artifact": {"artifact": artifact}}
            else:
                inputs[name] = {
                    "from_step": {"key": source["step"], "output": source["output"]}
                }
        steps.append({
            "key": step["key"],
            "component": component,
            "capability": step["capability"],
            "parameters": copy.deepcopy(step["parameters"]),
            "inputs": inputs,
            "timeout_ms": step["timeout_milliseconds"],
            "after": list(step["after"]),
        })
    workflow = {
        "schema_version": value["hub_contract"]["workflow_schema_version"],
        "name": value["name"],
        "max_concurrency": value["max_concurrency"],
        "steps": steps,
    }
    return {
        "schema": 1,
        "kind": "tdi-hub-workflow-preview",
        "execution_authorized": False,
        "hub_contract": copy.deepcopy(value["hub_contract"]),
        "workflow": workflow,
        "component_pins": sorted(component_pins.values(), key=lambda item: item["alias"]),
        "capability_pins": capability_pins,
    }
