"""Actual Elastic, Linux sensors/wait4, and local Hub resource admission."""
import copy
import json
import os
from pathlib import Path
import sys
import unittest

import test_tdi_engine_integration as operational
import tdi_experiment_supervisor as durable
from tdi_engine_store import EngineStore, identity
from tdi_physical_telemetry import capacity_snapshot, measured_process
from tdi_resource_admission import elastic_decision


class ResourceIntegrationTests(unittest.TestCase):
    setUp = operational.OperationalIntegrationTests.setUp
    start_hub = operational.OperationalIntegrationTests.start_hub
    stop_hub = operational.OperationalIntegrationTests.stop_hub
    cli = operational.OperationalIntegrationTests.cli

    @classmethod
    def setUpClass(cls):
        cls.hubd = Path(os.environ["TDI_HUBD_BIN"]).resolve(strict=True)
        cls.worker = Path(os.environ["TDI_DURABLE_WORKER"]).resolve(strict=True)
        cls.elastic = Path(os.environ["TDI_ELASTIC_BIN"]).resolve(strict=True)
        cls.source = os.environ["TDI_ELASTIC_SOURCE_COMMIT"]

    def policy(self):
        return {"max_concurrency": 1, "memory_bytes_per_trial": 1048576,
                "reserve_memory_bytes": 1048576, "max_age_milliseconds": 10000}

    def test_actual_elastic_admission_changes_only_width_and_reconciles(self):
        snapshot = self.cli("capacity")
        self.assertEqual("available", snapshot["capacity"]["status"], snapshot)
        plan, policy_path = self.root / "plan.json", self.root / "policy.json"
        self.cli("fixture-plan", "--worker", self.worker, "--trials", 3, "--output", plan)
        original = json.loads(plan.read_text())
        original_id = identity("tdi-operational-campaign/v1", original)
        self.assertEqual(3, original["graph"]["max_concurrency"])
        policy_path.write_text(json.dumps(self.policy()))
        command = ("run-local-admitted", plan, "--elastic-worker", self.elastic,
                   "--worker-sha256", durable.file_digest(self.elastic), "--source-commit", self.source,
                   "--resource-policy", policy_path)
        completed = self.cli(*command)
        self.assertEqual("completed", completed["phase"])
        actual_spec = copy.deepcopy(completed["spec"])
        actual_spec["graph"]["max_concurrency"] = 3
        self.assertEqual(original, actual_spec)
        self.assertEqual(1, completed["spec"]["graph"]["max_concurrency"])
        self.assertGreater(completed["cost_measurements"]["peak_rss_bytes"], 0)
        resumed = self.cli(*command)
        self.assertEqual(completed["snapshot"], resumed["snapshot"])
        self.assertTrue(resumed["resource_reconciled"])
        with EngineStore(self.catalogue, readonly=True) as store:
            events = store.events(completed["id"])
            self.assertTrue(any(e["kind"] == "resource-width-verified" for e in events))
            admission = next(e["payload"] for e in events if e["kind"] == "resource-admission")
            self.assertTrue(admission["elastic"]["report"]["committed"])
            self.assertEqual("Pass", admission["elastic"]["report"]["verification"])
            original_events = store.events(original_id)
            dispatch = next(e["payload"] for e in original_events if e["kind"] == "resource-dispatch")
            self.assertEqual(identity("tdi-resource-root-bindings/v1", {}),
                             dispatch["binding"]["root_bindings_identity"])

    def test_real_capacity_rejection_has_cost_and_never_submits_work(self):
        plan = self.root / "plan.json"
        self.cli("fixture-plan", "--worker", self.worker, "--trials", 1, "--output", plan)
        policy = self.policy(); policy["memory_bytes_per_trial"] = 2**64 - 1
        policy_path = self.root / "reject.json"; policy_path.write_text(json.dumps(policy))
        result = self.cli("run-local-admitted", plan, "--elastic-worker", self.elastic,
                          "--worker-sha256", durable.file_digest(self.elastic), "--source-commit", self.source,
                          "--resource-policy", policy_path, expected_code=durable.EXIT_TRIAL_FAILURE)
        self.assertEqual("resource-rejected", result["status"])
        self.assertEqual("insufficient-capacity", result["reason"])
        inspected = self.cli("inspect", result["campaign"])
        self.assertIsNone(inspected["campaign"]["workflow"])
        self.assertEqual([], inspected["results"])
        self.assertGreater(result["cost_measurements"]["peak_rss_bytes"], 0)
        with self.assertRaises(durable.ContractError):
            elastic_decision(json.loads(plan.read_text()), self.elastic, "0" * 64, self.source, policy)
        failing = self.root / "failing-worker"
        failing.write_text("#!/bin/sh\nexit 7\n"); failing.chmod(0o700)
        failure = elastic_decision(json.loads(plan.read_text()), failing, durable.file_digest(failing), self.source, policy)
        self.assertIsNone(failure["elastic"]["report"])
        self.assertIsNotNone(failure["elastic"]["protocol_failure"])
        self.assertEqual(7, failure["cost_measurements"]["exit_code"])

    def test_refuses_retroactive_admission_for_existing_hub_campaign(self):
        plan, policy_path = self.root / "plan.json", self.root / "policy.json"
        self.cli("fixture-plan", "--worker", self.worker, "--trials", 1, "--output", plan)
        campaign = self.cli("submit", plan)["campaign"]
        policy_path.write_text(json.dumps(self.policy()))
        result = self.cli("run-local-admitted", plan, "--elastic-worker", self.elastic,
                          "--worker-sha256", durable.file_digest(self.elastic), "--source-commit", self.source,
                          "--resource-policy", policy_path, expected_code=durable.EXIT_CONTRACT)
        self.assertEqual("contract-error", result["status"])
        self.assertIn("already dispatched without a resource-admission binding", result["error"])
        with EngineStore(self.catalogue, readonly=True) as store:
            self.assertEqual("admitted", store.get(campaign)["phase"])
            kinds = {event["kind"] for event in store.events(campaign)}
            self.assertNotIn("resource-admission", kinds)
            self.assertNotIn("resource-dispatch", kinds)

    def test_process_measurements_failure_timeout_and_output_bounds(self):
        code, out, _, measured = measured_process([sys.executable, "-c", "print('measured'); raise SystemExit(7)"])
        self.assertEqual(7, code); self.assertEqual(b"measured\n", out)
        self.assertGreater(measured["peak_rss_bytes"], 0)
        self.assertGreater(int(measured["wall_ns"]), 0)
        self.assertIsNone(measured["energy_joules"])
        _, _, _, timeout = measured_process([sys.executable, "-c", "import time; time.sleep(10)"], timeout=0.05)
        self.assertEqual("timeout", timeout["technical_failure"])
        _, out, _, budget = measured_process([sys.executable, "-c", "print('x'*100000)"], max_output=1024)
        self.assertEqual("output-budget", budget["technical_failure"]); self.assertEqual(1024, len(out))

    def test_actual_launch_failures_are_durable_and_never_dispatch(self):
        plan, policy_path = self.root / "plan.json", self.root / "policy.json"
        self.cli("fixture-plan", "--worker", self.worker, "--trials", 1, "--output", plan)
        policy_path.write_text(json.dumps(self.policy()))
        for name, content, mode in [("no-exec", "#!/bin/sh\nexit 0\n", 0o600),
                                    ("bad-format", "invalid executable format\n", 0o700),
                                    ("missing-loader", "#!/nonexistent/tdi-fixture-loader\n", 0o700)]:
            worker = self.root / name; worker.write_text(content); worker.chmod(mode)
            result = self.cli("run-local-admitted", plan, "--elastic-worker", worker,
                              "--worker-sha256", durable.file_digest(worker), "--source-commit", self.source,
                              "--resource-policy", policy_path, expected_code=durable.EXIT_TRIAL_FAILURE)
            self.assertEqual("resource-rejected", result["status"])
            self.assertEqual("process-launch-failed", result["cost_measurements"]["technical_failure"])
            self.assertIsNone(result["cost_measurements"]["exit_code"])
            self.assertIsNone(result["cost_measurements"]["peak_rss_bytes"])
            self.assertGreater(int(result["cost_measurements"]["launch_attempt_ns"]), 0)
            with EngineStore(self.catalogue, readonly=True) as store:
                self.assertIsNone(store.get(result["campaign"])["workflow"])
                evidence = [e["payload"] for e in store.events(result["campaign"]) if e["kind"] == "resource-admission"][-1]
                self.assertIsNone(evidence["elastic"]["report"])
                self.assertEqual("process-launch-failed", evidence["cost_measurements"]["technical_failure"])


if __name__ == "__main__":
    unittest.main()
