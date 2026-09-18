"""Bounded persistent transport for the actual Forge scientific state machine.

The caller persists each acknowledged command before acting on its permit.
Restart explicitly from that durable checkpoint; never redispatch external work
because a control response was lost. A source declaration is not an attestation.
"""
from __future__ import annotations

import copy
import math
import os
from pathlib import Path
import re
import selectors
import signal
import subprocess
import time

import tdi_experiment_supervisor as durable
from tdi_engine_store import identity
from tdi_forge_search import pinned_file

PROTOCOL = "forge-scientific-session/v1"


def recover_session_log(directory):
    """Reconstruct a portable checkpoint from a bounded, contiguous command log.

    Hashes detect corruption, not malicious owner replacement. Returned commands
    must be replayed by Forge; saved receipts never independently authorize work.
    An active external stage still requires caller-owned reconciliation.
    """
    directory = Path(directory)

    def read(path):
        if path.is_symlink() or not path.is_file():
            raise durable.ContractError("session log requires regular files")
        with path.open("rb") as stream:
            return durable.strict_json(stream.read(4 * 1024 * 1024 + 1),
                                       max_bytes=4 * 1024 * 1024, max_items=1000000)

    opened = read(directory / "session-open.json")
    if (set(opened) != {"schema_version", "spec", "response", "binding", "identity"}
            or type(opened["schema_version"]) is not int or opened["schema_version"] != 2
            or opened["identity"] != identity("tdi-forge-session-open/v2",
                                               {k: v for k, v in opened.items() if k != "identity"})):
        raise durable.ContractError("invalid session open identity")
    binding = opened["binding"]
    if (not isinstance(binding, dict)
            or set(binding) != {"path", "sha256", "source_commit", "repository", "protocol"}
            or not all(isinstance(v, str) for v in binding.values())
            or binding["protocol"] != PROTOCOL or binding["repository"] != "Memorithm/Forge"
            or not re.fullmatch(r"[0-9a-f]{40}", binding["source_commit"])
            or not re.fullmatch(r"[0-9a-f]{64}", binding["sha256"])
            or not Path(binding["path"]).is_absolute()):
        raise durable.ContractError("invalid session provenance structure")
    checkpoint = copy.deepcopy(opened["response"]["checkpoint"])
    previous = opened["identity"]
    journal = directory / "session-commands"
    if journal.is_symlink() or not journal.is_dir():
        raise durable.ContractError("invalid session journal directory")
    paths = sorted(journal.iterdir())
    if len(paths) + len(checkpoint["commands"]) > 2048:
        raise durable.ContractError("session log exceeds command budget")
    for index, path in enumerate(paths):
        if path.name != f"{index:04d}.json":
            raise durable.ContractError("noncontiguous session command log")
        delta = read(path)
        if (set(delta) != {"sequence", "previous", "spec_sha256", "command", "receipt", "identity"}
                or type(delta["sequence"]) is not int
                or delta["sequence"] != len(checkpoint["commands"]) + 1
                or delta["previous"] != previous or delta["spec_sha256"] != checkpoint["spec_sha256"]
                or delta["identity"] != identity("tdi-forge-session-command/v1",
                                                 {k: v for k, v in delta.items() if k != "identity"})):
            raise durable.ContractError("session command log identity mismatch")
        checkpoint["commands"].append(delta["command"])
        if len(durable.canonical(checkpoint)) > 3 * 1024 * 1024:
            raise durable.ContractError("session checkpoint exceeds byte budget")
        previous = delta["identity"]
    return {"spec": opened["spec"], "checkpoint": checkpoint, "binding": opened["binding"]}


class ForgeSession:
    """One trusted local executable and one immutable specification per process."""

    def __init__(self, binary, sha256, source_commit, spec, checkpoint=None, *, timeout=30):
        self.process = None
        self.closed = False
        self.frames = self.input_bytes = self.output_bytes = 0
        self.control_ns = 0
        self.spec = copy.deepcopy(spec)
        self.checkpoint = copy.deepcopy(checkpoint)
        if not isinstance(source_commit, str) or not re.fullmatch(r"[0-9a-f]{40}", source_commit):
            raise durable.ContractError("Forge source must be an exact declared commit")
        self.source_commit = source_commit
        self.binary = pinned_file(binary, sha256)
        self.stat_identity = self._stat()
        try:
            self.process = subprocess.Popen([self.binary["path"], "--session"],
                stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                bufsize=0, start_new_session=True, env={"LANG": "C", "LC_ALL": "C"})
            for pipe in (self.process.stdin, self.process.stdout, self.process.stderr):
                os.set_blocking(pipe.fileno(), False)
            pinned_file(self.binary["path"], self.binary["sha256"])
            opened = self._exchange({"op": "open", "spec": self.spec,
                                     "checkpoint": self.checkpoint}, timeout)
            self.initial = self._snapshot(opened, "opened")
            self.checkpoint = copy.deepcopy(self.initial["checkpoint"])
        except BaseException:
            self.abort()
            raise

    @property
    def binding(self):
        return dict(self.binary, source_commit=self.source_commit,
                    repository="Memorithm/Forge", protocol=PROTOCOL)

    def _stat(self):
        s = os.stat(self.binary["path"], follow_symlinks=False)
        return s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns, s.st_mode

    def _exchange(self, action, timeout):
        if (isinstance(timeout, bool) or not isinstance(timeout, (int, float))
                or not math.isfinite(timeout) or timeout <= 0):
            raise durable.ContractError("invalid session timeout")
        started = time.perf_counter_ns()
        try:
            if self.closed or self.process.poll() is not None or self._stat() != self.stat_identity:
                raise durable.ContractError("session stopped or deployment changed")
            raw = (durable.canonical({"protocol": PROTOCOL, "action": action}) + "\n").encode()
            if len(raw) > 4 * 1024 * 1024:
                raise durable.ContractError("session request exceeds 4 MiB")
            deadline = time.monotonic() + timeout
            sent, stdout, stderr = 0, bytearray(), bytearray()
            with selectors.DefaultSelector() as events:
                events.register(self.process.stdin, selectors.EVENT_WRITE, "input")
                events.register(self.process.stdout, selectors.EVENT_READ, "output")
                events.register(self.process.stderr, selectors.EVENT_READ, "error")
                while True:
                    remaining = deadline - time.monotonic()
                    if remaining <= 0:
                        raise TimeoutError("Forge session response deadline exceeded")
                    for key, _ in events.select(remaining):
                        if key.data == "input":
                            sent += os.write(key.fd, raw[sent:sent + 65536])
                            if sent == len(raw):
                                events.unregister(self.process.stdin)
                        else:
                            chunk = os.read(key.fd, 65536)
                            if not chunk:
                                events.unregister(key.fileobj)
                                if key.data == "output":
                                    raise durable.ContractError("Forge session closed without a complete response")
                                continue
                            target = stdout if key.data == "output" else stderr
                            target.extend(chunk)
                            if len(stdout) > 8 * 1024 * 1024 or len(stderr) > 65536:
                                raise durable.ContractError("Forge session output limit exceeded")
                    if b"\n" in stdout:
                        if sent != len(raw) or stdout[-1:] != b"\n" or stdout.count(b"\n") != 1:
                            raise durable.ContractError("unexpected Forge response framing")
                        if self._stat() != self.stat_identity:
                            raise durable.ContractError("session deployment changed")
                        reply = durable.strict_json(bytes(stdout), max_bytes=8 * 1024 * 1024,
                                                    max_items=1000000)
                        if set(reply) != {"protocol", "result"} or reply["protocol"] != PROTOCOL:
                            raise durable.ContractError("Forge session protocol mismatch")
                        self.frames += 1
                        self.input_bytes += len(raw)
                        self.output_bytes += len(stdout)
                        return reply["result"]
        except BaseException:
            self.abort()
            raise
        finally:
            self.control_ns += time.perf_counter_ns() - started

    def _snapshot(self, result, kind):
        if set(result) != {"kind", "response"} or result["kind"] != kind:
            raise durable.ContractError("Forge session snapshot kind mismatch")
        response = result["response"]
        external = self.spec["manifest"]["external_domain"]
        generation = {"upstream": external["upstream"],
                      "allowed_candidate_dimensions": external["allowed_candidate_dimensions"],
                      "generation_sources": external["data_boundary"]["generation_sources"]}
        cp = response["checkpoint"]
        if (set(response) != {"schema_version", "checkpoint", "snapshot", "generation_view"}
                or type(response["schema_version"]) is not int or response["schema_version"] != 1
                or set(cp) != {"schema_version", "spec_sha256", "commands"}
                or type(cp["schema_version"]) is not int or cp["schema_version"] != 1
                or not re.fullmatch(r"[0-9a-f]{64}", cp["spec_sha256"])
                or cp["commands"] != (self.checkpoint["commands"] if self.checkpoint else [])
                or (self.checkpoint and cp["spec_sha256"] != self.checkpoint["spec_sha256"])
                or response["generation_view"] != generation
                or response["snapshot"]["scientific_verdict"] != "not-assessed"):
            raise durable.ContractError("Forge session snapshot binding mismatch")
        return response

    def submit(self, command, *, timeout=30):
        try:
            command = copy.deepcopy(command)
            count = len(self.checkpoint["commands"])
            result = self._exchange({"op": "command", "spec_sha256": self.checkpoint["spec_sha256"],
                                     "expected_sequence": count, "command": command}, timeout)
            if (set(result) != {"kind", "spec_sha256", "sequence", "receipt"}
                    or result["kind"] != "receipt"
                    or result["spec_sha256"] != self.checkpoint["spec_sha256"]
                    or type(result["sequence"]) is not int or result["sequence"] != count + 1
                    or result["receipt"].get("status") not in
                    ("proposed", "started", "finished", "abandoned", "rejected", "duplicate")):
                raise durable.ContractError("Forge session receipt binding mismatch")
            self.checkpoint["commands"].append(command)
            return result["receipt"]
        except BaseException:
            self.abort()
            raise

    def inspect(self, *, timeout=30):
        try:
            result = self._exchange({"op": "inspect", "spec_sha256": self.checkpoint["spec_sha256"],
                                     "expected_sequence": len(self.checkpoint["commands"])}, timeout)
            return self._snapshot(result, "snapshot")
        except BaseException:
            self.abort()
            raise

    def close(self, *, timeout=5):
        if self.closed:
            return
        try:
            self.process.stdin.close()
            if self.process.wait(timeout=timeout) != 0:
                raise durable.ContractError("Forge session did not terminate successfully")
            pinned_file(self.binary["path"], self.binary["sha256"])
        finally:
            self.abort()

    def abort(self):
        self.closed = True
        if self.process is not None:
            if self.process.poll() is None:
                try:
                    os.killpg(self.process.pid, signal.SIGKILL)
                except ProcessLookupError:
                    pass
                self.process.wait(timeout=5)
            for pipe in (self.process.stdin, self.process.stdout, self.process.stderr):
                if pipe is not None and not pipe.closed:
                    pipe.close()

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.abort()
