"""Durable TDI campaign catalogue and scientific cache index.

This stores TDI execution evidence and references, not Hub payloads, leases or
scheduler state. SQLite transactions preserve intent before network mutations.
Use one trusted local directory with working fsync/flock; remote filesystems
and adversarial writers are outside this storage profile.
"""
from __future__ import annotations

import contextlib
import fcntl
import hashlib
import os
from pathlib import Path
import sqlite3
import time
import urllib.parse

import tdi_artifact_contract as artifacts
import tdi_experiment_supervisor as durable

SCHEMA = 1
MAX_PAGE = 200


def identity(kind, value):
    """Return a content identity independent of insertion order and timestamps."""
    return hashlib.sha256(kind.encode() + b"\0" + durable.canonical(value).encode()).hexdigest()


def atomic_json(path, value):
    """Publish a new JSON file atomically, refusing an existing destination.

    Example: ``atomic_json(Path('export.json'), {'schema': 1})``. The parent
    directory must exist. Failure leaves no successful-looking partial file.
    """
    path = Path(path)
    raw = (durable.canonical(value) + "\n").encode()
    import tempfile
    fd, temp = tempfile.mkstemp(prefix=".tdi-", dir=path.parent)
    try:
        with os.fdopen(fd, "wb") as stream:
            stream.write(raw)
            stream.flush()
            os.fsync(stream.fileno())
        os.link(temp, path, follow_symlinks=False)
        durable._fsync_directory(path.parent)
    finally:
        os.unlink(temp)


class EngineStore:
    """Open a transaction-serialized catalogue; readers never create a database.

    Example: ``with EngineStore(Path('campaigns.sqlite')) as store: ...``.
    ``readonly=True`` allows paginated inspection while another process runs.
    Errors are surfaced as SQLite/OS failures; committed intent is never erased
    to make an ambiguous network operation retryable.
    """

    def __init__(self, path, *, readonly=False):
        self.path = Path(path).absolute()
        self.readonly = readonly
        self.lock = None
        if self.path.is_symlink() or Path(str(self.path) + ".lock").is_symlink():
            raise durable.StorageError("symlink catalogue is forbidden")
        try:
            if readonly:
                uri = "file:" + urllib.parse.quote(str(self.path), safe="/") + "?mode=ro"
                self.db = sqlite3.connect(uri, uri=True, timeout=5)
            else:
                self.lock = open(str(self.path) + ".lock", "a+b")
                fcntl.flock(self.lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
                self.db = sqlite3.connect(self.path, timeout=5)
                self.db.execute("PRAGMA synchronous=FULL")
                self.db.execute("PRAGMA journal_mode=DELETE")
                self._initialize()
                # Initialization is exclusive; SQLite serializes subsequent
                # short mutations. Never hold a file lock across a network wait:
                # another CLI process must be able to persist cancellation.
                fcntl.flock(self.lock, fcntl.LOCK_UN)
            if self.db.execute("PRAGMA user_version").fetchone()[0] != SCHEMA:
                raise durable.StorageError("unsupported catalogue schema")
            self.db.row_factory = sqlite3.Row
            self.db.execute("PRAGMA foreign_keys=ON")
        except BaseException:
            if hasattr(self, "db"):
                self.db.close()
            if self.lock:
                self.lock.close()
            raise

    def _initialize(self):
        version = self.db.execute("PRAGMA user_version").fetchone()[0]
        if version == SCHEMA:
            return
        if version != 0 or self.db.execute("SELECT name FROM sqlite_master WHERE type='table'").fetchone():
            raise durable.StorageError("unknown database cannot become a TDI catalogue")
        statements = [
            "CREATE TABLE campaigns (id TEXT PRIMARY KEY, spec TEXT NOT NULL, endpoint TEXT NOT NULL, phase TEXT NOT NULL, workflow TEXT UNIQUE, admission TEXT, snapshot TEXT, created_ns INTEGER NOT NULL)",
            "CREATE TABLE events (sequence INTEGER PRIMARY KEY, campaign TEXT NOT NULL REFERENCES campaigns(id), kind TEXT NOT NULL, payload TEXT NOT NULL, recorded_ns INTEGER NOT NULL)",
            "CREATE INDEX campaign_events ON events(campaign, sequence)",
            "CREATE TABLE results (campaign TEXT NOT NULL REFERENCES campaigns(id), step TEXT NOT NULL, output TEXT NOT NULL, evidence TEXT NOT NULL, PRIMARY KEY(campaign,step,output))",
            "CREATE TABLE cache (key TEXT PRIMARY KEY, entry TEXT NOT NULL, campaign TEXT NOT NULL REFERENCES campaigns(id), step TEXT NOT NULL, output TEXT NOT NULL)",
            "CREATE TABLE restored_artifacts (campaign TEXT NOT NULL REFERENCES campaigns(id), artifact TEXT NOT NULL, location TEXT NOT NULL, PRIMARY KEY(campaign,artifact))",
        ]
        self.db.execute("BEGIN IMMEDIATE")
        try:
            for sql in statements:
                self.db.execute(sql)
            self.db.execute(f"PRAGMA user_version={SCHEMA}")
            self.db.commit()
            durable._fsync_directory(self.path.parent)
        except BaseException:
            self.db.rollback()
            raise

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.db.close()
        if self.lock:
            self.lock.close()

    def create(self, spec, endpoint):
        """Persist an immutable campaign before submission; exact replays return its ID."""
        campaign = identity("tdi-operational-campaign/v1", spec)
        raw = durable.canonical(spec)
        with self.db:
            row = self.db.execute("SELECT spec,endpoint FROM campaigns WHERE id=?", (campaign,)).fetchone()
            if row:
                if tuple(row) != (raw, endpoint):
                    raise durable.ContractError("campaign cannot move to a different Hub")
                return campaign
            self.db.execute("INSERT INTO campaigns VALUES (?,?,?,?,NULL,NULL,NULL,?)",
                            (campaign, raw, endpoint, "prepared", time.time_ns()))
            self._event(campaign, "prepared", {"spec_identity": campaign})
        return campaign

    def _event(self, campaign, kind, payload):
        self.db.execute("INSERT INTO events(campaign,kind,payload,recorded_ns) VALUES (?,?,?,?)",
                        (campaign, kind, durable.canonical(payload), time.time_ns()))

    def event(self, campaign, kind, payload):
        """Append an operational diagnostic without changing the scientific result."""
        with self.db:
            self._event(campaign, kind, payload)

    def get(self, campaign):
        """Return one campaign including its immutable spec and current Hub snapshot."""
        row = self.db.execute("SELECT * FROM campaigns WHERE id=?", (campaign,)).fetchone()
        if row is None:
            raise durable.ContractError("unknown campaign")
        value = dict(row)
        for key in ("spec", "admission", "snapshot"):
            if value[key] is not None:
                value[key] = durable.strict_json(value[key])
        if identity("tdi-operational-campaign/v1", value["spec"]) != campaign:
            raise durable.ContractError("catalogue specification identity mismatch")
        return value

    def transition(self, campaign, expected, phase, *, workflow=None, admission=None, snapshot=None):
        """Atomically compare the prior phase, update evidence, and append its event."""
        allowed = {
            "prepared": {"submitting"},
            "submitting": {"admitted", "submission-unknown"},
            "submission-unknown": {"admitted"},
            "admitted": {"executing", "cancel-requested", "completed", "failed", "cancelled"},
            "executing": {"executing", "completed", "failed", "cancelled", "cancel-requested"},
            "cancel-requested": {"cancel-requested", "completed", "failed", "cancelled"},
            "completed": {"completed"}, "failed": {"failed"}, "cancelled": {"cancelled"},
        }
        if phase not in allowed.get(expected, set()):
            raise durable.ContractError("invalid campaign transition")
        with self.db:
            cursor = self.db.execute(
                "UPDATE campaigns SET phase=?,workflow=COALESCE(?,workflow),admission=COALESCE(?,admission),snapshot=COALESCE(?,snapshot) WHERE id=? AND phase=?",
                (phase, workflow, durable.canonical(admission) if admission else None,
                 durable.canonical(snapshot) if snapshot else None, campaign, expected),
            )
            if cursor.rowcount != 1:
                raise durable.ContractError("campaign state changed or is unknown")
            self._event(campaign, phase, {"workflow": workflow})

    def put_result(self, campaign, step, output, evidence):
        """Store verified evidence once; changed authoritative results are rejected."""
        raw = durable.canonical(evidence)
        with self.db:
            row = self.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?",
                                  (campaign, step, output)).fetchone()
            if row:
                if row[0] != raw:
                    raise durable.ContractError("authoritative result changed")
                return
            self.db.execute("INSERT INTO results VALUES (?,?,?,?)", (campaign, step, output, raw))
            self._event(campaign, "result-verified", {"step": step, "output": output,
                                                      "evidence_id": identity("tdi-result-evidence/v1", evidence)})

    def results(self, campaign, *, after=0, limit=MAX_PAGE):
        """Return a bounded canonical page of result evidence, without payload downloads."""
        self._page(after, limit)
        rows = self.db.execute("SELECT step,output,evidence FROM results WHERE campaign=? ORDER BY step,output LIMIT ? OFFSET ?",
                               (campaign, limit, after))
        return [{"step": r[0], "output": r[1], "evidence": durable.strict_json(r[2])} for r in rows]

    def result(self, campaign, step, output):
        """Read one exact verified output, rejecting absent or incomplete evidence."""
        row = self.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?",
                              (campaign, step, output)).fetchone()
        if row is None:
            raise durable.ContractError("verified output is unavailable")
        return durable.strict_json(row[0])

    @staticmethod
    def _page(after, limit):
        if type(after) is not int or after < 0 or type(limit) is not int or not 1 <= limit <= MAX_PAGE:
            raise durable.ContractError("invalid catalogue pagination")

    def list(self, *, after=0, limit=MAX_PAGE, phase=None):
        """List campaigns in insertion order with an optional exact phase filter."""
        self._page(after, limit)
        rows = self.db.execute("SELECT id,phase,workflow,created_ns FROM campaigns WHERE (? IS NULL OR phase=?) ORDER BY created_ns,id LIMIT ? OFFSET ?",
                               (phase, phase, limit, after))
        return [dict(row) for row in rows]

    def events(self, campaign, *, after=0, limit=MAX_PAGE):
        """Read cursor-paginated events; `after` is the last observed sequence."""
        self._page(after, limit)
        rows = self.db.execute("SELECT sequence,kind,payload,recorded_ns FROM events WHERE campaign=? AND sequence>? ORDER BY sequence LIMIT ?",
                               (campaign, after, limit))
        return [dict(row) | {"payload": durable.strict_json(row["payload"])} for row in rows]

    def cache_put(self, entry, request, campaign, step, output, *, authorized):
        """Index one exact-domain deterministic success with its source provenance.

        The caller's policy must authorize reuse. Errors/rejections and timing
        observations are ineligible; the client verifies bytes again on a hit.
        """
        artifacts.validate_cache_reuse(entry, request, cache_authorized=authorized)
        row = self.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?",
                              (campaign, step, output)).fetchone()
        if not row or self.get(campaign)["phase"] != "completed":
            raise durable.ContractError("cache source must be a completed verified result")
        evidence = durable.strict_json(row[0])
        if (evidence.get("cache_eligible") is not True
                or evidence["artifact_identity"] != entry["artifact_identity"]
                or evidence["provenance_identity"] != entry["provenance_identity"]):
            raise durable.ContractError("cache source identity or eligibility mismatch")
        raw = durable.canonical(entry)
        with self.db:
            old = self.db.execute("SELECT entry FROM cache WHERE key=?", (entry["cache_key"],)).fetchone()
            if old and old[0] != raw:
                raise durable.ContractError("conflicting exact cache publication")
            self.db.execute("INSERT OR IGNORE INTO cache VALUES (?,?,?,?,?)",
                            (entry["cache_key"], raw, campaign, step, output))

    def cache_get(self, request, *, authorized):
        """Return a cache reference after authorization and exact-request validation."""
        if authorized is not True:
            raise durable.ContractError("cache lookup lacks explicit authorization")
        key = artifacts.cache_key(request)
        row = self.db.execute("SELECT * FROM cache WHERE key=?", (key,)).fetchone()
        if row is None:
            return None
        entry = durable.strict_json(row["entry"])
        artifacts.validate_cache_reuse(entry, request, cache_authorized=authorized)
        return {"entry": entry, "campaign": row["campaign"], "step": row["step"], "output": row["output"]}

    def backup(self, destination):
        """Create a coherent SQLite backup and refuse an existing target."""
        destination = Path(destination)
        fd = os.open(destination, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
        os.close(fd)
        try:
            with contextlib.closing(sqlite3.connect(destination)) as target:
                self.db.backup(target)
                if target.execute("PRAGMA integrity_check").fetchone() != ("ok",):
                    raise durable.StorageError("backup integrity failure")
            with destination.open("rb") as stream:
                os.fsync(stream.fileno())
            durable._fsync_directory(destination.parent)
        except BaseException:
            destination.unlink()
            raise

    def restore(self, record, results, locations, endpoint):
        """Commit an imported evidence catalogue after all payloads were verified.

        Original provenance/workflow identities remain unchanged. New Hub UUIDs
        live in separate transfer receipts; imported campaigns cannot execute.
        The caller must first validate the complete portable archive.
        """
        campaign = record["id"]
        if identity("tdi-operational-campaign/v1", record["spec"]) != campaign:
            raise durable.ContractError("imported campaign identity mismatch")
        with self.db:
            if self.db.execute("SELECT 1 FROM campaigns WHERE id=?", (campaign,)).fetchone():
                raise durable.ContractError("import refuses an existing campaign")
            self.db.execute("INSERT INTO campaigns VALUES (?,?,?,?,?,?,?,?)",
                            (campaign, durable.canonical(record["spec"]), endpoint, "imported", record["workflow"],
                             durable.canonical(record["admission"]), durable.canonical(record["snapshot"]), record["created_ns"]))
            for result in results:
                self.db.execute("INSERT INTO results VALUES (?,?,?,?)", (campaign, result["step"], result["output"], durable.canonical(result["evidence"])))
            for artifact, location in locations.items():
                self.db.execute("INSERT INTO restored_artifacts VALUES (?,?,?)", (campaign, artifact, durable.canonical(location)))
            self._event(campaign, "imported", {"source_endpoint": record["endpoint"], "source_phase": record["phase"]})
        return campaign

    def location(self, campaign, artifact):
        """Return an optional new transfer location, preserving the source evidence."""
        row = self.db.execute("SELECT location FROM restored_artifacts WHERE campaign=? AND artifact=?", (campaign, artifact)).fetchone()
        return durable.strict_json(row[0]) if row else None
