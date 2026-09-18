"""Public Development-only comparison of the real Forge bridge and Optuna.

This measures finite-space proposal quality and adapter costs, not Hub execution,
model quality, optimizer-only speed, or a protected/confirmatory research stage.
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.metadata
import itertools
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import sys
import time

import tdi_experiment_supervisor as durable
from tdi_engine_store import atomic_json, identity
from tdi_forge_search import ForgeClient

FORGE_COMMIT = "28067ab0aa1d52a2260d9bb2bf35a346547a292a"
ADAPTIVE_FORGE_COMMIT = "e23f945d4f8bcc283f09826f513f208836efc1c4"
OPTUNA_VERSION = "5.0.0"
VALUES = tuple(range(-8, 8))
TASKS = ("shifted-bowl", "coupled-ridge", "double-well")
ARMS = ("forge-grid", "forge-random", "optuna-random", "optuna-tpe")
ADAPTIVE_TASKS = TASKS + ("shifted-absolute", "rotated-valley", "categorical-interaction")
ADAPTIVE_ARMS = ("forge-random", "forge-tpe", "forge-gp", "optuna-random", "optuna-tpe", "optuna-tpe-multivariate")
PROFILES = {"smoke": (2, 8), "development": (20, 32),
            "adaptive-smoke": (2, 16), "adaptive-development": (20, 32)}
ROOT = Path(__file__).resolve().parents[1]


def objective(task, x, y):
    """Exact public integer objective; lower is better, no external data."""
    if task == "shifted-bowl":
        return (x - 3) ** 2 + 2 * (y + 2) ** 2
    if task == "coupled-ridge":
        return (x + y - 1) ** 2 + (x - 2 * y + 4) ** 2
    if task == "double-well":
        return (x * x - 9) ** 2 + (y * y - 4) ** 2 + (x - y) ** 2
    if task == "shifted-absolute":
        return 3 * abs(x + 5) + abs(y - 4)
    if task == "rotated-valley":
        return (3*x + 2*y - 5)**2 + (x - y + 1)**2
    if task == "categorical-interaction":
        return (7*(x+8) + 11*(y+8)) % 17 + 3*((x-y) % 5)
    raise ValueError("unknown public task")


def oracle(task, x, y):
    """Separately expanded polynomial, audited over the complete finite domain."""
    if task == "shifted-bowl":
        return x*x - 6*x + 2*y*y + 8*y + 17
    if task == "coupled-ridge":
        return 2*x*x - 2*x*y + 5*y*y + 6*x - 18*y + 17
    if task == "double-well":
        return x**4 + y**4 - 17*x*x - 7*y*y - 2*x*y + 97
    if task == "shifted-absolute":
        return (3*x + 15 if x >= -5 else -3*x - 15) + (y - 4 if y >= 4 else 4 - y)
    if task == "rotated-valley":
        return 10*x*x + 10*x*y + 5*y*y - 28*x - 22*y + 26
    if task == "categorical-interaction":
        a, b = 7*x + 11*y + 144, x-y
        return a - 17*(a//17) + 3*(b - 5*(b//5))
    raise ValueError("unknown public task")


def checked_point(point):
    if (set(point) != {"x", "y"} or any(type(v) is not int or v not in VALUES
                                       for v in point.values())):
        raise ValueError("proposal outside the common finite domain")
    return point


def task_bounds(task):
    losses = []
    for x, y in itertools.product(VALUES, repeat=2):
        loss = oracle(task, x, y)
        if loss != objective(task, x, y):
            raise ValueError("objective/oracle disagreement")
        losses.append(loss)
    return {"minimum": min(losses), "maximum": max(losses)}


def protocol(profile):
    if profile not in PROFILES:
        raise ValueError("unknown profile")
    seeds, budget = PROFILES[profile]
    if profile.startswith("adaptive-"):
        p = protocol("smoke")
        p.update({"kind": "tdi-optuna-adaptive-comparison/v1", "profile": profile,
                  "tasks": list(ADAPTIVE_TASKS), "arms": list(ADAPTIVE_ARMS),
                  "seeds": list(range(seeds)), "evaluations_per_arm": budget,
                  "forge_source_commit": ADAPTIVE_FORGE_COMMIT,
                  "forge_tpe": {"generator_version": "forge-finite-tpe/v1",
                                "startup_successes": 10, "elite_fraction": "1/5",
                                "uniform_kernel_mass": "1/5", "joint_mixture_weight": "1/2",
                                "uniform_pseudo_observations": 1, "explore_every": 5,
                                "acquisition": "enumerate-all-admissible-untried"},
                  "forge_gp": {"generator_version": "forge-finite-gp/v1",
                               "scirust_commit": "146575107005c24a47682dcaa08c4cd9464d1cc3",
                               "startup_successes": 10, "kernel": "half-additive-half-exp-hamming-2",
                               "acquisition": "mean-minus-two-stddev", "noise_variance": "1e-6",
                               "normalization": "maxabs-then-center-and-standardize",
                               "explore_every": 5},
                  "multivariate_tpe": dict(p["tpe"], multivariate=True),
                  "interpretation": "public-development-extension-frozen-before-first-run-not-independent-confirmation"})
        p["sampling"].pop("forge-grid")
        p["sampling"].update({"forge-gp": "scirust-categorical-gp-lcb-without-replacement",
                                "forge-tpe": "adaptive-categorical-tpe-without-replacement",
                                "optuna-tpe-multivariate": "multivariate-categorical-tpe-with-replacement"})
        return p
    return {"schema": 1, "kind": "tdi-optuna-finite-comparison/v1",
            "domain": "Development", "confirmatory": False, "profile": profile,
            "tasks": list(TASKS), "arms": list(ARMS), "values": list(VALUES),
            "seeds": list(range(seeds)), "evaluations_per_arm": budget,
            "baseline": {"x": -8, "y": -8}, "baseline_counts_as_evaluation": True,
            "optuna_version": OPTUNA_VERSION, "forge_source_commit": FORGE_COMMIT,
            "tpe": {"n_startup_trials": 10, "n_ei_candidates": 24,
                    "multivariate": False, "constant_liar": False},
            "pruning": False, "concurrency": 1, "cache": False,
            "sampling": {"forge-grid": "lexicographic-without-replacement",
                         "forge-random": "random-without-replacement-after-baseline",
                         "optuna-random": "random-with-replacement-after-baseline",
                         "optuna-tpe": "categorical-tpe-after-enqueued-baseline"},
            "primary_metric": "final-best-loss-minus-grid-minimum",
            "secondary_metric": "mean-normalized-incumbent-regret-over-all-evaluations",
            "order": "rotate-arms-by-task-index-plus-seed",
            "failure_policy": "preserve-partial-records-stop-no-automatic-retry",
            "timing_scope": "adapter-plus-verification-plus-evaluation-plus-durable-IO",
            "hardware_isolation": False, "scientific_verdict": "not-assessed"}


def file_hash(path):
    return durable.file_digest(Path(path))


def sources():
    # Include transitive TDI client imports in provenance, without reading data.
    return {p.name: file_hash(p) for p in sorted((ROOT / "scripts").glob("tdi_*.py"))}


def package_inventory():
    return dict(sorted((d.metadata["Name"], d.version)
                       for d in importlib.metadata.distributions()))


def check_packages():
    lock = ROOT / "scripts/requirements-optuna-benchmark.txt"
    for line in lock.read_text().splitlines():
        if not line or line.startswith("#"):
            continue
        name, expected = line.split("==")
        if importlib.metadata.version(name) != expected:
            raise ValueError("dependency differs from benchmark lock: " + name)
    return file_hash(lock)


def git_revision():
    return subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT,
                                   text=True, timeout=10).strip()


def record(stream, value):
    stream.write(durable.canonical(value) + "\n")
    stream.flush()
    os.fsync(stream.fileno())


def require_before_deadline(deadline, context):
    """Fail closed once the whole-run monotonic acceptance deadline expires."""
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError(f"whole-run deadline reached {context}")
    return remaining


def forge_timeout(deadline):
    """Bound one Forge subprocess by the remaining whole-run budget."""
    return min(30.0, require_before_deadline(deadline, "before Forge control call"))


class ForgeArm:
    """Use the existing TDI client and real Rust ask/begin/finish state machine."""
    def __init__(self, name, seed, budget, task, client, manifest_id, env_id,
                 source_commit, directory, deadline):
        self.client, self.directory, self.env_id = client, directory, env_id
        self.deadline = deadline
        self.checkpoint = None
        self.snapshot = None
        self.sequence = 0
        external = {
            "schema_version": 1, "domain_id": "tdi/public-optuna-comparison",
            "upstream": {"repository": "Memorithm/TDI", "commit_id": source_commit,
                         "contract_sha256": manifest_id},
            "allowed_candidate_dimensions": ["x", "y"],
            "data_boundary": {"generation_sources": ["public-polynomials/development/v1"],
                              "verification_sources": ["expanded-polynomial-oracles/v1"],
                              "final_holdout_sources": []},
            "verification": {"adapter_id": "tdi-polynomial-oracle/v1",
                             "adapter_sha256": file_hash(__file__)},
            "objectives": [{"name": "loss", "direction": "minimize"}],
            "environment": {"fingerprint_required": True, "isolation_required": False}}
        self.spec = {"schema_version": 1, "generator_version": "forge-finite-search/v1",
                     "manifest": {"schema_version": 1, "external_domain": external},
                     "dimensions": [{"name": k, "values": [str(v) for v in VALUES]}
                                    for k in ("x", "y")],
                     "forbidden_combinations": [], "objective_units": ["loss"],
                     "strategy": "grid" if name == "forge-grid" else "random-without-replacement",
                     "seed": str(seed),
                     "budget": {"max_proposals": budget, "max_stage_attempts": budget * 3,
                                "max_attempts_per_stage": 1, "stage_timeout_ms": 1000,
                                "max_reserved_ms": budget * 3000}}
        if name == "forge-tpe":
            self.spec.update(strategy="adaptive-tpe", generator_version="forge-finite-tpe/v1")
        elif name == "forge-gp":
            self.spec.update(strategy="adaptive-gp", generator_version="forge-finite-gp/v1")
        elif name not in ("forge-grid", "forge-random"):
            raise ValueError("unknown Forge arm")
        self.task = task
        # Actual protocol validation, included in setup and bounded by the
        # remaining whole-run wall-clock budget.
        self.client.call(self.spec, timeout=forge_timeout(self.deadline))

    def command(self, operation):
        response, _ = self.client.call(
            self.spec, self.checkpoint,
            {"request_id": str(self.sequence), "operation": operation},
            timeout=forge_timeout(self.deadline))
        receipt = response["snapshot"]["receipts"][-1]
        if receipt["status"] in ("rejected", "duplicate"):
            raise ValueError("Forge refused transition: " + durable.canonical(receipt))
        # Persist the complete replayable history before executing an issued stage.
        durable._atomic_text(self.directory / "checkpoint.json", durable.canonical(response["checkpoint"]))
        self.checkpoint, self.snapshot = response["checkpoint"], response["snapshot"]
        self.sequence += 1
        return receipt

    def ask(self):
        receipt = self.command({"op": "ask"})
        if receipt["status"] != "proposed" or receipt["constraint_rejected"]:
            raise ValueError("unexpected Forge proposal")
        self.proposal = receipt["proposal"]
        return checked_point({k: int(v) for k, v in self.proposal["parameters"].items()})

    def begin(self, stage):
        self.permit = self.command({"op": "begin", "candidate_id": self.proposal["candidate_id"],
                                    "stage": stage})["permit"]

    def finish(self, stage, evidence, elapsed_ns):
        external = self.spec["manifest"]["external_domain"]
        if stage == "compile":
            self.artifact = evidence["artifact_sha256"]
            outcome = {"kind": "compiled", "artifact_sha256": self.artifact,
                       "materialization": "precompiled-configuration"}
        elif stage == "verify":
            self.evidence_id = identity("tdi-optuna-verification/v1", evidence)
            outcome = {"kind": "verified", "artifact_sha256": self.artifact,
                       "evidence": {"upstream": external["upstream"],
                                    "verification": external["verification"],
                                    "verification_source": external["data_boundary"]["verification_sources"][0],
                                    "candidate_id": self.proposal["candidate_id"], "passed": True,
                                    "evidence_id": self.evidence_id,
                                    "environment_fingerprint": self.env_id}}
        else:
            outcome = {"kind": "measured", "artifact_sha256": self.artifact,
                       "verification_evidence_id": self.evidence_id,
                       "evidence_sha256": identity("tdi-optuna-measurement/v1", evidence),
                       "environment_id": self.env_id,
                       "metrics": [{"name": "loss", "unit": "loss", "value": float(evidence["loss"])}]}
        self.command({"op": "finish", "attempt_id": self.permit["attempt_id"],
                      "wall_ms": (elapsed_ns + 999999) // 1000000, "outcome": outcome})

    def close(self):
        if not self.snapshot["baseline_qualified"] or any(
                c["metrics"] is None for c in self.snapshot["candidates"]):
            raise ValueError("incomplete Forge measurements")
        atomic_json(self.directory / "forge-final.json", {"spec": self.spec,
                    "checkpoint": self.checkpoint, "snapshot": self.snapshot})


class OptunaArm:
    def __init__(self, name, seed, settings):
        import optuna
        if name not in ("optuna-random", "optuna-tpe", "optuna-tpe-multivariate"):
            raise ValueError("unknown Optuna arm")
        if name == "optuna-tpe-multivariate":
            settings = dict(settings, multivariate=True)
        sampler = (optuna.samplers.RandomSampler(seed=seed) if name == "optuna-random"
                   else optuna.samplers.TPESampler(seed=seed, **settings))
        self.study = optuna.create_study(direction="minimize", sampler=sampler,
                                        pruner=optuna.pruners.NopPruner())
        # Forge reserves the first point as its baseline; give Optuna the same
        # first observation and charge it to every arm's evaluation budget.
        self.study.enqueue_trial({"x": -8, "y": -8})

    def ask(self):
        self.trial = self.study.ask()
        return checked_point({k: self.trial.suggest_categorical(k, list(VALUES)) for k in ("x", "y")})

    def begin(self, stage):
        pass

    def finish(self, stage, evidence, elapsed_ns):
        if stage == "measure":
            self.study.tell(self.trial, evidence["loss"])

    def close(self):
        pass


def run_arm(arm, task, seed, budget, directory, stream, deadline):
    best, seen, rows = None, set(), []
    for index in range(budget):
        require_before_deadline(deadline, "before trial")
        start = time.perf_counter_ns()
        point = arm.ask()
        require_before_deadline(deadline, "after proposal")
        checked_point(point)
        if index == 0 and point != {"x": -8, "y": -8}:
            raise ValueError("common baseline missing")
        point_key = (point["x"], point["y"])
        duplicate = point_key in seen
        seen.add(point_key)
        stages = {}
        for stage in ("compile", "verify", "measure"):
            arm.begin(stage)
            require_before_deadline(deadline, f"after {stage} begin")
            tick = time.perf_counter_ns()
            if stage == "compile":
                payload = {"task": task, "parameters": point, "materialization": "precompiled-configuration"}
                evidence = {"artifact_sha256": identity("tdi-optuna-configuration/v1", payload)}
            elif stage == "verify":
                checked_point(point)
                expected = oracle(task, **point)
                evidence = {"task": task, "parameters": point, "expected": expected}
            else:
                loss = objective(task, **point)
                if loss != expected:
                    raise ValueError("evaluated objective disagrees with independent polynomial")
                evidence = {"task": task, "parameters": point, "loss": loss}
            elapsed = time.perf_counter_ns() - tick
            # Save evidence before telling the optimizer; reuse no hidden cache.
            atomic_json(directory / f"trial-{index:03d}-{stage}.json", evidence)
            require_before_deadline(deadline, f"after {stage} evidence persistence")
            arm.finish(stage, evidence, elapsed)
            require_before_deadline(deadline, f"after {stage} finish")
            stages[stage + "_body_ns"] = str(elapsed)
        # Do not accept a completed trial after the run deadline. Durable stage
        # evidence may remain for diagnosis, but it is not counted as a result.
        require_before_deadline(deadline, "before accepting completed trial")
        best = loss if best is None else min(best, loss)
        row = {"trial": index, "parameters": point, "loss": loss, "best_loss": best,
               "duplicate": duplicate, **stages,
               "adapter_trial_ns": str(time.perf_counter_ns() - start)}
        rows.append(row)
        record(stream, {"kind": "trial", "task": task, "seed": seed,
                        "arm": directory.name, **row})
        require_before_deadline(deadline, "after durable trial record")
    arm.close()
    require_before_deadline(deadline, "after arm close")
    return rows


def summarize(manifest, runs):
    """Recompute scores, inventories and matched-budget completeness from raw rows."""
    p = manifest["protocol"]
    if p != protocol(p["profile"]):
        raise ValueError("protocol differs from the declared comparison profile")
    if manifest["task_bounds"] != {task: task_bounds(task) for task in p["tasks"]}:
        raise ValueError("incorrect objective reference bounds")
    expected = set(itertools.product(p["tasks"], p["seeds"], p["arms"]))
    keys = [(r["task"], r["seed"], r["arm"]) for r in runs]
    if len(keys) != len(set(keys)) or set(keys) != expected:
        raise ValueError("missing/duplicate task-seed-arm run")
    summaries = []
    for run in runs:
        rows = run["trials"]
        if len(rows) != p["evaluations_per_arm"]:
            raise ValueError("unequal or incomplete evaluation budget")
        best, seen, regrets = None, set(), []
        bounds = manifest["task_bounds"][run["task"]]
        for index, row in enumerate(rows):
            point = checked_point(row["parameters"])
            key = tuple(point[k] for k in ("x", "y"))
            loss = oracle(run["task"], **point)
            best = loss if best is None else min(best, loss)
            if (type(row["loss"]) is not int or type(row["best_loss"]) is not int
                    or row["trial"] != index or row["loss"] != loss or row["best_loss"] != best
                    or row["duplicate"] != (key in seen)
                    or (run["arm"].startswith("forge-") and key in seen)
                    or (index == 0 and point != p["baseline"])):
                raise ValueError("invalid raw trajectory or baseline")
            seen.add(key)
            regrets.append(best - bounds["minimum"])
        summaries.append({"task": run["task"], "seed": run["seed"], "arm": run["arm"],
                          "final_regret": regrets[-1], "unique_points": len(seen),
                          "normalized_auc": statistics.mean(regrets) / (bounds["maximum"] - bounds["minimum"]),
                          "observed_adapter_ns": run["observed_adapter_ns"]})
    aggregates = []
    for task, arm in itertools.product(p["tasks"], p["arms"]):
        selected = [x for x in summaries if x["task"] == task and x["arm"] == arm]
        aggregates.append({"task": task, "arm": arm, "seeds": len(selected),
                           "median_final_regret": statistics.median(x["final_regret"] for x in selected),
                           "median_normalized_auc": statistics.median(x["normalized_auc"] for x in selected),
                           "median_unique_points": statistics.median(x["unique_points"] for x in selected)})
    return {"runs": summaries, "aggregates": aggregates,
            "inference": "descriptive-public-development-only; no significance or superiority claim"}


def verify_report(path):
    """Verify identity and recompute all quality summaries; hashes are not signatures."""
    report = durable.strict_json(Path(path).read_bytes(), max_bytes=16 * 1024 * 1024,
                                 max_items=1000000)
    content = {k: v for k, v in report.items() if k != "identity"}
    if (report["status"] != "complete"
            or report["identity"] != identity("tdi-optuna-report/v1", content)
            or report["manifest_identity"] != identity("tdi-optuna-manifest/v1", report["manifest"])
            or report["summary"] != summarize(report["manifest"], report["raw_runs"])):
        raise ValueError("comparison report identity or summary mismatch")
    return report


def run(output, forge_binary, profile, max_seconds=1800):
    import optuna
    if importlib.metadata.version("optuna") != OPTUNA_VERSION:
        raise ValueError("install the pinned Optuna requirements")
    requirements_sha256 = check_packages()
    optuna.logging.set_verbosity(optuna.logging.WARNING)
    p = protocol(profile)
    binary = Path(forge_binary).resolve(strict=True)
    client = ForgeClient(binary, file_hash(binary), p["forge_source_commit"])
    source_hashes = sources()
    environment = {"python": platform.python_version(), "platform": platform.platform(),
                   "machine": platform.machine(), "cpu_count": os.cpu_count(),
                   "packages": package_inventory(), "python_sha256": file_hash(Path(sys.executable).resolve())}
    manifest = {"protocol": p, "tdi_source_commit": git_revision(), "source_sha256": source_hashes,
                "requirements_sha256": requirements_sha256,
                "forge": client.binding, "environment": environment,
                "task_bounds": {task: task_bounds(task) for task in p["tasks"]},
                "max_seconds": max_seconds,
                "limits": ["finite categorical public polynomials, not an ML or continuous-space benchmark",
                           "same numeric seeds do not mean identical random draws",
                           "Forge uses subprocess replay; Optuna uses in-process memory storage",
                           "adapter timings include this asymmetry and are not optimizer-only speed",
                           "all arms retain durable evidence; storage policies differ",
                           "no Hub, GPU, hostile-code, recovery, pruning or multi-objective qualification"]}
    manifest_id = identity("tdi-optuna-manifest/v1", manifest)
    env_id = identity("tdi-optuna-environment/v1", environment)
    output = Path(output)
    output.mkdir(parents=False, exist_ok=False)
    atomic_json(output / "manifest.json", dict(manifest, identity=manifest_id))
    deadline = time.monotonic() + max_seconds
    runs = []
    try:
        with (output / "events.jsonl").open("x", encoding="utf-8") as stream:
            for task_index, task in enumerate(p["tasks"]):
                for seed in p["seeds"]:
                    offset = (task_index + seed) % len(p["arms"])
                    for name in p["arms"][offset:] + p["arms"][:offset]:
                        directory = output / task / str(seed) / name
                        directory.mkdir(parents=True, exist_ok=False)
                        start = time.perf_counter_ns()
                        record(stream, {"kind": "start", "task": task, "seed": seed, "arm": name})
                        arm = (ForgeArm(name, seed, p["evaluations_per_arm"], task, client,
                                        manifest_id, env_id, manifest["tdi_source_commit"], directory,
                                        deadline)
                               if name.startswith("forge-") else OptunaArm(name, seed, p["tpe"]))
                        rows = run_arm(arm, task, seed, p["evaluations_per_arm"], directory, stream, deadline)
                        require_before_deadline(deadline, "before accepting completed arm")
                        result = {"task": task, "seed": seed, "arm": name, "trials": rows,
                                  "observed_adapter_ns": str(time.perf_counter_ns() - start)}
                        atomic_json(directory / "run.json", result)
                        runs.append(result)
                    print(f"completed {task} seed {seed}", flush=True)
        require_before_deadline(deadline, "before final source verification")
        if sources() != source_hashes or file_hash(binary) != client.binding["sha256"]:
            raise ValueError("source or Forge binary changed during comparison")
        require_before_deadline(deadline, "after final source verification")
        report = {"schema": 1, "status": "complete", "manifest_identity": manifest_id,
                  "manifest": manifest, "raw_runs": runs, "summary": summarize(manifest, runs)}
        require_before_deadline(deadline, "before complete report publication")
        report["identity"] = identity("tdi-optuna-report/v1", report)
        atomic_json(output / "report.json", report)
        return report
    except BaseException as error:
        atomic_json(output / "failure.json", {"status": "incomplete", "manifest_identity": manifest_id,
                    "completed_runs": len(runs), "error_type": type(error).__name__, "error": str(error)[:1000]})
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--forge-worker", type=Path)
    parser.add_argument("--profile", choices=tuple(PROFILES), default="smoke")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--verify-report", type=Path)
    parser.add_argument("--max-seconds", type=int, default=1800)
    args = parser.parse_args()
    if args.verify_report:
        if args.forge_worker or args.output:
            parser.error("verification cannot be combined with execution")
        try:
            report = verify_report(args.verify_report)
        except (OSError, ValueError, KeyError, TypeError) as error:
            print(f"invalid report: {error}", file=sys.stderr)
            return 21
        print(durable.canonical({"verified": True, "identity": report["identity"]}))
        return 0
    if not args.forge_worker or not args.output:
        parser.error("execution requires --forge-worker and --output")
    if not 1 <= args.max_seconds <= 7200:
        parser.error("max-seconds must be 1..7200")
    try:
        report = run(args.output, args.forge_worker, args.profile, args.max_seconds)
    except (OSError, ValueError, RuntimeError, TimeoutError) as error:
        print(f"comparison incomplete: {error}", file=sys.stderr)
        return 20
    print(durable.canonical({"status": report["status"], "identity": report["identity"],
                            "report": str(args.output / "report.json")}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
