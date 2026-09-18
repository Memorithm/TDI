"""Failure-boundary tests; real worker execution is covered in integration tests."""
import copy
import errno
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from unittest.mock import patch

import tdi_engine_archive as archive
import tdi_engine_runtime as runtime
from tdi_engine_store import EngineStore, atomic_json
from tdi_engine_viewer import render_catalogue
from tdi_hub_client import HubClient, HubClientError, HubTransportUnknown
import tdi_experiment_supervisor as durable
from test_tdi_hub_admission_contract import fixture_graph, root_artifact_bindings, workflow_response


def campaign_fixture():
    graph = fixture_graph()
    return {"schema": 1, "purpose": "development-software", "domain": "Development", "graph": graph,
            "policy": {"trust": "trusted-software", "allowed_steps": {
                step["key"]: {k: step[k] for k in ("component_id", "component_version", "component_manifest_digest", "capability", "capability_contract_version")}
                for step in graph["steps"]}},
            "outputs": {step["key"]: {name: {"media_type": "application/json", "access_class": "development", "json_fields": {}, "cache": "disabled"}
                                      for name in step["outputs"]} for step in graph["steps"]}}


class StoreAndPolicyTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_readonly_and_unknown_schema_never_create_or_rewrite_catalogue(self):
        path = self.root / "missing.sqlite"
        with self.assertRaises(sqlite3.Error):
            EngineStore(path, readonly=True)
        self.assertFalse(path.exists())
        with sqlite3.connect(path) as db:
            db.execute("CREATE TABLE unrelated (value TEXT)")
            db.execute("INSERT INTO unrelated VALUES ('keep')")
        with self.assertRaises(durable.StorageError):
            EngineStore(path)
        with sqlite3.connect(path) as db:
            self.assertEqual(("keep",), db.execute("SELECT value FROM unrelated").fetchone())

    def test_failed_fsync_or_disk_full_does_not_publish_partial_json(self):
        target = self.root / "artifact.json"
        for call in ("os.fsync", "os.link"):
            with patch(call, side_effect=OSError(errno.ENOSPC, "synthetic disk full")):
                with self.assertRaises(OSError):
                    atomic_json(target, {"value": "complete"})
            self.assertFalse(target.exists())
            self.assertEqual([], list(self.root.iterdir()))

        with patch("tdi_engine_store.durable._fsync_directory",
                   side_effect=OSError(errno.EIO, "synthetic directory fsync failure")):
            with self.assertRaises(OSError):
                atomic_json(target, {"value": "complete"})
        self.assertFalse(target.exists())
        self.assertEqual([], list(self.root.iterdir()))

        def competing_replacement(_directory):
            target.unlink()
            target.write_text(json.dumps({"value": "external-writer"}))
            raise OSError(errno.EIO, "synthetic directory fsync failure")

        with patch("tdi_engine_store.durable._fsync_directory", side_effect=competing_replacement):
            with self.assertRaises(OSError):
                atomic_json(target, {"value": "complete"})
        self.assertEqual({"value": "external-writer"}, json.loads(target.read_text()))
        target.unlink()

        atomic_json(target, {"value": "original"})
        with self.assertRaises(FileExistsError):
            atomic_json(target, {"value": "replacement"})
        self.assertEqual({"value": "original"}, json.loads(target.read_text()))

    def _run_crash_child(self, code, *args, expected_returncode):
        env = os.environ.copy()
        scripts = str(Path(__file__).resolve().parent)
        env["PYTHONPATH"] = scripts + (os.pathsep + env["PYTHONPATH"] if env.get("PYTHONPATH") else "")
        completed = subprocess.run(
            [sys.executable, "-c", code, *map(str, args)],
            cwd=Path(__file__).resolve().parents[1],
            env=env,
            check=False,
            capture_output=True,
            text=True,
            timeout=20,
        )
        self.assertEqual(
            expected_returncode,
            completed.returncode,
            msg=f"child stderr={completed.stderr!r}; stdout={completed.stdout!r}",
        )

    def _run_signaled_child(self, code, *args, marker, signum):
        env = os.environ.copy()
        scripts = str(Path(__file__).resolve().parent)
        env["PYTHONPATH"] = scripts + (os.pathsep + env["PYTHONPATH"] if env.get("PYTHONPATH") else "")
        process = subprocess.Popen(
            [sys.executable, "-c", code, *map(str, args), str(marker)],
            cwd=Path(__file__).resolve().parents[1],
            env=env,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.monotonic() + 10
        try:
            while not marker.exists():
                returncode = process.poll()
                if returncode is not None:
                    stdout, stderr = process.communicate()
                    self.fail(
                        f"child exited before interruption point: returncode={returncode}; "
                        f"stderr={stderr!r}; stdout={stdout!r}"
                    )
                if time.monotonic() >= deadline:
                    self.fail("child did not reach interruption point")
                time.sleep(0.01)
            process.send_signal(signum)
            stdout, stderr = process.communicate(timeout=10)
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=5)
        self.assertEqual(
            -signum,
            process.returncode,
            msg=f"child stderr={stderr!r}; stdout={stdout!r}",
        )

    def test_atomic_json_process_crash_never_exposes_partial_named_payload(self):
        child = r'''\
import os
from pathlib import Path
import sys
import tdi_engine_store as store

target = Path(sys.argv[1])
stage = sys.argv[2]
if stage == "before-link":
    store._publish_completed_temp = lambda *_args: os._exit(91)
elif stage == "after-link-before-directory-fsync":
    store.durable._fsync_directory = lambda *_args: os._exit(92)
elif stage == "after-directory-fsync":
    original = store._publish_completed_temp
    def publish_then_crash(temporary, destination):
        original(temporary, destination)
        os._exit(93)
    store._publish_completed_temp = publish_then_crash
else:
    raise SystemExit("unknown stage")
store.atomic_json(target, {"value": "complete"})
raise SystemExit("crash hook was not reached")
'''
        expected = (durable.canonical({"value": "complete"}) + "\n").encode()
        cases = (
            ("before-link", 91, False),
            ("after-link-before-directory-fsync", 92, True),
            ("after-directory-fsync", 93, True),
        )
        for stage, returncode, published in cases:
            with self.subTest(stage=stage):
                root = self.root / f"json-{stage}"
                root.mkdir()
                target = root / "artifact.json"
                self._run_crash_child(child, target, stage, expected_returncode=returncode)
                self.assertEqual(published, target.exists())
                if published:
                    self.assertEqual(expected, target.read_bytes())
                else:
                    temporaries = list(root.glob(".tdi-*"))
                    self.assertEqual(1, len(temporaries))
                    self.assertEqual(expected, temporaries[0].read_bytes())

    def test_backup_process_crash_never_exposes_partial_named_catalogue(self):
        child = r'''\
import os
from pathlib import Path
import sys
import tdi_engine_store as store

source = Path(sys.argv[1])
destination = Path(sys.argv[2])
stage = sys.argv[3]
with store.EngineStore(source) as catalogue:
    if stage == "before-link":
        store._publish_completed_temp = lambda *_args: os._exit(94)
    elif stage == "after-link-before-directory-fsync":
        store.durable._fsync_directory = lambda *_args: os._exit(95)
    elif stage == "after-directory-fsync":
        original = store._publish_completed_temp
        def publish_then_crash(temporary, target):
            original(temporary, target)
            os._exit(96)
        store._publish_completed_temp = publish_then_crash
    else:
        raise SystemExit("unknown stage")
    catalogue.backup(destination)
raise SystemExit("crash hook was not reached")
'''
        cases = (
            ("before-link", 94, False),
            ("after-link-before-directory-fsync", 95, True),
            ("after-directory-fsync", 96, True),
        )
        for stage, returncode, published in cases:
            with self.subTest(stage=stage):
                root = self.root / f"backup-{stage}"
                root.mkdir()
                source = root / "catalogue.sqlite"
                destination = root / "backup.sqlite"
                self._run_crash_child(
                    child, source, destination, stage, expected_returncode=returncode
                )
                self.assertEqual(published, destination.exists())
                candidates = [destination] if published else list(root.glob(".tdi-backup-*"))
                self.assertEqual(1, len(candidates))
                with sqlite3.connect(candidates[0]) as backup:
                    self.assertEqual(("ok",), backup.execute("PRAGMA integrity_check").fetchone())
                    self.assertEqual((4,), backup.execute("PRAGMA user_version").fetchone())
                    tables = {
                        row[0]
                        for row in backup.execute(
                            "SELECT name FROM sqlite_master WHERE type='table'"
                        )
                    }
                    self.assertEqual(
                        {
                            "campaigns",
                            "events",
                            "results",
                            "cache",
                            "restored_artifacts",
                            "exports",
                            "searches",
                            "search_events",
                            "search_stages",
                            "shared_proofs",
                        },
                        tables,
                    )

    @unittest.skipUnless(hasattr(signal, "SIGHUP"), "requires POSIX SIGHUP")
    def test_atomic_json_external_sighup_preserves_publication_boundary(self):
        child = r'''\
import os
from pathlib import Path
import signal
import sys
import tdi_engine_store as store

target = Path(sys.argv[1])
stage = sys.argv[2]
marker = Path(sys.argv[3])

def pause_for_parent_signal():
    signal.signal(signal.SIGHUP, signal.SIG_DFL)
    if hasattr(signal, "pthread_sigmask"):
        signal.pthread_sigmask(signal.SIG_UNBLOCK, {signal.SIGHUP})
    with marker.open("xb") as stream:
        stream.write(b"ready")
        stream.flush()
        os.fsync(stream.fileno())
    signal.pause()

if stage == "before-link":
    store._publish_completed_temp = lambda *_args: pause_for_parent_signal()
elif stage == "after-link-before-directory-fsync":
    store.durable._fsync_directory = lambda *_args: pause_for_parent_signal()
elif stage == "after-directory-fsync":
    original = store._publish_completed_temp
    def publish_then_pause(temporary, destination):
        original(temporary, destination)
        pause_for_parent_signal()
    store._publish_completed_temp = publish_then_pause
else:
    raise SystemExit("unknown stage")
store.atomic_json(target, {"value": "complete"})
raise SystemExit("signal interruption point was not reached")
'''
        expected = (durable.canonical({"value": "complete"}) + "\n").encode()
        cases = (
            ("before-link", False),
            ("after-link-before-directory-fsync", True),
            ("after-directory-fsync", True),
        )
        for stage, published in cases:
            with self.subTest(stage=stage):
                root = self.root / f"json-sighup-{stage}"
                root.mkdir()
                target = root / "artifact.json"
                marker = root / ".ready"
                self._run_signaled_child(
                    child, target, stage, marker=marker, signum=signal.SIGHUP
                )
                self.assertEqual(published, target.exists())
                if published:
                    self.assertEqual(expected, target.read_bytes())
                else:
                    temporaries = list(root.glob(".tdi-*"))
                    self.assertEqual(1, len(temporaries))
                    self.assertEqual(expected, temporaries[0].read_bytes())

    @unittest.skipUnless(hasattr(signal, "SIGHUP"), "requires POSIX SIGHUP")
    def test_backup_external_sighup_preserves_publication_boundary(self):
        child = r'''\
import os
from pathlib import Path
import signal
import sys
import tdi_engine_store as store

source = Path(sys.argv[1])
destination = Path(sys.argv[2])
stage = sys.argv[3]
marker = Path(sys.argv[4])

def pause_for_parent_signal():
    signal.signal(signal.SIGHUP, signal.SIG_DFL)
    if hasattr(signal, "pthread_sigmask"):
        signal.pthread_sigmask(signal.SIG_UNBLOCK, {signal.SIGHUP})
    with marker.open("xb") as stream:
        stream.write(b"ready")
        stream.flush()
        os.fsync(stream.fileno())
    signal.pause()

with store.EngineStore(source) as catalogue:
    if stage == "before-link":
        store._publish_completed_temp = lambda *_args: pause_for_parent_signal()
    elif stage == "after-link-before-directory-fsync":
        store.durable._fsync_directory = lambda *_args: pause_for_parent_signal()
    elif stage == "after-directory-fsync":
        original = store._publish_completed_temp
        def publish_then_pause(temporary, target):
            original(temporary, target)
            pause_for_parent_signal()
        store._publish_completed_temp = publish_then_pause
    else:
        raise SystemExit("unknown stage")
    catalogue.backup(destination)
raise SystemExit("signal interruption point was not reached")
'''
        cases = (
            ("before-link", False),
            ("after-link-before-directory-fsync", True),
            ("after-directory-fsync", True),
        )
        expected_tables = {
            "campaigns",
            "events",
            "results",
            "cache",
            "restored_artifacts",
            "exports",
            "searches",
            "search_events",
            "search_stages",
            "shared_proofs",
        }
        for stage, published in cases:
            with self.subTest(stage=stage):
                root = self.root / f"backup-sighup-{stage}"
                root.mkdir()
                source = root / "catalogue.sqlite"
                destination = root / "backup.sqlite"
                marker = root / ".ready"
                self._run_signaled_child(
                    child,
                    source,
                    destination,
                    stage,
                    marker=marker,
                    signum=signal.SIGHUP,
                )
                self.assertEqual(published, destination.exists())
                candidates = [destination] if published else list(root.glob(".tdi-backup-*"))
                self.assertEqual(1, len(candidates))
                with sqlite3.connect(candidates[0]) as backup:
                    self.assertEqual(("ok",), backup.execute("PRAGMA integrity_check").fetchone())
                    self.assertEqual((4,), backup.execute("PRAGMA user_version").fetchone())
                    tables = {
                        row[0]
                        for row in backup.execute(
                            "SELECT name FROM sqlite_master WHERE type='table'"
                        )
                    }
                    self.assertEqual(expected_tables, tables)

    def test_backup_storage_failures_never_publish_named_partial_catalogue(self):
        destination = self.root / "backup.sqlite"
        with EngineStore(self.root / "catalogue.sqlite") as store:
            for call in ("os.fsync", "os.link"):
                with self.subTest(call=call):
                    with patch(call, side_effect=OSError(errno.ENOSPC, "synthetic disk full")):
                        with self.assertRaises(OSError):
                            store.backup(destination)
                    self.assertFalse(destination.exists())
                    self.assertEqual([], list(self.root.glob(".tdi-backup-*")))

            with patch("tdi_engine_store.durable._fsync_directory",
                       side_effect=OSError(errno.EIO, "synthetic directory fsync failure")):
                with self.assertRaises(OSError):
                    store.backup(destination)
            self.assertFalse(destination.exists())
            self.assertEqual([], list(self.root.glob(".tdi-backup-*")))

            def competing_publication(_source, target, **_kwargs):
                Path(target).write_bytes(b"external-writer")
                raise FileExistsError(target)

            with patch("os.link", side_effect=competing_publication):
                with self.assertRaises(FileExistsError):
                    store.backup(destination)
            self.assertEqual(b"external-writer", destination.read_bytes())
            destination.unlink()
            self.assertEqual([], list(self.root.glob(".tdi-backup-*")))

            store.backup(destination)
            self.assertTrue(destination.exists())
            with sqlite3.connect(destination) as backup:
                self.assertEqual(("ok",), backup.execute("PRAGMA integrity_check").fetchone())

            original = destination.read_bytes()
            with self.assertRaises(FileExistsError):
                store.backup(destination)
            self.assertEqual(original, destination.read_bytes())

    def test_failure_to_persist_submission_intent_prevents_network_mutation(self):
        spec = campaign_fixture()
        spec["graph"]["steps"][0]["inputs"] = {}
        class Client:
            endpoint = "http://127.0.0.1:8477"
            calls = 0
            def request(self, *args, **kwargs):
                self.calls += 1
                raise AssertionError("must persist intent first")
        client = Client()
        with EngineStore(self.root / "catalogue.sqlite") as store:
            with patch.object(store, "transition", side_effect=sqlite3.OperationalError("disk full")):
                with self.assertRaises(sqlite3.OperationalError):
                    runtime.submit(client, store, spec, {})
            self.assertEqual("prepared", store.list()[0]["phase"])
        self.assertEqual(0, client.calls)

    def test_catalogue_rejects_final_or_unversioned_campaigns_at_persistence_boundary(self):
        development = campaign_fixture()
        validation = campaign_fixture()
        validation["domain"] = "Validation"
        validation["graph"]["name"] = "validation-graph"

        with EngineStore(self.root / "catalogue.sqlite") as store:
            self.assertIsInstance(store.create(development, "http://127.0.0.1:8477"), str)
            self.assertIsInstance(store.create(validation, "http://127.0.0.1:8477"), str)

            for mutate in (
                lambda spec: spec.__setitem__("domain", "Final"),
                lambda spec: spec.__setitem__("domain", "Protected"),
                lambda spec: spec.__setitem__("purpose", "confirmatory"),
                lambda spec: spec.__setitem__("schema", 2),
                lambda spec: spec.__setitem__("schema", True),
                lambda spec: spec.__setitem__("schema", 1.0),
            ):
                candidate = campaign_fixture()
                mutate(candidate)
                with self.assertRaisesRegex(durable.ContractError, "non-final|Development or Validation"):
                    store.create(candidate, "http://127.0.0.1:8477")

            injected = campaign_fixture()
            injected["domain"] = "Final"
            with store.db:
                store.db.execute(
                    "UPDATE campaigns SET spec=? WHERE id=(SELECT id FROM campaigns ORDER BY created_ns LIMIT 1)",
                    (durable.canonical(injected),),
                )
            with self.assertRaisesRegex(durable.ContractError, "Development or Validation"):
                store.access_audit()
            with self.assertRaisesRegex(durable.ContractError, "Development or Validation"):
                store.backup(self.root / "must-not-backup-final.sqlite")
            self.assertFalse((self.root / "must-not-backup-final.sqlite").exists())

    def test_search_stage_rejects_final_campaign_before_transaction(self):
        with EngineStore(self.root / "catalogue.sqlite") as store:
            search = store.create_search({"kind": "boundary-test"}, {"binding": "test"}, {})
            running = store.update_search(search["id"], 0, "running", {}, {"started": True})
            self.assertEqual("running", running["phase"])

            final = campaign_fixture()
            final["domain"] = "Final"
            with self.assertRaisesRegex(durable.ContractError, "Development or Validation"):
                store.bind_search_stage(
                    search["id"], "attempt-1", final, {}, "http://127.0.0.1:8477"
                )

            self.assertEqual(0, store.db.execute("SELECT COUNT(*) FROM campaigns").fetchone()[0])
            self.assertEqual(0, store.db.execute("SELECT COUNT(*) FROM search_stages").fetchone()[0])
            self.assertEqual("running", store.get_search(search["id"])["phase"])

    def test_restore_rejects_final_campaign_before_any_catalogue_write(self):
        record = {
            "id": "0" * 64,
            "spec": dict(campaign_fixture(), domain="Final"),
            "workflow": None,
            "admission": None,
            "snapshot": None,
            "created_ns": 1,
            "endpoint": "https://source.example",
            "phase": "completed",
        }
        with EngineStore(self.root / "catalogue.sqlite") as store:
            with self.assertRaisesRegex(durable.ContractError, "Development or Validation"):
                store.restore(record, [], {}, "https://hub.example")
            self.assertEqual([], store.list())

    def test_access_audit_preserves_existing_search_parser_budget(self):
        large_response = {"values": [0] * 100_001}
        with EngineStore(self.root / "catalogue.sqlite") as store:
            store.create_search({"kind": "large-search"}, {"binding": "test"}, large_response)
            audit = store.access_audit()
            self.assertEqual("clear", audit["status"])
            self.assertGreater(audit["structured_values_checked"], 0)

    def test_restricted_bundle_is_rejected_before_first_upload(self):
        class NoUploadClient:
            endpoint = "https://hub.example"

            def upload(self, *_args, **_kwargs):
                raise AssertionError("restricted bundle must be rejected before upload")

        class NoRestoreStore:
            def restore(self, *_args, **_kwargs):
                raise AssertionError("restricted bundle must be rejected before catalogue restore")

        bundle = {
            "campaign": {
                "admission": {
                    "root_artifact_bindings": [
                        {"descriptor": {"access_class": "restricted-reference"}}
                    ]
                }
            },
            "members": [{"step": "step", "output": "out", "evidence": {
                "artifact_identity": "a" * 64,
                "descriptor": {"access_class": "development"},
            }}],
        }
        with patch.object(archive, "verify_bundle", return_value={("step", "out"): b"bytes"}):
            with self.assertRaisesRegex(durable.ContractError, "restricted-reference"):
                archive.restore_bundle(NoUploadClient(), NoRestoreStore(), bundle)

    def test_restricted_metadata_cannot_persist_in_events_cache_exports_or_candidates(self):
        tagged = {"nested": {"access_class": "restricted-reference", "opaque": "reference-only"}}
        with EngineStore(self.root / "catalogue.sqlite") as store:
            campaign = store.create(campaign_fixture(), "http://127.0.0.1:8477")
            for label, operation in (
                ("event", lambda: store.event(campaign, "diagnostic", tagged)),
                ("result", lambda: store.put_result(campaign, "prepare", "result", tagged)),
                ("cache", lambda: store.cache_put(tagged, {}, campaign, "prepare", "result", authorized=True)),
                ("export", lambda: store.prepare_export(campaign, "otlp", "https://collector.example", tagged)),
                ("candidate", lambda: store.create_search(tagged, {}, {})),
            ):
                with self.subTest(boundary=label), self.assertRaisesRegex(durable.ContractError, "restricted-reference"):
                    operation()
            clean = store.access_audit()
            self.assertEqual("clear", clean["status"])
            self.assertEqual(0, clean["restricted_reference_records"])
            with store.db:
                store.db.execute(
                    "INSERT INTO events(campaign,kind,payload,recorded_ns) VALUES (?,?,?,?)",
                    (campaign, "legacy-injected", durable.canonical(tagged), 1),
                )
            with self.assertRaisesRegex(durable.ContractError, "events.payload.*restricted-reference"):
                store.access_audit()
            with self.assertRaisesRegex(durable.ContractError, "events.payload.*restricted-reference"):
                store.backup(self.root / "must-not-exist.sqlite")
            self.assertFalse((self.root / "must-not-exist.sqlite").exists())

    def test_restricted_roots_rejected_before_download_or_submission(self):
        spec = campaign_fixture()
        roots = root_artifact_bindings()
        import tdi_hub_edge_contract as edge
        original = next(iter(roots.values()))
        descriptor = dict(original["descriptor"], access_class="restricted-reference")
        restricted = edge.bind_portable_artifact(descriptor, {"id": original["hub_artifact_id"],
            "hub_digest": original["hub_digest"], "raw_sha256": descriptor["raw_sha256"], "size": descriptor["size_bytes"]})
        class Client:
            endpoint = "http://127.0.0.1:8477"
            def download(self, *_):
                raise AssertionError("restricted bytes must not be read")
        with EngineStore(self.root / "catalogue.sqlite") as store:
            with self.assertRaisesRegex(durable.ContractError, "access"):
                runtime.submit(Client(), store, spec, {descriptor["raw_sha256"]: restricted})
            self.assertEqual([], store.list())

    def test_running_attach_cannot_return_a_dispatchable_admitted_state(self):
        spec = campaign_fixture()
        spec["graph"]["steps"][0]["inputs"] = {}
        response = workflow_response(spec["graph"])
        response["state"] = "running"
        response["steps"] = []
        class Client:
            endpoint = "http://127.0.0.1:8477"
            def request(self, method, *_):
                if method != "GET":
                    raise AssertionError("running workflow must not be dispatched again")
                return response
        with EngineStore(self.root / "catalogue.sqlite") as store:
            campaign = store.create(runtime.canonical_campaign(spec), Client.endpoint)
            store.transition(campaign, "prepared", "submitting")
            self.assertEqual("executing", runtime.attach(Client(), store, campaign, response["id"], {})["phase"])
            self.assertEqual("executing", runtime.execute(Client(), store, campaign)["phase"])

    def test_complete_workflow_requires_unique_complete_successful_steps(self):
        spec = campaign_fixture()
        for steps in ([], [{"key": "prepare", "state": "succeeded"}],
                      [{"key": "prepare", "state": "succeeded"}] * 2,
                      [{"key": [], "state": "succeeded"}]):
            with self.subTest(steps=steps), self.assertRaises(durable.ContractError):
                runtime.validated_snapshot(spec, {"state": "succeeded", "steps": steps})

    def test_viewer_escapes_producer_strings_and_displays_incomplete_state(self):
        spec = campaign_fixture()
        spec["graph"]["steps"][0]["parameters"]["unsafe"] = '<script>alert("x")</script>'
        with EngineStore(self.root / "catalogue.sqlite") as store:
            campaign = store.create(spec, "http://127.0.0.1:8477")
            page = render_catalogue(store, campaign)
        self.assertNotIn(b"<script>", page)
        self.assertIn(b"&lt;script&gt;", page)
        self.assertIn(b"No verified outputs", page)


class TransportTests(unittest.TestCase):
    def test_origin_and_credential_policy(self):
        for endpoint, kwargs in (
            ("http://example.com", {"allow_loopback_http": True}),
            ("http://localhost", {"allow_loopback_http": True}),
            ("http://127.0.0.1", {}), ("https://example.com", {}),
            ("https://user:pass@example.com", {"token": "test"}),
            ("https://example.com/path", {"token": "test"}),
            ("https://127.0.0.1", {"token": "line\nbreak"}),
            ("https://127.0.0.1", {"timeout": float("nan")}),
        ):
            with self.subTest(endpoint=endpoint), self.assertRaises(HubClientError):
                HubClient(endpoint, **kwargs)

    def test_redirect_overflow_duplicates_truncation_and_lost_mutation(self):
        seen = []
        class Handler(BaseHTTPRequestHandler):
            def log_message(self, *_):
                pass
            def do_GET(self):
                seen.append(self.path)
                if self.path.endswith("redirect"):
                    self.send_response(302)
                    self.send_header("Location", "/api/v1/secret-target")
                    self.end_headers()
                    return
                bodies = {"duplicate": b'{"a":1,"a":2}', "overflow": b'{"a":1e999}',
                          "truncated": b"{}", "huge": b"x" * 300, "array": b"[]"}
                body = bodies.get(self.path.rsplit("/", 1)[-1], b"{}")
                self.send_response(200)
                self.send_header("Content-Length", "100" if self.path.endswith("truncated") else str(len(body)))
                self.end_headers()
                self.wfile.write(body)
            do_POST = do_GET
        with HTTPServer(("127.0.0.1", 0), Handler) as server:
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            client = HubClient(f"http://127.0.0.1:{server.server_port}", token="private-test-token", allow_loopback_http=True, max_bytes=128)
            try:
                for path in ("redirect", "duplicate", "overflow", "truncated", "huge", "array"):
                    with self.subTest(path=path), self.assertRaises(HubClientError) as failure:
                        client.request("GET", "/api/v1/" + path)
                    self.assertNotIn("private-test-token", str(failure.exception))
                with self.assertRaises(HubTransportUnknown):
                    client.request("POST", "/api/v1/duplicate", value={})
                self.assertEqual(2, seen.count("/api/v1/duplicate"))
                self.assertNotIn("/api/v1/secret-target", seen)
            finally:
                server.shutdown()
                thread.join(timeout=5)


if __name__ == "__main__":
    unittest.main()
