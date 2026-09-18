"""Invoke the exact NNIS-owned hardware-manifest validator without device access.

Successful validation of the current unresolved manifest is recorded explicitly
as unresolved. It never becomes hardware or scientific execution authorization.
"""
import hashlib
from pathlib import Path
import subprocess
import sys

import tdi_experiment_supervisor as durable
from tdi_engine_store import identity
from tdi_nnis_partner_contract import (NNIS_SOURCE_SHA, NNIS_MANIFEST_PATH, NNIS_MANIFEST_BLOB_SHA,
    NNIS_VALIDATOR_PATH, NNIS_VALIDATOR_BLOB_SHA, NNIS_MANIFEST_STATUS)
from tdi_physical_telemetry import measured_process


def _pinned_blob(root, name, expected):
    path = root / name
    if any(p.is_symlink() for p in [path, *path.parents]):
        raise durable.ContractError('NNIS reviewed source cannot contain symlink components')
    with path.open('rb') as stream: raw = stream.read(1024 * 1024 + 1)
    if len(raw) > 1024 * 1024 or hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() != expected:
        raise durable.ContractError('NNIS reviewed blob differs from the exact audited source')
    return path, raw


def review_checkout(checkout):
    """Return provenance-bound review status for the pinned NNIS checkout.

    Requires an exact HEAD identity and exact manifest/validator blobs.
    Other checkout files are not part of this review and are never inspected.
    The subprocess has a 10 s / 1 MiB budget, imports no model, and receives no
    evidence or device selection. A future qualified manifest needs a separately
    reviewed pin; it cannot pass this unresolved-source adapter by substitution.
    """
    root = Path(checkout)
    if root.is_symlink() or not root.is_dir(): raise durable.ContractError('explicit NNIS checkout directory required')
    root = root.resolve(strict=True)
    command = subprocess.run(['git', '-C', str(root), 'rev-parse', 'HEAD'], capture_output=True, timeout=10, check=False)
    if command.returncode or command.stdout.decode().strip() != NNIS_SOURCE_SHA:
        raise durable.ContractError('NNIS checkout is not the exact audited revision')
    manifest, raw = _pinned_blob(root, NNIS_MANIFEST_PATH, NNIS_MANIFEST_BLOB_SHA)
    validator, code = _pinned_blob(root, NNIS_VALIDATOR_PATH, NNIS_VALIDATOR_BLOB_SHA)
    data = durable.strict_json(raw)
    if data['status'] != NNIS_MANIFEST_STATUS: raise durable.ContractError('NNIS qualification status drift')
    status, out, diagnostic, costs = measured_process([str(Path(sys.executable).resolve()), '-I', str(validator)], timeout=10,
        env={'LANG': 'C', 'LC_ALL': 'C', 'NNIS_EXPECTED_REVISION': NNIS_SOURCE_SHA})
    _pinned_blob(root, NNIS_MANIFEST_PATH, NNIS_MANIFEST_BLOB_SHA)
    _pinned_blob(root, NNIS_VALIDATOR_PATH, NNIS_VALIDATOR_BLOB_SHA)
    if status != 0 or costs['technical_failure']: raise durable.ContractError('NNIS-owned qualification validator rejected source')
    result = {'schema': 1, 'kind': 'tdi-nnis-qualification-review', 'source_commit': NNIS_SOURCE_SHA,
              'manifest_sha256': hashlib.sha256(raw).hexdigest(), 'validator_sha256': hashlib.sha256(code).hexdigest(),
              'status': data['status'], 'validator_stdout': out.decode('utf-8'), 'stderr_sha256': hashlib.sha256(diagnostic).hexdigest(),
              'cost_measurements': costs, 'hardware_execution_performed': False, 'qualification_resolved': False,
              'production_routing_authorized': False, 'performance_claim_authorized': False,
              'scientific_verdict': 'not-assessed'}
    result['identity'] = identity('tdi-nnis-qualification-review/v1', result)
    return result
