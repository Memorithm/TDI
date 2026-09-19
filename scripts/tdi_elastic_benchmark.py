#!/usr/bin/env python3
"""Bounded, real Elastic/Hub admission comparison on public counter fixtures.

One isolated Hub per case; no shared-capacity or multi-tenant qualification.
The fixed arm and Elastic arm execute the same work, not the same width when
Elastic reduces admission. Rejections are retained, never scored as fast runs.
"""
from __future__ import annotations

import argparse
import copy
import math
from pathlib import Path
import re
import statistics
import sys

import tdi_engine_archive as archive
import tdi_engine_runtime as runtime
from tdi_engine_benchmark import CountedHub, DEPLOYMENT_MODULES, LocalHub, measure
from tdi_engine_store import EngineStore, atomic_json, identity
import tdi_experiment_supervisor as durable
from tdi_hub_fixture import prepare_fixture
from tdi_physical_telemetry import capacity_snapshot
from tdi_resource_admission import _policy, execute_local_admitted


def schedule(widths, repeats):
    """Predeclared balanced arm order; never choose cases from observed timings."""
    if (not widths or len(set(widths)) != len(widths)
            or any(type(w) is not int or not 1 <= w <= 8 for w in widths)
            or type(repeats) is not int or not 1 <= repeats <= 20):
        raise durable.ContractError("unique widths 1..8 and repeats 1..20 required")
    return [(repeat, width, arm) for repeat in range(repeats) for width in widths
            for arm in (("fixed", "elastic") if repeat % 2 == 0 else ("elastic", "fixed"))]


def payload_identity(rows, trials):
    expected = {(f"{mode}-{i}", "file:result") for i in range(trials) for mode in ("run", "verify")}
    if len(rows) != len(expected) or {(r["step"], r["output"]) for r in rows} != expected:
        raise durable.ContractError("benchmark incomplete or duplicate outputs")
    values = sorted((r["step"], r["output"], r["evidence"]["json"]) for r in rows)
    return identity("tdi-counter-benchmark-payloads/v1", values)


def summarize(records):
    """Descriptive per-width summaries; failures remain in every denominator."""
    summaries = []
    for width, arm in sorted({(r["requested_width"], r["arm"]) for r in records}):
        group = [r for r in records if (r["requested_width"], r["arm"]) == (width, arm)]
        complete = [r for r in group if r["status"] == "completed"]
        times = sorted(r["submit_execute_collect"]["wall_ns"] for r in complete)
        elapsed = sum(r.get("submit_execute_collect", {}).get("wall_ns", 0) for r in group)
        summaries.append({"requested_width": width, "arm": arm, "attempted": len(group),
                          "completed": len(complete), "failed_or_rejected": len(group) - len(complete),
                          "actual_widths": [r["actual_width"] for r in complete],
                          "median_wall_ns": statistics.median(times) if times else None,
                          "p95_wall_ns_nearest_rank": times[math.ceil(.95 * len(times)) - 1] if times else None,
                          "successful_trials_per_second_in_timed_windows":
                              sum(r["trials"] for r in complete) * 1e9 / elapsed if elapsed else None})
    return summaries


def verify_report(root, *, candidate=None):
    """Recheck the full matrix, artifact bytes/provenance and derived summaries.

    This is internal consistency, not authentication of measured wall clocks.
    """
    def read(path):
        with path.open("rb") as stream:
            return durable.strict_json(stream.read(32 * 1024**2 + 1), max_bytes=32 * 1024**2,
                                       max_items=2_000_000, max_string_bytes=16 * 1024**2)
    manifest = read(root / "manifest.json")
    report = read(root / "report.json") if candidate is None else candidate
    if (report["manifest_identity"] != identity("tdi-elastic-benchmark-manifest/v1", manifest)
            or report["identity"] != identity("tdi-elastic-benchmark-report/v1", {k: v for k, v in report.items() if k != "identity"})
            or len(report["records"]) != len(manifest["schedule"])):
        raise durable.ContractError("benchmark report identity or matrix mismatch")
    payloads = set()
    for index, (record, case) in enumerate(zip(report["records"], manifest["schedule"])):
        if ([record["repeat"], record["requested_width"], record["arm"]] != case
                or record["trials"] != manifest["trials"]
                or read(root / f"case-{index:03d}.json") != record
                or record["status"] not in ("completed", "rejected")):
            raise durable.ContractError("benchmark case binding mismatch")
        if record["status"] == "completed":
            bundle = read(root / f"case-{index:03d}" / "bundle.json")
            archive.verify_bundle(bundle, expected_identity=record["bundle"]["identity"])
            fingerprint = payload_identity(bundle["members"], manifest["trials"])
            if (fingerprint != record["payload_identity"] or record["verified_outputs"] != 2 * manifest["trials"]
                    or record["restart_unchanged"] is not True
                    or bundle["campaign"]["spec"]["graph"]["max_concurrency"] != record["actual_width"]
                    or not 1 <= record["actual_width"] <= record["requested_width"]
                    or (record["arm"] == "fixed" and record["actual_width"] != record["requested_width"])):
                raise durable.ContractError("benchmark result binding mismatch")
            payloads.add(fingerprint)
    status = "completed" if all(r["status"] == "completed" for r in report["records"]) else "contains-rejections"
    if len(payloads) > 1 or report["status"] != status or report["summary"] != summarize(report["records"]):
        raise durable.ContractError("benchmark summary or cross-arm payload mismatch")
    return report["identity"]


def run_case(root, *, arm, width, trials, hubd, worker, elastic, elastic_sha256, source, policy):
    root.mkdir()
    record = {"arm": arm, "requested_width": width, "trials": trials, "status": "incomplete"}
    hub = LocalHub(root, str(hubd))
    try:
        _, record["hub_startup"] = measure(hub.start)
        client = CountedHub(hub.endpoint, token=hub.token, allow_loopback_http=True)
        spec, record["registration"] = measure(lambda: prepare_fixture(client, worker, trials=trials))
        spec["graph"]["max_concurrency"] = width
        spec = runtime.canonical_campaign(spec)
        original = copy.deepcopy(spec)
        record["capacity_before"] = capacity_snapshot()
        with EngineStore(root / "catalogue.sqlite") as store:
            def execute():
                if arm == "elastic":
                    return execute_local_admitted(client, store, spec, {}, elastic,
                                                  elastic_sha256, source, dict(policy, max_concurrency=width))
                campaign = runtime.submit(client, store, spec, {})
                return runtime.execute(client, store, campaign)
            result, record["submit_execute_collect"] = measure(execute)
            record["capacity_after"] = capacity_snapshot()
            original_id = identity("tdi-operational-campaign/v1", original)
            if arm == "elastic":
                record["admission_evidence"] = [e["payload"] for e in store.events(original_id)
                                                 if e["kind"] == "resource-admission"]
            if result.get("status") == "resource-rejected":
                record.update(status="rejected", reason=result["reason"])
                return record
            if result.get("phase") != "completed":
                raise durable.ContractError("benchmark campaign did not complete")
            actual = copy.deepcopy(result["spec"])
            record["actual_width"] = actual["graph"]["max_concurrency"]
            if not 1 <= record["actual_width"] <= width or (arm == "fixed" and record["actual_width"] != width):
                raise durable.ContractError("benchmark execution width mismatch")
            actual["graph"]["max_concurrency"] = width
            if actual != original:
                raise durable.ContractError("admission changed workload beyond width")
            rows = store.results(result["id"])
            record["payload_identity"] = payload_identity(rows, trials)
            # Export independently verifies all selected artifact bytes and provenance.
            record["bundle"], record["export_verify"] = measure(
                lambda: archive.export_bundle(client, store, result["id"], root / "bundle.json"))
            snapshot = result["snapshot"]
            hub.stop()
            _, record["hub_restart"] = measure(hub.start)
            resumed, record["reconcile"] = measure(lambda: runtime.execute(client, store, result["id"]))
            if resumed["snapshot"] != snapshot or payload_identity(store.results(result["id"]), trials) != record["payload_identity"]:
                raise durable.ContractError("benchmark restart changed execution or payloads")
            record.update(status="completed", restart_unchanged=True, verified_outputs=len(rows))
        return record
    finally:
        try:
            atomic_json(root / "diagnostic.json", record)
        finally:
            try:
                hub.stop()
            finally:
                hub.log.close()


def run(args):
    cases = schedule(args.widths, args.repeats)
    if type(args.trials) is not int or not max(args.widths) <= args.trials <= 32:
        raise durable.ContractError("trials must cover all widths and be at most 32")
    policy = _policy({"max_concurrency": max(args.widths), "memory_bytes_per_trial": args.memory_bytes_per_trial,
                      "reserve_memory_bytes": args.reserve_memory_bytes, "max_age_milliseconds": 10000})
    for source in (args.elastic_source, args.hub_source):
        if not re.fullmatch(r"[0-9a-f]{40}", source):
            raise durable.ContractError("exact source commits required")
    binaries = {name: Path(getattr(args, name)).resolve(strict=True) for name in ("hubd", "worker", "elastic")}
    hashes = {name: durable.file_digest(path) for name, path in binaries.items()}
    modules = sorted(set(DEPLOYMENT_MODULES) | {"tdi_elastic_benchmark.py", "tdi_resource_admission.py"})
    manifest = {"schema": "tdi-elastic-benchmark/v1", "trials": args.trials, "schedule": cases, "policy": policy,
                "binary_sha256": hashes, "source_declarations": {"elastic": args.elastic_source, "hub": args.hub_source},
                "source_build_attestation": False,
                "scripts": {name: durable.file_digest(Path(__file__).with_name(name)) for name in modules},
                "scope": "one local graph at a time; public counter fixture; no optimizer quality or multi-campaign claim",
                "timing_scope": "submit/admission + execute + collect; setup, export, restart measured separately",
                "peak_total_memory_bytes": None, "power_loss_durability_qualified": False}
    args.output.mkdir(parents=True, exist_ok=False)
    atomic_json(args.output / "manifest.json", manifest)
    records = []
    for index, (repeat, width, arm) in enumerate(cases):
        case_root = args.output / f"case-{index:03d}"
        try:
            record = run_case(case_root, arm=arm, width=width, trials=args.trials, **binaries,
                              elastic_sha256=hashes["elastic"], source=args.elastic_source, policy=policy)
        except Exception as error:
            # Keep attempted-case failure and all prior artifacts; never fabricate a timing.
            record = {"arm": arm, "requested_width": width, "trials": args.trials,
                      "status": "failed", "error_type": type(error).__name__, "error": str(error)}
            atomic_json(args.output / f"case-{index:03d}.json", dict(record, repeat=repeat))
            raise
        record["repeat"] = repeat
        atomic_json(args.output / f"case-{index:03d}.json", record)
        records.append(record)
        print(f"{index + 1}/{len(cases)} {arm} width={width}: {record['status']}", file=sys.stderr, flush=True)
    if hashes != {name: durable.file_digest(path) for name, path in binaries.items()}:
        raise durable.ContractError("benchmark binary changed")
    payloads = {r["payload_identity"] for r in records if r["status"] == "completed"}
    if len(payloads) > 1:
        raise durable.ContractError("benchmark arms produced different payloads")
    report = {"manifest_identity": identity("tdi-elastic-benchmark-manifest/v1", manifest),
              "status": "completed" if all(r["status"] == "completed" for r in records) else "contains-rejections",
              "records": records, "summary": summarize(records)}
    report["identity"] = identity("tdi-elastic-benchmark-report/v1", report)
    verify_report(args.output, candidate=report)
    atomic_json(args.output / "report.json", report)
    return 0 if report["status"] == "completed" else durable.EXIT_TRIAL_FAILURE


def main():
    if len(sys.argv) == 3 and sys.argv[1] == "--verify":
        print(verify_report(Path(sys.argv[2])))
        return 0
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ("hubd", "worker", "elastic"):
        parser.add_argument("--" + name, type=Path, required=True)
    parser.add_argument("--elastic-source", required=True)
    parser.add_argument("--hub-source", required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--widths", type=int, nargs="+", default=[1, 2, 4, 8])
    parser.add_argument("--trials", type=int, default=8)
    parser.add_argument("--repeats", type=int, default=4)
    parser.add_argument("--memory-bytes-per-trial", type=int, default=64 * 1024**2)
    parser.add_argument("--reserve-memory-bytes", type=int, default=128 * 1024**2)
    return run(parser.parse_args())


if __name__ == "__main__":
    raise SystemExit(main())
