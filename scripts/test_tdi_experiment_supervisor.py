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
        self.worker.write_text(f"#!{sys.executable}\n" + '''import json,sys
args=dict(zip(sys.argv[1::2],sys.argv[2::2]))
seed=int(args['--tdi-seed'])
print(json.dumps({'seed':seed,'plan_id':args['--tdi-plan-id'],'status':'Evaluated','value':seed*2}))
''')
        self.worker.chmod(0o755)
        self.plan = {"schema": 1, "purpose": "development-software", "domain": "Development",
                     "indices": [0, 1, 2], "argv": ["worker"],
                     "artifacts": {"worker": supervisor.file_digest(self.worker)},
                     "timeout_seconds": 2, "max_output_bytes": 4096, "max_trials": 3}

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
        with self.assertRaises(sqlite3.DatabaseError):
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
        for code, limit, cancel, expected in [
            ("import time;time.sleep(30)", 1000, lambda: False, "Timeout"),
            ("print('x'*10000)", 100, lambda: False, "OutputLimit"),
            ("import time;time.sleep(30)", 1000, lambda: True, "Cancelled"),
        ]:
            started = time.monotonic()
            result = supervisor.supervise([sys.executable, "-c", code], 0.3, limit, cancel)
            self.assertEqual(result["status"], expected)
            self.assertLess(time.monotonic() - started, 5)
            self.assertLessEqual(len(result["stdout"]) + len(result["stderr"]), limit)

    def test_worker_failure_and_bad_binding_keep_diagnostics(self):
        for body, expected in [("import sys;print('diagnostic',file=sys.stderr);sys.exit(3)", "WorkerFailed"),
                               ("print('{}')", "ContractRejected")]:
            self.worker.write_text(f"#!{sys.executable}\n{body}\n")
            self.plan["artifacts"]["worker"] = supervisor.file_digest(self.worker)
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
        with self.assertRaises(supervisor.ContractError):
            supervisor.strict_json('{"a":1,"a":2}')
        with self.assertRaises(supervisor.ContractError):
            supervisor.strict_json('{"a":NaN}')

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
