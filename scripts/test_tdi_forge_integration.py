"""Actual Forge/Hub/TDI search, independent oracle, restart and failure recovery."""
import copy
import json
import os
from pathlib import Path
import sqlite3
import shutil
import tempfile
import unittest
from unittest.mock import patch

import test_tdi_engine_integration as operational
import tdi_experiment_supervisor as durable
from tdi_engine_store import EngineStore
import tdi_engine_runtime as runtime
import tdi_forge_search as search
from tdi_hub_client import HubTransportUnknown


class SearchIntegrationTests(unittest.TestCase):
    setUp = operational.OperationalIntegrationTests.setUp
    start_hub = operational.OperationalIntegrationTests.start_hub
    stop_hub = operational.OperationalIntegrationTests.stop_hub
    cli = operational.OperationalIntegrationTests.cli

    @classmethod
    def setUpClass(cls):
        cls.hubd = Path(os.environ["TDI_HUBD_BIN"]).resolve(strict=True)
        cls.worker = Path(os.environ["TDI_SEARCH_WORKER"]).resolve(strict=True)
        cls.forge_worker = Path(os.environ["TDI_FORGE_WORKER"]).resolve(strict=True)
        cls.forge_source = os.environ["TDI_FORGE_SOURCE_COMMIT"]
        cls.tdi_source = os.environ["TDI_SEARCH_SOURCE_COMMIT"]

    def prepare(self):
        return self.cli("search-fixture", "--worker", self.worker, "--tdi-source-commit", self.tdi_source,
                        "--forge-worker", self.forge_worker, "--forge-sha256", durable.file_digest(self.forge_worker),
                        "--forge-source-commit", self.forge_source, "--seed", str(2**64 - 1))

    def test_actual_search_restart_incorrect_excluded_and_no_repeat_measurement(self):
        prepared = self.prepare(); key = prepared["id"]
        paused = self.cli("search-run", key, "--max-stages", 1)
        self.assertEqual("paused", paused["phase"])
        self.assertEqual(1, paused["response"]["snapshot"]["attempts"])
        self.stop_hub(); self.start_hub()
        completed = self.cli("search-resume", key)
        self.assertEqual("completed", completed["phase"])
        state = completed["response"]["snapshot"]
        self.assertEqual(["measured", "measured", "incorrect"], [x["status"] for x in state["candidates"]])
        self.assertIsNone(state["candidates"][2]["metrics"])
        self.assertEqual(8, state["attempts"])
        self.assertTrue(state["baseline_qualified"])
        self.assertEqual("not-assessed", state["scientific_verdict"])
        for c in state["candidates"][:2]:
            self.assertGreater(c["metrics"][0]["value"], 0)
            self.assertGreater(c["metrics"][1]["value"], 0)
        self.assertEqual(completed, self.cli("search-resume", key))
        with EngineStore(self.catalogue, readonly=True) as store:
            stages = store.db.execute("SELECT campaign FROM search_stages WHERE search=?", (key,)).fetchall()
            self.assertEqual(8, len(stages))
            values = [store.results(row[0])[0]["evidence"]["json"] for row in stages]
            self.assertEqual(2, sum(v["stage"] == "measure" for v in values))
            self.assertTrue(all(v["wall_ms"] >= 0 for v in values))
            self.assertEqual(24, len(next(v for v in values if v["stage"] == "verify")["cost_measurements"]))
            self.assertTrue(all(not e["evidence"]["cache_eligible"] for row in stages for e in store.results(row[0])))
            raw = json.dumps(completed["response"]["generation_view"])
            self.assertNotIn("verification", raw); self.assertNotIn("validation", raw)

    def test_completed_stage_survives_tell_persistence_failure_without_redispatch(self):
        key = self.prepare()["id"]
        original = EngineStore.update_search
        def fail_tell(store, key, sequence, phase, response, event):
            if event.get("command", {}).get("operation", {}).get("op") == "finish":
                raise sqlite3.OperationalError("injected persistence failure after actual Hub completion")
            return original(store, key, sequence, phase, response, event)
        with EngineStore(self.catalogue) as store, patch.object(EngineStore, "update_search", fail_tell):
            with self.assertRaises(sqlite3.OperationalError):
                search.run_search(self.client, store, key, max_stages=1)
            pending = store.get_search(key)["response"]["snapshot"]["active_attempt"]
            mapping = store.search_stage(key, pending["attempt_id"])
            first = store.get(mapping["campaign"])["snapshot"]
            self.assertEqual("succeeded", first["state"])
        self.stop_hub(); self.start_hub()
        resumed = self.cli("search-resume", key, "--max-stages", 1)
        self.assertIsNone(resumed["response"]["snapshot"]["active_attempt"])
        self.assertEqual(1, resumed["response"]["snapshot"]["attempts"])
        with EngineStore(self.catalogue, readonly=True) as store:
            self.assertEqual(first, store.get(mapping["campaign"])["snapshot"])
            self.assertEqual(1, store.db.execute("SELECT COUNT(*) FROM search_stages WHERE search=?", (key,)).fetchone()[0])

    def test_cancellation_before_dispatch_and_corrupt_projection_fail_closed(self):
        key = self.prepare()["id"]
        with EngineStore(self.catalogue) as store:
            record = search._control(store, key, {"op": "ask"})
            candidate = record["response"]["snapshot"]["candidates"][0]["proposal"]["candidate_id"]
            record = search._control(store, key, {"op": "begin", "candidate_id": candidate, "stage": "compile"})
            permit = record["response"]["snapshot"]["active_attempt"]
            mapping = search._prepare_stage(self.client, store, record, permit)
        cancelled = self.cli("search-cancel", key, expected_code=durable.EXIT_TRIAL_FAILURE)
        self.assertEqual("cancelled", cancelled["phase"])
        with EngineStore(self.catalogue) as store:
            campaign = store.get(mapping["campaign"])
            self.assertEqual("cancelled", campaign["phase"]); self.assertIsNone(campaign["workflow"])
            with self.assertRaises(durable.ContractError):
                store.bind_search_stage(key, "a" * 64, campaign["spec"], {}, self.endpoint)
        self.assertEqual(cancelled, self.cli("search-resume", key, expected_code=durable.EXIT_TRIAL_FAILURE))
        # A separate non-final contract is used for the corruption check.
        with EngineStore(self.catalogue) as store:
            forge = search.ForgeClient(self.forge_worker, durable.file_digest(self.forge_worker), self.forge_source)
            second = search.prepare_fixture(self.client, store, forge, self.worker, self.tdi_source, seed="1")
            forged = copy.deepcopy(second["response"]); forged["snapshot"]["baseline_qualified"] = True
            with store.db:
                store.db.execute("UPDATE searches SET response=? WHERE id=?", (durable.canonical(forged), second["id"]))
            with self.assertRaisesRegex(durable.ContractError, "differs from Forge replay"):
                search.run_search(self.client, store, second["id"])

    def test_ambiguous_submission_requires_attach_before_search_cancellation(self):
        key = self.prepare()["id"]
        with EngineStore(self.catalogue) as store:
            record = search._control(store, key, {"op": "ask"})
            candidate = record["response"]["snapshot"]["candidates"][0]["proposal"]["candidate_id"]
            record = search._control(store, key, {"op": "begin", "candidate_id": candidate, "stage": "compile"})
            permit = record["response"]["snapshot"]["active_attempt"]
            mapping = search._prepare_stage(self.client, store, record, permit)
            campaign = store.get(mapping["campaign"])
            submitted = {}

            def lose_binding(spec, response, roots):
                submitted["workflow"] = response["id"]
                raise ValueError("injected lost workflow response")

            with patch.object(runtime, "_bind_record", side_effect=lose_binding):
                with self.assertRaises(HubTransportUnknown):
                    runtime.submit(self.client, store, campaign["spec"], mapping["roots"])
            self.assertEqual("submission-unknown", store.get(campaign["id"])["phase"])
            before = store.get_search(key)
            with self.assertRaisesRegex(durable.ContractError, "must be attached before search cancellation"):
                search.cancel_search(self.client, store, key)
            self.assertEqual(before["phase"], store.get_search(key)["phase"])
            runtime.attach(self.client, store, campaign["id"], submitted["workflow"], mapping["roots"])
            cancelled = search.cancel_search(self.client, store, key)
            self.assertEqual("cancelled", cancelled["phase"])
            self.assertEqual("cancelled", store.get(campaign["id"])["phase"])

    def test_cancelled_search_blocks_already_admitted_workflow_execution(self):
        key = self.prepare()["id"]
        with EngineStore(self.catalogue) as store:
            record = search._control(store, key, {"op": "ask"})
            candidate = record["response"]["snapshot"]["candidates"][0]["proposal"]["candidate_id"]
            record = search._control(store, key, {"op": "begin", "candidate_id": candidate, "stage": "compile"})
            mapping = search._prepare_stage(self.client, store, record, record["response"]["snapshot"]["active_attempt"])
            campaign = store.get(mapping["campaign"])
            runtime.submit(self.client, store, campaign["spec"], mapping["roots"])
            store.update_search(key, record["sequence"], "cancelled", record["response"], {"cancellation_requested": True})
            with self.assertRaises(durable.ContractError):
                runtime.execute(self.client, store, campaign["id"])
            self.assertEqual("admitted", store.get(campaign["id"])["phase"])
            self.assertEqual([], store.results(campaign["id"]))
            search.cancel_search(self.client, store, key)
            self.assertEqual("cancelled", store.get(campaign["id"])["phase"])

    def test_launch_failure_cost_is_kept_and_explicit_retry_consumes_budget(self):
        worker = self.root / "copied-finite-worker"
        shutil.copyfile(self.worker, worker); worker.chmod(0o600)
        prepared = self.cli("search-fixture", "--worker", worker, "--tdi-source-commit", self.tdi_source,
                            "--forge-worker", self.forge_worker, "--forge-sha256", durable.file_digest(self.forge_worker),
                            "--forge-source-commit", self.forge_source)
        failed = self.cli("search-run", prepared["id"], expected_code=durable.EXIT_TRIAL_FAILURE)
        self.assertEqual("paused", failed["phase"]); self.assertTrue(failed["execution_error"])
        self.assertEqual(2, failed["response"]["snapshot"]["attempts"])
        self.assertEqual("pending", failed["response"]["snapshot"]["candidates"][0]["status"])
        with EngineStore(self.catalogue, readonly=True) as store:
            evidence = search._stage_evidence(store, prepared["id"], failed["response"]["snapshot"]["candidates"][0]["proposal"]["candidate_id"], "verify")
            value = evidence["json"]
            self.assertEqual("failed", value["outcome"]["kind"])
            self.assertEqual("process-launch-failed", value["cost_measurements"][0]["technical_failure"])
            self.assertIsNone(value["cost_measurements"][0]["peak_rss_bytes"])
        worker.chmod(0o700)
        completed = self.cli("search-resume", prepared["id"])
        self.assertEqual("completed", completed["phase"])
        self.assertEqual(9, completed["response"]["snapshot"]["attempts"])
        self.assertEqual(135000, completed["response"]["snapshot"]["charged_ms"])


class SearchMigrationTests(unittest.TestCase):
    def test_v2_migration_rollback_and_backup_preserve_legacy_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "v2.sqlite"
            with sqlite3.connect(path) as db:
                db.executescript((Path(__file__).parent / "fixtures/tdi-catalogue-v2.sql").read_text())
                db.execute("INSERT INTO campaigns VALUES ('legacy','{}','http://127.0.0.1:8477','prepared',NULL,NULL,NULL,1)")
                db.execute("INSERT INTO exports VALUES ('export','legacy','otlp','http://127.0.0.1:4318','{}','failed',?,1)", ('{ \"original\": true }',))
            with EngineStore(path, readonly=True) as old:
                self.assertEqual([], old.list_searches())
            original = EngineStore._create_searches
            def fail(store):
                original(store)
                raise sqlite3.OperationalError("injected search migration commit failure")
            with patch.object(EngineStore, "_create_searches", fail), self.assertRaises(sqlite3.Error):
                EngineStore(path)
            with sqlite3.connect(path) as db:
                self.assertEqual(2, db.execute("PRAGMA user_version").fetchone()[0])
                self.assertIsNone(db.execute("SELECT name FROM sqlite_master WHERE name='searches'").fetchone())
            with EngineStore(path) as current:
                self.assertEqual(3, current.db.execute("PRAGMA user_version").fetchone()[0])
                self.assertEqual('{ \"original\": true }', current.db.execute("SELECT receipt FROM exports").fetchone()[0])
                current.backup(Path(directory) / "backup.sqlite")
            with EngineStore(Path(directory) / "backup.sqlite", readonly=True) as backup:
                self.assertEqual([], backup.list_searches())
                self.assertEqual('{ \"original\": true }', backup.db.execute("SELECT receipt FROM exports").fetchone()[0])


if __name__ == "__main__":
    unittest.main()
