"""Execution engine for schema-2 TDI cgroup-v2 campaigns."""
from pathlib import Path
import subprocess
import sys

import tdi_experiment_supervisor as durable
from tdi_linux_containment import CgroupV2Attempt, ContainmentError, spawn_reaper
from tdi_linux_contract import (
    attempt_identity, execution_profile, journal_binding, recovery_marker,
    validate_plan,
)

TECHNICAL_FAILURES = set(durable.TECHNICAL_FAILURES) | {"ContainmentFailed"}


def _launcher_argv(group, worker_argv):
    return [sys.executable, str(Path(__file__).with_name("tdi_cgroup_exec.py").resolve()),
            "--cgroup-procs", str(group.path / "cgroup.procs"), "--", *worker_argv]


def _terminate_and_measure(group):
    evidence = group.metrics()
    if group.is_populated():
        group.kill()
    return evidence


def run(plan, root, journal_path, cgroup_parent, cancelled=lambda: False,
        after_commit=lambda _: None, anchor_path=None, recovery_dir=None):
    """Run one schema-2 campaign with one isolated cgroup per durable attempt."""
    root = Path(root).resolve(strict=True)
    plan_id = validate_plan(plan, root)
    profile = execution_profile(plan)
    cgroup_parent = Path(cgroup_parent).resolve(strict=True)
    recovery_dir = Path(recovery_dir) if recovery_dir else Path(str(journal_path) + ".recovery")
    recovery_dir.mkdir(parents=True, exist_ok=True)
    journal = durable.Journal(journal_path, journal_binding(plan), anchor_path=anchor_path)
    try:
        done, active, _ = journal.read()
        if active is not None:
            attempt_id = attempt_identity(plan_id, active)
            marker_path = recovery_dir / f"{attempt_id}.json"
            interrupted = {
                "status": "Interrupted",
                "reason": "previous supervisor ended before durable completion; not retried",
                "attempt_id": attempt_id,
                "recovery": CgroupV2Attempt.reconcile_existing(cgroup_parent, attempt_id),
            }
            marker = recovery_marker(marker_path)
            if marker is not None:
                interrupted["reaper_marker"] = marker
            journal.append({"kind": "Finish", "index": active, "result": interrupted})
            done, _, _ = journal.read()

        for index in plan["indices"][len(done):]:
            if cancelled():
                break
            validate_plan(plan, root)
            seed = index | (2**63 if plan["domain"] == "Validation" else 0)
            attempt_id = attempt_identity(plan_id, index)
            marker_path = recovery_dir / f"{attempt_id}.json"
            if marker_path.exists():
                raise ContainmentError("recovery marker already exists for a new attempt identity")
            group = CgroupV2Attempt(cgroup_parent, attempt_id, profile).create()
            reaper = spawn_reaper(group.path, marker_path)
            journal.append({"kind": "Start", "index": index})
            worker_argv = [str(durable.artifact(root, plan["argv"][0])), *plan["argv"][1:],
                           "--tdi-seed", str(seed), "--tdi-plan-id", plan_id]
            result = {"attempt_id": attempt_id, "backend": "linux-cgroup-v2"}
            cleanup_errors = []
            try:
                result.update(durable.supervise(_launcher_argv(group, worker_argv),
                                                plan["timeout_seconds"],
                                                plan["max_output_bytes"], cancelled))
                result["resource_evidence"] = _terminate_and_measure(group)
                if result["status"] == "Completed":
                    if result["stdout"] is None:
                        raise durable.ContractError("worker stdout is not valid UTF-8")
                    response = durable.strict_json(result["stdout"],
                                                   max_bytes=plan["max_output_bytes"])
                    if (not isinstance(response, dict) or response.get("seed") != seed
                            or type(response.get("seed")) is not int
                            or response.get("plan_id") != plan_id
                            or response.get("status") not in ("Evaluated", "Rejected")):
                        raise durable.ContractError("worker response binding or status mismatch")
                    result["response"] = response
                validate_plan(plan, root)
            except (durable.ContractError, OSError, subprocess.SubprocessError) as error:
                result.update(status="ContractRejected", error=str(error)[:4096])
            except ContainmentError as error:
                result.update(status="ContainmentFailed", error=str(error)[:4096])
            finally:
                try:
                    if group.path.exists():
                        if group.is_populated():
                            group.kill()
                        if "resource_evidence" not in result:
                            result["resource_evidence"] = group.metrics()
                        group.cleanup()
                except ContainmentError as error:
                    cleanup_errors.append(f"cgroup cleanup: {error}")
                try:
                    reaper.release()
                except ContainmentError as error:
                    cleanup_errors.append(f"reaper release: {error}")
                if cleanup_errors:
                    prior = result.get("status")
                    result.update(status="ContainmentFailed", containment_errors=cleanup_errors)
                    if prior and prior != "ContainmentFailed":
                        result["prior_status"] = prior
            journal.append({"kind": "Finish", "index": index, "result": result})
            after_commit(index)
        return journal.read()[0]
    finally:
        journal.close()


def exit_code_for_records(records, stopped=False):
    if stopped:
        return durable.EXIT_CANCELLED
    return durable.EXIT_TRIAL_FAILURE if any(
        record.get("result", {}).get("status") in TECHNICAL_FAILURES for record in records
    ) else durable.EXIT_OK
