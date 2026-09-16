"""Bounded process client for the SciRust research-stats JSON v1 example.

The caller explicitly trusts and pins an installed binary and its source SHA.
No Python statistical implementation substitutes for that executable. This is
process containment for trusted software, not an arbitrary-code sandbox.
"""
from pathlib import Path
import os
import signal
import subprocess
import tempfile
import time

import tdi_execution_graph as graphs
import tdi_experiment_supervisor as durable

LIMIT = 1024 * 1024


class SciRustStats:
    """Call a source-identified, byte-pinned SciRust process with finite budgets.

    Example: ``SciRustStats(binary, sha256, source_commit).call('holm', p_values=[.01])``.
    Requests and each diagnostic stream are bounded to 1 MiB; wall time is
    bounded to 30 seconds. Child processes are killed on failure/interruption.
    The binary and deployment directory must remain trusted and immutable.
    """
    def __init__(self, binary, sha256, source_commit):
        self.binary = Path(binary)
        if self.binary.is_symlink() or not self.binary.is_file():
            raise durable.ContractError("SciRust worker must be a regular non-symlink binary")
        self.binary = self.binary.resolve(strict=True)
        graphs._sha256(sha256, "SciRust binary identity")
        if not isinstance(source_commit, str) or len(source_commit) != 40 or any(c not in "0123456789abcdef" for c in source_commit):
            raise durable.ContractError("SciRust source must be an exact commit SHA")
        self.sha256, self.source_commit = sha256, source_commit
        if durable.file_digest(self.binary) != sha256:
            raise durable.ContractError("SciRust binary identity mismatch")

    @property
    def provenance(self):
        """Declared source and observed binary identity; no build attestation implied."""
        return {"repository": "Memorithm/scirust", "source_commit": self.source_commit,
                "binary_sha256": self.sha256, "protocol": "scirust-research-stats-json/v1"}

    def call(self, operation, **parameters):
        """Compute one bounded request or raise an explicit contract error."""
        if operation not in ("paired_mean_percentile", "holm", "morris", "sobol"):
            raise durable.ContractError("unknown SciRust research operation")
        raw = durable.canonical(dict(schema=1, operation=operation, **parameters)).encode()
        if len(raw) > LIMIT or durable.file_digest(self.binary) != self.sha256:
            raise durable.ContractError("SciRust request budget or binary identity mismatch")
        with tempfile.TemporaryDirectory(prefix="tdi-stats-") as root:
            root = Path(root)
            request, output, diagnostic = (root / x for x in ("input.json", "output.json", "diagnostic.txt"))
            request.write_bytes(raw)
            with request.open("rb") as stdin, output.open("wb") as stdout, diagnostic.open("wb") as stderr:
                process = subprocess.Popen([str(self.binary)], stdin=stdin, stdout=stdout, stderr=stderr,
                    start_new_session=True, cwd=root, env={"LANG": "C", "LC_ALL": "C"})
                try:
                    deadline = time.monotonic() + 30
                    while process.poll() is None:
                        if time.monotonic() > deadline or max(output.stat().st_size, diagnostic.stat().st_size) > LIMIT:
                            raise durable.ContractError("SciRust worker exceeded time/output budget")
                        time.sleep(0.01)
                finally:
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    process.wait(timeout=5)
            if max(output.stat().st_size, diagnostic.stat().st_size) > LIMIT:
                raise durable.ContractError("SciRust worker output exceeds 1 MiB")
            response = durable.strict_json(output.read_bytes(), max_bytes=LIMIT)
        if durable.file_digest(self.binary) != self.sha256:
            raise durable.ContractError("SciRust binary changed during evaluation")
        if (process.returncode != 0 or not isinstance(response, dict)
                or set(response) != {"schema", "status", "result"}
                or type(response["schema"]) is not int or response["schema"] != 1
                or response["status"] != "computed" or not isinstance(response["result"], dict)):
            raise durable.ContractError("SciRust rejected the request or returned an invalid response")
        return response["result"]
