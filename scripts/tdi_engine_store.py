"""Durable TDI campaign catalogue and scientific cache index.

This stores TDI execution evidence and references, not Hub payloads, leases or
scheduler state. SQLite transactions preserve intent before network mutations.
Use one trusted local directory with working fsync/flock; remote filesystems
and adversarial writers are outside this storage profile.
"""
from __future__ import annotations

import contextlib
import copy
import fcntl
import hashlib
import os
from pathlib import Path
import sqlite3
import time
import urllib.parse

import tdi_artifact_contract as artifacts
import tdi_experiment_supervisor as durable

SCHEMA = 4
MAX_PAGE = 200
MAX_ACCESS_SCAN_NODES = 1_000_000


def _reject_restricted_reference(value, context):
    """Reject tagged restricted-reference metadata before local persistence.

    This is a defence-in-depth metadata invariant, not a content classifier or
    an authorization system.  It deliberately keys only on the canonical
    ``access_class`` field; unlabelled sensitive bytes require upstream policy.
    """
    stack = [value]
    visited = 0
    while stack:
        current = stack.pop()
        visited += 1
        if visited > MAX_ACCESS_SCAN_NODES:
            raise durable.ContractError("access metadata scan budget exceeded")
        if isinstance(current, dict):
            if current.get("access_class") == "restricted-reference":
                raise durable.ContractError(f"{context} contains restricted-reference access metadata")
            stack.extend(current.values())
        elif isinstance(current, (list, tuple)):
            stack.extend(current)


def identity(kind, value):
    """Return a content identity independent of insertion order and timestamps."""
    return hashlib.sha256(kind.encode() + b"\0" + durable.canonical(value).encode()).hexdigest()


def _unlink_if_same_file(path, reference):
    """Remove ``path`` only while it still names the completed reference file."""
    try:
        reference_stat = Path(reference).lstat()
        path_stat = Path(path).lstat()
        if (reference_stat.st_dev, reference_stat.st_ino) == (path_stat.st_dev, path_stat.st_ino):
            Path(path).unlink()
    except FileNotFoundError:
        pass


def _publish_completed_temp(temporary, destination):
    """Publish one complete temporary file without deleting a competing writer."""
    temporary = Path(temporary)
    destination = Path(destination)
    os.link(temporary, destination, follow_symlinks=False)
    try:
        durable._fsync_directory(destination.parent)
    except BaseException:
        _unlink_if_same_file(destination, temporary)
        raise


def atomic_json(path, value):
    """Publish a new JSON file atomically, refusing an existing destination.

    Example: ``atomic_json(Path('export.json'), {'schema': 1})``. The parent
    directory must exist. A failed publication removes only the name created by
    this call and never deletes a destination concurrently replaced by another
    writer. Broader crash/power-loss durability remains a separate qualification.
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
        _publish_completed_temp(temp, path)
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
            if self.db.execute("PRAGMA user_version").fetchone()[0] not in ((1, 2, 3, SCHEMA) if readonly else (SCHEMA,)):
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
        if version in (1, 2, 3):
            tables = {row[0] for row in self.db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
            expected = {"campaigns", "events", "results", "cache", "restored_artifacts"}
            if version >= 2:
                expected.add("exports")
            if version >= 3:
                expected.update(("searches", "search_events", "search_stages"))
            if tables != expected:
                raise durable.StorageError("unrecognized previous catalogue tables")
            self.db.execute("BEGIN IMMEDIATE")
            try:
                if version == 1:
                    self._create_exports()
                if version < 3:
                    self._create_searches()
                self._create_shared_proofs()
                self.db.execute(f"PRAGMA user_version={SCHEMA}")
                self.db.commit()
                durable._fsync_directory(self.path.parent)
            except BaseException:
                self.db.rollback()
                raise
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
            self._create_exports()
            self._create_searches()
            self._create_shared_proofs()
            self.db.execute(f"PRAGMA user_version={SCHEMA}")
            self.db.commit()
            durable._fsync_directory(self.path.parent)
        except BaseException:
            self.db.rollback()
            raise

    def _create_exports(self):
        self.db.execute("CREATE TABLE exports (id TEXT PRIMARY KEY, campaign TEXT NOT NULL REFERENCES campaigns(id), kind TEXT NOT NULL, endpoint TEXT NOT NULL, plan TEXT NOT NULL, state TEXT NOT NULL, receipt TEXT NOT NULL, updated_ns INTEGER NOT NULL)")
        self.db.execute("CREATE INDEX campaign_exports ON exports(campaign, updated_ns)")

    def _create_shared_proofs(self):
        self.db.execute("CREATE TABLE shared_proofs (id TEXT PRIMARY KEY, payload TEXT NOT NULL)")
        self.db.execute("CREATE INDEX campaigns_by_time ON campaigns(created_ns,id)")
        self.db.execute("CREATE INDEX campaigns_by_phase_time ON campaigns(phase,created_ns,id)")

    def _pack_result(self, evidence):
        """Intern one repeated G3 admission inside the caller's transaction.

        Legacy rows remain byte-for-byte unchanged. Public readers reconstruct
        the complete original evidence; the private storage envelope never
        becomes an artifact, exported proof or changed scientific identity.
        """
        if evidence.get("kind") == "tdi-stored-result/v1":
            raise durable.ContractError("private result storage envelope is reserved")
        binding = evidence.get("binding")
        if not isinstance(binding, dict) or not isinstance(binding.get("workflow_admission_binding"), dict):
            return durable.canonical(evidence)
        compact = copy.deepcopy(evidence)
        proof = compact["binding"].pop("workflow_admission_binding")
        key = identity("tdi-shared-admission/v1", proof)
        raw = durable.canonical(proof)
        self.db.execute("INSERT OR IGNORE INTO shared_proofs VALUES (?,?)", (key, raw))
        if self.db.execute("SELECT payload FROM shared_proofs WHERE id=?", (key,)).fetchone()[0] != raw:
            raise durable.ContractError("shared proof identity collision or corruption")
        stored = {"kind": "tdi-stored-result/v1", "admission_identity": key,
                  "evidence_identity": identity("tdi-result-evidence/v1", evidence), "evidence": compact}
        return durable.canonical(stored)

    def _unpack_result(self, raw):
        """Verify the stored reference and reconstruct the exact complete evidence."""
        value = durable.strict_json(raw)
        if not isinstance(value, dict) or value.get("kind") != "tdi-stored-result/v1":
            return value
        if set(value) != {"kind", "admission_identity", "evidence_identity", "evidence"}:
            raise durable.ContractError("invalid stored result envelope")
        row = self.db.execute("SELECT payload FROM shared_proofs WHERE id=?", (value["admission_identity"],)).fetchone()
        if row is None:
            raise durable.ContractError("shared admission proof missing")
        proof = durable.strict_json(row[0])
        if identity("tdi-shared-admission/v1", proof) != value["admission_identity"]:
            raise durable.ContractError("shared admission proof corrupted")
        evidence = value["evidence"]
        if not isinstance(evidence, dict) or not isinstance(evidence.get("binding"), dict) or "workflow_admission_binding" in evidence["binding"]:
            raise durable.ContractError("invalid compact evidence binding")
        evidence["binding"]["workflow_admission_binding"] = proof
        if identity("tdi-result-evidence/v1", evidence) != value["evidence_identity"]:
            raise durable.ContractError("stored evidence identity mismatch")
        return evidence

    def _create_searches(self):
        self.db.execute("CREATE TABLE searches (id TEXT PRIMARY KEY, spec TEXT NOT NULL, binding TEXT NOT NULL, response TEXT NOT NULL, phase TEXT NOT NULL, sequence INTEGER NOT NULL, created_ns INTEGER NOT NULL)")
        self.db.execute("CREATE TABLE search_events (sequence INTEGER PRIMARY KEY, search TEXT NOT NULL REFERENCES searches(id), kind TEXT NOT NULL, payload TEXT NOT NULL, recorded_ns INTEGER NOT NULL)")
        self.db.execute("CREATE INDEX search_events_by_search ON search_events(search,sequence)")
        self.db.execute("CREATE TABLE search_stages (search TEXT NOT NULL REFERENCES searches(id), attempt TEXT NOT NULL, campaign TEXT NOT NULL REFERENCES campaigns(id), roots TEXT NOT NULL, PRIMARY KEY(search,attempt))")

    def create_search(self, spec, binding, response):
        """Persist an immutable search and its initial Forge checkpoint, without execution."""
        _reject_restricted_reference({"spec": spec, "binding": binding, "response": response}, "search")
        key = identity("tdi-scientific-search/v1", {"spec": spec, "binding": binding})
        with self.db:
            row = self.db.execute("SELECT id FROM searches WHERE id=?", (key,)).fetchone()
            if row is None:
                self.db.execute("INSERT INTO searches VALUES (?,?,?,?, 'prepared',0,?)",
                    (key, durable.canonical(spec), durable.canonical(binding), durable.canonical(response), time.time_ns()))
                self._search_event(key, "prepared", {"identity": key})
        return self.get_search(key)

    def get_search(self, key):
        """Read administrative state; candidate proposal code must not receive it."""
        row = self.db.execute("SELECT * FROM searches WHERE id=?", (key,)).fetchone()
        if row is None:
            raise durable.ContractError("unknown scientific search")
        record = dict(row)
        for field in ("spec", "binding", "response"):
            record[field] = durable.strict_json(record[field], max_bytes=8 * 1024 * 1024, max_items=1000000)
        if identity("tdi-scientific-search/v1", {k: record[k] for k in ("spec", "binding")}) != key:
            raise durable.ContractError("scientific search contract/binding integrity mismatch")
        row = self.db.execute("SELECT payload FROM search_events WHERE search=? AND kind='transition' ORDER BY sequence DESC LIMIT 1", (key,)).fetchone()
        record["last_transition"] = durable.strict_json(row[0]) if row else None
        record["execution_error"] = record["phase"] == "cancel-requested" or bool(record["last_transition"] and
                                         ({"stage_failure", "budget_or_prerequisite_stop"} & set(record["last_transition"])))
        record["stages"] = [{"attempt": r[0], "campaign": r[1], "roots": durable.strict_json(r[2]), "phase": r[3], "workflow": r[4]}
                            for r in self.db.execute("SELECT ss.attempt,ss.campaign,ss.roots,c.phase,c.workflow FROM search_stages ss JOIN campaigns c ON c.id=ss.campaign WHERE ss.search=? ORDER BY ss.rowid", (key,))]
        return record

    def _search_event(self, key, kind, payload):
        _reject_restricted_reference(payload, "search event")
        self.db.execute("INSERT INTO search_events(search,kind,payload,recorded_ns) VALUES (?,?,?,?)",
                        (key, kind, durable.canonical(payload), time.time_ns()))

    def search_event(self, key, kind, payload):
        """Retain execution mappings/diagnostics without changing Forge authority."""
        with self.db:
            self._search_event(key, kind, payload)

    def update_search(self, key, sequence, phase, response, event):
        """CAS checkpoint and audit event in one FULL-synchronous transaction."""
        _reject_restricted_reference({"response": response, "event": event}, "search transition")
        origins = {"prepared": ("prepared",), "running": ("prepared", "running", "paused"),
                   "paused": ("running",), "completed": ("running", "completed"),
                   "cancel-requested": ("prepared", "running", "paused", "cancel-requested"),
                   "cancelled": ("cancel-requested", "cancelled")}
        if phase not in origins:
            raise durable.ContractError("unknown scientific search phase")
        placeholders = ",".join("?" for _ in origins[phase])
        with self.db:
            changed = self.db.execute("UPDATE searches SET response=?,phase=?,sequence=sequence+1 WHERE id=? AND sequence=? "
                f"AND phase IN ({placeholders}) "
                "AND NOT (? AND EXISTS (SELECT 1 FROM search_stages ss JOIN campaigns c ON c.id=ss.campaign WHERE ss.search=searches.id AND c.phase NOT IN ('completed','failed','cancelled')))",
                (durable.canonical(response), phase, key, sequence, *origins[phase], phase == "cancelled"))
            if changed.rowcount != 1:
                raise durable.ContractError("scientific search transition invalid, cleanup incomplete or state changed")
            self._search_event(key, "transition", event)
        return self.get_search(key)

    def list_searches(self, *, after=0, limit=MAX_PAGE):
        """Page actual search phases; v1/v2 read-only catalogues contain no searches."""
        self._page(after, limit)
        if self.db.execute("PRAGMA user_version").fetchone()[0] < 3:
            return []
        return [dict(row) for row in self.db.execute("SELECT id,phase,sequence,created_ns FROM searches ORDER BY created_ns,id LIMIT ? OFFSET ?", (limit, after))]

    def bind_search_stage(self, key, attempt, spec, roots, endpoint):
        """Bind a prepared Hub campaign before dispatch, atomically with cancellation."""
        _reject_restricted_reference({"spec": spec, "roots": roots}, "search stage")
        campaign = identity("tdi-operational-campaign/v1", spec)
        self.db.execute("BEGIN IMMEDIATE")
        with self.db:
            row = self.db.execute("SELECT phase FROM searches WHERE id=?", (key,)).fetchone()
            if row is None or row[0] != "running":
                raise durable.ContractError("search stopped before stage admission")
            existing = self.db.execute("SELECT campaign,roots FROM search_stages WHERE search=? AND attempt=?", (key, attempt)).fetchone()
            if existing:
                if tuple(existing) != (campaign, durable.canonical(roots)):
                    raise durable.ContractError("search stage plan changed after binding")
                return campaign
            previous = self.db.execute("SELECT spec,endpoint FROM campaigns WHERE id=?", (campaign,)).fetchone()
            if previous and tuple(previous) != (durable.canonical(spec), endpoint):
                raise durable.ContractError("search stage campaign binding mismatch")
            if previous is None:
                self.db.execute("INSERT INTO campaigns VALUES (?,?,?,?,NULL,NULL,NULL,?)",
                                (campaign, durable.canonical(spec), endpoint, "prepared", time.time_ns()))
                self._event(campaign, "prepared", {"spec_identity": campaign})
            self.db.execute("INSERT INTO search_stages VALUES (?,?,?,?)", (key, attempt, campaign, durable.canonical(roots)))
            self._search_event(key, "stage-plan", {"attempt_id": attempt, "campaign": campaign})
        return campaign

    def search_stage(self, key, attempt):
        """Recover an exact attempt mapping without creating another workflow."""
        row = self.db.execute("SELECT campaign,roots FROM search_stages WHERE search=? AND attempt=?", (key, attempt)).fetchone()
        return {"campaign": row[0], "roots": durable.strict_json(row[1])} if row else None

    @contextlib.contextmanager
    def delivery_lock(self):
        """Serialize external delivery/reconciliation only; never block campaign cancellation."""
        path = Path(str(self.path) + ".delivery.lock")
        if path.is_symlink():
            raise durable.StorageError("symlink delivery lock is forbidden")
        with path.open("a+b") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            yield

    def list_exports(self, *, after=0, limit=MAX_PAGE):
        """List bounded delivery state without loading payloads or credentials."""
        self._page(after, limit)
        return [dict(row) for row in self.db.execute("SELECT id,campaign,kind,endpoint,state,updated_ns FROM exports ORDER BY updated_ns,id LIMIT ? OFFSET ?", (limit, after))]

    def prepare_export(self, campaign, kind, endpoint, plan):
        """Persist a bounded immutable export plan without changing campaign evidence.

        At most 32 unfinished exports may be queued. Existing identical plans
        return their durable state and are never implicitly redispatched.
        """
        _reject_restricted_reference(plan, "external export plan")
        key = identity("tdi-external-export/v1", {"campaign": campaign, "kind": kind, "endpoint": endpoint, "plan": plan})
        raw = durable.canonical(plan)
        if kind not in ("mlflow", "otlp") or len(raw.encode()) > 1024 * 1024:
            raise durable.ContractError("unsupported or oversized export plan")
        self.db.execute("BEGIN IMMEDIATE")
        with self.db:
            row = self.db.execute("SELECT id FROM exports WHERE id=?", (key,)).fetchone()
            if row is None:
                if self.db.execute("SELECT COUNT(*) FROM exports WHERE state != 'sent'").fetchone()[0] >= 32:
                    raise durable.ContractError("external export queue is full")
                self.db.execute("INSERT INTO exports VALUES (?,?,?,?,?,'prepared','{}',?)", (key, campaign, kind, endpoint, raw, time.time_ns()))
                self._event(campaign, "export-prepared", {"export": key, "kind": kind})
        return self.get_export(key)

    def get_export(self, key):
        """Read one export intent/receipt without exposing credentials."""
        row = self.db.execute("SELECT * FROM exports WHERE id=?", (key,)).fetchone()
        if row is None:
            raise durable.ContractError("unknown external export")
        record = dict(row)
        record["plan"], record["receipt"] = durable.strict_json(record["plan"]), durable.strict_json(record["receipt"])
        expected = identity("tdi-external-export/v1", {k: record[k] for k in ("campaign", "kind", "endpoint", "plan")})
        if key != expected:
            raise durable.ContractError("external export plan integrity mismatch")
        return record

    def export_transition(self, key, expected, state, receipt):
        """Compare-and-set delivery state; export failure cannot overwrite results."""
        _reject_restricted_reference(receipt, "external export receipt")
        allowed = {"prepared": {"sending"}, "sending": {"sending", "sent", "failed"}, "failed": {"sending"}}
        if state not in allowed.get(expected, set()):
            raise durable.ContractError("invalid export transition")
        with self.db:
            changed = self.db.execute("UPDATE exports SET state=?,receipt=?,updated_ns=? WHERE id=? AND state=?",
                (state, durable.canonical(receipt), time.time_ns(), key, expected))
            if changed.rowcount != 1:
                raise durable.ContractError("export state changed; inspect before continuing")
            self._event(self.get_export(key)["campaign"], "export-" + state, {"export": key, "receipt": receipt})

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.db.close()
        if self.lock:
            self.lock.close()

    def create(self, spec, endpoint):
        """Persist an immutable campaign before submission; exact replays return its ID."""
        _reject_restricted_reference(spec, "campaign specification")
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
        _reject_restricted_reference(payload, "campaign event")
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
            "prepared": {"submitting", "cancelled"},
            "submitting": {"admitted", "submission-unknown"},
            "submission-unknown": {"admitted"},
            "admitted": {"executing", "cancel-requested", "completed", "failed", "cancelled"},
            "executing": {"executing", "completed", "failed", "cancelled", "cancel-requested"},
            "cancel-requested": {"cancel-requested", "completed", "failed", "cancelled"},
            "completed": {"completed"}, "failed": {"failed"}, "cancelled": {"cancelled"},
        }
        if phase not in allowed.get(expected, set()):
            raise durable.ContractError("invalid campaign transition")
        _reject_restricted_reference({"admission": admission, "snapshot": snapshot}, "campaign transition")
        with self.db:
            cursor = self.db.execute(
                "UPDATE campaigns SET phase=?,workflow=COALESCE(?,workflow),admission=COALESCE(?,admission),snapshot=COALESCE(?,snapshot) WHERE id=? AND phase=? "
                "AND NOT (? AND EXISTS (SELECT 1 FROM search_stages ss JOIN searches s ON s.id=ss.search WHERE ss.campaign=campaigns.id AND s.phase IN ('cancel-requested','cancelled')))",
                (phase, workflow, durable.canonical(admission) if admission else None,
                 durable.canonical(snapshot) if snapshot else None, campaign, expected, phase == "executing" and expected == "admitted"),
            )
            if cursor.rowcount != 1:
                raise durable.ContractError("campaign state changed or is unknown")
            self._event(campaign, phase, {"workflow": workflow})

    def put_result(self, campaign, step, output, evidence):
        """Store verified evidence once; changed authoritative results are rejected."""
        _reject_restricted_reference(evidence, "result evidence")
        raw = durable.canonical(evidence)
        with self.db:
            row = self.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?",
                                  (campaign, step, output)).fetchone()
            if row:
                if durable.canonical(self._unpack_result(row[0])) != raw:
                    raise durable.ContractError("authoritative result changed")
                return
            packed = self._pack_result(evidence)
            self.db.execute("INSERT INTO results VALUES (?,?,?,?)", (campaign, step, output, packed))
            self._event(campaign, "result-verified", {"step": step, "output": output,
                                                      "evidence_id": identity("tdi-result-evidence/v1", evidence)})

    def results(self, campaign, *, after=0, limit=MAX_PAGE):
        """Return a bounded canonical page of result evidence, without payload downloads."""
        self._page(after, limit)
        rows = self.db.execute("SELECT step,output,evidence FROM results WHERE campaign=? ORDER BY step,output LIMIT ? OFFSET ?",
                               (campaign, limit, after))
        return [{"step": r[0], "output": r[1], "evidence": self._unpack_result(r[2])} for r in rows]

    def result(self, campaign, step, output):
        """Read one exact verified output, rejecting absent or incomplete evidence."""
        row = self.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?",
                              (campaign, step, output)).fetchone()
        if row is None:
            raise durable.ContractError("verified output is unavailable")
        return self._unpack_result(row[0])

    @staticmethod
    def _page(after, limit):
        if type(after) is not int or after < 0 or type(limit) is not int or not 1 <= limit <= MAX_PAGE:
            raise durable.ContractError("invalid catalogue pagination")

    def list(self, *, after=0, limit=MAX_PAGE, phase=None, domain=None):
        """List campaigns in insertion order with optional exact phase/domain filters."""
        self._page(after, limit)
        if domain is not None:
            if domain not in ("Development", "Validation"):
                raise durable.ContractError("only non-final catalogue filters are supported")
            clause = "json_extract(spec,'$.domain')=?"
            values = [domain]
            if phase is not None:
                clause += " AND phase=?"
                values.append(phase)
            rows = self.db.execute(
                "SELECT id,phase,workflow,created_ns FROM campaigns WHERE " + clause + " ORDER BY created_ns,id LIMIT ? OFFSET ?",
                (*values, limit, after),
            )
        elif phase is None:
            rows = self.db.execute("SELECT id,phase,workflow,created_ns FROM campaigns ORDER BY created_ns,id LIMIT ? OFFSET ?", (limit, after))
        else:
            rows = self.db.execute("SELECT id,phase,workflow,created_ns FROM campaigns WHERE phase=? ORDER BY created_ns,id LIMIT ? OFFSET ?", (phase, limit, after))
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
        _reject_restricted_reference({"entry": entry, "request": request}, "cache publication")
        artifacts.validate_cache_reuse(entry, request, cache_authorized=authorized)
        row = self.db.execute("SELECT evidence FROM results WHERE campaign=? AND step=? AND output=?",
                              (campaign, step, output)).fetchone()
        if not row or self.get(campaign)["phase"] != "completed":
            raise durable.ContractError("cache source must be a completed verified result")
        evidence = self._unpack_result(row[0])
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

    def access_audit(self):
        """Audit structured catalogue metadata for tagged restricted references.

        The audit is label based: it proves only that persisted JSON metadata
        does not declare ``access_class=restricted-reference``.  It does not
        classify unlabelled payload content and is not a mixed-store ACL.
        """
        tables = {row[0] for row in self.db.execute("SELECT name FROM sqlite_master WHERE type='table'")}
        columns = {
            "campaigns": ("spec", "admission", "snapshot"),
            "events": ("payload",),
            "results": ("evidence",),
            "cache": ("entry",),
            "restored_artifacts": ("location",),
            "exports": ("plan", "receipt"),
            "searches": ("spec", "binding", "response"),
            "search_events": ("payload",),
            "search_stages": ("roots",),
            "shared_proofs": ("payload",),
        }
        checked = 0
        for table, names in columns.items():
            if table not in tables:
                continue
            quoted = ",".join(names)
            for row in self.db.execute(f"SELECT {quoted} FROM {table}"):
                for name, raw in zip(names, row):
                    if raw is None:
                        continue
                    value = durable.strict_json(raw)
                    _reject_restricted_reference(value, f"{table}.{name}")
                    checked += 1
        return {
            "schema": 1,
            "status": "clear",
            "structured_values_checked": checked,
            "restricted_reference_records": 0,
            "scope": "declared access_class metadata only",
        }

    def backup(self, destination):
        """Publish a coherent SQLite backup atomically and refuse an existing target.

        The externally named destination is linked only after SQLite backup,
        integrity validation and file ``fsync`` have completed. A process crash
        or storage failure before that point can therefore leave at most a hidden
        temporary file, never a successful-looking partial export. Directory
        ``fsync`` failure removes the just-linked destination best-effort before
        surfacing the persistence error. Existing tagged restricted-reference
        metadata blocks backup publication.
        """
        self.access_audit()
        import tempfile

        destination = Path(destination)
        if destination.exists() or destination.is_symlink():
            raise FileExistsError(destination)
        fd, temporary = tempfile.mkstemp(prefix=".tdi-backup-", dir=destination.parent)
        os.close(fd)
        temporary = Path(temporary)
        try:
            with contextlib.closing(sqlite3.connect(temporary)) as target:
                self.db.backup(target)
                if target.execute("PRAGMA integrity_check").fetchone() != ("ok",):
                    raise durable.StorageError("backup integrity failure")
            with temporary.open("rb") as stream:
                os.fsync(stream.fileno())
            _publish_completed_temp(temporary, destination)
        finally:
            temporary.unlink(missing_ok=True)

    def restore(self, record, results, locations, endpoint):
        """Commit an imported evidence catalogue after all payloads were verified.

        Original provenance/workflow identities remain unchanged. New Hub UUIDs
        live in separate transfer receipts; imported campaigns cannot execute.
        The caller must first validate the complete portable archive.
        """
        _reject_restricted_reference({"record": record, "results": results, "locations": locations}, "bundle restore")
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
                self.db.execute("INSERT INTO results VALUES (?,?,?,?)", (campaign, result["step"], result["output"], self._pack_result(result["evidence"])))
            for artifact, location in locations.items():
                self.db.execute("INSERT INTO restored_artifacts VALUES (?,?,?)", (campaign, artifact, durable.canonical(location)))
            self._event(campaign, "imported", {"source_endpoint": record["endpoint"], "source_phase": record["phase"]})
        return campaign

    def location(self, campaign, artifact):
        """Return an optional new transfer location, preserving the source evidence."""
        row = self.db.execute("SELECT location FROM restored_artifacts WHERE campaign=? AND artifact=?", (campaign, artifact)).fetchone()
        return durable.strict_json(row[0]) if row else None
