#!/usr/bin/env python3
"""Measured Linux engine baselines, with complete raw repetitions and budgets.

Runs only public software controls. Each case executes in a fresh process.
Performance thresholds must be declared with a baseline before a comparison;
there is no default passing latency limit and no hardware speedup claim.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import platform
import secrets
import socket
import sqlite3
import statistics
import subprocess
import sys
import tempfile
import time
import tracemalloc
import urllib.error
import urllib.request

import tdi_artifact_contract as artifacts
import tdi_engine_runtime as runtime
from tdi_engine_store import EngineStore, atomic_json, identity
import tdi_experiment_supervisor as durable
from tdi_hub_client import HubClient
from tdi_hub_fixture import prepare_fixture
from tdi_observability import OTLPClient, prepare_otlp
from tdi_physical_telemetry import capacity_snapshot, measured_process


def proc_io():
    """Read actual Linux process I/O counters; unavailable sensors remain null."""
    try:
        return {line.split(':')[0]: int(line.split(':')[1]) for line in Path('/proc/self/io').read_text().splitlines()}
    except (OSError, ValueError):
        return None


def hardware_environment():
    """Record visible CPU and benchmark filesystem semantics without host paths."""
    cpu = {}
    try:
        for line in Path('/proc/cpuinfo').read_text().split('\n\n')[0].splitlines():
            if ':' in line:
                key, value = (x.strip() for x in line.split(':', 1))
                if key in ('vendor_id', 'model name', 'cpu family', 'model', 'stepping', 'microcode'):
                    cpu[key] = value
    except OSError:
        pass
    directory = Path(tempfile.gettempdir()).resolve()
    mounts = []
    for line in Path('/proc/self/mountinfo').read_text().splitlines():
        left, right = line.split(' - ', 1); fields = left.split(); after = right.split()
        mount = Path(fields[4])
        if directory == mount or mount in directory.parents:
            options = after[2].split(',')
            mounts.append((len(mount.parts), {"filesystem_type": after[0], "mount_options": fields[5].split(','),
                "semantic_options": [x for x in options if '=' not in x or x.split('=')[0] in ('fsync', 'index', 'metacopy', 'data', 'barrier', 'sync')]}))
    filesystem = max(mounts, key=lambda x: x[0])[1] if mounts else None
    return {"visible_cpu": cpu or None, "temporary_filesystem": filesystem,
            "storage_durability_qualified": False,
            "storage_scope": 'latency on the visible filesystem; successful fsync is not a power-loss experiment'}


def measure(function):
    """Return actual wall/CPU and scoped Linux I/O deltas plus the function value."""
    before = proc_io(); wall, cpu = time.perf_counter_ns(), time.process_time_ns()
    value = function()
    elapsed, used = time.perf_counter_ns() - wall, time.process_time_ns() - cpu
    after = proc_io()
    return value, {"wall_ns": elapsed, "cpu_ns": used,
                   "io": {k: after[k] - before[k] for k in before} if before and after else None}


def journal_case(root, count, payload_bytes):
    """Measure actual FULL-synchronous append, hashing volume and independent audit."""
    plan = {"schema": 1, "purpose": "journal-microbenchmark", "indices": list(range(count))}
    path = root / 'journal.sqlite'
    journal = durable.Journal(path, plan)
    counts = {"hash_calls": 0, "hash_input_bytes": 0, "full_scans": 0}
    digest, scan = durable.digest, journal._scan
    def counted(raw):
        counts['hash_calls'] += 1; counts['hash_input_bytes'] += len(raw)
        return digest(raw)
    def scanned():
        counts['full_scans'] += 1
        return scan()
    durable.digest, journal._scan = counted, scanned
    tracemalloc.start()
    def write():
        for i in range(count):
            journal.append({"kind": "Start", "index": i})
            # Distinct payload objects exercise retained result memory.
            journal.append({"kind": "Finish", "index": i, "result": {"status": "microbenchmark", "payload": str(i).zfill(payload_bytes)}})
    try:
        _, writing = measure(write)
        retained, peak = tracemalloc.get_traced_memory()
        append_counts = dict(counts)
        _, audit = measure(journal.audit)
        audit_counts = {k: counts[k] - append_counts[k] for k in counts}
        _, read = measure(journal.read)
        db_bytes = path.stat().st_size
    finally:
        tracemalloc.stop(); durable.digest = digest; journal.close()
    reopened, restart = measure(lambda: durable.Journal(path, plan))
    try:
        if len(reopened.read()[0]) != count:
            raise durable.ContractError("journal benchmark lost committed trials")
    finally:
        reopened.close()
    if append_counts['full_scans'] or append_counts['hash_calls'] != 2 * count:
        raise durable.ContractError("append rehashed history or changed its hashing budget")
    return {"write": writing, "audit": audit, "cached_read": read, "reopen_audit": restart,
            "append_work": append_counts, "audit_work": audit_counts, "database_bytes": db_bytes, "python_retained_bytes": retained,
            "python_peak_bytes": peak, "payload_bytes_per_trial": payload_bytes,
            "scope": "SQLite FULL/DELETE; tracemalloc active; I/O includes counter reads, caches may serve reads"}


def serialization_case(size):
    """Measure repeated actual canonical encoding/parsing and SHA-256 separately."""
    value = {"schema": 1, "purpose": "synthetic-serialization-benchmark", "payload": 'x' * size}
    raw = durable.canonical(value).encode()
    rounds = max(4, min(256, 1048576 // max(size, 1)))
    def repeat(function):
        for _ in range(rounds):
            function()
    _, encoding = measure(lambda: repeat(lambda: durable.canonical(value)))
    _, decoding = measure(lambda: repeat(lambda: durable.strict_json(raw, max_bytes=2 * 1024 * 1024, max_string_bytes=2 * 1024 * 1024)))
    _, hashing = measure(lambda: repeat(lambda: hashlib.sha256(raw).digest()))
    return {"bytes_per_operation": len(raw), "rounds": rounds, "encoding": encoding, "decoding": decoding, "sha256": hashing}


class CountedHub(HubClient):
    """Actual HTTP client counting JSON-body representations and artifact bytes."""
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.counts = {"requests": 0, "request_canonical_json_bytes": 0, "response_canonical_json_bytes": 0, "downloaded_artifact_bytes": 0}

    def request(self, method, path, **kwargs):
        self.counts['requests'] += 1
        if kwargs.get('value') is not None:
            self.counts['request_canonical_json_bytes'] += len(durable.canonical(kwargs['value']).encode())
        result = super().request(method, path, **kwargs)
        if not kwargs.get('binary'):
            self.counts['response_canonical_json_bytes'] += len(durable.canonical(result).encode())
        return result

    def download(self, *args, **kwargs):
        raw, binding = super().download(*args, **kwargs)
        self.counts['downloaded_artifact_bytes'] += len(raw)
        return raw, binding


class LocalHub:
    """Own an isolated real loopback Hub for bounded benchmark execution."""
    def __init__(self, root, binary):
        self.root, self.binary = root, binary
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0)); self.port = sock.getsockname()[1]
        self.endpoint = f'http://127.0.0.1:{self.port}'
        self.token = secrets.token_hex(24)
        self.log = (root / 'hub.log').open('ab')
        self.process = None

    def start(self):
        self.process = subprocess.Popen([self.binary, '--listen', f'127.0.0.1:{self.port}', '--data-dir', str(self.root / 'hub')],
            env={'PATH': os.defpath, 'SCIRUST_HUB_TOKEN': self.token}, stdin=subprocess.DEVNULL, stdout=self.log, stderr=self.log)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise durable.ContractError('benchmark Hub exited before becoming healthy')
            try:
                with urllib.request.urlopen(self.endpoint + '/health', timeout=.2): return
            except (OSError, urllib.error.URLError): time.sleep(.02)
        raise durable.ContractError('benchmark Hub startup timeout')

    def stop(self):
        if self.process is not None and self.process.poll() is None:
            self.process.terminate()
            try: self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill(); self.process.wait(timeout=5)
                raise durable.ContractError('benchmark Hub required forced termination')


def hub_case(root, count, concurrency, hubd, worker):
    """Measure real registration/admission/execute/transfer/restart/export preparation."""
    hub = LocalHub(root, hubd)
    try:
        _, startup = measure(hub.start)
        client = CountedHub(hub.endpoint, token=hub.token, allow_loopback_http=True)
        spec, registration = measure(lambda: prepare_fixture(client, worker, trials=count))
        spec['graph']['max_concurrency'] = concurrency
        spec = runtime.canonical_campaign(spec)
        with EngineStore(root / 'catalogue.sqlite') as store:
            campaign, admission = measure(lambda: runtime.submit(client, store, spec, {}))
            result, execution = measure(lambda: runtime.execute(client, store, campaign))
            if result['phase'] != 'completed':
                raise durable.ContractError('benchmark campaign failed')
            rows = store.results(campaign)
            payload_bytes = sum(r['evidence']['descriptor']['size_bytes'] for r in rows)
            evidence_bytes = sum(len(durable.canonical(r['evidence']).encode()) for r in rows)
            stored_result_bytes = store.db.execute('SELECT SUM(length(CAST(evidence AS BLOB))) FROM results WHERE campaign=?', (campaign,)).fetchone()[0]
            has_shared = store.db.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='shared_proofs'").fetchone()
            shared_proof_bytes = store.db.execute('SELECT COALESCE(SUM(length(CAST(payload AS BLOB))),0) FROM shared_proofs').fetchone()[0] if has_shared else 0
            evidence = rows[0]['evidence']
            def rehash():
                for _ in range(128):
                    artifacts.provenance_identity(evidence['provenance'])
            _, provenance = measure(rehash)
            transfer_before_restart = dict(client.counts)
            hub.stop()
            _, restart = measure(hub.start)
            _, refresh = measure(lambda: runtime.refresh(client, store, campaign))
            endpoint = OTLPClient('http://127.0.0.1:4318', allow_loopback_http=True)
            export, observability = measure(lambda: prepare_otlp(store, campaign, endpoint))
            receipt_bytes = len(durable.canonical(export['plan']).encode())
        return {"hub_startup": startup, "registration": registration, "admission": admission,
                "execute_collect": execution, "hub_restart": restart, "reconcile_after_restart": refresh,
                "provenance_128_hashes": provenance, "local_otlp_prepare": observability,
                "otlp_plan_bytes": receipt_bytes, "remote_export_delivery": None,
                "verified_outputs": len(rows), "artifact_payload_bytes": payload_bytes, "catalogue_evidence_json_bytes": evidence_bytes,
                "stored_result_json_bytes": stored_result_bytes, "shared_proof_json_bytes": shared_proof_bytes,
                "total_stored_evidence_json_bytes": stored_result_bytes + shared_proof_bytes,
                "transfer_before_restart": transfer_before_restart, "transfer_total": client.counts,
                "scope": "real public counter/verify workers; execution includes Hub scheduling and TDI G3 verification; OTLP encoding/persistence only, no remote collector"}
    finally:
        try: hub.stop()
        finally: hub.log.close()


def catalogue_case(root, count):
    """Measure actual registry writes and queries over explicitly synthetic rows."""
    # No Hub execution or result is asserted for these inventory-only records.
    with EngineStore(root / 'catalogue.sqlite') as store:
        def populate():
            for i in range(count):
                store.create({"kind": "synthetic-benchmark-catalogue-record", "index": i}, 'http://127.0.0.1:8477')
        _, writing = measure(populate)
        statements = []; store.db.set_trace_callback(statements.append)
        store.list(after=max(0, count - 50), limit=50, phase='prepared')
        store.db.set_trace_callback(None)
        query = next(sql for sql in statements if sql.startswith('SELECT id,phase'))
        plan = [list(row) for row in store.db.execute('EXPLAIN QUERY PLAN ' + query)]
        def queries():
            for _ in range(64):
                rows = store.list(after=max(0, count - 50), limit=50, phase='prepared')
                if len(rows) != min(50, count): raise durable.ContractError('catalogue benchmark lost rows')
        _, reads = measure(queries)
        database_bytes = (root / 'catalogue.sqlite').stat().st_size
    return {"populate": writing, "query_last_page_64_times": reads, "query_plan": plan,
            "database_bytes": database_bytes, "scope": "synthetic inventory, FULL SQLite transactions; no workflow/result claims"}


def run_case(case, hubd, worker):
    """Execute one bounded case in temporary storage and preserve its measurements."""
    with tempfile.TemporaryDirectory(prefix='tdi-engine-benchmark-') as folder:
        root = Path(folder)
        if case['kind'] == 'journal': return journal_case(root, case['count'], case['payload_bytes'])
        if case['kind'] == 'serialization': return serialization_case(case['payload_bytes'])
        if case['kind'] == 'catalogue': return catalogue_case(root, case['count'])
        if case['kind'] == 'hub': return hub_case(root, case['count'], case['concurrency'], hubd, worker)
        if case['kind'] == 'minimal-worker':
            code, out, _, costs = measured_process([worker, '--tdi-seed', '0', '--tdi-plan-id', '1' * 64], timeout=10, max_output=65536)
            result = durable.strict_json(out)
            if code != 0 or result.get('status') != 'Evaluated' or result.get('scores') != [2, 2, 2, 2]:
                raise durable.ContractError('minimal public worker failed its independent oracle')
            return {"worker_process": costs, "stdout_bytes": len(out)}
    raise durable.ContractError('unknown benchmark case')


def numeric_metrics(value, prefix=''):
    """Flatten physical measurements for comparisons; exclude counters and IDs."""
    found = {}
    if isinstance(value, dict):
        for key, item in value.items():
            name = prefix + '/' + key
            if type(item) is int and (key.endswith('_ns') or key.endswith('_bytes')):
                found[name] = item
            elif isinstance(item, dict): found.update(numeric_metrics(item, name))
    return found


def summarize(records):
    """Report raw-observation median/min/max without statistical coverage claims."""
    metrics = [numeric_metrics(r['measurements']) for r in records if r['phase'] == 'measured']
    if not metrics: raise durable.ContractError('no measured benchmark repetitions')
    keys = set.intersection(*(set(m) for m in metrics))
    return {key: {"median": statistics.median(m[key] for m in metrics), "min": min(m[key] for m in metrics),
                  "max": max(m[key] for m in metrics), "repetitions": len(metrics)} for key in sorted(keys)}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path)
    parser.add_argument('--hubd', type=Path, required=True); parser.add_argument('--worker', type=Path, required=True)
    parser.add_argument('--source-commit', required=True); parser.add_argument('--hub-source-commit', required=True)
    parser.add_argument('--repetitions', type=int, default=5); parser.add_argument('--warmup', type=int, default=1)
    parser.add_argument('--sizes', type=int, nargs='+', default=[64, 256, 1024])
    parser.add_argument('--hub-trials', type=int, nargs='+', default=[1, 4, 16])
    parser.add_argument('--max-seconds', type=int, default=600)
    parser.add_argument('--baseline', type=Path); parser.add_argument('--policy', type=Path)
    parser.add_argument('--case-json', help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    try:
        for name in ('source_commit', 'hub_source_commit'):
            value = getattr(args, name)
            if len(value) != 40 or any(c not in '0123456789abcdef' for c in value): raise durable.ContractError('exact source revisions required')
        for path in (args.hubd, args.worker):
            if path.is_symlink() or not path.is_file(): raise durable.ContractError('regular explicitly selected binaries required')
        if (not 2 <= args.repetitions <= 10 or not 0 <= args.warmup <= 2 or not 30 <= args.max_seconds <= 1800
                or not 1 <= len(args.sizes) <= 4 or any(not 16 <= x <= 4096 for x in args.sizes)
                or len(set(args.sizes)) != len(args.sizes) or not 1 <= len(args.hub_trials) <= 4
                or any(not 1 <= x <= 32 for x in args.hub_trials) or len(set(args.hub_trials)) != len(args.hub_trials)):
            raise durable.ContractError('benchmark budget outside the bounded profile')
        if args.case_json:
            case = durable.strict_json(args.case_json, max_bytes=4096)
            if case not in cases(args.sizes, args.hub_trials): raise durable.ContractError('case is outside the declared plan')
            print(durable.canonical(run_case(case, str(args.hubd.resolve()), str(args.worker.resolve())))); return 0
        if args.output is None or args.output.exists() or args.output.is_symlink():
            raise durable.ContractError('a new output directory is required')
        if bool(args.baseline) != bool(args.policy): raise durable.ContractError('baseline and predeclared policy must be supplied together')
        baseline = policy = None
        if args.baseline:
            from tdi_engine import read_json
            baseline, policy = read_json(args.baseline), read_json(args.policy)
            validate_baseline(baseline, policy)
        args.output.mkdir(parents=False)
        capacity = capacity_snapshot()
        environment = {"python": platform.python_version(), "platform": platform.platform(), "machine": platform.machine(),
                       "sqlite": sqlite3.sqlite_version, "cpu_affinity": sorted(os.sched_getaffinity(0)),
                       "hardware": hardware_environment(), "capacity": capacity}
        modules = ('tdi_engine_benchmark.py', 'tdi_engine_store.py', 'tdi_engine_runtime.py', 'tdi_experiment_supervisor.py',
                   'tdi_hub_client.py', 'tdi_hub_fixture.py', 'tdi_physical_telemetry.py', 'tdi_observability.py',
                   'tdi_artifact_contract.py', 'tdi_execution_graph.py', 'tdi_hub_edge_contract.py', 'tdi_hub_admission_contract.py',
                   'tdi_experiment_contract.py')
        source_files = {name: durable.file_digest(Path(__file__).with_name(name)) for name in modules}
        plan = {"schema": 1, "kind": "tdi-engine-benchmark-plan", "scope": 'non-final public software; serial cases; no GPU/energy claim',
                "source_commit": args.source_commit, "hub_source_commit": args.hub_source_commit,
                "files": source_files, "binary_sha256": {"hubd": durable.file_digest(args.hubd), "worker": durable.file_digest(args.worker)},
                "environment": environment, "cases": cases(args.sizes, args.hub_trials), "warmup": args.warmup,
                "repetitions": args.repetitions, "max_seconds": args.max_seconds, "policy": policy,
                "baseline_identity": baseline['identity'] if baseline else None}
        plan['identity'] = identity('tdi-engine-benchmark-plan/v1', plan)
        atomic_json(args.output / 'plan.json', plan)
        records = []; started = time.monotonic(); failure = None
        for case in plan['cases']:
            for rep in range(args.warmup + args.repetitions):
                remaining = args.max_seconds - (time.monotonic() - started)
                if remaining <= 0: failure = 'total wall budget exceeded'; break
                command = [sys.executable, str(Path(__file__).resolve()), '--case-json', durable.canonical(case),
                           '--hubd', str(args.hubd.resolve()), '--worker', str(args.worker.resolve()),
                           '--source-commit', args.source_commit, '--hub-source-commit', args.hub_source_commit,
                           '--sizes', *map(str, args.sizes), '--hub-trials', *map(str, args.hub_trials)]
                code, out, err, costs = measured_process(command, timeout=min(120, remaining), max_output=4 * 1024 * 1024)
                record = {"case": case, "repetition": rep, "phase": 'warmup' if rep < args.warmup else 'measured',
                          "outer_process": costs, "exit_code": code, "measurements": None, "diagnostic": err.decode(errors='replace')[-4096:]}
                if code == 0 and not costs['technical_failure']:
                    record['measurements'] = durable.strict_json(out, max_bytes=4 * 1024 * 1024)
                else: failure = 'case failed or exceeded its process budget'
                if (plan['binary_sha256'] != {"hubd": durable.file_digest(args.hubd), "worker": durable.file_digest(args.worker)}
                        or any(durable.file_digest(Path(__file__).with_name(name)) != digest for name, digest in source_files.items())):
                    failure = 'benchmark deployment changed during measurement'
                records.append(record); atomic_json(args.output / ('record-' + str(len(records)).zfill(4) + '.json'), record)
                if failure: break
            if failure: break
        summaries = [{"case": case, "metrics": summarize([r for r in records if r['case'] == case])} for case in plan['cases']] if failure is None else []
        report = {"schema": 1, "kind": "tdi-engine-benchmark", "plan": plan, "records": records, "summary": summaries,
                  "status": 'measured' if failure is None else 'incomplete', "error": failure,
                  "scientific_verdict": 'not-assessed', "comparison": None}
        if baseline and failure is None: report['comparison'] = compare(baseline, report, policy)
        report['identity'] = identity('tdi-engine-benchmark/v1', report)
        atomic_json(args.output / 'report.json', report)
        print(durable.canonical({"status": report['status'], "identity": report['identity'], "report": str(args.output / 'report.json'), "error": failure}))
        return 20 if failure or report['comparison'] and report['comparison']['status'] != 'within-policy' else 0
    except (ValueError, OSError, sqlite3.Error) as error:
        print(durable.canonical({"schema": 1, "status": 'benchmark-error', "error": str(error)}), file=sys.stderr)
        return 21


def cases(sizes, hub_trials):
    """Return the predeclared workload inventory, including equal-budget concurrency."""
    return ([{"kind": 'minimal-worker'}]
            + [{"kind": 'journal', "count": n, "payload_bytes": size} for n in sizes for size in (64, 4096)]
            + [{"kind": 'serialization', "payload_bytes": size} for size in (1024, 65536, 1048576)]
            + [{"kind": 'catalogue', "count": n} for n in sizes]
            + [{"kind": 'hub', "count": n, "concurrency": c} for n in hub_trials for c in sorted({1, min(4, n)})])


def validate_baseline(baseline, policy):
    """Bind a complete baseline and nonempty finite metric budget before execution."""
    if (not isinstance(baseline, dict) or baseline.get('kind') != 'tdi-engine-benchmark'
            or baseline.get('status') != 'measured' or baseline.get('identity') != identity('tdi-engine-benchmark/v1', {k: v for k, v in baseline.items() if k != 'identity'})):
        raise durable.ContractError('invalid baseline identity/status')
    if (not isinstance(policy, dict) or set(policy) != {'schema', 'baseline_identity', 'reason', 'limits'} or type(policy['schema']) is not int
            or policy['schema'] != 1 or policy['baseline_identity'] != baseline['identity'] or not isinstance(policy['reason'], str)
            or not 1 <= len(policy['reason']) <= 1024 or not isinstance(policy['limits'], list) or not 1 <= len(policy['limits']) <= 128):
        raise durable.ContractError('invalid predeclared baseline comparison policy')
    seen = set()
    for limit in policy['limits']:
        if (not isinstance(limit, dict) or set(limit) != {'case', 'metric', 'max_ratio'} or type(limit['max_ratio']) not in (float, int)
                or not 1 <= limit['max_ratio'] <= 10 or not isinstance(limit['metric'], str) or not 1 <= len(limit['metric']) <= 512):
            raise durable.ContractError('invalid regression metric budget')
        key = durable.canonical(limit['case']), limit['metric']
        matching = next((row for row in baseline['summary'] if row['case'] == limit['case']), None)
        if key in seen or matching is None or limit['metric'] not in matching['metrics'] or matching['metrics'][limit['metric']]['median'] <= 0:
            raise durable.ContractError('duplicate, missing or zero baseline metric')
        seen.add(key)


def compare(baseline, current, policy):
    """Compare only matching workloads/environment, retaining every selected ratio."""
    old, new = baseline['plan'], current['plan']
    fields = ('python', 'platform', 'machine', 'sqlite', 'cpu_affinity', 'hardware')
    if old['cases'] != new['cases'] or old['repetitions'] != new['repetitions'] or old['warmup'] != new['warmup'] or any(old['environment'][f] != new['environment'][f] for f in fields):
        return {"status": 'incompatible', "reason": 'workload or declared environment differs', "metrics": []}
    metrics = []
    for limit in policy['limits']:
        a = next(row for row in baseline['summary'] if row['case'] == limit['case'])['metrics'][limit['metric']]['median']
        row = next(row for row in current['summary'] if row['case'] == limit['case'])
        if limit['metric'] not in row['metrics']:
            return {"status": 'incompatible', "reason": 'selected metric unavailable', "metrics": metrics}
        b = row['metrics'][limit['metric']]['median']; ratio = b / a
        metrics.append({**limit, "baseline_median": a, "candidate_median": b, "ratio": ratio, "within_budget": ratio <= limit['max_ratio']})
    return {"status": 'within-policy' if all(row['within_budget'] for row in metrics) else 'regression', "metrics": metrics,
            "limitations": 'observed medians only; shared-host noise and cache state are not controlled; no significance claim'}


if __name__ == '__main__': raise SystemExit(main())
