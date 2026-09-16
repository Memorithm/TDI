"""Bounded non-final sensitivity plans and evidence-preserving SciRust analysis.

SALib owns the qualified sampling algorithms. SciRust owns numerical estimates.
TDI declares factors, units, ordering, budgets, completeness and provenance.
This module never selects or executes a model, final population or optimizer.
"""
from __future__ import annotations

import copy
import importlib.metadata
import inspect
from pathlib import Path
import re

import tdi_artifact_contract as artifacts
import tdi_engine_runtime as runtime
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_store import identity
from tdi_research_analysis import finite, label

MAX_ROWS = 4096
MAX_SCALARS = 16384
SAMPLERS = {"SALib": "1.5.2", "numpy": "2.5.3", "scipy": "1.18.1"}


def protocol(value):
    """Validate the complete design and declared budget before sampling."""
    fields = {"schema", "purpose", "domain", "name", "method", "factors", "output_unit",
              "samples", "seed", "max_evaluations", "missing", "assumptions"}
    if (not isinstance(value, dict) or set(value) != fields or type(value["schema"]) is not int
            or value["schema"] != 1 or value["purpose"] != "exploratory-sensitivity"
            or value["domain"] not in ("Development", "Validation")):
        raise durable.ContractError("invalid non-final sensitivity protocol")
    p = copy.deepcopy(value)
    label(p["name"]); label(p["output_unit"])
    if p["method"] not in ("morris", "sobol", "ablation"):
        raise durable.ContractError("unsupported sensitivity method")
    if p["missing"] != "reject-incomplete":
        raise durable.ContractError("complete sampling blocks are required")
    expected = ["deterministic-response", "independent-uniform-factors"] if p["method"] != "ablation" else ["deterministic-response", "declared-one-factor-interventions"]
    if p["assumptions"] != expected:
        raise durable.ContractError("sampling assumptions must be explicitly declared")
    if not isinstance(p["seed"], str) or not re.fullmatch(r"0|[1-9][0-9]{0,19}", p["seed"]) or int(p["seed"]) > 2**64 - 1:
        raise durable.ContractError("seed must be canonical decimal u64")
    factors = p["factors"]
    if not isinstance(factors, list) or not 1 <= len(factors) <= 8:
        raise durable.ContractError("requires 1..8 factors")
    names = set()
    for f in factors:
        if not isinstance(f, dict) or set(f) != {"name", "unit", "lower", "upper", "baseline", "intervention"}:
            raise durable.ContractError("invalid factor schema")
        label(f["name"]); label(f["unit"])
        if f["name"] in names:
            raise durable.ContractError("duplicate factor")
        names.add(f["name"])
        low, high = finite(f["lower"]), finite(f["upper"])
        if not low < high or finite(high - low) <= 0:
            raise durable.ContractError("factor requires a finite positive range")
        if p["method"] == "ablation":
            if not low <= finite(f["baseline"]) <= high or not low <= finite(f["intervention"]) <= high or f["baseline"] == f["intervention"]:
                raise durable.ContractError("ablation requires distinct explicit in-range baseline/intervention")
        elif f["baseline"] is not None or f["intervention"] is not None:
            raise durable.ContractError("non-ablation intervention fields must be null")
    n, d = p["samples"], len(factors)
    if type(n) is not int:
        raise durable.ContractError("sample count must be integer")
    if p["method"] == "ablation":
        if n != 1 or p["seed"] != "0":
            raise durable.ContractError("deterministic ablation requires samples=1 and seed=0")
        count = d + 1
    elif p["method"] == "morris":
        if not 2 <= n <= 128:
            raise durable.ContractError("requires 2..128 Morris trajectories")
        count = n * (d + 1)
    else:
        if not 16 <= n <= 2048 or n & (n - 1):
            raise durable.ContractError("Sobol base samples must be a power of two in 16..2048")
        count = n * (d + 2)
    if (type(p["max_evaluations"]) is not int or not count <= p["max_evaluations"] <= MAX_ROWS
            or count * d > MAX_SCALARS):
        raise durable.ContractError("sampling exceeds the predeclared evaluation/scalar budget")
    return p


def make_plan(value):
    """Materialize the immutable ordered design without evaluating any response.

    Sobol uses scrambled LMS+shift, skip_values=0, no second-order rows. Morris
    uses four levels and unoptimized complete trajectories. Duplicate points
    remain distinct planned evaluations; row IDs include their ordinal.
    """
    p = protocol(value)
    factors, method = p["factors"], p["method"]
    if method == "ablation":
        physical = [[f["baseline"] for f in factors]]
        for j, f in enumerate(factors):
            row = list(physical[0]); row[j] = f["intervention"]; physical.append(row)
        normalized = [[(x - f["lower"]) / (f["upper"] - f["lower"]) for x, f in zip(row, factors)] for row in physical]
        generator = {"name": "tdi-declared-oat/v1", "packages": {}, "source_sha256": durable.file_digest(Path(__file__))}
    else:
        try:
            versions = {name: importlib.metadata.version(name) for name in SAMPLERS}
        except importlib.metadata.PackageNotFoundError as error:
            raise durable.ContractError("install the pinned optional sensitivity requirements") from error
        if versions != SAMPLERS:
            raise durable.ContractError("sampling dependency versions differ from the qualified profile")
        from SALib.sample import morris, sobol
        sample = morris.sample if method == "morris" else sobol.sample
        problem = {"num_vars": len(factors), "names": [f["name"] for f in factors], "bounds": [[0, 1] for _ in factors]}
        options = ({"num_levels": 4, "optimal_trajectories": None, "local_optimization": False} if method == "morris"
                   else {"calc_second_order": False, "scramble": True, "skip_values": 0})
        normalized = sample(problem, p["samples"], seed=int(p["seed"]), **options).tolist()
        physical = [[finite(f["lower"] + x * (f["upper"] - f["lower"])) for x, f in zip(row, factors)] for row in normalized]
        generator = {"name": "SALib.sample." + method, "packages": versions, "options": options,
                     "source_sha256": durable.file_digest(Path(inspect.getsourcefile(sample))),
                     "adapter_sha256": durable.file_digest(Path(__file__))}
    protocol_id = identity("tdi-sensitivity-protocol/v1", p)
    rows = [{"id": identity("tdi-sensitivity-row/v1", {"protocol": protocol_id, "ordinal": i, "normalized": row}),
             "ordinal": i, "normalized": row, "values": physical[i]} for i, row in enumerate(normalized)]
    plan = {"schema": 1, "kind": "tdi-sensitivity-plan", "protocol": p, "protocol_identity": protocol_id,
            "generator": generator, "rows": rows}
    plan["identity"] = identity("tdi-sensitivity-plan/v1", plan)
    return plan


def validate_plan(value):
    """Regenerate and compare the entire plan before preparing or analyzing it."""
    if not isinstance(value, dict) or set(value) != {"schema", "kind", "protocol", "protocol_identity", "generator", "rows", "identity"}:
        raise durable.ContractError("invalid sensitivity plan envelope")
    p = make_plan(value["protocol"])
    if value != p:
        raise durable.ContractError("sensitivity plan differs from deterministic regeneration")
    return p


def collect(store, plan, selections):
    """Select verified terminal catalogue batches, preserving every row identity.

    A selection names campaign/step/output. The artifact's payload must echo
    the full plan identity and ordered row IDs. Failed or missing batches are
    rejected; callers inspect their unchanged campaign evidence for diagnostics.
    """
    plan = validate_plan(plan)
    if not isinstance(selections, list) or not 1 <= len(selections) <= 128:
        raise durable.ContractError("requires 1..128 batch selectors")
    rows, seen = {}, set()
    records = {}
    costs = []
    for selection in selections:
        if not isinstance(selection, dict) or set(selection) != {"campaign", "step", "output"}:
            raise durable.ContractError("invalid sensitivity batch selector")
        key = tuple(selection[k] for k in ("campaign", "step", "output"))
        if key in seen:
            raise durable.ContractError("duplicate batch selector")
        seen.add(key)
        if key[0] not in records:
            record = store.get(key[0]); spec = runtime.canonical_campaign(record["spec"])
            if spec["domain"] != plan["protocol"]["domain"] or record["phase"] not in ("completed", "failed", "cancelled", "imported"):
                raise durable.ContractError("sensitivity campaign is incomplete or in another domain")
            state, steps = runtime.validated_snapshot(spec, record["snapshot"])
            if state not in runtime.TERMINAL:
                raise durable.ContractError("terminal source snapshot required")
            records[key[0]] = (record, steps)
        record, steps = records[key[0]]
        step = steps.get(key[1])
        found = store.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?", key).fetchone()
        if step is None or step["state"] != "succeeded" or found is None or key[2] not in record["spec"]["outputs"].get(key[1], {}):
            raise durable.ContractError("incomplete sensitivity batch; inspect the source campaign")
        evidence = durable.strict_json(found[0], max_items=100000)
        descriptor = artifacts.canonical_artifact(evidence["descriptor"])
        if (descriptor["access_class"] != plan["protocol"]["domain"].lower()
                or artifacts.artifact_identity(descriptor) != evidence["artifact_identity"]
                or artifacts.provenance_identity(evidence["provenance"]) != evidence["provenance_identity"]):
            raise durable.ContractError("sensitivity source integrity mismatch")
        batch = evidence["json"]
        if (not isinstance(batch, dict) or batch.get("plan_identity") != plan["identity"]
                or batch.get("output_unit") != plan["protocol"]["output_unit"]
                or batch.get("status") != "SensitivityEvaluated" or not isinstance(batch.get("rows"), list)
                or not 1 <= len(batch["rows"]) <= 32):
            raise durable.ContractError("sensitivity result plan/unit/schema mismatch")
        for index, row in enumerate(batch["rows"]):
            if not isinstance(row, dict) or set(row) != {"id", "ordinal", "value"} or type(row["ordinal"]) is not int or not 0 <= row["ordinal"] < len(plan["rows"]):
                raise durable.ContractError("invalid sensitivity response row")
            expected = plan["rows"][row["ordinal"]]
            if row["id"] != expected["id"] or row["id"] in rows:
                raise durable.ContractError("duplicate or unplanned sensitivity response")
            rows[row["id"]] = {**row, "value": finite(row["value"]), "source": {**selection,
                "artifact_identity": evidence["artifact_identity"], "provenance_identity": evidence["provenance_identity"], "pointer": f"/rows/{index}/value"}}
        costs.append({**selection, "measurements": batch.get("cost_measurements"),
                      "status": "recorded" if batch.get("cost_measurements") is not None else "unavailable"})
    if len(rows) != len(plan["rows"]):
        raise durable.ContractError("incomplete sensitivity design; no rows may be discarded")
    return {"schema": 1, "plan_identity": plan["identity"], "rows": [rows[r["id"]] for r in plan["rows"]], "cost_measurements": costs}


def analyze(plan, observations, worker):
    """Compute point estimates from complete blocks through the actual SciRust API."""
    plan = validate_plan(plan)
    if (not isinstance(observations, dict) or set(observations) != {"schema", "plan_identity", "rows", "cost_measurements"}
            or type(observations["schema"]) is not int or observations["schema"] != 1
            or observations["plan_identity"] != plan["identity"] or not isinstance(observations["rows"], list)
            or len(observations["rows"]) != len(plan["rows"])):
        raise durable.ContractError("observations must contain the complete ordered design")
    ys, sources = [], set()
    for planned, row in zip(plan["rows"], observations["rows"]):
        if not isinstance(row, dict) or set(row) != {"id", "ordinal", "value", "source"} or row["id"] != planned["id"] or type(row["ordinal"]) is not int or row["ordinal"] != planned["ordinal"]:
            raise durable.ContractError("observation row order/identity mismatch")
        source = row["source"]
        if not isinstance(source, dict) or set(source) != {"campaign", "step", "output", "artifact_identity", "provenance_identity", "pointer"}:
            raise durable.ContractError("observation requires source provenance")
        for field in ("campaign", "artifact_identity", "provenance_identity"):
            graphs._sha256(source[field], "sensitivity source")
        if not re.fullmatch(r"/rows/(0|[1-9][0-9]?)/value", source["pointer"]):
            raise durable.ContractError("invalid row source pointer")
        source_key = source["provenance_identity"], source["pointer"]
        if source_key in sources:
            raise durable.ContractError("one observed scalar cannot replace multiple design rows")
        sources.add(source_key); ys.append(finite(row["value"]))
    p = plan["protocol"]; method = p["method"]; d = len(p["factors"])
    if method == "ablation":
        result = {"method": "declared-oat-difference/v1", "baseline": ys[0],
                  "effects": [finite(y - ys[0]) for y in ys[1:]]}
        implementation = {"repository": "Memorithm/TDI", "source_sha256": durable.file_digest(Path(__file__))}
    else:
        if worker is None:
            raise durable.ContractError("a pinned SciRust worker is required")
        if method == "morris":
            result = worker.call("morris", inputs=[r["normalized"] for r in plan["rows"]], outputs=ys)
        else:
            width = d + 2
            result = worker.call("sobol", a=ys[0::width], b=ys[d + 1::width], ab=[ys[j + 1::width] for j in range(d)])
        implementation = worker.provenance
    report = {"schema": 1, "kind": "tdi-sensitivity-analysis", "plan": plan, "observations": observations,
              "implementation": implementation, "result": result, "planned_rows": len(ys), "included_rows": len(ys),
              "excluded_rows": 0, "confidence_intervals": None, "scientific_verdict": "not-assessed",
              "limitations": ["exploratory point estimates; no confidence intervals or second-order Sobol",
                              "independence and deterministic-response assumptions are caller declarations",
                              "Morris effects use normalized factor ranges; Sobol indices are dimensionless",
                              "ablation differences apply only to the declared baseline and interventions",
                              "direct observations are caller assertions; catalogue extraction preserves verified references"]}
    report["identity"] = identity("tdi-sensitivity-analysis/v1", report)
    return report
