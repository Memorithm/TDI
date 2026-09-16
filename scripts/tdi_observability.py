"""Optional MLflow 2.0 REST and OTLP/HTTP JSON exports from durable evidence.

No exporter participates in scientific execution or success decisions. Immutable
plans and bounded delivery state live in the catalogue; failures remain visible.
Only selected scalar metrics and non-final provenance references leave TDI.
"""
from __future__ import annotations

import copy
import math
import re
import urllib.parse

import tdi_artifact_contract as artifacts
import tdi_engine_runtime as runtime
import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable
from tdi_engine_store import identity
from tdi_hub_client import BoundedOriginClient, HubClientError

MAX_EXPORT_RESULTS = 200
MAX_EXPORT_EVENTS = 1000
MAX_EXPORT_METRICS = 100
EXIT_EXPORT = 23


class ExportError(durable.ContractError):
    """External delivery failed or remains ambiguous; scientific data is intact."""


class MLflowClient(BoundedOriginClient):
    """Versioned MLflow REST client using the bounded, authenticated origin policy."""
    path_prefix = "/api/2.0/mlflow/"


class OTLPClient(BoundedOriginClient):
    """OTLP/HTTP JSON transport; no automatic retry or credential-bearing redirect."""
    path_prefix = "/v1/"


def _label(value, label, limit=128):
    if not isinstance(value, str) or not re.fullmatch(r"[A-Za-z0-9_. /-]{1," + str(limit) + "}", value):
        raise durable.ContractError("invalid " + label)
    return value


def _source(store, campaign):
    record = store.get(campaign)
    spec = runtime.canonical_campaign(record["spec"])
    if record["phase"] not in ("completed", "failed", "cancelled", "imported"):
        raise durable.ContractError("external export requires terminal evidence")
    state, steps = runtime.validated_snapshot(spec, record["snapshot"])
    if state not in runtime.TERMINAL:
        raise durable.ContractError("exported snapshot must be terminal")
    count = store.db.execute("SELECT COUNT(*) FROM results WHERE campaign=?", (campaign,)).fetchone()[0]
    if count > MAX_EXPORT_RESULTS:
        raise durable.ContractError("export result inventory exceeds 200; select a smaller campaign")
    results = store.results(campaign, limit=MAX_EXPORT_RESULTS)
    for result in results:
        evidence = result["evidence"]
        descriptor = artifacts.canonical_artifact(evidence["descriptor"])
        provenance = artifacts.canonical_provenance(evidence["provenance"])
        if (descriptor["access_class"] != spec["domain"].lower()
                or artifacts.artifact_identity(descriptor) != evidence["artifact_identity"]
                or artifacts.provenance_identity(provenance) != evidence["provenance_identity"]):
            raise durable.ContractError("external export evidence integrity/access mismatch")
    # Source timestamps describe catalogue lifecycle, not GPU/worker compute.
    terminal = store.db.execute("SELECT recorded_ns FROM events WHERE campaign=? AND kind IN ('completed','failed','cancelled','imported') ORDER BY sequence LIMIT 1", (campaign,)).fetchone()
    if terminal is None or terminal[0] < record["created_ns"]:
        raise durable.ContractError("missing or reversed catalogue lifecycle clock")
    return record, steps, results, terminal[0]


def _selected_metrics(results, selections, timestamp):
    if not isinstance(selections, list) or len(selections) > MAX_EXPORT_METRICS:
        raise durable.ContractError("metric selection exceeds 100 entries")
    outputs = {(r["step"], r["output"]): r["evidence"] for r in results}
    metrics, units, seen = [], [], set()
    for selection in selections:
        if not isinstance(selection, dict) or set(selection) != {"key", "step", "output", "field", "unit"}:
            raise durable.ContractError("invalid metric selection schema")
        key = _label(selection["key"], "metric key")
        unit = _label(selection["unit"], "metric unit", 32)
        if key.startswith("tdi.") or key in seen:
            raise durable.ContractError("metric key is reserved or duplicated")
        seen.add(key)
        if not all(isinstance(selection[k], str) for k in ("step", "output", "field")):
            raise durable.ContractError("metric selectors must be strings")
        evidence = outputs.get((selection["step"], selection["output"]))
        if evidence is None or selection["field"] not in evidence["json"]:
            raise durable.ContractError("selected metric is absent from verified evidence")
        value = evidence["json"][selection["field"]]
        if (type(value) not in (int, float) or (type(value) is int and abs(value) > 2**53 - 1)
                or not math.isfinite(value)):
            raise durable.ContractError("selected metric is not a finite binary64-safe scalar")
        metrics.append({"key": key, "value": value, "timestamp": timestamp, "step": 0})
        units.append({"key": "tdi.unit." + key, "value": unit})
    return metrics, units


def prepare_mlflow(store, campaign, client, experiment_id, selections=None):
    """Freeze a bounded export; experiment must already exist on the chosen server.

    Example: ``prepare_mlflow(store, campaign, client, '0', [])``. Detailed raw
    JSON, environment variables, reserved evaluator labels and tokens are never
    auto-discovered. Selected field units are explicit and preserved as tags.
    """
    experiment_id = _label(experiment_id, "MLflow experiment id", 128)
    record, steps, results, end = _source(store, campaign)
    timestamp = end // 1_000_000
    metrics, units = _selected_metrics(results, [] if selections is None else selections, timestamp)
    metrics += [{"key": "tdi." + name, "value": value, "timestamp": timestamp, "step": 0} for name, value in (
        ("verified_outputs", len(results)), ("steps_planned", len(record["spec"]["graph"]["steps"])),
        ("steps_succeeded", sum(s.get("state") == "succeeded" for s in steps.values())),
        ("attempts_observed", sum(len(s.get("attempts", [])) for s in steps.values())),
    )]
    params = [{"key": key, "value": value} for key, value in {
        "tdi.domain": record["spec"]["domain"], "tdi.root_plan": record["spec"]["graph"]["root_plan_id"],
        "tdi.graph": graphs.graph_identity(record["spec"]["graph"]), "tdi.purpose": record["spec"]["purpose"],
    }.items()]
    tags = units + [{"key": "tdi.phase", "value": record["phase"]}, {"key": "tdi.workflow", "value": record["workflow"]},
                    {"key": "tdi.result_semantics", "value": "technical evidence; no automatic beneficial verdict"}]
    for index, result in enumerate(results):
        evidence = result["evidence"]
        tags += [{"key": f"tdi.artifact.{index}", "value": evidence["artifact_identity"]},
                 {"key": f"tdi.provenance.{index}", "value": evidence["provenance_identity"]}]
    batches = []
    for offset in range(0, max(len(metrics), len(params), len(tags)), 100):
        batches.append({"metrics": metrics[offset:offset + 100], "params": params[offset:offset + 100], "tags": tags[offset:offset + 100]})
    plan = {"schema": 1, "create": {"experiment_id": experiment_id, "run_name": "TDI " + campaign[:16],
                                    "start_time": record["created_ns"] // 1_000_000},
            "batches": batches, "finish": {"status": "FINISHED" if record["snapshot"]["state"] == "succeeded" else "FAILED", "end_time": timestamp}}
    return store.prepare_export(campaign, "mlflow", client.endpoint, plan)


def _attribute(key, value):
    return {"key": key, "value": {"stringValue": str(value)}}


def prepare_otlp(store, campaign, client):
    """Freeze OTLP JSON metrics, traces and lifecycle logs from actual catalogue data.

    Metric dimensions are the finite domain/state vocabulary. Campaign identity
    belongs to spans and logs. Durations cover the catalogue lifecycle, including
    queue/user wait; no GPU, energy, worker CPU or fresh cached timing is inferred.
    """
    record, steps, results, end = _source(store, campaign)
    resource = {"attributes": [_attribute("service.name", "tdi-research-engine")]}
    scope = {"name": "tdi.catalogue", "version": "1.0.0"}
    trace = identity("tdi-otlp-trace/v1", {"campaign": campaign, "workflow": record["workflow"]})[:32]
    span = identity("tdi-otlp-span/v1", {"campaign": campaign, "workflow": record["workflow"]})[:16]
    dimensions = [_attribute("tdi.domain", record["spec"]["domain"]), _attribute("tdi.state", record["snapshot"]["state"])]
    metrics = [{"name": "tdi.catalogue." + name, "unit": "1", "description": "Count for the selected durable campaign; trace identifies the source.",
                "gauge": {"dataPoints": [{"attributes": dimensions, "timeUnixNano": str(end), "asInt": str(value),
                    "exemplars": [{"timeUnixNano": str(end), "asInt": str(value), "traceId": trace, "spanId": span}]}]}}
               for name, value in (("verified_outputs", len(results)), ("steps_observed", len(steps)),
                                   ("attempts_observed", sum(len(s.get("attempts", [])) for s in steps.values())))]
    span_data = {"traceId": trace, "spanId": span, "name": "tdi.catalogue.campaign", "kind": 1,
                 "startTimeUnixNano": str(record["created_ns"]), "endTimeUnixNano": str(end),
                 "attributes": dimensions + [_attribute("tdi.campaign", campaign), _attribute("tdi.workflow", record["workflow"]),
                     _attribute("tdi.clock_scope", "catalogue wall-clock lifecycle, includes queue and user wait")],
                 "status": {"code": 1 if record["snapshot"]["state"] == "succeeded" else 2}}
    logs, cursor = [], 0
    allowed = {"prepared", "submitting", "submission-unknown", "admitted", "executing", "cancel-requested", "completed", "failed", "cancelled", "result-verified", "imported"}
    while True:
        page = store.events(campaign, after=cursor)
        if not page:
            break
        for event in page:
            cursor = event["sequence"]
            if event["kind"] not in allowed or event["recorded_ns"] > end:
                continue
            if len(logs) >= MAX_EXPORT_EVENTS:
                raise durable.ContractError("OTLP lifecycle inventory exceeds 1000 events")
            logs.append({"timeUnixNano": str(event["recorded_ns"]), "severityNumber": 9,
                         "body": {"stringValue": event["kind"]}, "traceId": trace, "spanId": span,
                         "attributes": [_attribute("tdi.campaign", campaign), _attribute("tdi.sequence", event["sequence"])]})
    plan = {"schema": 1, "requests": [
        {"path": "/v1/metrics", "value": {"resourceMetrics": [{"resource": resource, "scopeMetrics": [{"scope": scope, "metrics": metrics}]}]}},
        {"path": "/v1/traces", "value": {"resourceSpans": [{"resource": resource, "scopeSpans": [{"scope": scope, "spans": [span_data]}]}]}},
        {"path": "/v1/logs", "value": {"resourceLogs": [{"resource": resource, "scopeLogs": [{"scope": scope, "logRecords": logs}]}]}},
    ]}
    return store.prepare_export(campaign, "otlp", client.endpoint, plan)


def send_export(store, key, client, *, retry=False, resume_run=None):
    """Deliver a persisted plan once; explicit failed-export retry may repeat data.

    A lost run-creation reply requires an explicitly selected existing run with
    the matching export tag. No new MLflow run is created on retry. Remote
    delivery is not exactly-once; original scientific results remain immutable.
    """
    with store.delivery_lock():
        return _send_export_locked(store, key, client, retry=retry, resume_run=resume_run)


def _send_export_locked(store, key, client, *, retry, resume_run):
    record = store.get_export(key)
    expected_type = MLflowClient if record["kind"] == "mlflow" else OTLPClient
    if not isinstance(client, expected_type) or client.endpoint != record["endpoint"]:
        raise durable.ContractError("export client kind/origin mismatch")
    if record["state"] == "sent":
        return record
    if record["state"] == "sending" and not (record["kind"] == "otlp" and retry is True):
        raise ExportError("export has an unfinished delivery; inspect the receiver before reconciliation")
    if record["state"] != "prepared" and not (retry is True and record["state"] in ("failed", "sending")):
        raise ExportError("failed export requires an explicit retry")
    receipt = copy.deepcopy(record["receipt"])
    receipt.pop("delivery_error", None)
    receipt["delivery_attempt"] = receipt.get("delivery_attempt", 0) + 1
    receipt["requests_acknowledged"] = 0
    receipt["batches_acknowledged"] = 0
    if record["kind"] == "mlflow" and record["state"] == "failed" and not (resume_run or receipt.get("run_id")):
        raise ExportError("run creation outcome unknown; supply the existing run id after receiver inspection")
    store.export_transition(key, record["state"], "sending", receipt)
    try:
        if record["kind"] == "mlflow":
            _send_mlflow(store, record, client, receipt, resume_run)
        else:
            for index, request in enumerate(record["plan"]["requests"]):
                response = client.request("POST", request["path"], value=request["value"])
                partial = response.get("partialSuccess", {})
                if not isinstance(partial, dict) or any(str(v) not in ("", "0") for v in partial.values()):
                    raise ExportError("collector reported partial delivery")
                receipt["requests_acknowledged"] = index + 1
                store.export_transition(key, "sending", "sending", receipt)
        store.export_transition(key, "sending", "sent", receipt)
    except (HubClientError, ExportError, ValueError, KeyError, TypeError):
        receipt["delivery_error"] = "receiver failed or returned an invalid/partial response; data may have been written"
        store.export_transition(key, "sending", "failed", receipt)
        raise ExportError("external export failed; durable campaign and results are unchanged; inspect export " + key) from None
    return store.get_export(key)


def reconcile_mlflow(store, key, client, run_id=None):
    """Read an existing matching MLflow run and reconcile interrupted delivery.

    The delivery lock proves no sender is active in this local catalogue. This
    never creates or mutates a remote run. Complete payload/status agreement
    closes the receipt; a partial run becomes explicitly retryable using its id.
    """
    with store.delivery_lock():
        record = store.get_export(key)
        if record["kind"] != "mlflow" or not isinstance(client, MLflowClient) or client.endpoint != record["endpoint"]:
            raise durable.ContractError("reconciliation requires the matching MLflow origin")
        if record["state"] == "sent":
            return record
        if record["state"] not in ("sending", "failed"):
            raise durable.ContractError("only unfinished delivery can be reconciled")
        run_id = _label(run_id or record["receipt"].get("run_id"), "existing MLflow run id")
        run = client.request("GET", "/api/2.0/mlflow/runs/get?" + urllib.parse.urlencode({"run_id": run_id}))["run"]
        tags = {t["key"]: t["value"] for t in run["data"].get("tags", [])}
        if tags.get("tdi.export_id") != key or run["info"]["experiment_id"] != record["plan"]["create"]["experiment_id"]:
            raise durable.ContractError("receiver run belongs to another export")
        observed = {kind: {item["key"]: item for item in run["data"].get(kind, [])} for kind in ("metrics", "params", "tags")}
        complete = (run["info"]["status"] == record["plan"]["finish"]["status"]
                    and run["info"].get("end_time") == record["plan"]["finish"]["end_time"]
                    and run["info"].get("start_time") == record["plan"]["create"]["start_time"])
        for batch in record["plan"]["batches"]:
            for kind in observed:
                for item in batch[kind]:
                    actual = observed[kind].get(item["key"], {})
                    fields = ("value", "timestamp", "step") if kind == "metrics" else ("value",)
                    complete &= all(actual.get(field) == item[field] for field in fields)
        receipt = dict(record["receipt"], run_id=run_id, reconciled=True)
        if record["state"] == "failed":
            store.export_transition(key, "failed", "sending", receipt)
        store.export_transition(key, "sending", "sent" if complete else "failed", receipt)
        return store.get_export(key)


def _send_mlflow(store, record, client, receipt, resume_run):
    key, plan = record["id"], record["plan"]
    run_id = resume_run or receipt.get("run_id")
    if run_id:
        _label(run_id, "MLflow run id")
        response = client.request("GET", "/api/2.0/mlflow/runs/get?" + urllib.parse.urlencode({"run_id": run_id}))
        run = response["run"]
        tags = {t["key"]: t["value"] for t in run["data"].get("tags", [])}
        if tags.get("tdi.export_id") != key or run["info"]["experiment_id"] != plan["create"]["experiment_id"]:
            raise durable.ContractError("existing MLflow run belongs to another export")
    else:
        value = dict(plan["create"], tags=[{"key": "tdi.export_id", "value": key}, {"key": "tdi.campaign", "value": record["campaign"]}])
        response = client.request("POST", "/api/2.0/mlflow/runs/create", value=value)
        run_id = _label(response["run"]["info"]["run_id"], "MLflow run id")
    receipt["run_id"] = run_id
    store.export_transition(key, "sending", "sending", receipt)
    for index, batch in enumerate(plan["batches"]):
        client.request("POST", "/api/2.0/mlflow/runs/log-batch", value=dict(batch, run_id=run_id))
        receipt["batches_acknowledged"] = index + 1
        store.export_transition(key, "sending", "sending", receipt)
    client.request("POST", "/api/2.0/mlflow/runs/update", value=dict(plan["finish"], run_id=run_id))
