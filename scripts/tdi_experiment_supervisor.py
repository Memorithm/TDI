#!/usr/bin/env python3
"""Linux non-final experiment supervision; standard library only.

This module provides a durable single-writer execution boundary for explicitly
allowed Development/Validation software trials. It is not a scientific stage
authorizer and it is not a complete sandbox.
"""
import argparse
import base64
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
import sys
import time
import uuid


EXIT_OK = 0
EXIT_TRIAL_FAILURE = 20
EXIT_CONTRACT = 21
EXIT_STORAGE = 22
EXIT_CANCELLED = 130
JOURNAL_FORMAT_VERSION = 2
ANCHOR_SCHEMA = 1
MAX_JSON_BYTES = 16_777_216
MAX_JSON_DEPTH = 64
MAX_JSON_ITEMS = 100_000
MAX_JSON_STRING_BYTES = 1_048_576
TECHNICAL_FAILURES = {
    "WorkerFailed", "Timeout", "OutputLimit", "Cancelled",
    "ContractRejected", "Interrupted",
}


class ContractError(ValueError):
    """Input or protocol data violated a declared contract."""


class StorageError(RuntimeError):
    """Durable journal or anchor storage could not satisfy its contract."""


def canonical(value):
    """Return canonical JSON used by the current v1 plan/journal identity."""
    return json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False)


def _finite_float(token):
    value = float(token)
    if not math.isfinite(value):
        raise ContractError("nonfinite JSON number")
    return value


def _validate_json_shape(value, max_depth, max_items, max_string_bytes):
    stack = [(value, 0)]
    seen = 0
    while stack:
        current, depth = stack.pop()
        if depth > max_depth:
            raise ContractError("JSON nesting depth exceeded")
        seen += 1
        if seen > max_items:
            raise ContractError("JSON item count exceeded")
        if isinstance(current, str):
            if len(current.encode("utf-8")) > max_string_bytes:
                raise ContractError("JSON string length exceeded")
        elif isinstance(current, dict):
            stack.extend((key, depth + 1) for key in current)
            stack.extend((item, depth + 1) for item in current.values())
        elif isinstance(current, list):
            stack.extend((item, depth + 1) for item in current)


def strict_json(raw, *, max_bytes=MAX_JSON_BYTES, max_depth=MAX_JSON_DEPTH,
                max_items=MAX_JSON_ITEMS, max_string_bytes=MAX_JSON_STRING_BYTES):
    """Decode bounded UTF-8 JSON and reject duplicate/non-finite values.

    The decoder rejects non-finite constants (NaN/Infinity) and decimal
    literals whose conversion overflows to infinity (for example ``1e999``).
    Duplicate keys, excessive input size, nesting, item counts and string sizes
    are rejected before the value can enter the durable protocol.
    """
    if isinstance(raw, bytes):
        if len(raw) > max_bytes:
            raise ContractError("JSON byte size exceeded")
        try:
            raw = raw.decode("utf-8", errors="strict")
        except UnicodeDecodeError as error:
            raise ContractError("invalid UTF-8 JSON") from error
    elif isinstance(raw, str):
        if len(raw.encode("utf-8")) > max_bytes:
            raise ContractError("JSON byte size exceeded")
    else:
        raise ContractError("JSON input must be text or bytes")

    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise ContractError("duplicate JSON key")
            result[key] = value
        return result

    def nonfinite(_token):
        raise ContractError("nonfinite JSON number")

    try:
        value = json.loads(raw, object_pairs_hook=pairs, parse_float=_finite_float,
                           parse_constant=nonfinite)
    except ContractError:
        raise
    except (ValueError, RecursionError) as error:
        raise ContractError("invalid JSON") from error
    _validate_json_shape(value, max_depth, max_items, max_string_bytes)
    return value


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


def _decode_output(data):
    try:
        return data.decode("utf-8", errors="strict"), None
    except UnicodeDecodeError:
        return None, base64.b64encode(data).decode("ascii")


def supervise(argv, timeout, limit, cancelled=lambda: False):
    """Bound both output streams together and reap the direct child.

    The process group is killed on timeout/cancel/output limit and after direct
    child completion. This does not stop descendants that deliberately escape
    the process group; a qualified containment backend is required for that.
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

    stdout, stdout_b64 = _decode_output(bytes(output["stdout"]))
    stderr, stderr_b64 = _decode_output(bytes(output["stderr"]))
    result = {
        "status": reason or ("Completed" if process.returncode == 0 else "WorkerFailed"),
        "returncode": process.returncode,
        "stdout": stdout,
        "stderr": stderr,
    }
    if stdout_b64 is not None:
        result["stdout_encoding"] = "base64"
        result["stdout_base64"] = stdout_b64
    if stderr_b64 is not None:
        result["stderr_encoding"] = "base64"
        result["stderr_base64"] = stderr_b64
    return result


def _fsync_directory(path):
    descriptor = os.open(str(path), os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _atomic_text(path, text):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_name(f".{path.name}.tmp.{os.getpid()}.{uuid.uuid4().hex}")
    try:
        with temporary.open("x", encoding="utf-8") as stream:
            stream.write(text)
            stream.flush()
            os.fsync(stream.fileno())
        os.replace(temporary, path)
        _fsync_directory(path.parent)
    except OSError as error:
        with contextlib.suppress(OSError):
            temporary.unlink()
        raise StorageError("failed to publish durable anchor") from error


class Journal:
    """Single-writer, FULL-synchronous append journal with incremental head state.

    Existing version-1 journals are validated and migrated non-destructively by
    adding version/index metadata. The event payload/hash chain is unchanged.
    ``audit()`` performs the independent full scan; ordinary ``append()`` uses
    the already-validated in-memory head and therefore does not rehash history.

    An optional anchor file binds journal identity, sequence and head hash. It
    detects a clean suffix deletion only when the anchor is kept on a separately
    trusted boundary. A colocated/rewriteable anchor is not tamper-proof.
    """
    def __init__(self, path, plan, anchor_path=None):
        self.path = Path(path)
        self.anchor_path = Path(anchor_path) if anchor_path is not None else None
        if self.path.is_symlink() or Path(str(self.path) + ".lock").is_symlink():
            raise ContractError("symlink journal")
        self.lock = open(str(self.path) + ".lock", "a+b")
        try:
            fcntl.flock(self.lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            existed = self.path.exists()
            try:
                self.db = sqlite3.connect(self.path)
                self.db.execute("PRAGMA synchronous=FULL")
                self.db.execute("PRAGMA journal_mode=DELETE")
            except sqlite3.Error as error:
                raise StorageError("failed to open journal") from error
            if not existed:
                self._create(plan)
            if self.db.execute("PRAGMA integrity_check").fetchone() != ("ok",):
                raise ContractError("journal integrity failure")
            if self.db.execute("SELECT plan FROM meta").fetchall() != [(canonical(plan),)]:
                raise ContractError("journal plan mismatch")
            self.plan = plan
            scanned = self._scan()
            self._install_or_validate_metadata(scanned)
            self._done, self._active, self._head, self._next_seq = scanned
            self._verify_index()
            self._verify_or_initialize_anchor()
        except BaseException:
            if hasattr(self, "db"):
                self.db.close()
            self.lock.close()
            raise

    def _create(self, plan):
        try:
            with self.db:
                self.db.execute("CREATE TABLE meta (plan TEXT NOT NULL)")
                self.db.execute("INSERT INTO meta VALUES (?)", (canonical(plan),))
                self.db.execute("CREATE TABLE events (seq INTEGER PRIMARY KEY, payload TEXT NOT NULL, hash TEXT NOT NULL)")
                self.db.execute("CREATE TABLE journal_schema (version INTEGER NOT NULL, journal_id TEXT NOT NULL)")
                self.db.execute("INSERT INTO journal_schema VALUES (?,?)", (JOURNAL_FORMAT_VERSION, uuid.uuid4().hex))
                self.db.execute("CREATE TABLE trial_state (trial_index INTEGER PRIMARY KEY, state TEXT NOT NULL, finish_payload TEXT, last_seq INTEGER NOT NULL)")
            _fsync_directory(self.path.parent)
        except (sqlite3.Error, OSError) as error:
            raise StorageError("failed to initialize journal") from error

    def _scan(self):
        previous = digest(canonical(self.plan if hasattr(self, "plan") else strict_json(
            self.db.execute("SELECT plan FROM meta").fetchone()[0])).encode())
        plan = self.plan if hasattr(self, "plan") else strict_json(self.db.execute("SELECT plan FROM meta").fetchone()[0])
        done, active = [], None
        try:
            rows = self.db.execute("SELECT seq,payload,hash FROM events ORDER BY seq")
            for expected_seq, (seq, raw, recorded) in enumerate(rows):
                if seq != expected_seq or digest((previous + raw).encode()) != recorded:
                    raise ContractError("journal sequence or hash mismatch")
                event = strict_json(raw)
                if not isinstance(event, dict) or set(event) not in ({"kind", "index"}, {"kind", "index", "result"}):
                    raise ContractError("invalid journal event shape")
                if event.get("kind") == "Start":
                    if set(event) != {"kind", "index"}:
                        raise ContractError("invalid Start event shape")
                    if active is not None or len(done) >= len(plan["indices"]) or event["index"] != plan["indices"][len(done)]:
                        raise ContractError("invalid start sequence")
                    active = event["index"]
                elif event.get("kind") == "Finish":
                    if set(event) != {"kind", "index", "result"} or active is None or event["index"] != active:
                        raise ContractError("invalid finish sequence")
                    done.append(event)
                    active = None
                else:
                    raise ContractError("invalid journal transition")
                previous = recorded
            return done, active, previous, len(done) * 2 + (1 if active is not None else 0)
        except sqlite3.Error as error:
            raise StorageError("failed to read journal") from error

    def _install_or_validate_metadata(self, scanned):
        done, active, _head, _next_seq = scanned
        has_schema = self.db.execute(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='journal_schema'").fetchone()
        has_index = self.db.execute(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='trial_state'").fetchone()
        try:
            with self.db:
                if not has_schema:
                    self.db.execute("CREATE TABLE journal_schema (version INTEGER NOT NULL, journal_id TEXT NOT NULL)")
                    self.db.execute("INSERT INTO journal_schema VALUES (?,?)", (JOURNAL_FORMAT_VERSION, uuid.uuid4().hex))
                else:
                    row = self.db.execute("SELECT version,journal_id FROM journal_schema").fetchall()
                    if len(row) != 1 or row[0][0] not in (1, JOURNAL_FORMAT_VERSION) or not row[0][1]:
                        raise ContractError("unsupported journal metadata")
                    if row[0][0] != JOURNAL_FORMAT_VERSION:
                        self.db.execute("UPDATE journal_schema SET version=?", (JOURNAL_FORMAT_VERSION,))
                if not has_index:
                    self.db.execute("CREATE TABLE trial_state (trial_index INTEGER PRIMARY KEY, state TEXT NOT NULL, finish_payload TEXT, last_seq INTEGER NOT NULL)")
                    seq = 0
                    for event in done:
                        self.db.execute("INSERT INTO trial_state VALUES (?,?,?,?)",
                                        (event["index"], "Finished", canonical(event), seq + 1))
                        seq += 2
                    if active is not None:
                        self.db.execute("INSERT INTO trial_state VALUES (?,?,?,?)", (active, "Started", None, seq))
            if not has_schema or not has_index:
                _fsync_directory(self.path.parent)
        except sqlite3.Error as error:
            raise StorageError("failed to migrate journal metadata") from error

    @property
    def journal_id(self):
        rows = self.db.execute("SELECT journal_id FROM journal_schema").fetchall()
        if len(rows) != 1:
            raise ContractError("invalid journal identity metadata")
        return rows[0][0]

    def _verify_index(self):
        expected = []
        seq = 0
        for event in self._done:
            expected.append((event["index"], "Finished", canonical(event), seq + 1))
            seq += 2
        if self._active is not None:
            expected.append((self._active, "Started", None, seq))
        actual = self.db.execute(
            "SELECT trial_index,state,finish_payload,last_seq FROM trial_state ORDER BY last_seq").fetchall()
        if actual != expected:
            raise ContractError("journal index mismatch")

    def _anchor_record(self):
        return {"schema": ANCHOR_SCHEMA, "journal_id": self.journal_id,
                "seq": self._next_seq - 1, "head": self._head}

    def _verify_or_initialize_anchor(self):
        if self.anchor_path is None:
            return
        if self.anchor_path.is_symlink():
            raise ContractError("symlink anchor")
        expected = self._anchor_record()
        if self.anchor_path.exists():
            actual = strict_json(self.anchor_path.read_bytes(), max_bytes=4096, max_depth=4,
                                 max_items=16, max_string_bytes=256)
            if actual != expected:
                raise ContractError("journal external anchor mismatch")
        else:
            _atomic_text(self.anchor_path, canonical(expected) + "\n")

    def _publish_anchor(self):
        if self.anchor_path is not None:
            _atomic_text(self.anchor_path, canonical(self._anchor_record()) + "\n")

    def close(self):
        self.db.close()
        self.lock.close()

    def read(self):
        """Return the already-validated in-memory journal state without rescanning."""
        return list(self._done), self._active, self._head

    def audit(self):
        """Independently rescan the complete hash chain and compare the durable index."""
        scanned = self._scan()
        done, active, head, next_seq = scanned
        if (done, active, head, next_seq) != (self._done, self._active, self._head, self._next_seq):
            raise ContractError("journal in-memory state diverged from durable state")
        self._verify_index()
        if self.anchor_path is not None:
            actual = strict_json(self.anchor_path.read_bytes(), max_bytes=4096, max_depth=4,
                                 max_items=16, max_string_bytes=256)
            if actual != self._anchor_record():
                raise ContractError("journal external anchor mismatch")
        return self.read()

    def _validate_transition(self, event):
        if not isinstance(event, dict):
            raise ContractError("journal event must be an object")
        kind = event.get("kind")
        if kind == "Start":
            if set(event) != {"kind", "index"} or self._active is not None:
                raise ContractError("invalid Start transition")
            position = len(self._done)
            if position >= len(self.plan["indices"]) or event["index"] != self.plan["indices"][position]:
                raise ContractError("invalid Start trial identity")
        elif kind == "Finish":
            if set(event) != {"kind", "index", "result"} or self._active is None or event["index"] != self._active:
                raise ContractError("invalid Finish transition")
        else:
            raise ContractError("invalid journal event kind")

    def append(self, event):
        self._validate_transition(event)
        raw = canonical(event)
        recorded = digest((self._head + raw).encode())
        seq = self._next_seq
        try:
            self.db.execute("BEGIN IMMEDIATE")
            self.db.execute("INSERT INTO events VALUES (?,?,?)", (seq, raw, recorded))
            if event["kind"] == "Start":
                self.db.execute("INSERT INTO trial_state VALUES (?,?,?,?)",
                                (event["index"], "Started", None, seq))
            else:
                self.db.execute("UPDATE trial_state SET state='Finished',finish_payload=?,last_seq=? WHERE trial_index=? AND state='Started'",
                                (raw, seq, event["index"]))
                if self.db.execute("SELECT changes()").fetchone()[0] != 1:
                    raise ContractError("journal state index transition failed")
            self.db.commit()
        except ContractError:
            self.db.rollback()
            raise
        except sqlite3.Error as error:
            self.db.rollback()
            raise StorageError("failed to append journal event") from error

        self._head = recorded
        self._next_seq += 1
        if event["kind"] == "Start":
            self._active = event["index"]
        else:
            self._done.append(event)
            self._active = None
        self._publish_anchor()


def run(plan, root, journal_path, cancelled=lambda: False, after_commit=lambda _: None,
        anchor_path=None):
    root = Path(root).resolve(strict=True)
    plan_id = validate(plan, root)
    journal = Journal(journal_path, plan, anchor_path=anchor_path)
    try:
        done, active, _ = journal.read()
        if active is not None:
            journal.append({"kind": "Finish", "index": active, "result": {
                "status": "Interrupted",
                "reason": "previous supervisor ended before durable completion; not retried",
            }})
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
                    if result["stdout"] is None:
                        raise ContractError("worker stdout is not valid UTF-8")
                    response = strict_json(result["stdout"], max_bytes=plan["max_output_bytes"])
                    if (not isinstance(response, dict) or response.get("seed") != seed
                            or type(response.get("seed")) is not int or response.get("plan_id") != plan_id
                            or response.get("status") not in ("Evaluated", "Rejected")):
                        raise ContractError("worker response binding or status mismatch")
                    result["response"] = response
                validate(plan, root)
            except (ContractError, OSError) as error:
                result.update(status="ContractRejected", error=str(error))
            journal.append({"kind": "Finish", "index": index, "result": result})
            after_commit(index)
        return journal.read()[0]
    finally:
        journal.close()


def exit_code_for_records(records, stopped=False):
    if stopped:
        return EXIT_CANCELLED
    return EXIT_TRIAL_FAILURE if any(
        record.get("result", {}).get("status") in TECHNICAL_FAILURES for record in records
    ) else EXIT_OK


def _emit(payload, diagnostic=None):
    print(canonical(payload))
    if diagnostic:
        print(diagnostic, file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True, type=Path)
    parser.add_argument("--root", required=True, type=Path)
    parser.add_argument("--journal", required=True, type=Path)
    parser.add_argument("--anchor", type=Path,
                        help="optional separately trusted external journal-head anchor")
    parser.add_argument("--audit-only", action="store_true",
                        help="validate journal/hash/index/anchor without executing workers")
    args = parser.parse_args()
    stopped = False

    def stop(_signum, _frame):
        nonlocal stopped
        stopped = True

    signal.signal(signal.SIGINT, stop)
    signal.signal(signal.SIGTERM, stop)
    try:
        plan = strict_json(args.plan.read_bytes())
        root = args.root.resolve(strict=True)
        validate(plan, root)
        if args.audit_only:
            journal = Journal(args.journal, plan, anchor_path=args.anchor)
            try:
                records, active, head = journal.audit()
            finally:
                journal.close()
            _emit({"schema": 1, "operation": "audit", "status": "ok",
                   "records": records, "active": active, "head": head})
            return EXIT_OK
        records = run(plan, root, args.journal, lambda: stopped, anchor_path=args.anchor)
        code = exit_code_for_records(records, stopped=stopped)
        status = "cancelled" if code == EXIT_CANCELLED else ("ok" if code == EXIT_OK else "trial-failure")
        _emit({"schema": 1, "operation": "run", "status": status,
               "exit_code": code, "records": records},
              None if code == EXIT_OK else "TDI supervisor recorded a technical trial failure")
        return code
    except ContractError as error:
        _emit({"schema": 1, "operation": "run", "status": "contract-error",
               "exit_code": EXIT_CONTRACT, "error": str(error)}, str(error))
        return EXIT_CONTRACT
    except (StorageError, sqlite3.Error, OSError) as error:
        _emit({"schema": 1, "operation": "run", "status": "storage-error",
               "exit_code": EXIT_STORAGE, "error": str(error)}, str(error))
        return EXIT_STORAGE


if __name__ == "__main__":
    raise SystemExit(main())
