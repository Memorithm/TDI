"""Real proof sharing, exact portable reconstruction, corruption and migration."""
import copy
from pathlib import Path
import sqlite3
import tempfile
import unittest
from unittest.mock import patch

import tdi_engine_archive as archive
import tdi_engine_runtime as runtime
import tdi_experiment_supervisor as durable
from tdi_engine_store import EngineStore
from tdi_hub_fixture import prepare_fixture
import test_tdi_engine_integration as hub_fixture


class SharedEvidenceTests(unittest.TestCase):
    def hub(self):
        hub_fixture.OperationalIntegrationTests.setUpClass()
        hub = hub_fixture.OperationalIntegrationTests(); self.addCleanup(hub.doCleanups); hub.setUp()
        return hub

    def test_actual_proof_sharing_export_restore_and_corruption(self):
        hub = self.hub()
        spec = prepare_fixture(hub.client, hub.worker, trials=4)
        with EngineStore(hub.catalogue) as store:
            key = runtime.submit(hub.client, store, spec, {})
            runtime.execute(hub.client, store, key)
            results = store.results(key)
            self.assertEqual(8, len(results))
            self.assertEqual(1, store.db.execute('SELECT COUNT(*) FROM shared_proofs').fetchone()[0])
            reference_bytes = sum(len(durable.canonical(r['evidence']).encode()) for r in results)
            stored_bytes = store.db.execute('SELECT SUM(length(CAST(evidence AS BLOB))) FROM results').fetchone()[0]
            stored_bytes += store.db.execute('SELECT SUM(length(CAST(payload AS BLOB))) FROM shared_proofs').fetchone()[0]
            self.assertLess(stored_bytes, reference_bytes)
            first = results[0]
            store.put_result(key, first['step'], first['output'], first['evidence'])
            bad = copy.deepcopy(first['evidence']); bad['json']['unexpected'] = True
            with self.assertRaises(durable.ContractError): store.put_result(key, first['step'], first['output'], bad)
            output = hub.root / 'archive.json'
            archive.export_bundle(hub.client, store, key, output)
            bundle = durable.strict_json(output.read_bytes(), max_bytes=16 * 1024 * 1024)
            archive.verify_bundle(bundle)
            self.assertEqual(results, [{k: m[k] for k in ('step', 'output', 'evidence')} for m in bundle['members']])
            proof = store.db.execute('SELECT id,payload FROM shared_proofs').fetchone()
            with store.db: store.db.execute('UPDATE shared_proofs SET payload=? WHERE id=?', ('{}', proof[0]))
            with self.assertRaisesRegex(durable.ContractError, 'proof corrupted'): store.result(key, first['step'], first['output'])
            with store.db: store.db.execute('UPDATE shared_proofs SET payload=? WHERE id=?', (proof[1], proof[0]))
            raw = store.db.execute('SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?', (key, first['step'], first['output'])).fetchone()[0]
            compact = durable.strict_json(raw); compact['evidence']['json']['unexpected'] = True
            with store.db: store.db.execute('UPDATE results SET evidence=? WHERE campaign=? AND step=? AND output=?', (durable.canonical(compact), key, first['step'], first['output']))
            with self.assertRaisesRegex(durable.ContractError, 'identity mismatch'): store.result(key, first['step'], first['output'])
            with store.db: store.db.execute('UPDATE results SET evidence=? WHERE campaign=? AND step=? AND output=?', (raw, key, first['step'], first['output']))
            store.backup(hub.root / 'backup.sqlite')
        with EngineStore(hub.root / 'backup.sqlite', readonly=True) as backup:
            self.assertEqual(results, backup.results(key))
        with EngineStore(hub.root / 'restored.sqlite') as restored:
            archive.restore_bundle(hub.client, restored, bundle)
            self.assertEqual(results, restored.results(key))
            self.assertEqual(1, restored.db.execute('SELECT COUNT(*) FROM shared_proofs').fetchone()[0])
            with restored.db: restored.db.execute('DELETE FROM shared_proofs')
            with self.assertRaisesRegex(durable.ContractError, 'proof missing'): restored.result(key, first['step'], first['output'])

    def test_failed_publication_rolls_back_new_shared_proof_then_reconciles(self):
        hub = self.hub()
        with EngineStore(hub.catalogue) as store:
            spec = prepare_fixture(hub.client, hub.worker, trials=1)
            key = runtime.submit(hub.client, store, spec, {})
            event = store._event
            def fail(campaign, kind, payload):
                if kind == 'result-verified': raise sqlite3.OperationalError('injected durable evidence commit failure')
                return event(campaign, kind, payload)
            with patch.object(store, '_event', side_effect=fail), self.assertRaises(sqlite3.Error):
                runtime.execute(hub.client, store, key)
            self.assertEqual(0, store.db.execute('SELECT COUNT(*) FROM results').fetchone()[0])
            self.assertEqual(0, store.db.execute('SELECT COUNT(*) FROM shared_proofs').fetchone()[0])
            self.assertEqual('completed', runtime.refresh(hub.client, store, key)['phase'])
            self.assertEqual(2, len(store.results(key)))

    def test_v3_migration_is_atomic_and_keeps_all_legacy_bytes(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / 'legacy.sqlite'
            with sqlite3.connect(path) as db:
                db.executescript(Path(__file__).with_name('fixtures').joinpath('tdi-catalogue-v3.sql').read_text())
                db.execute("INSERT INTO campaigns VALUES ('legacy','{ \"original\": true }','http://127.0.0.1:8477','prepared',NULL,NULL,NULL,1)")
                db.execute("INSERT INTO results VALUES ('legacy','step','out','{ \"legacy\": true }')")
                db.execute("INSERT INTO events VALUES (1,'legacy','legacy','{ \"preserved\": true }',2)")
            original = EngineStore._create_shared_proofs
            def fail(store):
                original(store); raise sqlite3.OperationalError('injected schema-4 DDL failure')
            with patch.object(EngineStore, '_create_shared_proofs', fail), self.assertRaises(sqlite3.Error): EngineStore(path)
            with sqlite3.connect(path) as db:
                self.assertEqual(3, db.execute('PRAGMA user_version').fetchone()[0])
                self.assertIsNone(db.execute("SELECT name FROM sqlite_master WHERE name='shared_proofs'").fetchone())
            with EngineStore(path, readonly=True) as old:
                self.assertEqual({'legacy': True}, old.result('legacy', 'step', 'out'))
            with EngineStore(path) as current:
                self.assertEqual(4, current.db.execute('PRAGMA user_version').fetchone()[0])
                self.assertEqual('{ "legacy": true }', current.db.execute('SELECT evidence FROM results').fetchone()[0])
                self.assertEqual('{ "preserved": true }', current.db.execute('SELECT payload FROM events').fetchone()[0])
                self.assertEqual({'legacy': True}, current.result('legacy', 'step', 'out'))

    def test_indexed_pagination_keeps_exact_order_and_phase_filter(self):
        with tempfile.TemporaryDirectory() as directory, EngineStore(Path(directory) / 'catalogue.sqlite') as store:
            keys = [store.create({'synthetic': i}, 'http://127.0.0.1:8477') for i in range(80)]
            self.assertEqual(keys[30:50], [row['id'] for row in store.list(after=30, limit=20)])
            self.assertEqual(keys[30:50], [row['id'] for row in store.list(after=30, limit=20, phase='prepared')])
            self.assertEqual([], store.list(phase='failed'))
            traced = []; store.db.set_trace_callback(traced.append)
            store.list(after=30, limit=20, phase='prepared'); store.db.set_trace_callback(None)
            query = next(sql for sql in traced if sql.startswith('SELECT id,phase'))
            plan = [row[3] for row in store.db.execute('EXPLAIN QUERY PLAN ' + query)]
            self.assertTrue(any('campaigns_by_phase_time' in step for step in plan))
            self.assertFalse(any('TEMP B-TREE' in step for step in plan))


if __name__ == '__main__': unittest.main()
