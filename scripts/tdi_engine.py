#!/usr/bin/env python3
"""TDI operational CLI: prepare, execute, inspect, resume, cancel and export.

All commands emit versioned JSON. The local catalogue is usable without a
network service for status, inspection, comparison, backup and bundle checks.
Execution uses an explicitly selected Hub deployment and trusted-software policy.
"""
from __future__ import annotations

import argparse
import os
from pathlib import Path
import sqlite3
import sys

import tdi_experiment_supervisor as durable
import tdi_engine_runtime as runtime
from tdi_engine_store import EngineStore, atomic_json, identity
from tdi_hub_client import HubClient, HubTransportUnknown
from tdi_engine_archive import export_bundle, restore_bundle, verify_bundle
from tdi_observability import ExportError, EXIT_EXPORT


def read_json(path, limit=16 * 1024 * 1024, *, max_items=100_000):
    """Read bounded UTF-8 JSON from an explicitly selected regular file."""
    path = Path(path)
    if path.is_symlink() or not path.is_file():
        raise durable.ContractError("expected an explicit regular non-symlink file")
    with path.open("rb") as stream:
        return durable.strict_json(stream.read(limit + 1), max_bytes=limit, max_items=max_items)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalogue", type=Path, default=Path("tdi-campaigns.sqlite"))
    parser.add_argument("--hub", default="https://127.0.0.1:8477")
    parser.add_argument("--allow-loopback-http", action="store_true")
    parser.add_argument("--timeout", type=float, default=30)
    sub = parser.add_subparsers(dest="operation", required=True)
    sub.add_parser("doctor")
    p = sub.add_parser("sensitivity-plan")
    p.add_argument("--protocol", type=Path, required=True); p.add_argument("--output", type=Path, required=True)
    p = sub.add_parser("sensitivity-fixture-plan")
    p.add_argument("--plan", type=Path, required=True); p.add_argument("--function", choices=("additive", "ishigami"), required=True)
    p.add_argument("--output", type=Path, required=True)
    p = sub.add_parser("sensitivity-analyze")
    p.add_argument("--plan", type=Path, required=True); p.add_argument("--output", type=Path, required=True)
    source = p.add_mutually_exclusive_group(required=True)
    source.add_argument("--selections", type=Path); source.add_argument("--observations", type=Path); source.add_argument("--campaign")
    p.add_argument("--worker", type=Path); p.add_argument("--worker-sha256"); p.add_argument("--source-commit")
    for verb in ("validate", "plan"):
        p = sub.add_parser(verb); p.add_argument("spec", type=Path)
    p = sub.add_parser("fixture-plan")
    p.add_argument("--worker", type=Path, required=True); p.add_argument("--output", type=Path, required=True)
    p.add_argument("--trials", type=int, default=3); p.add_argument("--domain", choices=("Development", "Validation"), default="Development")
    p = sub.add_parser("library-fixture-plan")
    p.add_argument("--worker", type=Path, required=True); p.add_argument("--output", type=Path, required=True)
    p.add_argument("--adapter", choices=("finite", "jacobi"), required=True)
    p.add_argument("--trials", type=int, default=2); p.add_argument("--domain", choices=("Development", "Validation"), default="Development")
    p = sub.add_parser("submit"); p.add_argument("spec", type=Path); p.add_argument("--roots", type=Path)
    p = sub.add_parser("run-local-admitted")
    p.add_argument("spec", type=Path); p.add_argument("--roots", type=Path)
    p.add_argument("--elastic-worker", type=Path, required=True); p.add_argument("--worker-sha256", required=True)
    p.add_argument("--source-commit", required=True); p.add_argument("--resource-policy", type=Path, required=True)
    sub.add_parser("capacity")
    for verb in ("run", "resume", "cancel", "inspect"):
        p = sub.add_parser(verb); p.add_argument("campaign")
    p = sub.add_parser("attach"); p.add_argument("campaign"); p.add_argument("workflow"); p.add_argument("--roots", type=Path)
    p = sub.add_parser("status"); p.add_argument("--after", type=int, default=0); p.add_argument("--limit", type=int, default=100); p.add_argument("--phase")
    p = sub.add_parser("events"); p.add_argument("campaign"); p.add_argument("--after", type=int, default=0)
    p = sub.add_parser("compare"); p.add_argument("left"); p.add_argument("right")
    p = sub.add_parser("export"); p.add_argument("campaign"); p.add_argument("output", type=Path)
    for verb in ("verify", "restore"):
        p = sub.add_parser(verb); p.add_argument("archive", type=Path); p.add_argument("--expected-identity")
    p = sub.add_parser("backup"); p.add_argument("output", type=Path)
    p = sub.add_parser("cache-request"); p.add_argument("campaign"); p.add_argument("step"); p.add_argument("output")
    p = sub.add_parser("cache-get"); p.add_argument("request", type=Path)
    p.add_argument("--allow-exact-reuse", action="store_true")
    p = sub.add_parser("view"); p.add_argument("--port", type=int, default=8765)
    p = sub.add_parser("export-mlflow"); p.add_argument("campaign"); p.add_argument("--endpoint", required=True); p.add_argument("--experiment-id", required=True); p.add_argument("--metrics", type=Path)
    p = sub.add_parser("export-otlp"); p.add_argument("campaign"); p.add_argument("--endpoint", required=True)
    p = sub.add_parser("exports"); p.add_argument("--after", type=int, default=0)
    p = sub.add_parser("inspect-export"); p.add_argument("export_id")
    p = sub.add_parser("analysis-validate"); p.add_argument("protocol", type=Path)
    p = sub.add_parser("analyze"); p.add_argument("--protocol", type=Path, required=True)
    source = p.add_mutually_exclusive_group(required=True)
    source.add_argument("--selections", type=Path); source.add_argument("--observations", type=Path)
    p.add_argument("--worker", type=Path, required=True); p.add_argument("--worker-sha256", required=True)
    p.add_argument("--source-commit", required=True); p.add_argument("--output", type=Path, required=True)
    for verb in ("retry-export", "reconcile-export"):
        p = sub.add_parser(verb); p.add_argument("export_id"); p.add_argument("--run-id")
    p = sub.add_parser("search-fixture")
    p.add_argument("--worker", type=Path, required=True); p.add_argument("--tdi-source-commit", required=True)
    p.add_argument("--forge-worker", type=Path, required=True); p.add_argument("--forge-sha256", required=True); p.add_argument("--forge-source-commit", required=True)
    p.add_argument("--strategy", choices=("grid", "random-without-replacement"), default="grid"); p.add_argument("--seed", default="0")
    p = sub.add_parser("searches"); p.add_argument("--after", type=int, default=0); p.add_argument("--limit", type=int, default=100)
    p = sub.add_parser("search-inspect"); p.add_argument("search")
    for verb in ("search-run", "search-resume", "search-cancel"):
        p = sub.add_parser(verb); p.add_argument("search")
        if verb != "search-cancel":
            p.add_argument("--max-stages", type=int, default=128)
    args = parser.parse_args(argv)
    try:
        result = dispatch(args)
        code = durable.EXIT_OK
        if isinstance(result, dict) and (result.get("phase") in ("failed", "cancelled") or result.get("execution_error") is True):
            code = durable.EXIT_TRIAL_FAILURE
        if isinstance(result, dict) and result.get("status") == "resource-rejected":
            code = durable.EXIT_TRIAL_FAILURE
        print(durable.canonical({"schema": 1, "operation": args.operation, "exit_code": code, "result": result}))
        return code
    except ExportError as error:
        code, status, message = EXIT_EXPORT, "external-export-error", str(error)
    except HubTransportUnknown as error:
        code, status = durable.EXIT_TRIAL_FAILURE, "outcome-unknown"
        message = str(error)
    except (durable.ContractError, ValueError, KeyError, TypeError) as error:
        code, status = durable.EXIT_CONTRACT, "contract-error"
        message = str(error)
    except (durable.StorageError, sqlite3.Error, OSError) as error:
        code, status = durable.EXIT_STORAGE, "storage-error"
        message = str(error)
    except KeyboardInterrupt:
        code, status, message = durable.EXIT_CANCELLED, "client-interrupted", "inspect the durable campaign before another execution"
    print(durable.canonical({"schema": 1, "operation": args.operation, "exit_code": code,
                             "status": status, "error": message}))
    print(message, file=sys.stderr)
    return code


def dispatch(args):
    """Execute one CLI operation; network clients are created only when needed."""
    op = args.operation
    if op in ("sensitivity-plan", "sensitivity-fixture-plan", "sensitivity-analyze"):
        from tdi_sensitivity import make_plan, validate_plan, collect, analyze
        if args.output.exists() or args.output.is_symlink():
            raise durable.StorageError("sensitivity output already exists")
        if op == "sensitivity-plan":
            plan = make_plan(read_json(args.protocol))
            atomic_json(args.output, plan)
            return {"identity": plan["identity"], "rows": len(plan["rows"]), "path": str(args.output), "execution_started": False}
        plan = validate_plan(read_json(args.plan, max_items=1000000))
        if op == "sensitivity-fixture-plan":
            from tdi_sensitivity_fixture import prepare
            client = HubClient(args.hub, token=os.environ.get("TDI_HUB_TOKEN"), allow_loopback_http=args.allow_loopback_http, timeout=args.timeout)
            spec = prepare(client, plan, args.function)
            atomic_json(args.output, spec)
            return {"campaign_identity": identity("tdi-operational-campaign/v1", spec), "path": str(args.output), "execution_started": False}
        worker = None
        if plan["protocol"]["method"] != "ablation":
            if not all((args.worker, args.worker_sha256, args.source_commit)):
                raise durable.ContractError("sensitivity analysis requires a pinned SciRust worker")
            from tdi_scirust_client import SciRustStats
            worker = SciRustStats(args.worker, args.worker_sha256, args.source_commit)
        if args.observations:
            observations = read_json(args.observations, max_items=1000000)
        else:
            with EngineStore(args.catalogue, readonly=True) as store:
                if args.campaign:
                    campaign = store.get(args.campaign)
                    selections = [{"campaign": args.campaign, "step": step, "output": output}
                                  for step, outputs in campaign["spec"]["outputs"].items() for output, contract in outputs.items()
                                  if contract["json_fields"].get("plan_identity") == plan["identity"]]
                else:
                    selections = read_json(args.selections)
                observations = collect(store, plan, selections)
        report = analyze(plan, observations, worker)
        atomic_json(args.output, report)
        return {"identity": report["identity"], "path": str(args.output), "result": report["result"], "scientific_verdict": report["scientific_verdict"]}
    if op == "capacity":
        from tdi_physical_telemetry import capacity_snapshot
        return capacity_snapshot()
    if op in ("analysis-validate", "analyze"):
        from tdi_research_analysis import MAX_ANALYSIS_ITEMS, canonical_protocol, observations_from_catalogue, analyze
        protocol = canonical_protocol(read_json(args.protocol, max_items=MAX_ANALYSIS_ITEMS))
        if op == "analysis-validate":
            return {"protocol_identity": identity("tdi-analysis-protocol/v1", protocol), "protocol": protocol}
        from tdi_scirust_client import SciRustStats
        worker = SciRustStats(args.worker, args.worker_sha256, args.source_commit)
        if args.selections:
            with EngineStore(args.catalogue, readonly=True) as store:
                observations = observations_from_catalogue(store, protocol, read_json(args.selections, max_items=MAX_ANALYSIS_ITEMS))
        else:
            observations = read_json(args.observations, max_items=MAX_ANALYSIS_ITEMS)
        report = analyze(protocol, observations, worker)
        atomic_json(args.output, report)
        return {"identity": report["identity"], "output": str(args.output), "results": report["results"], "scientific_verdict": report["scientific_verdict"]}
    if op in ("validate", "plan"):
        spec = runtime.canonical_campaign(read_json(args.spec))
        return {"campaign_identity": identity("tdi-operational-campaign/v1", spec), "steps": len(spec["graph"]["steps"]), "spec": spec}
    if op == "verify":
        value = read_json(args.archive)
        payloads = verify_bundle(value, expected_identity=args.expected_identity)
        return {"identity": value["identity"], "verified_members": len(payloads)}
    if op == "doctor":
        import platform
        return {"python": platform.python_version(), "platform": platform.platform(),
                "sqlite": sqlite3.sqlite_version, "catalogue_exists": args.catalogue.is_file(),
                "cgroup_v2_visible": Path("/sys/fs/cgroup/cgroup.controllers").is_file(),
                "hard_gpu_quota_qualified": False, "hostile_code_isolation_qualified": False,
                "execution_backend": "explicit Hub deployment", "scientific_execution_started": False}
    if op == "view":
        from tdi_engine_viewer import serve
        serve(args.catalogue, args.port)
        return {"status": "stopped"}
    readonly = op in ("status", "inspect", "compare", "events", "backup", "export", "cache-request", "cache-get", "exports", "inspect-export", "searches", "search-inspect")
    if op in ("fixture-plan", "library-fixture-plan", "submit", "run-local-admitted", "run", "resume", "cancel", "attach", "export", "restore", "cache-get", "search-fixture", "search-run", "search-resume", "search-cancel"):
        client = HubClient(args.hub, token=os.environ.get("TDI_HUB_TOKEN"),
                           allow_loopback_http=args.allow_loopback_http, timeout=args.timeout)
    if op in ("fixture-plan", "library-fixture-plan"):
        from tdi_hub_fixture import prepare_fixture
        if args.output.exists() or args.output.is_symlink():
            raise durable.ContractError("fixture plan output already exists")
        if op == "library-fixture-plan":
            from tdi_library_fixture import prepare_library_fixture
            spec = prepare_library_fixture(client, args.worker, adapter=args.adapter, domain=args.domain, trials=args.trials)
        else:
            spec = prepare_fixture(client, args.worker, domain=args.domain, trials=args.trials)
        atomic_json(args.output, spec)
        return {"path": str(args.output), "campaign_identity": identity("tdi-operational-campaign/v1", spec)}
    with EngineStore(args.catalogue, readonly=readonly) as store:
        if op == "searches":
            return store.list_searches(after=args.after, limit=args.limit)
        if op == "search-inspect":
            return store.get_search(args.search)
        if op in ("search-fixture", "search-run", "search-resume", "search-cancel"):
            from tdi_forge_search import ForgeClient, prepare_fixture, run_search, cancel_search
            if op == "search-fixture":
                forge = ForgeClient(args.forge_worker, args.forge_sha256, args.forge_source_commit)
                return prepare_fixture(client, store, forge, args.worker, args.tdi_source_commit, strategy=args.strategy, seed=args.seed)
            if op == "search-cancel":
                return cancel_search(client, store, args.search)
            return run_search(client, store, args.search, max_stages=args.max_stages)
        if op == "run-local-admitted":
            from tdi_resource_admission import execute_local_admitted
            return execute_local_admitted(client, store, read_json(args.spec), read_json(args.roots) if args.roots else {},
                                          args.elastic_worker, args.worker_sha256, args.source_commit, read_json(args.resource_policy))
        if op == "exports":
            return store.list_exports(after=args.after)
        if op == "inspect-export":
            return store.get_export(args.export_id)
        if op in ("export-mlflow", "export-otlp", "retry-export", "reconcile-export"):
            from tdi_observability import MLflowClient, OTLPClient, prepare_mlflow, prepare_otlp, send_export, reconcile_mlflow
            record = store.get_export(args.export_id) if op in ("retry-export", "reconcile-export") else None
            kind = record["kind"] if record else ("mlflow" if op == "export-mlflow" else "otlp")
            client_type = MLflowClient if kind == "mlflow" else OTLPClient
            client = client_type(record["endpoint"] if record else args.endpoint, token=os.environ.get("TDI_EXPORT_TOKEN"),
                                 allow_loopback_http=args.allow_loopback_http, timeout=args.timeout, max_bytes=1024 * 1024)
            if op == "reconcile-export":
                return reconcile_mlflow(store, record["id"], client, args.run_id)
            if record is None:
                record = prepare_mlflow(store, args.campaign, client, args.experiment_id, read_json(args.metrics) if args.metrics else []) if kind == "mlflow" else prepare_otlp(store, args.campaign, client)
            return send_export(store, record["id"], client, retry=op == "retry-export", resume_run=getattr(args, "run_id", None))
        if op == "cache-request":
            from tdi_engine_cache import cache_request
            return cache_request(store, store.get(args.campaign), args.step, args.output)
        if op == "cache-get":
            from tdi_engine_cache import lookup
            return lookup(client, store, read_json(args.request), authorized=args.allow_exact_reuse)
        if op == "status":
            return store.list(after=args.after, limit=args.limit, phase=args.phase)
        if op == "inspect":
            return {"campaign": store.get(args.campaign), "results": store.results(args.campaign)}
        if op == "events":
            return store.events(args.campaign, after=args.after)
        if op == "compare":
            left, right = store.get(args.left), store.get(args.right)
            return {"same_protocol": left["spec"]["graph"]["root_plan_id"] == right["spec"]["graph"]["root_plan_id"],
                    "same_spec": left["spec"] == right["spec"],
                    "left": left, "right": right,
                    "left_results": store.results(args.left), "right_results": store.results(args.right)}
        if op == "backup":
            store.backup(args.output)
            return {"path": str(args.output), "status": "verified-backup"}
        if op == "submit":
            campaign = runtime.submit(client, store, read_json(args.spec), read_json(args.roots) if args.roots else {})
            return {"campaign": campaign, "phase": store.get(campaign)["phase"]}
        if op == "attach":
            return runtime.attach(client, store, args.campaign, args.workflow, read_json(args.roots) if args.roots else {})
        if op == "run":
            return runtime.execute(client, store, args.campaign)
        if op == "resume":
            return runtime.refresh(client, store, args.campaign)
        if op == "cancel":
            return runtime.cancel(client, store, args.campaign)
        if op == "export":
            return export_bundle(client, store, args.campaign, args.output)
        if op == "restore":
            return {"campaign": restore_bundle(client, store, read_json(args.archive), expected_identity=args.expected_identity), "phase": "imported"}
    raise durable.ContractError("unsupported operation")


if __name__ == "__main__":
    raise SystemExit(main())
