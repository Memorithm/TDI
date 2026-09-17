"""Minimal-install export, migration and failure-boundary qualification."""
import json
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

import tdi_experiment_supervisor as durable
from tdi_engine_store import EngineStore, identity
from tdi_observability import _selected_metrics, prepare_mlflow, MLflowClient
from test_tdi_engine_unit import campaign_fixture


class ObservabilityUnitTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "catalogue.sqlite"

    def legacy(self):
        spec = campaign_fixture()
        campaign = identity("tdi-operational-campaign/v1", spec)
        with sqlite3.connect(self.path) as db:
            db.executescript((Path(__file__).parent / "fixtures" / "tdi-catalogue-v1.sql").read_text())
            db.execute("INSERT INTO campaigns VALUES (?,?,?,?,NULL,NULL,NULL,?)", (campaign, durable.canonical(spec), "http://127.0.0.1:8477", "prepared", 1))
            db.execute("INSERT INTO events VALUES (1,?,'prepared',?,1)", (campaign, '{"legacy": true}'))
            db.execute("INSERT INTO results VALUES (?,'fixture','value',?)", (campaign, '{"historical": "unaltered bytes"}'))
        return campaign

    def test_v1_readonly_and_atomic_current_migration_preserve_existing_evidence(self):
        campaign = self.legacy()
        with EngineStore(self.path, readonly=True) as old:
            original = old.get(campaign)
            self.assertEqual(1, old.db.execute("PRAGMA user_version").fetchone()[0])
        with EngineStore(self.path) as upgraded:
            self.assertEqual(3, upgraded.db.execute("PRAGMA user_version").fetchone()[0])
            self.assertEqual(original, upgraded.get(campaign))
            self.assertEqual('{"legacy": true}', upgraded.db.execute("SELECT payload FROM events WHERE sequence=1").fetchone()[0])
            self.assertEqual('{"historical": "unaltered bytes"}', upgraded.db.execute("SELECT evidence FROM results").fetchone()[0])
            self.assertEqual([], upgraded.list_exports())
            upgraded.backup(Path(self.temp.name) / "backup.sqlite")

    def test_interrupted_migration_rolls_back_ddl_and_schema_version(self):
        campaign = self.legacy()
        original = EngineStore._create_exports
        def fail(store):
            original(store)
            raise sqlite3.OperationalError("injected migration failure")
        with patch.object(EngineStore, "_create_exports", fail), self.assertRaises(sqlite3.Error):
            EngineStore(self.path)
        with sqlite3.connect(self.path) as db:
            self.assertEqual(1, db.execute("PRAGMA user_version").fetchone()[0])
            self.assertIsNone(db.execute("SELECT name FROM sqlite_master WHERE name='exports'").fetchone())
        with EngineStore(self.path) as recovered:
            self.assertEqual("prepared", recovered.get(campaign)["phase"])

    def test_delivery_queue_is_bounded_and_same_plan_is_idempotent(self):
        with EngineStore(self.path) as store:
            campaign = store.create(campaign_fixture(), "http://127.0.0.1:8477")
            first = None
            for index in range(32):
                row = store.prepare_export(campaign, "otlp", "http://127.0.0.1:4318", {"schema": 1, "test_index": index})
                first = first or row
            self.assertEqual(first, store.prepare_export(campaign, "otlp", "http://127.0.0.1:4318", {"schema": 1, "test_index": 0}))
            with self.assertRaisesRegex(durable.ContractError, "queue is full"):
                store.prepare_export(campaign, "otlp", "http://127.0.0.1:4318", {"schema": 1, "test_index": 33})
            self.assertEqual(32, len(store.list_exports()))

    def test_delivery_lock_does_not_block_campaign_catalogue(self):
        with EngineStore(self.path) as one, EngineStore(self.path) as two:
            with one.delivery_lock():
                with self.assertRaises(BlockingIOError), two.delivery_lock():
                    pass
                campaign = two.create(campaign_fixture(), "http://127.0.0.1:8477")
                self.assertEqual("prepared", one.get(campaign)["phase"])
            with two.delivery_lock():
                pass

    def test_metric_export_requires_explicit_safe_scalars_and_units(self):
        rows = [{"step": "run", "output": "result", "evidence": {"json": {"score": 2.5}}}]
        choice = {"key": "quality", "step": "run", "output": "result", "field": "score", "unit": "distance"}
        metrics, units = _selected_metrics(rows, [choice], 1000)
        self.assertEqual(2.5, metrics[0]["value"])
        self.assertEqual("distance", units[0]["value"])
        for value in (True, "2.5", [], float("nan"), float("inf"), 10**1000):
            rows[0]["evidence"]["json"]["score"] = value
            with self.subTest(value=type(value).__name__), self.assertRaises(durable.ContractError):
                _selected_metrics(rows, [choice], 1000)
        with self.assertRaises(durable.ContractError):
            _selected_metrics([], [choice], 1000)

    def test_incomplete_campaign_cannot_create_external_export(self):
        client = MLflowClient("http://127.0.0.1:5000", allow_loopback_http=True)
        with EngineStore(self.path) as store:
            campaign = store.create(campaign_fixture(), "http://127.0.0.1:8477")
            with self.assertRaisesRegex(durable.ContractError, "terminal"):
                prepare_mlflow(store, campaign, client, "0")
            self.assertEqual([], store.list_exports())


if __name__ == "__main__":
    unittest.main()
