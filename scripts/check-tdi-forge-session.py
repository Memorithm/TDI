#!/usr/bin/env python3
"""Actual Forge session/replay, interrupted-permit and bounded transport checks."""
import argparse
import copy
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import time
from unittest import mock

import tdi_experiment_supervisor as durable
import tdi_optuna_benchmark as bench
from tdi_forge_session import ForgeSession, PROTOCOL, recover_session_log
from tdi_physical_telemetry import measured_process

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--forge-worker", type=Path, required=True)
parser.add_argument("--historical-worker", type=Path)
args = parser.parse_args()
binary = args.forge_worker.resolve(strict=True)
client = bench.ForgeClient(binary, bench.file_hash(binary), "0" * 40)  # synthetic contract identity only


def rejected(action):
    try:
        action()
    except (durable.ContractError, ValueError, TimeoutError, OSError):
        return
    raise AssertionError("invalid operation was accepted")


with tempfile.TemporaryDirectory() as tmp:
    root = Path(tmp)
    for name in ("forge-tpe-session", "forge-gp-session", "forge-tpe-early-session"):
        directory = root / name
        directory.mkdir()
        arm = bench.SessionForgeArm(name, 7, 16, "shifted-bowl", client,
                "1" * 64, "2" * 64, "0" * 40, directory, time.monotonic() + 60)
        try:
            with (directory / "events.jsonl").open("x") as stream:
                bench.run_arm(arm, "shifted-bowl", 7, 16, directory, stream, time.monotonic() + 60)
        finally:
            arm.session.abort()
        recovered = recover_session_log(directory)
        assert recovered["checkpoint"] == arm.checkpoint
        replay, _ = client.call(arm.spec, arm.checkpoint)
        assert replay["snapshot"] == arm.snapshot
        if args.historical_worker and name != "forge-tpe-early-session":
            old_binary = args.historical_worker.resolve(strict=True)
            old = bench.ForgeClient(old_binary, bench.file_hash(old_binary), bench.ADAPTIVE_FORGE_COMMIT)
            historical, _ = old.call(arm.spec, arm.checkpoint)
            assert historical == replay
        entry = directory / "session-commands/0001.json"
        saved = entry.read_bytes()
        entry.unlink()
        rejected(lambda: recover_session_log(directory))
        entry.write_bytes(saved)
        value = json.loads(saved)
        value["previous"] = "f" * 64
        entry.write_text(json.dumps(value))
        rejected(lambda: recover_session_log(directory))
        entry.write_bytes(saved)
    spec = arm.spec
    with ForgeSession(binary, client.binary["sha256"], "0" * 40, spec) as session:
        proposal = session.submit({"request_id": "ask", "operation": {"op": "ask"}})["proposal"]
        begin = {"request_id": "begin", "operation": {"op": "begin",
                 "candidate_id": proposal["candidate_id"], "stage": "compile"}}
        permit = session.submit(begin)["permit"]
        persisted = copy.deepcopy(session.checkpoint)
        # Crash with an active issued permit; recovery must not allocate another.
        session.process.kill()
        session.process.wait(timeout=5)
    with ForgeSession(binary, client.binary["sha256"], "0" * 40, spec, persisted) as resumed:
        assert resumed.inspect()["snapshot"]["active_attempt"] == permit
        assert resumed.submit(begin) == {"status": "duplicate", "original_index": 1}
        before = resumed.inspect()["snapshot"]["charged_ms"]
        other = dict(begin, request_id="new-begin")
        assert resumed.submit(other)["status"] == "rejected"
        assert resumed.inspect()["snapshot"]["charged_ms"] == before
        resumed.close()
    changed = dict(spec, seed="99")
    rejected(lambda: ForgeSession(binary, client.binary["sha256"], "0" * 40, changed, persisted))

    open_frame = {"protocol": PROTOCOL, "action": {"op": "open", "spec": spec, "checkpoint": None}}
    encoded = (json.dumps(open_frame) + "\n").encode()
    for bad in (b'{"protocol":"x","protocol":"y"}\n',
                b'{"protocol":"forge-scientific-session/v1","action":{"op":"inspect","spec_sha256":"x","expected_sequence":0}}\n',
                b"x" * (4*1024*1024 + 1) + b"\n", encoded.rstrip(b"\n")):
        result = subprocess.run([str(binary), "--session"], input=bad, capture_output=True, timeout=10)
        assert result.returncode == 21 and not result.stdout

    # Fault-only fixtures test transport cleanup, not optimizer performance.
    for name, body in (("silent", "import time; time.sleep(10)"),
                       ("flood", "import sys; sys.stdout.write('x' * (9*1024*1024)); sys.stdout.flush()"),
                       ("wrong", "print('{\"protocol\":\"wrong\",\"result\":{}}', flush=True)")):
        worker = root / name
        worker.write_text(f"#!{sys.executable}\n{body}\n")
        worker.chmod(0o700)
        rejected(lambda: ForgeSession(worker, bench.file_hash(worker), "0"*40, spec, timeout=0.2))
    copied = root / "forge-copy"
    shutil.copy2(binary, copied)
    with ForgeSession(copied, bench.file_hash(copied), "0"*40, spec) as session:
        stat = copied.stat()
        os.utime(copied, ns=(stat.st_atime_ns, stat.st_mtime_ns + 1000000))
        rejected(session.inspect)
        assert session.closed and session.process.poll() is not None

for use_pidfd in (True, False):
    context = (mock.patch("tdi_physical_telemetry.os.pidfd_open", side_effect=OSError("unavailable"))
               if not use_pidfd else mock.patch("tdi_physical_telemetry.os.pidfd_open", wraps=os.pidfd_open))
    with context:
        code, out, _, costs = measured_process([sys.executable, "-c", "print('ok'); raise SystemExit(7)"])
        assert code == 7 and out == b"ok\n" and costs["peak_rss_bytes"] > 0
        assert costs["completion_wait_method"] == ("pidfd" if use_pidfd else "poll")
        _, _, _, costs = measured_process([sys.executable, "-c", "import time; time.sleep(10)"], timeout=0.02)
        assert costs["technical_failure"] == "timeout"
        _, out, _, costs = measured_process([sys.executable, "-c", "print('x'*100000)"], max_output=1024)
        assert costs["technical_failure"] == "output-budget" and len(out) == 1024
print("PASS: actual replay/session parity, log corruption/gaps, crash with active permit, duplicate/no-charge, protocol bounds, timeout/flood/identity cleanup, pidfd and polling telemetry")
