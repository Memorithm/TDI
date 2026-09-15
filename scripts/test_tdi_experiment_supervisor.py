import copy
import importlib.util
import json
import os
from pathlib import Path
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest

MODULE = Path(__file__).with_name("tdi_experiment_supervisor.py")
spec = importlib.util.spec_from_file_location("supervisor", MODULE)
supervisor = importlib.util.module_from_spec(spec)
spec.loader.exec_module(supervisor)


class DurableTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.worker = self.root / "worker"
        self._write_worker('''import json,sys
args=dict(zip(sys.argv[1::2],sys.argv[2::2]))
seed=int(args['--tdi-seed'])
print(json.dumps({'seed':seed,'plan_id':args['--tdi-plan-id'],'status':'Evaluated','value':seed*2}))
''')
        self.plan = {"schema": 1, "purpose": "development-software", "domain": "Development",
                     "indices": [0, 1, 2], "argv": ["worker"],
                     "artifacts": {"worker": supervisor.file_digest(self.worker)},
                     "timeout_seconds": 2, "max_output_bytes": 4096, "max_trials": 3}

    def _write_worker(self, body):
        self.worker.write_text(f"#!{sys.executable}\n" + body)
        self.worker.chmod(0o755)

    def _sync_worker_digest(self):
        self.plan["artifacts"]["worker"] = supervisor.file_digest(self.worker)

    def _write_plan(self, name="plan.json"):
        path = self.root / name
        path.write_text(supervisor.canonical(self.plan) + "\n")
        return path

    def test_reopen_matches_continuous_without_repeating_completed_trials(self):
        full = supervisor.run(self.plan, self.root, self.root / "full.db")
        def crash(_):
            raise RuntimeError("simulated supervisor crash after durable commit")
        with self.assertRaises(RuntimeError):
            supervisor.run(self.plan, self.root, self.root / "partial.db", after_commit=crash)
        resumed = supervisor.run(self.plan, self.root, self.root / "partial.db")
        self.assertEqual(full, resumed)
        self.assertEqual(resumed, supervisor.run(self.plan, self.root, self.root / "partial.db"))
        with sqlite3.connect(self.root / "partial.db") as db:
            self.assertEqual(db.execute("SELECT count(*) FROM events").fetchone()[0], 6)

    def test_unfinished_trial_is_retained_as_interrupted_not_rerun(self):
        path = self.root / "trial.db"
        journal = supervisor.Journal(path, self.plan)
        journal.append({"kind": "Start", "index": 0})
        journal.close()
        records = supervisor.run(self.plan, self.root, path)
        self.assertEqual(records[0]["result"]["status"], "Interrupted")
        self.assertEqual(len(records), 3)
        self.assertEqual(supervisor.exit_code_for_records(records), supervisor.EXIT_TRIAL_FAILURE)

    def test_real_process_crash_releases_lock_and_preserves_start(self):
        path = self.root / "crashed.db"
        script = f'''import sys,os
sys.path.insert(0,{str(MODULE.parent)!r})
import tdi_experiment_supervisor as s
j=s.Journal({str(path)!r},{self.plan!r})
j.append({{"kind":"Start","index":0}})
os._exit(19)
'''
        self.assertEqual(subprocess.run([sys.executable, "-c", script], check=False).returncode, 19)
        self.assertEqual(supervisor.run(self.plan, self.root, path)[0]["result"]["status"], "Interrupted")

    def test_artifact_and_plan_drift_fail_before_new_work(self):
        path = self.root / "trial.db"
        supervisor.run(self.plan, self.root, path)
        changed = copy.deepcopy(self.plan)
        changed["timeout_seconds"] = 3
        with self.assertRaises(supervisor.ContractError):
            supervisor.run(changed, self.root, path)
        self.worker.write_text(self.worker.read_text() + "# changed\n")
        with self.assertRaises(supervisor.ContractError):
            supervisor.run(self.plan, self.root, path)

    def test_corruption_and_truncation_fail_closed(self):
        path = self.root / "bad.db"
        supervisor.run(self.plan, self.root, path)
        with sqlite3.connect(path) as db:
            db.execute("UPDATE events SET payload='{}' WHERE seq=1")
        with self.assertRaises(supervisor.ContractError):
            supervisor.run(self.plan, self.root, path)
        path = self.root / "truncated.db"
        path.write_bytes(b"SQLite format 3\0broken")
        with self.assertRaises((supervisor.StorageError, sqlite3.DatabaseError)):
            supervisor.run(self.plan, self.root, path)

    def test_concurrent_writer_is_rejected(self):
        path = self.root / "trial.db"
        journal = supervisor.Journal(path, self.plan)
        try:
            with self.assertRaises(BlockingIOError):
                supervisor.Journal(path, self.plan)
        finally:
            journal.close()

    def test_timeout_output_limit_and_cancellation(self):
        for code, timeout, limit, cancel, expected in [
            ("import time;time.sleep(30)", 0.3, 1000, lambda: False, "Timeout"),
            ("print('x'*10000)", 2.0, 100, lambda: False, "OutputLimit"),
            ("import time;time.sleep(30)", 0.3, 1000, lambda: True, "Cancelled"),
        ]:
            started = time.monotonic()
            result = supervisor.supervise([sys.executable, "-c", code], timeout, limit, cancel)
            self.assertEqual(result["status"], expected)
            self.assertLess(time.monotonic() - started, 5)
            stdout = result["stdout"] or ""
            stderr = result["stderr"] or ""
            self.assertLessEqual(len(stdout.encode()) + len(stderr.encode()), limit)

    def test_worker_failure_and_bad_binding_keep_diagnostics(self):
        for body, expected in [("import sys;print('diagnostic',file=sys.stderr);sys.exit(3)", "WorkerFailed"),
                               ("print('{}')", "ContractRejected")]:
            self._write_worker(body + "\n")
            self._sync_worker_digest()
            result = supervisor.run(self.plan, self.root, self.root / (expected + ".db"))[0]["result"]
            self.assertEqual(result["status"], expected)
            self.assertIn("stdout", result)
            self.assertIn("stderr", result)

    def test_plan_validation_and_domains(self):
        for key, value in [("domain", "Final"), ("indices", [1, 1]), ("timeout_seconds", float('nan')),
                           ("max_trials", 1), ("max_output_bytes", 0), ("argv", ["../worker"])]:
            plan = copy.deepcopy(self.plan)
            plan[key] = value
            with self.assertRaises(supervisor.ContractError):
                supervisor.validate(plan, self.root)
        self.plan["domain"] = "Validation"
        records = supervisor.run(self.plan, self.root, self.root / "validation.db")
        self.assertEqual(records[0]["result"]["response"]["seed"], 2**63)
        for raw in ['{"a":1,"a":2}', '{"a":NaN}', '{"a":Infinity}', '{"a":1e999}']:
            with self.assertRaises(supervisor.ContractError):
                supervisor.strict_json(raw)

    def test_json_limits_and_invalid_utf8_are_explicit(self):
        with self.assertRaises(supervisor.ContractError):
            supervisor.strict_json(b'"\xff"')
        nested = "0"
        for _ in range(supervisor.MAX_JSON_DEPTH + 2):
            nested = "[" + nested + "]"
        with self.assertRaises(supervisor.ContractError):
            supervisor.strict_json(nested)
        with self.assertRaises(supervisor.ContractError):
            supervisor.strict_json('"' + ('x' * 128) + '"', max_string_bytes=64)
        with self.assertRaises(supervisor.ContractError):
            supervisor.strict_json('[0,1,2,3]', max_items=3)

    def test_invalid_utf8_worker_stdout_is_preserved_and_rejected(self):
        self._write_worker("import os;os.write(1,b'\\xff')\n")
        self._sync_worker_digest()
        result = supervisor.run(self.plan, self.root, self.root / "utf8.db")[0]["result"]
        self.assertEqual(result["status"], "ContractRejected")
        self.assertEqual(result["stdout_encoding"], "base64")
        self.assertIsNone(result["stdout"])
        self.assertEqual(result["stdout_base64"], "/w==")

    def test_append_hashing_is_incremental_not_quadratic(self):
        plan = copy.deepcopy(self.plan)
        plan["indices"] = [0, 1]
        plan["max_trials"] = 2
        journal = supervisor.Journal(self.root / "linear.db", plan)
        calls = 0
        original = supervisor.digest
        def counted(data):
            nonlocal calls
            calls += 1
            return original(data)
        supervisor.digest = counted
        try:
            journal.append({"kind": "Start", "index": 0})
            journal.append({"kind": "Finish", "index": 0, "result": {"status": "Completed"}})
            journal.append({"kind": "Start", "index": 1})
            journal.append({"kind": "Finish", "index": 1, "result": {"status": "Completed"}})
            self.assertEqual(calls, 4)
        finally:
            supervisor.digest = original
            journal.close()
        reopened = supervisor.Journal(self.root / "linear.db", plan)
        try:
            self.assertEqual(len(reopened.audit()[0]), 2)
        finally:
            reopened.close()

    def test_impossible_transitions_are_rejected_before_storage(self):
        journal = supervisor.Journal(self.root / "state.db", self.plan)
        try:
            with self.assertRaises(supervisor.ContractError):
                journal.append({"kind": "Start", "index": 1})
            journal.append({"kind": "Start", "index": 0})
            journal.append({"kind": "Finish", "index": 0, "result": {"status": "Completed"}})
            with self.assertRaises(supervisor.ContractError):
                journal.append({"kind": "Finish", "index": 0, "result": {"status": "Completed"}})
        finally:
            journal.close()

    def test_external_anchor_detects_clean_suffix_deletion(self):
        path = self.root / "anchored.db"
        anchor = self.root / "trusted" / "journal.anchor"
        supervisor.run(self.plan, self.root, path, anchor_path=anchor)
        with sqlite3.connect(path) as db:
            db.execute("DELETE FROM events WHERE seq >= 4")
            db.execute("DELETE FROM trial_state WHERE trial_index = 2")
        with self.assertRaises(supervisor.ContractError):
            supervisor.Journal(path, self.plan, anchor_path=anchor)

    def test_legacy_journal_migrates_without_rewriting_event_chain(self):
        path = self.root / "legacy.db"
        previous = supervisor.digest(supervisor.canonical(self.plan).encode())
        events = [
            {"kind": "Start", "index": 0},
            {"kind": "Finish", "index": 0, "result": {"status": "Completed"}},
        ]
        with sqlite3.connect(path) as db:
            db.execute("CREATE TABLE meta (plan TEXT NOT NULL)")
            db.execute("INSERT INTO meta VALUES (?)", (supervisor.canonical(self.plan),))
            db.execute("CREATE TABLE events (seq INTEGER PRIMARY KEY, payload TEXT NOT NULL, hash TEXT NOT NULL)")
            for seq, event in enumerate(events):
                raw = supervisor.canonical(event)
                recorded = supervisor.digest((previous + raw).encode())
                db.execute("INSERT INTO events VALUES (?,?,?)", (seq, raw, recorded))
                previous = recorded
        journal = supervisor.Journal(path, self.plan)
        try:
            self.assertEqual(journal.read()[0], [events[1]])
            self.assertEqual(journal.db.execute("SELECT version FROM journal_schema").fetchone()[0],
                             supervisor.JOURNAL_FORMAT_VERSION)
            self.assertEqual(journal.db.execute("SELECT count(*) FROM events").fetchone()[0], 2)
            journal.audit()
        finally:
            journal.close()

    def test_cli_worker_failure_returns_nonzero(self):
        self._write_worker("import sys;print('diagnostic',file=sys.stderr);sys.exit(3)\n")
        self._sync_worker_digest()
        plan = self._write_plan()
        completed = subprocess.run(
            [sys.executable, str(MODULE), "--root", str(self.root), "--plan", str(plan),
             "--journal", str(self.root / "cli-failed.db")],
            text=True, capture_output=True, check=False)
        self.assertEqual(completed.returncode, supervisor.EXIT_TRIAL_FAILURE)
        payload = json.loads(completed.stdout)
        self.assertEqual(payload["status"], "trial-failure")
        self.assertEqual(payload["records"][0]["result"]["status"], "WorkerFailed")
        self.assertIn("technical trial failure", completed.stderr)

    def test_cli_overflow_is_contract_error_and_audit_is_structured(self):
        overflow = self.root / "overflow.json"
        overflow.write_text('{"schema":1e999}')
        rejected = subprocess.run(
            [sys.executable, str(MODULE), "--root", str(self.root), "--plan", str(overflow),
             "--journal", str(self.root / "overflow.db")],
            text=True, capture_output=True, check=False)
        self.assertEqual(rejected.returncode, supervisor.EXIT_CONTRACT)
        self.assertEqual(json.loads(rejected.stdout)["status"], "contract-error")

        journal_path = self.root / "audit.db"
        supervisor.run(self.plan, self.root, journal_path)
        plan = self._write_plan("audit-plan.json")
        audited = subprocess.run(
            [sys.executable, str(MODULE), "--root", str(self.root), "--plan", str(plan),
             "--journal", str(journal_path), "--audit-only"],
            text=True, capture_output=True, check=False)
        self.assertEqual(audited.returncode, 0, audited.stderr)
        self.assertEqual(json.loads(audited.stdout)["operation"], "audit")

    def test_scientific_rejection_does_not_become_technical_failure(self):
        self._write_worker('''import json,sys
args=dict(zip(sys.argv[1::2],sys.argv[2::2]))
seed=int(args['--tdi-seed'])
print(json.dumps({'seed':seed,'plan_id':args['--tdi-plan-id'],'status':'Rejected'}))
''')
        self._sync_worker_digest()
        plan = self._write_plan()
        completed = subprocess.run(
            [sys.executable, str(MODULE), "--root", str(self.root), "--plan", str(plan),
             "--journal", str(self.root / "cli-rejected.db")],
            text=True, capture_output=True, check=False)
        self.assertEqual(completed.returncode, supervisor.EXIT_OK, completed.stderr)
        self.assertEqual(json.loads(completed.stdout)["status"], "ok")

    @unittest.skipUnless(os.environ.get("TDI_DURABLE_WORKER"), "Rust worker built by CI")
    def test_real_rust_worker_resume(self):
        import shutil
        worker = Path(os.environ["TDI_DURABLE_WORKER"]).resolve()
        shutil.copy2(worker, self.worker)
        self.plan["artifacts"]["worker"] = supervisor.file_digest(self.worker)
        records = supervisor.run(self.plan, self.root, self.root / "rust.db")
        self.assertEqual(len(records), 3)
        for record in records:
            self.assertEqual(record["result"]["status"], "Completed")
            self.assertEqual(record["result"]["response"]["scores"], [2, 2, 2, 2])
        self.assertEqual(records, supervisor.run(self.plan, self.root, self.root / "rust.db"))


if __name__ == "__main__":
    unittest.main()
