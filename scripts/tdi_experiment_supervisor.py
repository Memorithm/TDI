#!/usr/bin/env python3
"""Linux non-final experiment supervision; standard library only.

This is not a sandbox or authorization for a scientific stage. Workers and
artifact directories must be trusted and immutable during execution.
"""
import argparse
import contextlib
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import selectors
import signal
import sqlite3
import subprocess
import time


class ContractError(ValueError):
    pass


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def strict_json(raw):
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ContractError("duplicate JSON key")
            result[key] = value
        return result

    return json.loads(raw, object_pairs_hook=pairs,
                      parse_constant=lambda _: (_ for _ in ()).throw(ContractError("nonfinite JSON")))


def digest(data):
    return hashlib.sha256(data).hexdigest()


def artifact(root, relative):
    if not isinstance(relative, str) or not relative or Path(relative).is_absolute():
        raise ContractError("artifact must have a relative path")
    if any(part in (".", "..") for part in Path(relative).parts):
        raise ContractError("artifact traversal")
    path = root / relative
    for parent in [path, *path.parents]:
        if parent == root:
            break
        if parent.is_symlink():
            raise ContractError("symlink artifact")
    if not path.is_file() or not path.resolve().is_relative_to(root):
        raise ContractError("missing or external artifact")
    return path


def file_digest(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def validate(plan, root):
    required = {"schema", "purpose", "domain", "indices", "argv", "artifacts",
                "timeout_seconds", "max_output_bytes", "max_trials"}
    if not isinstance(plan, dict) or set(plan) != required:
        raise ContractError("unknown or missing plan fields")
    if plan["schema"] != 1 or plan["purpose"] != "development-software":
        raise ContractError("only versioned non-final software trials are supported")
    if plan["domain"] not in ("Development", "Validation"):
        raise ContractError("unsupported domain")
    indices = plan["indices"]
    if (not isinstance(indices, list) or not indices
            or any(type(i) is not int or not 0 <= i < 2**63 for i in indices)
            or len(set(indices)) != len(indices)):
        raise ContractError("invalid or duplicate trial index")
    if type(plan["max_trials"]) is not int or not 0 < len(indices) <= plan["max_trials"]:
        raise ContractError("trial bound")
    timeout = plan["timeout_seconds"]
    if type(timeout) not in (float, int) or not math.isfinite(timeout) or timeout <= 0:
        raise ContractError("invalid timeout")
    if type(plan["max_output_bytes"]) is not int or not 0 < plan["max_output_bytes"] <= 16_777_216:
        raise ContractError("output limit must be between 1 byte and 16 MiB")
    argv = plan["argv"]
    if not isinstance(argv, list) or not argv or any(not isinstance(x, str) or "\0" in x for x in argv):
        raise ContractError("argv must be strings, without shell interpretation")
    artifacts = plan["artifacts"]
    if not isinstance(artifacts, dict) or not artifacts or argv[0] not in artifacts:
        raise ContractError("executable must be an explicitly pinned artifact")
    for name, expected in artifacts.items():
        if not isinstance(expected, str) or len(expected) != 64 or any(c not in "0123456789abcdef" for c in expected):
            raise ContractError("expected SHA-256 must be lowercase hexadecimal")
        if file_digest(artifact(root, name)) != expected:
            raise ContractError("artifact digest mismatch: " + name)
    return digest(canonical(plan).encode())


def supervise(argv, timeout, limit, cancelled=lambda: False):
    """Bound both output streams together; always reap the direct child.

    Kill the process group on timeout/cancel/output limit and after completion.
    Descendants deliberately escaping the group require an external sandbox.
    No RAM/GPU bound is implied by the output cap or wall-clock deadline.
    """
    output = {"stdout": bytearray(), "stderr": bytearray()}
    reason = None
    process = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, start_new_session=True,
                               env={"LANG": "C", "LC_ALL": "C"})
    started = time.monotonic()
    try:
        with selectors.DefaultSelector() as selector:
            for name, stream in (("stdout", process.stdout), ("stderr", process.stderr)):
                os.set_blocking(stream.fileno(), False)
                selector.register(stream, selectors.EVENT_READ, name)
            while selector.get_map() or process.poll() is None:
                if cancelled():
                    reason = "Cancelled"
                    break
                if time.monotonic() - started >= timeout:
                    reason = "Timeout"
                    break
                for key, _ in selector.select(min(0.02, timeout)):
                    chunk = os.read(key.fd, 65536)
                    if not chunk:
                        selector.unregister(key.fileobj)
                        continue
                    remaining = limit - sum(len(x) for x in output.values())
                    output[key.data].extend(chunk[:remaining])
                    if len(chunk) > remaining:
                        reason = "OutputLimit"
                        break
                if reason:
                    break
    finally:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        process.wait()
        process.stdout.close()
        process.stderr.close()
    return {"status": reason or ("Completed" if process.returncode == 0 else "WorkerFailed"),
            "returncode": process.returncode,
            "stdout": output["stdout"].decode("utf-8", errors="replace"),
            "stderr": output["stderr"].decode("utf-8", errors="replace")}


class Journal:
    """Single writer, SQLite FULL synchronous transactions, append-only events.

    Hash chains detect accidental corruption, not an adversary rewriting the
    entire database. A deleted suffix cannot be detected without an external
    trusted checkpoint. Use a local filesystem with reliable fsync/locking.
    """
    def __init__(self, path, plan):
        path = Path(path)
        if path.is_symlink() or Path(str(path) + ".lock").is_symlink():
            raise ContractError("symlink journal")
        self.lock = open(str(path) + ".lock", "a+b")
        try:
            fcntl.flock(self.lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            existed = path.exists()
            self.db = sqlite3.connect(path)
            self.db.execute("PRAGMA synchronous=FULL")
            self.db.execute("PRAGMA journal_mode=DELETE")
            if not existed:
                with self.db:
                    self.db.execute("CREATE TABLE meta (plan TEXT NOT NULL)")
                    self.db.execute("INSERT INTO meta VALUES (?)", (canonical(plan),))
                    self.db.execute("CREATE TABLE events (seq INTEGER PRIMARY KEY, payload TEXT NOT NULL, hash TEXT NOT NULL)")
                directory = os.open(str(path.parent), os.O_RDONLY | os.O_DIRECTORY)
                try:
                    os.fsync(directory)
                finally:
                    os.close(directory)
            if self.db.execute("PRAGMA integrity_check").fetchone() != ("ok",):
                raise ContractError("journal integrity failure")
            if self.db.execute("SELECT plan FROM meta").fetchall() != [(canonical(plan),)]:
                raise ContractError("journal plan mismatch")
            self.plan = plan
            self.read()
        except BaseException:
            if hasattr(self, "db"):
                self.db.close()
            self.lock.close()
            raise

    def close(self):
        self.db.close()
        self.lock.close()

    def read(self):
        previous = digest(canonical(self.plan).encode())
        done, active = [], None
        for expected_seq, (seq, raw, recorded) in enumerate(self.db.execute("SELECT seq,payload,hash FROM events ORDER BY seq")):
            if seq != expected_seq or digest((previous + raw).encode()) != recorded:
                raise ContractError("journal sequence or hash mismatch")
            event = strict_json(raw)
            if event["kind"] == "Start":
                if active is not None or len(done) >= len(self.plan["indices"]) or event["index"] != self.plan["indices"][len(done)]:
                    raise ContractError("invalid start sequence")
                active = event["index"]
            elif event["kind"] == "Finish" and active is not None and event["index"] == active:
                done.append(event)
                active = None
            else:
                raise ContractError("invalid journal transition")
            previous = recorded
        return done, active, previous

    def append(self, event):
        _, _, previous = self.read()
        seq = self.db.execute("SELECT count(*) FROM events").fetchone()[0]
        raw = canonical(event)
        with self.db:
            self.db.execute("INSERT INTO events VALUES (?,?,?)", (seq, raw, digest((previous + raw).encode())))


def run(plan, root, journal_path, cancelled=lambda: False, after_commit=lambda _: None):
    root = Path(root).resolve(strict=True)
    plan_id = validate(plan, root)
    journal = Journal(journal_path, plan)
    try:
        done, active, _ = journal.read()
        if active is not None:
            journal.append({"kind": "Finish", "index": active, "result": {"status": "Interrupted", "reason": "previous supervisor ended before durable completion; not retried"}})
            done, _, _ = journal.read()
        for index in plan["indices"][len(done):]:
            if cancelled():
                break
            validate(plan, root)
            seed = index | (2**63 if plan["domain"] == "Validation" else 0)
            journal.append({"kind": "Start", "index": index})
            argv = [str(artifact(root, plan["argv"][0])), *plan["argv"][1:],
                    "--tdi-seed", str(seed), "--tdi-plan-id", plan_id]
            result = {}
            try:
                result = supervise(argv, plan["timeout_seconds"], plan["max_output_bytes"], cancelled)
                if result["status"] == "Completed":
                    response = strict_json(result["stdout"])
                    if (not isinstance(response, dict) or response.get("seed") != seed
                            or type(response.get("seed")) is not int or response.get("plan_id") != plan_id
                            or response.get("status") not in ("Evaluated", "Rejected")):
                        raise ContractError("worker response binding or status mismatch")
                    result["response"] = response
                validate(plan, root)
            except (ValueError, OSError) as error:
                result.update(status="ContractRejected", error=str(error))
            journal.append({"kind": "Finish", "index": index, "result": result})
            after_commit(index)
        return journal.read()[0]
    finally:
        journal.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True, type=Path)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--journal", required=True, type=Path)
    args = parser.parse_args()
    stopped = False

    def stop(_signum, _frame):
        nonlocal stopped
        stopped = True

    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGTERM, stop)
    records = run(strict_json(args.plan.read_text()), args.root, args.journal, lambda: stopped)
    print(canonical(records))
    return 130 if stopped else 0


if __name__ == "__main__":
    raise SystemExit(main())
