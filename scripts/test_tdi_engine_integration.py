"""Real Hub/SQLite/process/Rust-library qualification; binaries are mandatory.

Run with TDI_HUBD_BIN and TDI_DURABLE_WORKER pointing at the built executables.
These fixtures use only a non-final finite counter oracle and synthetic faults.
No network service, dataset, model, protected payload or mock result is assumed.
"""
from __future__ import annotations

import copy
from http.server import HTTPServer
import json
import os
from pathlib import Path
import secrets
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
import urllib.error
import urllib.request
import uuid

import tdi_artifact_contract as artifacts
import tdi_engine_archive as archive
import tdi_engine_runtime as runtime
from tdi_engine_store import EngineStore, identity
from tdi_engine_viewer import make_handler
from tdi_hub_client import HubClient, HubTransportUnknown
from tdi_hub_fixture import prepare_fixture
import tdi_experiment_supervisor as durable


class OperationalIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.hubd = Path(os.environ["TDI_HUBD_BIN"]).resolve(strict=True)
        cls.worker = Path(os.environ["TDI_DURABLE_WORKER"]).resolve(strict=True)

    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="tdi-engine-test-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.catalogue = self.root / "catalogue.sqlite"
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            self.port = sock.getsockname()[1]
        self.endpoint = f"http://127.0.0.1:{self.port}"
        self.token = secrets.token_hex(24)
        self.process = None
        self.log = (self.root / "hub.log").open("ab")
        self.addCleanup(self.log.close)
        self.addCleanup(self.stop_hub)
        self.start_hub()
        self.client = HubClient(self.endpoint, token=self.token, allow_loopback_http=True, timeout=30)

    def start_hub(self):
        self.process = subprocess.Popen(
            [str(self.hubd), "--listen", f"127.0.0.1:{self.port}", "--data-dir", str(self.root / "hub")],
            env={"PATH": os.defpath, "SCIRUST_HUB_TOKEN": self.token},
            stdin=subprocess.DEVNULL, stdout=self.log, stderr=self.log,
        )
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                self.fail("Hub failed to start: " + (self.root / "hub.log").read_text()[-4000:])
            try:
                with urllib.request.urlopen(self.endpoint + "/health", timeout=0.2):
                    return
            except (OSError, urllib.error.URLError):
                time.sleep(0.02)
        self.fail("Hub did not become healthy")

    def stop_hub(self):
        if self.process is not None and self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait(timeout=5)
                self.fail("Hub required forced termination")

    def cli(self, *args, catalogue=None, expected_code=0):
        result = subprocess.run(
            [sys.executable, str(Path(__file__).with_name("tdi_engine.py")),
             "--catalogue", str(catalogue or self.catalogue), "--hub", self.endpoint,
             "--allow-loopback-http", *map(str, args)],
            env={"PATH": os.defpath, "TDI_HUB_TOKEN": self.token, "PYTHONDONTWRITEBYTECODE": "1"},
            capture_output=True, timeout=40, check=False,
        )
        self.assertEqual(expected_code, result.returncode, result.stderr.decode()[-4000:])
        value = json.loads(result.stdout)
        self.assertEqual(expected_code, value["exit_code"])
        return value.get("result", value)

    def test_real_rust_workflow_restart_export_restore_and_viewer(self):
        plan = self.root / "plan.json"
        self.cli("fixture-plan", "--worker", self.worker, "--trials", 2, "--output", plan)
        self.assertEqual(4, self.cli("validate", plan)["steps"])
        campaign = self.cli("submit", plan)["campaign"]
        completed = self.cli("run", campaign)
        self.assertEqual("completed", completed["phase"])
        outputs = self.cli("inspect", campaign)["results"]
        self.assertEqual(4, len(outputs))
        self.assertEqual({"Evaluated", "Verified"}, {r["evidence"]["json"]["status"] for r in outputs})
        for step in completed["snapshot"]["steps"]:
            self.assertEqual(1, len(step["attempts"]))
        cache_request = self.cli("cache-request", campaign, "run-0", "file:result")
        cache_path = self.root / "cache-request.json"
        cache_path.write_text(json.dumps(cache_request))
        self.cli("cache-get", cache_path, expected_code=durable.EXIT_CONTRACT)
        cached = self.cli("cache-get", cache_path, "--allow-exact-reuse")
        self.assertTrue(cached["cache_hit"])
        self.assertFalse(cached["new_execution"])
        self.assertFalse(cached["fresh_timing_measurement"])
        self.assertIn(cached["evidence"], [result["evidence"] for result in outputs])
        for field in ("plan_id", "step_identity", "implementation_identity", "backend_identity", "parameters_identity"):
            changed = dict(cache_request, **{field: "f" * 64})
            cache_path.write_text(json.dumps(changed))
            self.assertFalse(self.cli("cache-get", cache_path, "--allow-exact-reuse")["cache_hit"])
        changed = dict(cache_request, domain="Validation")
        cache_path.write_text(json.dumps(changed))
        self.assertFalse(self.cli("cache-get", cache_path, "--allow-exact-reuse")["cache_hit"])
        self.stop_hub()
        self.start_hub()
        resumed = self.cli("resume", campaign)
        self.assertEqual(completed["snapshot"], resumed["snapshot"])
        self.assertEqual(campaign, self.cli("submit", plan)["campaign"])
        self.assertEqual(completed["snapshot"], self.cli("run", campaign)["snapshot"])
        bundle_path = self.root / "bundle.json"
        receipt = self.cli("export", campaign, bundle_path)
        self.assertEqual(4, receipt["members"])
        self.assertEqual(4, self.cli("verify", bundle_path, "--expected-identity", receipt["identity"])["verified_members"])
        bundle = json.loads(bundle_path.read_text())
        lineage_mutations = (
            ("plan_id", "f" * 64),
            ("step_identity", "e" * 64),
            ("trial_id", "d" * 64),
            ("attempt_id", "c" * 64),
            ("inputs", [{"name": "graph", "identity": "b" * 64}]),
            ("dependencies", [{"name": "admission", "identity": "a" * 64}]),
        )
        for field, changed_value in lineage_mutations:
            forged = copy.deepcopy(bundle)
            evidence = forged["members"][0]["evidence"]
            evidence["provenance"][field] = changed_value
            evidence["provenance_identity"] = artifacts.provenance_identity(
                artifacts.canonical_provenance(evidence["provenance"])
            )
            forged["identity"] = identity(
                "tdi-evidence-bundle/v1",
                {k: v for k, v in forged.items() if k != "identity"},
            )
            with self.subTest(provenance_field=field), self.assertRaisesRegex(
                durable.ContractError, "provenance policy"
            ):
                archive.verify_bundle(forged)
        corrupt = copy.deepcopy(bundle)
        corrupt["members"].pop()
        corrupt["identity"] = identity("tdi-evidence-bundle/v1", {k: v for k, v in corrupt.items() if k != "identity"})
        with self.assertRaisesRegex(durable.ContractError, "incomplete"):
            archive.verify_bundle(corrupt)
        with self.assertRaisesRegex(durable.ContractError, "identity"):
            archive.verify_bundle(corrupt, expected_identity=receipt["identity"])
        restored_path = self.root / "restored.sqlite"
        self.assertEqual("imported", self.cli("restore", bundle_path, "--expected-identity", receipt["identity"], catalogue=restored_path)["phase"])
        imported = self.cli("inspect", campaign, catalogue=restored_path)
        self.assertEqual(outputs, imported["results"])
        self.cli("run", campaign, catalogue=restored_path, expected_code=durable.EXIT_CONTRACT)
        self.cli("cancel", campaign, catalogue=restored_path, expected_code=durable.EXIT_CONTRACT)
        self.cli("export", campaign, self.root / "reexport.json", catalogue=restored_path)
        backup = self.root / "backup.sqlite"
        self.cli("backup", backup)
        self.assertEqual(outputs, self.cli("inspect", campaign, catalogue=backup)["results"])
        with EngineStore(restored_path, readonly=True) as store:
            for result in outputs:
                evidence = result["evidence"]
                self.assertNotEqual(evidence["hub_artifact"], store.location(campaign, evidence["artifact_identity"])["hub_artifact_id"])
        with HTTPServer(("127.0.0.1", 0), make_handler(self.catalogue)) as viewer:
            thread = threading.Thread(target=viewer.serve_forever, daemon=True)
            thread.start()
            try:
                url = f"http://127.0.0.1:{viewer.server_port}"
                with urllib.request.urlopen(url + "/?campaign=" + campaign, timeout=5) as response:
                    self.assertIn(b"Verified outputs", response.read())
                    self.assertIn("frame-ancestors 'none'", response.headers["Content-Security-Policy"])
                with urllib.request.urlopen(url + "/api/campaigns", timeout=5) as response:
                    self.assertEqual(campaign, json.load(response)["result"][0]["id"])
                with self.assertRaises(urllib.error.HTTPError) as failure:
                    urllib.request.urlopen(urllib.request.Request(url, headers={"Host": "untrusted.invalid"}), timeout=5)
                self.assertEqual(403, failure.exception.code)
                failure.exception.close()
            finally:
                viewer.shutdown()
                thread.join(timeout=5)

    def fault_plan(self, program):
        spec = prepare_fixture(self.client, self.worker, trials=1)
        component_id = str(uuid.uuid4())
        registered = self.client.request("POST", "/api/v1/components", value={"schema_version": 1, "manifest": {
            "id": component_id, "name": "tdi-synthetic-fault", "version": "1.0.0", "kind": "tool",
            "capabilities": [{"name": "tdi.fixture.fault", "contract_version": "1.0.0", "inputs": [], "outputs": []}],
            "execution": {"type": "process", "program": sys.executable, "args": ["-c", program]},
        }})["component"]
        step = next(s for s in spec["graph"]["steps"] if s["key"] == "run-0")
        step.update(component_id=component_id, component_manifest_digest=registered["manifest_digest"], capability="tdi.fixture.fault", outputs=[])
        spec["graph"]["steps"] = [step]
        spec["policy"]["allowed_steps"] = {"run-0": {k: step[k] for k in spec["policy"]["allowed_steps"]["run-0"]}}
        spec["outputs"] = {"run-0": {}}
        return runtime.canonical_campaign(spec)

    def test_worker_failure_remains_failed_after_restart(self):
        spec = self.fault_plan("raise SystemExit(7)")
        with EngineStore(self.catalogue) as store:
            campaign = runtime.submit(self.client, store, spec, {})
            failed = runtime.execute(self.client, store, campaign)
            self.assertEqual("failed", failed["phase"])
            self.assertEqual([], store.results(campaign))
        self.stop_hub()
        self.start_hub()
        failed_again = self.cli("run", campaign, expected_code=durable.EXIT_TRIAL_FAILURE)
        self.assertEqual(failed["snapshot"], failed_again["snapshot"])

    def test_concurrent_cancellation_does_not_wait_for_client_lock(self):
        spec = self.fault_plan("import time; time.sleep(60)")
        with EngineStore(self.catalogue) as store:
            campaign = runtime.submit(self.client, store, spec, {})
            workflow = store.get(campaign)["workflow"]
        failures = []
        def execute():
            try:
                with EngineStore(self.catalogue) as store:
                    runtime.execute(self.client, store, campaign)
            except Exception as error:
                failures.append(error)
        thread = threading.Thread(target=execute, daemon=True)
        thread.start()
        deadline = time.monotonic() + 5
        while time.monotonic() < deadline:
            snapshot = self.client.request("GET", f"/api/v1/workflows/{workflow}")
            if snapshot["state"] == "running":
                break
            time.sleep(0.02)
        self.assertEqual("running", snapshot["state"])
        with EngineStore(self.catalogue) as store:
            result = runtime.cancel(self.client, store, campaign)
            self.assertIn(result["phase"], ("cancel-requested", "cancelled"))
        thread.join(timeout=20)
        self.assertFalse(thread.is_alive(), "cancellation did not release execution client")
        self.assertEqual([], failures)
        self.assertEqual("cancelled", self.cli("resume", campaign, expected_code=durable.EXIT_TRIAL_FAILURE)["phase"])

    def test_lost_submission_response_requires_read_only_attach(self):
        spec = prepare_fixture(self.client, self.worker, trials=1)
        client = self.client
        class LostResponse:
            endpoint = client.endpoint
            created = None
            posts = 0
            def request(self, method, path, **kwargs):
                if method == "POST":
                    self.posts += 1
                    self.created = client.request(method, path, **kwargs)["workflow"]
                    raise HubTransportUnknown("synthetic lost response")
                return client.request(method, path, **kwargs)
        lost = LostResponse()
        with EngineStore(self.catalogue) as store:
            with self.assertRaises(HubTransportUnknown):
                runtime.submit(lost, store, spec, {})
            campaign = runtime.submit(lost, store, spec, {})
            self.assertEqual(1, lost.posts)
            self.assertEqual("submission-unknown", store.get(campaign)["phase"])
            self.assertEqual("admitted", runtime.attach(client, store, campaign, lost.created["id"], {})["phase"])
            self.assertEqual("completed", runtime.execute(client, store, campaign)["phase"])

    def test_lost_execution_response_reconciles_without_redispatch(self):
        spec = prepare_fixture(self.client, self.worker, trials=1)
        client = self.client

        class LostExecutionResponse:
            endpoint = client.endpoint
            posts = 0

            def request(self, method, path, **kwargs):
                if method == "POST" and path.endswith("/executions"):
                    self.posts += 1
                    client.request(method, path, **kwargs)
                    raise HubTransportUnknown("synthetic lost execution response")
                return client.request(method, path, **kwargs)

            def download(self, *args, **kwargs):
                return client.download(*args, **kwargs)

        lost = LostExecutionResponse()
        with EngineStore(self.catalogue) as store:
            campaign = runtime.submit(client, store, spec, {})
            workflow = store.get(campaign)["workflow"]
            with self.assertRaises(HubTransportUnknown):
                runtime.execute(lost, store, campaign)
            self.assertEqual(1, lost.posts)
            self.assertEqual("executing", store.get(campaign)["phase"])

            deadline = time.monotonic() + 10
            while time.monotonic() < deadline:
                snapshot = client.request("GET", f"/api/v1/workflows/{workflow}")
                if snapshot["state"] in ("succeeded", "failed", "cancelled"):
                    break
                time.sleep(0.02)
            self.assertEqual("succeeded", snapshot["state"])

            reconciled = runtime.execute(lost, store, campaign)
            self.assertEqual("completed", reconciled["phase"])
            self.assertEqual(1, lost.posts)
            self.assertEqual(2, len(store.results(campaign)))
            for step in reconciled["snapshot"]["steps"]:
                self.assertEqual(1, len(step["attempts"]))


if __name__ == "__main__":
    unittest.main()
