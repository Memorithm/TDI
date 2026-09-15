import contextlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock

SCRIPTS = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPTS))
import tdi_experiment_supervisor as durable
import tdi_linux_contract as contract
import tdi_linux_runner as runner
from tdi_linux_containment import ContainmentError


class FakeProcess:
    def __init__(self):
        self.wait_calls = 0
        self.killed = False
    def wait(self, timeout=None):
        self.wait_calls += 1
        return 0
    def kill(self):
        self.killed = True


class FakeReaper:
    def __init__(self):
        read_fd, self.control_fd = os.pipe()
        os.close(read_fd)
        self.process = FakeProcess()
        self.released = False
    def release(self):
        with contextlib.suppress(OSError):
            os.close(self.control_fd)
        self.control_fd = -1
        self.released = True


class FakeGroup:
    instances = []
    def __init__(self, parent, attempt_id, profile):
        self.parent, self.attempt_id, self.profile = Path(parent), attempt_id, profile
        self.path = self.parent / f"tdi-{attempt_id}"
        self.killed = self.cleaned = False
        self.path.mkdir(parents=True, exist_ok=False)
        (self.path / "cgroup.procs").write_text("")
        FakeGroup.instances.append(self)
    def create(self): return self
    def is_populated(self): return not self.cleaned
    def kill(self): self.killed = True
    def metrics(self):
        return {"backend": "linux-cgroup-v2", "attempt_cgroup": self.path.name,
                "effective_limits": {"memory_max_bytes": self.profile.memory_max_bytes,
                "swap_max_bytes": self.profile.swap_max_bytes,
                "cpu_max": f"{self.profile.cpu_quota_us} {self.profile.cpu_period_us}",
                "pids_max": self.profile.pids_max}}
    def cleanup(self):
        self.cleaned = True
        for child in self.path.iterdir(): child.unlink()
        self.path.rmdir()
    @classmethod
    def reconcile_existing(cls, parent, attempt_id):
        path = Path(parent) / f"tdi-{attempt_id}"; present = path.exists()
        if present:
            for child in path.iterdir(): child.unlink()
            path.rmdir()
        return {"present": present, "killed": present}


class FailCleanupGroup(FakeGroup):
    def cleanup(self):
        raise ContainmentError("synthetic populated cgroup")


class RunnerTests(unittest.TestCase):
    def setUp(self):
        FakeGroup.instances.clear()
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name); self.cgroup = self.root / "cgroup"; self.cgroup.mkdir()
        worker = self.root / "worker"; worker.write_text("#!/bin/sh\nexit 0\n"); worker.chmod(0o755)
        self.plan = {"schema": 2, "purpose": "development-software", "domain": "Development",
            "indices": [0, 1], "argv": ["worker"], "artifacts": {"worker": durable.file_digest(worker)},
            "timeout_seconds": 2, "max_output_bytes": 4096, "max_trials": 2,
            "execution": {"backend": "linux-cgroup-v2", "profile": {"schema": 1,
                "memory_max_bytes": 64*1024*1024, "swap_max_bytes": 0,
                "cpu_quota_us": 50_000, "cpu_period_us": 100_000, "pids_max": 32,
                "trust": "trusted", "gpu_required": False, "gpu_memory_max_bytes": None}}}

    def test_campaign_binds_attempt_and_resource_evidence(self):
        plan_id = contract.validate_plan(self.plan, self.root); reapers=[]
        def fake_supervise(argv, *_):
            seed=int(argv[-3]); return {"status":"Completed","returncode":0,
                "stdout":json.dumps({"seed":seed,"plan_id":argv[-1],"status":"Evaluated"}),"stderr":""}
        def fake_reaper(*_): h=FakeReaper(); reapers.append(h); return h
        with mock.patch.object(runner,"CgroupV2Attempt",FakeGroup), \
             mock.patch.object(runner,"require_inside_delegation",lambda *_: self.cgroup), \
             mock.patch.object(runner,"spawn_reaper",fake_reaper), \
             mock.patch.object(runner.durable,"supervise",fake_supervise):
            records=runner.run(self.plan,self.root,self.root/"campaign.db",self.cgroup,
                               recovery_dir=self.root/"recovery")
        self.assertEqual(len(records),2)
        for index,record in enumerate(records):
            result=record["result"]; self.assertEqual(result["status"],"Completed")
            self.assertEqual(result["response"]["plan_id"],plan_id)
            self.assertEqual(result["attempt_id"],contract.attempt_identity(plan_id,index))
            self.assertEqual(result["resource_evidence"]["backend"],"linux-cgroup-v2")
        self.assertTrue(all(g.killed and g.cleaned for g in FakeGroup.instances))
        self.assertTrue(all(h.released for h in reapers))

    def test_unfinished_attempt_is_reconciled_not_reexecuted(self):
        path=self.root/"partial.db"; journal=durable.Journal(path,contract.journal_binding(self.plan))
        journal.append({"kind":"Start","index":0}); journal.close()
        attempt=contract.attempt_identity(contract.validate_plan(self.plan,self.root),0)
        stale=self.cgroup/f"tdi-{attempt}"; stale.mkdir(); (stale/"cgroup.procs").write_text("")
        with mock.patch.object(runner,"CgroupV2Attempt",FakeGroup), \
             mock.patch.object(runner,"require_inside_delegation",lambda *_: self.cgroup), \
             mock.patch.object(runner,"spawn_reaper",lambda *_:FakeReaper()), \
             mock.patch.object(runner.durable,"supervise") as supervise:
            supervise.return_value={"status":"WorkerFailed","returncode":3,"stdout":"","stderr":"boom"}
            records=runner.run(self.plan,self.root,path,self.cgroup,recovery_dir=self.root/"recovery")
        self.assertEqual(records[0]["result"]["status"],"Interrupted")
        self.assertEqual(supervise.call_count,1)

    def test_orphaned_pre_start_cgroup_is_consumed_not_retried(self):
        plan_id=contract.validate_plan(self.plan,self.root)
        attempt=contract.attempt_identity(plan_id,0)
        stale=self.cgroup/f"tdi-{attempt}"; stale.mkdir(); (stale/"cgroup.procs").write_text("")
        with mock.patch.object(runner,"CgroupV2Attempt",FakeGroup), \
             mock.patch.object(runner,"require_inside_delegation",lambda *_: self.cgroup), \
             mock.patch.object(runner,"spawn_reaper",lambda *_:FakeReaper()), \
             mock.patch.object(runner.durable,"supervise") as supervise:
            supervise.return_value={"status":"WorkerFailed","returncode":3,"stdout":"","stderr":"boom"}
            records=runner.run(self.plan,self.root,self.root/"orphan.db",self.cgroup,
                               recovery_dir=self.root/"recovery")
        self.assertEqual(records[0]["result"]["status"],"Interrupted")
        self.assertTrue(records[0]["result"]["recovery"]["present"])
        self.assertEqual(supervise.call_count,1)

    def test_cleanup_failure_hands_off_to_reaper_and_stops_campaign(self):
        plan_id=contract.validate_plan(self.plan,self.root); reapers=[]
        def fake_supervise(argv, *_):
            seed=int(argv[-3]); return {"status":"Completed","returncode":0,
                "stdout":json.dumps({"seed":seed,"plan_id":argv[-1],"status":"Evaluated"}),"stderr":""}
        def fake_reaper(*_): h=FakeReaper(); reapers.append(h); return h
        journal_path=self.root/"cleanup-failure.db"
        with mock.patch.object(runner,"CgroupV2Attempt",FailCleanupGroup), \
             mock.patch.object(runner,"require_inside_delegation",lambda *_: self.cgroup), \
             mock.patch.object(runner,"spawn_reaper",fake_reaper), \
             mock.patch.object(runner.durable,"supervise",fake_supervise):
            with self.assertRaises(ContainmentError):
                runner.run(self.plan,self.root,journal_path,self.cgroup,
                           recovery_dir=self.root/"recovery")
        journal=durable.Journal(journal_path,contract.journal_binding(self.plan))
        try:
            records,active,_=journal.read()
        finally:
            journal.close()
        self.assertIsNone(active)
        self.assertEqual(len(records),1)
        self.assertEqual(records[0]["result"]["status"],"ContainmentFailed")
        self.assertEqual(records[0]["result"]["prior_status"],"Completed")
        self.assertFalse(reapers[0].released)
        self.assertEqual(reapers[0].process.wait_calls,1)

    def test_launcher_attaches_exact_pid_before_exec(self):
        procs=self.root/"cgroup.procs"; procs.write_text("")
        worker=self.root/"probe.py"; worker.write_text(f"#!{sys.executable}\nimport json,os\nprint(json.dumps({{'pid':os.getpid()}}))\n"); worker.chmod(0o755)
        completed=subprocess.run([sys.executable,str(SCRIPTS/"tdi_cgroup_exec.py"),"--cgroup-procs",str(procs),"--",str(worker)],text=True,capture_output=True,check=False)
        self.assertEqual(completed.returncode,0,completed.stderr)
        self.assertEqual(int(procs.read_text()),json.loads(completed.stdout)["pid"])


if __name__ == "__main__": unittest.main()
