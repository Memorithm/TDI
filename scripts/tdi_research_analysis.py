"""Non-final, protocol-declared paired analysis using shared SciRust primitives.

The protocol declares units, paired repeats, strata, arms, missing-data policy
and multiplicity before analysis. Every missing/error/rejected pair is counted.
Reports carry estimates, intervals and provenance, never an automatic beneficial
verdict. Frozen series-specific analysis contracts are not changed by this API.
"""
import copy
import math
import re

import tdi_artifact_contract as artifacts
import tdi_engine_runtime as runtime
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_store import identity

MAX_ROWS = 20_000
MAX_ANALYSIS_ITEMS = 1_000_000


def label(value):
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z0-9_.-]{1,64}", value):
        raise durable.ContractError("invalid analysis label")
    return value


def finite(value):
    if (type(value) not in (int, float) or (type(value) is int and abs(value) > 2**53 - 1)
            or not math.isfinite(value)):
        raise durable.ContractError("analysis value must be a finite binary64-safe scalar")
    return value


def _mean(values):
    try:
        total = math.fsum(values)
    except OverflowError:
        scale = max(abs(x) for x in values)
        return finite((math.fsum(x / scale for x in values) / len(values)) * scale)
    return finite(total / len(values))


def canonical_protocol(value):
    """Validate a bounded exploratory paired-unit protocol without reading results.

    Each unit declares a nonempty replicate list; both arms must supply all of
    those repeats or the entire unit is excluded under `exclude-incomplete-unit`.
    `reject-incomplete` instead refuses analysis. All units have equal weight.
    Intervals can use a declared Bonferroni family across comparisons and strata;
    bootstrap coverage remains approximate. No confirmatory stage is authorized.
    """
    required = {"schema", "purpose", "domain", "name", "unit_kind", "units", "comparisons", "strata",
                "missing", "confidence", "resamples", "seed", "multiplicity"}
    if (not isinstance(value, dict) or set(value) != required or type(value["schema"]) is not int or value["schema"] != 1
            or value["purpose"] != "exploratory-analysis" or value["domain"] not in ("Development", "Validation")):
        raise durable.ContractError("invalid non-final analysis protocol")
    p = copy.deepcopy(value)
    label(p["name"])
    if p["unit_kind"] not in ("task", "family", "seed", "episode", "hardware-repeat", "declared-cluster"):
        raise durable.ContractError("independent experimental unit must be declared")
    if not isinstance(p["units"], list) or not 2 <= len(p["units"]) <= 10_000:
        raise durable.ContractError("analysis requires 2..10000 planned independent units")
    seen = set()
    for unit in p["units"]:
        if not isinstance(unit, dict) or set(unit) != {"id", "stratum", "replicates"}:
            raise durable.ContractError("invalid planned unit")
        label(unit["id"]); label(unit["stratum"])
        if unit["id"] in seen:
            raise durable.ContractError("duplicate planned unit")
        seen.add(unit["id"])
        reps = unit["replicates"]
        if not isinstance(reps, list) or not 1 <= len(reps) <= 100:
            raise durable.ContractError("unit requires 1..100 paired repeats")
        for rep in reps: label(rep)
        if len(set(reps)) != len(reps):
            raise durable.ContractError("duplicate planned repeat")
    if not isinstance(p["comparisons"], list) or not 1 <= len(p["comparisons"]) <= 8:
        raise durable.ContractError("requires 1..8 declared comparisons")
    names = set()
    for comparison in p["comparisons"]:
        if not isinstance(comparison, dict) or set(comparison) != {"id", "reference", "candidate", "metric", "unit"}:
            raise durable.ContractError("invalid comparison schema")
        for item in comparison.values(): label(item)
        if comparison["id"] in names or comparison["reference"] == comparison["candidate"]:
            raise durable.ContractError("duplicate comparison or identical arms")
        names.add(comparison["id"])
    strata = p["strata"]
    if not isinstance(strata, list) or len(strata) > 16:
        raise durable.ContractError("invalid strata")
    for stratum in strata: label(stratum)
    if len(set(strata)) != len(strata) or not set(strata).issubset({u["stratum"] for u in p["units"]}):
        raise durable.ContractError("unknown or duplicate analysis stratum")
    if p["missing"] not in ("reject-incomplete", "exclude-incomplete-unit"):
        raise durable.ContractError("missing-data policy must be declared")
    if type(p["resamples"]) is not int or not 100 <= p["resamples"] <= 100_000 or len(p["units"]) * p["resamples"] > 10_000_000:
        raise durable.ContractError("analysis resampling budget exceeded")
    if (len(p["units"]) + sum(u["stratum"] in strata for u in p["units"])) * len(p["comparisons"]) * p["resamples"] > 20_000_000:
        raise durable.ContractError("complete analysis exceeds 20000000 resampled contributions")
    if not 0 < finite(p["confidence"]) < 1:
        raise durable.ContractError("invalid confidence level")
    if not isinstance(p["seed"], str) or not re.fullmatch(r"0|[1-9][0-9]{0,19}", p["seed"]) or int(p["seed"]) > 2**64 - 1:
        raise durable.ContractError("seed must be a canonical decimal u64")
    if p["multiplicity"] not in ("marginal-exploratory", "bonferroni-family"):
        raise durable.ContractError("multiplicity must be declared")
    arms_metrics = {(c[a], c["metric"]) for c in p["comparisons"] for a in ("reference", "candidate")}
    if sum(len(u["replicates"]) for u in p["units"]) * len(arms_metrics) > MAX_ROWS:
        raise durable.ContractError("planned analysis inventory exceeds 20000 observations")
    return p


def _pointer(value, pointer):
    if not isinstance(pointer, str) or not pointer.startswith("/") or len(pointer) > 256:
        raise durable.ContractError("an explicit JSON pointer is required")
    for part in pointer[1:].split("/"):
        if re.search(r"~(?![01])", part):
            raise durable.ContractError("invalid JSON pointer escape")
        part = part.replace("~1", "/").replace("~0", "~")
        if isinstance(value, list):
            if not re.fullmatch(r"0|[1-9][0-9]*", part) or len(part) > 8 or int(part) >= len(value):
                raise durable.ContractError("JSON pointer index missing")
            value = value[int(part)]
        elif isinstance(value, dict) and part in value:
            value = value[part]
        else:
            raise durable.ContractError("JSON pointer field missing")
    return finite(value)


def observations_from_catalogue(store, protocol, selections):
    """Project explicitly selected scalar result fields from the real catalogue.

    Selectors identify unit, replicate, arm, metric, campaign, step, output and
    pointer. Only terminal non-final campaigns in the protocol domain and
    verified artifact/provenance references are accepted. Failed/unexecuted
    steps retain their actual states as nonnumeric observations. At most 64
    campaign snapshots may be selected. No success or numeric value is imputed.
    """
    p = canonical_protocol(protocol)
    if not isinstance(selections, list) or len(selections) > MAX_ROWS:
        raise durable.ContractError("selection inventory exceeds the analysis budget")
    rows, records, sources = [], {}, set()
    for selection in selections:
        if not isinstance(selection, dict) or set(selection) != {"unit", "replicate", "arm", "metric", "campaign", "step", "output", "pointer"}:
            raise durable.ContractError("invalid analysis result selector")
        campaign = selection["campaign"]
        if campaign not in records:
            if len(records) >= 64:
                raise durable.ContractError("analysis source inventory exceeds 64 campaigns")
            record = store.get(campaign)
            spec = runtime.canonical_campaign(record["spec"])
            if spec["domain"] != p["domain"] or record["phase"] not in ("completed", "failed", "cancelled", "imported"):
                raise durable.ContractError("analysis source is incomplete or in another domain")
            state, steps = runtime.validated_snapshot(spec, record["snapshot"])
            if state not in runtime.TERMINAL:
                raise durable.ContractError("analysis requires a terminal workflow snapshot")
            records[campaign] = (record, steps)
        record, steps = records[campaign]
        step = steps.get(selection["step"])
        if selection["output"] not in record["spec"]["outputs"].get(selection["step"], {}):
            raise durable.ContractError("selected step/output was not declared")
        found = store.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?", (campaign, selection["step"], selection["output"])).fetchone()
        if found is None:
            if step is not None and step["state"] == "succeeded":
                raise durable.ContractError("successful step has no verified result")
            state = step["state"] if step is not None else "not-started"
            rows.append({**{k: selection[k] for k in ("unit", "replicate", "arm", "metric")},
                         "status": "technical-error" if state == "failed" else "missing", "value": None,
                         "reason": "hub-step-" + state, "source": None})
            continue
        if step is None or step["state"] != "succeeded":
            raise durable.ContractError("verified result conflicts with terminal step state")
        evidence = store.result(campaign, selection["step"], selection["output"])
        descriptor = artifacts.canonical_artifact(evidence["descriptor"])
        if (descriptor["access_class"] != p["domain"].lower()
                or artifacts.artifact_identity(descriptor) != evidence["artifact_identity"]
                or artifacts.provenance_identity(evidence["provenance"]) != evidence["provenance_identity"]):
            raise durable.ContractError("analysis source evidence integrity/access mismatch")
        source_key = (evidence["provenance_identity"], selection["pointer"], selection["metric"])
        if source_key in sources:
            raise durable.ContractError("one source observation cannot become multiple experimental observations")
        sources.add(source_key)
        rows.append({**{k: selection[k] for k in ("unit", "replicate", "arm", "metric")}, "status": "observed",
                     "value": _pointer(evidence["json"], selection["pointer"]), "reason": None,
                     "source": {"campaign": campaign, "artifact_identity": evidence["artifact_identity"],
                                "provenance_identity": evidence["provenance_identity"], "pointer": selection["pointer"]}})
    return {"schema": 1, "domain": p["domain"], "protocol_identity": identity("tdi-analysis-protocol/v1", p), "rows": rows}


def analyze(protocol, observations, worker):
    """Produce deterministic paired effects/intervals with complete denominators.

    Repeats are first paired and averaged inside each unit. The shared SciRust
    bootstrap then resamples complete unit contrasts with equal unit weights.
    Fewer than two complete units yields `insufficient-units` and a null interval.
    The selected independent-unit assumption remains a protocol responsibility.
    """
    p = canonical_protocol(protocol)
    protocol_id = identity("tdi-analysis-protocol/v1", p)
    if (not isinstance(observations, dict) or set(observations) != {"schema", "domain", "protocol_identity", "rows"}
            or type(observations["schema"]) is not int or observations["schema"] != 1
            or observations["domain"] != p["domain"] or observations["protocol_identity"] != protocol_id
            or not isinstance(observations["rows"], list) or len(observations["rows"]) > MAX_ROWS):
        raise durable.ContractError("observations do not match the declared analysis protocol")
    units = {u["id"]: u for u in p["units"]}
    arms_metrics = {(c[a], c["metric"]) for c in p["comparisons"] for a in ("reference", "candidate")}
    rows, sources = {}, set()
    for row in observations["rows"]:
        if not isinstance(row, dict) or set(row) != {"unit", "replicate", "arm", "metric", "status", "value", "reason", "source"}:
            raise durable.ContractError("invalid analysis observation")
        for key in ("unit", "replicate", "arm", "metric"): label(row[key])
        key = tuple(row[k] for k in ("unit", "replicate", "arm", "metric"))
        if (key in rows or key[0] not in units or key[1] not in units[key[0]]["replicates"] or key[2:] not in arms_metrics):
            raise durable.ContractError("duplicate or unplanned observation")
        if row["status"] == "observed":
            finite(row["value"])
            source = row["source"]
            if row["reason"] is not None or not isinstance(source, dict) or set(source) != {"campaign", "artifact_identity", "provenance_identity", "pointer"}:
                raise durable.ContractError("observed value requires exact source references")
            for name in ("campaign", "artifact_identity", "provenance_identity"):
                graphs._sha256(source[name], "analysis source")
            source_key = (source["provenance_identity"], source["pointer"], row["metric"])
            if source_key in sources:
                raise durable.ContractError("duplicated source observation")
            sources.add(source_key)
        elif row["status"] in ("missing", "technical-error", "rejected"):
            if row["value"] is not None or row["source"] is not None:
                raise durable.ContractError("failed observation cannot carry a numeric value")
            label(row["reason"])
        else:
            raise durable.ContractError("unknown observation state")
        rows[key] = row
    family_size = len(p["comparisons"]) * (1 + len(p["strata"]))
    confidence = 1 - (1 - p["confidence"]) / family_size if p["multiplicity"] == "bonferroni-family" else p["confidence"]
    results = []
    for c in p["comparisons"]:
        for stratum in [None, *p["strata"]]:
            expected = [u for u in p["units"] if stratum is None or u["stratum"] == stratum]
            included, exclusions, denominator = [], [], {arm: {"planned": 0, "observed": 0, "missing": 0, "technical-error": 0, "rejected": 0} for arm in (c["reference"], c["candidate"])}
            for unit in expected:
                values, reasons = {c["reference"]: [], c["candidate"]: []}, []
                for rep in unit["replicates"]:
                    for arm in values:
                        row = rows.get((unit["id"], rep, arm, c["metric"]))
                        status = row["status"] if row else "missing"
                        denominator[arm]["planned"] += 1
                        denominator[arm][status] += 1
                        if status == "observed": values[arm].append(row["value"])
                        else: reasons.append({"replicate": rep, "arm": arm, "status": status, "reason": row["reason"] if row else "no-observation"})
                if reasons:
                    exclusions.append({"unit": unit["id"], "reasons": reasons})
                    continue
                means = {arm: _mean(v) for arm, v in values.items()}
                included.append({"unit": unit["id"], "reference": means[c["reference"]], "candidate": means[c["candidate"]],
                                 "effect": finite(means[c["candidate"]] - means[c["reference"]]), "paired_repeats": len(unit["replicates"])})
            if exclusions and p["missing"] == "reject-incomplete":
                raise durable.ContractError("protocol rejects incomplete experimental units")
            interval = None
            if len(included) >= 2:
                # Derive independent streams from immutable comparison/scope
                # coordinates, not output values or completion order.
                stream = int(identity("tdi-analysis-stream/v1", {"seed": p["seed"], "comparison": c["id"], "stratum": stratum})[:16], 16)
                interval = worker.call("paired_mean_percentile", contrasts=[x["effect"] for x in included], resamples=p["resamples"], confidence=confidence, seed=str(stream))
            results.append({"comparison": c, "stratum": stratum, "expected_units": len(expected), "included_units": len(included),
                            "excluded_units": len(exclusions), "denominators": denominator, "unit_effects": included, "exclusions": exclusions,
                            "status": "analyzed" if interval else "insufficient-units", "interval": interval})
    report = {"schema": 1, "kind": "tdi-paired-analysis", "protocol": p, "protocol_identity": protocol_id,
              "observations_identity": identity("tdi-analysis-observations/v1", observations), "observations": observations,
              "implementation": worker.provenance, "family_size": family_size, "results": results,
              "cost_measurements": None, "cost_status": "not-selected-or-measured-by-this-analysis",
              "scientific_verdict": "not-assessed", "effect_convention": "candidate-minus-reference",
              "limitations": ["declared independent units are an assumption", "percentile coverage is approximate", "exploratory non-final analysis; no optional stopping decision", "rows supplied directly are caller assertions; catalogue extraction preserves verified source references"]}
    report["identity"] = identity("tdi-paired-analysis/v1", report)
    return report
