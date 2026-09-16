"""Actual MLflow server and official OTLP protobuf validation of real Hub results.

Requires TDI_HUBD_BIN, TDI_DURABLE_WORKER and TDI_MLFLOW_BIN. The optional test
environment pins mlflow==3.16.0 and opentelemetry-proto==1.44.0. No cloud account
or public endpoint is used; failure injection is confined to loopback receivers.
"""
import base64
import copy
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import socket
import subprocess
import threading
import time
import unittest
import urllib.error
import urllib.request

from google.protobuf.json_format import ParseDict
from opentelemetry.proto.collector.logs.v1.logs_service_pb2 import ExportLogsServiceRequest
from opentelemetry.proto.collector.metrics.v1.metrics_service_pb2 import ExportMetricsServiceRequest
from opentelemetry.proto.collector.trace.v1.trace_service_pb2 import ExportTraceServiceRequest

import tdi_engine_runtime as runtime
from tdi_engine_store import EngineStore
from tdi_hub_fixture import prepare_fixture
from tdi_observability import MLflowClient, OTLPClient, ExportError, prepare_mlflow, prepare_otlp, send_export, reconcile_mlflow
import test_tdi_engine_integration as hub_fixture


class ObservabilityIntegrationTests(unittest.TestCase):
    def setUp(self):
        hub_fixture.OperationalIntegrationTests.setUpClass()
        self.hub = hub_fixture.OperationalIntegrationTests()
        self.addCleanup(self.hub.doCleanups)
        self.hub.setUp()
        spec = prepare_fixture(self.hub.client, self.hub.worker, trials=1)
        with EngineStore(self.hub.catalogue) as store:
            self.campaign = runtime.submit(self.hub.client, store, spec, {})
            self.original = runtime.execute(self.hub.client, store, self.campaign)
            self.original_results = store.results(self.campaign)

    def test_real_mlflow_preserves_values_provenance_and_failure_recovery(self):
        binary = Path(os.environ["TDI_MLFLOW_BIN"]).resolve(strict=True)
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        endpoint = f"http://127.0.0.1:{port}"
        root = self.hub.root
        with (root / "mlflow.log").open("ab") as log:
            server = subprocess.Popen([str(binary), "server", "--host", "127.0.0.1", "--port", str(port), "--workers", "1",
                "--backend-store-uri", "sqlite:///" + str(root / "mlflow.sqlite"),
                "--artifacts-destination", str(root / "mlflow-artifacts")],
                env={"PATH": os.defpath, "MLFLOW_DISABLE_TELEMETRY": "true", "PYTHONDONTWRITEBYTECODE": "1"},
                stdin=subprocess.DEVNULL, stdout=log, stderr=log)
            try:
                deadline = time.monotonic() + 45
                while time.monotonic() < deadline:
                    if server.poll() is not None:
                        self.fail((root / "mlflow.log").read_text()[-4000:])
                    try:
                        with urllib.request.urlopen(endpoint + "/health", timeout=0.3):
                            break
                    except (OSError, urllib.error.URLError):
                        time.sleep(0.05)
                else:
                    self.fail("MLflow did not start: " + (root / "mlflow.log").read_text()[-4000:])
                client = MLflowClient(endpoint, allow_loopback_http=True, timeout=10)
                experiment = client.request("POST", "/api/2.0/mlflow/experiments/create", value={"name": "tdi-software-qualification"})["experiment_id"]
                with EngineStore(self.hub.catalogue) as store:
                    prepared = prepare_mlflow(store, self.campaign, client, experiment)
                    sent = send_export(store, prepared["id"], client)
                    self.assertEqual("sent", sent["state"])
                    run_id = sent["receipt"]["run_id"]
                    run = client.request("GET", "/api/2.0/mlflow/runs/get?run_id=" + run_id)["run"]
                    self.assertEqual("FINISHED", run["info"]["status"])
                    metrics = {m["key"]: m["value"] for m in run["data"]["metrics"]}
                    self.assertEqual(2, metrics["tdi.verified_outputs"])
                    tags = {t["key"]: t["value"] for t in run["data"]["tags"]}
                    self.assertEqual(self.campaign, tags["tdi.campaign"])
                    self.assertEqual(self.original_results[0]["evidence"]["artifact_identity"], tags["tdi.artifact.0"])
                    self.assertEqual(sent, send_export(store, prepared["id"], client))
                    # Preserve a receiver success whose final acknowledgement
                    # was lost by the sender; reconciliation performs GET only.
                    store.db.execute("UPDATE exports SET state='sending' WHERE id=?", (sent["id"],))
                    store.db.commit()
                    self.assertEqual("sent", reconcile_mlflow(store, sent["id"], client)["state"])
                    # Equal values with a different observation step must not
                    # be mistaken for the original acknowledged payload.
                    client.request("POST", "/api/2.0/mlflow/runs/log-batch", value={"run_id": run_id,
                        "metrics": [{"key": "tdi.verified_outputs", "value": 2, "step": 9,
                                     "timestamp": prepared["plan"]["finish"]["end_time"]}], "params": [], "tags": []})
                    store.db.execute("UPDATE exports SET state='sending' WHERE id=?", (sent["id"],))
                    store.db.commit()
                    self.assertEqual("failed", reconcile_mlflow(store, sent["id"], client)["state"])
                    # A distinct explicit experiment target creates another
                    # export; injecting the first log failure leaves its run id.
                    experiment2 = client.request("POST", "/api/2.0/mlflow/experiments/create", value={"name": "tdi-export-failure"})["experiment_id"]
                    second = prepare_mlflow(store, self.campaign, client, experiment2)
                    original_request = client.request
                    def fault(method, path, **kwargs):
                        if path.endswith("log-batch"):
                            raise ExportError("injected exporter failure")
                        return original_request(method, path, **kwargs)
                    client.request = fault
                    with self.assertRaises(ExportError):
                        send_export(store, second["id"], client)
                    self.assertEqual("failed", store.get_export(second["id"])["state"])
                    client.request = original_request
                    self.assertEqual("sent", send_export(store, second["id"], client, retry=True)["state"])
                    self.assertEqual(self.original, store.get(self.campaign))
                    self.assertEqual(self.original_results, store.results(self.campaign))
            finally:
                server.terminate()
                try:
                    server.wait(timeout=15)
                except subprocess.TimeoutExpired:
                    server.kill(); server.wait(timeout=5)
                    self.fail("MLflow did not stop cleanly")

    def test_otlp_official_schema_partial_failure_and_retry(self):
        messages, errors = [], []
        reject = {"enabled": True}
        schemas = {"/v1/metrics": ExportMetricsServiceRequest, "/v1/traces": ExportTraceServiceRequest, "/v1/logs": ExportLogsServiceRequest}
        def proto_json(value):
            # OTLP JSON deviates from protobuf JSON only for these byte IDs.
            if isinstance(value, dict):
                return {key: base64.b64encode(bytes.fromhex(item)).decode() if key in ("traceId", "spanId", "parentSpanId") else proto_json(item) for key, item in value.items()}
            if isinstance(value, list):
                return [proto_json(item) for item in value]
            return value
        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass
            def do_POST(self):
                try:
                    if self.headers["Content-Type"] != "application/json":
                        raise ValueError("wrong content type")
                    size = int(self.headers["Content-Length"])
                    if not 0 < size <= 1024 * 1024:
                        raise ValueError("oversized OTLP request")
                    value = json.loads(self.rfile.read(size))
                    message = ParseDict(proto_json(value), schemas[self.path](), ignore_unknown_fields=False)
                    messages.append((self.path, value, message))
                    response = {"partialSuccess": {"rejectedLogRecords": "1", "errorMessage": "injected rejection"}} if self.path == "/v1/logs" and reject["enabled"] else {}
                    raw = json.dumps(response).encode()
                    self.send_response(200)
                    self.send_header("Content-Type", "application/json")
                    self.send_header("Content-Length", str(len(raw)))
                    self.end_headers(); self.wfile.write(raw)
                except Exception as error:
                    errors.append(str(error))
                    self.send_error(400)
        with HTTPServer(("127.0.0.1", 0), Handler) as receiver:
            thread = threading.Thread(target=receiver.serve_forever, daemon=True)
            thread.start()
            try:
                client = OTLPClient(f"http://127.0.0.1:{receiver.server_port}", allow_loopback_http=True)
                with EngineStore(self.hub.catalogue) as store:
                    plan = prepare_otlp(store, self.campaign, client)
                    failure = self.hub.cli("export-otlp", self.campaign, "--endpoint", client.endpoint, expected_code=23)
                    self.assertEqual("external-export-error", failure["status"])
                    self.assertEqual([], errors)
                    self.assertEqual(3, len(messages))
                    self.assertEqual("failed", store.get_export(plan["id"])["state"])
                    self.assertEqual(self.original, store.get(self.campaign))
                    reject["enabled"] = False
                    self.assertEqual("sent", self.hub.cli("retry-export", plan["id"])["state"])
                    self.assertEqual("sent", self.hub.cli("inspect-export", plan["id"])["state"])
                    self.assertEqual(6, len(messages))
                    # Crash recovery is an explicit replay under an exclusive
                    # local sender lock. The receiver may observe duplicates.
                    store.db.execute("UPDATE exports SET state='sending' WHERE id=?", (plan["id"],))
                    store.db.commit()
                    with self.assertRaises(ExportError):
                        send_export(store, plan["id"], client)
                    self.assertEqual(6, len(messages))
                    self.assertEqual("sent", send_export(store, plan["id"], client, retry=True)["state"])
                    self.assertEqual(9, len(messages))
                    self.assertEqual(self.original_results, store.results(self.campaign))
                    metric = messages[0][1]["resourceMetrics"][0]["scopeMetrics"][0]["metrics"][0]
                    self.assertEqual({"tdi.domain", "tdi.state"}, {a["key"] for a in metric["gauge"]["dataPoints"][0]["attributes"]})
                    raw = json.dumps(plan["plan"])
                    self.assertNotIn("plan_id", raw)
                    self.assertNotIn("scores", raw)
                    self.assertNotIn(self.hub.token, raw)
            finally:
                receiver.shutdown(); thread.join(timeout=5)


if __name__ == "__main__":
    unittest.main()
